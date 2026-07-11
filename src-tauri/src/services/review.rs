//! Spaced-review engine (Phase 6) — the "hard-coded" half of the retention
//! feature (COURSE_BLUEPRINT.md §7, LESSON_COURSE_DESIGN.md §6): it wraps the
//! pure-Rust **`rs-fsrs`** crate (on-device, no network, no AI) to schedule when
//! a solved Stage-1 problem should be re-solved *cold*, and builds the
//! interleaved due queue + the honest habit header.
//!
//! We use `rs-fsrs`'s **long-term** scheduler (`enable_short_term = false`), so a
//! card is spaced in *days* from its very first review — the right granularity
//! for re-solving whole problems, not the minute-level learning steps meant for
//! flashcards. Fuzz is off by default, so scheduling is deterministic and
//! testable. We schedule with FSRS's default weights and never train them: the
//! Stage-1 slice has far too little review history to fit per-user parameters,
//! and the default curve is the honest, well-calibrated baseline.
//!
//! Why `rs-fsrs` and not the heavier `fsrs` crate: `fsrs` bundles an ML
//! optimizer (via `burn`) we don't need and that would bloat the offline
//! installer; `rs-fsrs` is the same project's pure-Rust *scheduler*, depending
//! only on `chrono` + `serde` — exactly the "pure-Rust on-device" fit §7 calls
//! for.
//!
//! Flow: solving/gating a course problem calls [`enqueue`] (idempotent, Stage-1
//! only); [`queue`] surfaces what's due, interleaved across patterns; a cold
//! re-solve is graded through [`record`], where an `again` **demotes** the card
//! (interval collapses, lapse counter bumps).

use std::collections::VecDeque;

use chrono::{DateTime, NaiveDate, Utc};
use rs_fsrs::{Card, Parameters, Rating, State, FSRS};

use crate::domain::review::{
    HabitState, ReviewCardState, ReviewItem, ReviewOutcome, ReviewQueue, ReviewRating,
};
use crate::error::{AppError, AppResult};
use crate::services::curriculum::CurriculumStore;
use crate::services::db::{attempts, review, Db};
use crate::services::progress;

/// An FSRS scheduler with default weights, long-term (day-level) spacing, and
/// fuzz disabled (deterministic). Cheap to build per call — it holds only the
/// parameter block.
fn scheduler() -> FSRS {
    let params = Parameters {
        enable_short_term: false,
        ..Parameters::default()
    };
    FSRS::new(params)
}

/// Adds a solved/gated course problem to the review queue as a fresh card due
/// now. No-ops (returns `false`) for a problem the course doesn't teach — the
/// wider catalog never enters spaced review — and for one already queued (its
/// existing schedule is preserved). Callers treat a DB hiccup here as
/// non-fatal: missing one enqueue must never fail a solve.
pub fn enqueue(
    store: &CurriculumStore,
    db: &Db,
    problem_id: &str,
    now: DateTime<Utc>,
) -> AppResult<bool> {
    if !store.is_course_problem(problem_id) {
        return Ok(false);
    }
    review::enqueue_new(db, problem_id, &now.to_rfc3339())
}

/// Partial-prereq-credit review (Phase 7, Math-Academy "repetition
/// compression", BLUEPRINT.md §7). When a learner solves a problem in some unit,
/// they've just *exercised* every pattern that unit builds on — so the
/// prerequisite cards already due for a cold re-solve get a lighter,
/// partial-credit review (a `Hard`-grade advancement) instead of demanding a
/// full separate re-solve. This keeps spaced review from ballooning as the
/// ladder grows: climbing the ladder pays down the review debt of its
/// foundations. Only graduated (`review`/`relearning`) cards that are actually
/// due are credited; new/learning cards and not-yet-due cards are untouched.
/// Best-effort, like [`enqueue`] — returns how many cards were credited.
pub fn partial_prereq_credit(
    store: &CurriculumStore,
    db: &Db,
    problem_id: &str,
    now: DateTime<Utc>,
) -> AppResult<u32> {
    let Some(unit) = store.unit_of_problem(problem_id) else {
        return Ok(0);
    };
    let ancestors = store.ancestors_of(unit);
    if ancestors.is_empty() {
        return Ok(0);
    }
    let sched = scheduler();
    let mut credited = 0u32;
    for row in review::list_all(db)? {
        if row.problem_id == problem_id {
            continue;
        }
        let Some(card_unit) = store.unit_of_problem(&row.problem_id) else {
            continue;
        };
        if !ancestors.contains(card_unit) {
            continue;
        }
        let state = state_from_wire(&row.state);
        if !(state == State::Review || state == State::Relearning) {
            continue;
        }
        let is_due = row
            .due_at
            .as_deref()
            .and_then(parse_dt)
            .map(|d| d <= now)
            .unwrap_or(false);
        if !is_due {
            continue;
        }
        let next = sched.next(card_from_row(&row, now), now, Rating::Hard).card;
        review::upsert(
            db,
            &review::ReviewRow {
                problem_id: row.problem_id.clone(),
                state: state_to_wire(next.state).to_string(),
                stability: Some(next.stability),
                difficulty: Some(next.difficulty),
                due_at: Some(next.due.to_rfc3339()),
                last_reviewed_at: Some(now.to_rfc3339()),
                lapses: next.lapses as i64,
            },
        )?;
        credited += 1;
    }
    Ok(credited)
}

/// Records one cold re-solve and reschedules the card via FSRS. Rejects a
/// problem that never entered the queue. An `again` grade is a demotion: FSRS
/// collapses the interval and, once the card has graduated to `review`, bumps
/// its lapse counter.
pub fn record(
    db: &Db,
    problem_id: &str,
    rating: ReviewRating,
    now: DateTime<Utc>,
) -> AppResult<ReviewOutcome> {
    let row = review::get(db, problem_id)?
        .ok_or_else(|| AppError::NotFound(format!("'{problem_id}' is not in the review queue")))?;

    let card = card_from_row(&row, now);
    let info = scheduler().next(card, now, to_fsrs_rating(rating));
    let next = info.card;

    let due_at = next.due.to_rfc3339();
    let new_row = review::ReviewRow {
        problem_id: problem_id.to_string(),
        state: state_to_wire(next.state).to_string(),
        stability: Some(next.stability),
        difficulty: Some(next.difficulty),
        due_at: Some(due_at.clone()),
        last_reviewed_at: Some(now.to_rfc3339()),
        lapses: next.lapses as i64,
    };
    review::upsert(db, &new_row)?;

    Ok(ReviewOutcome {
        problem_id: problem_id.to_string(),
        state: to_card_state(next.state),
        due_at,
        interval_days: (next.due - now).num_days().max(0),
        lapses: next.lapses.max(0) as u32,
        demoted: rating == ReviewRating::Again,
    })
}

/// Builds the review page payload: the cards due now (interleaved across
/// patterns so the learner never re-solves two of the same kind back-to-back),
/// how many are scheduled for later, and the habit header. `now` drives due-ness
/// and the "reviewed today" tally; `today` (the local calendar day) drives the
/// streak.
pub fn queue(
    store: &CurriculumStore,
    db: &Db,
    now: DateTime<Utc>,
    today: NaiveDate,
) -> AppResult<ReviewQueue> {
    let rows = review::list_all(db)?;

    let mut due = Vec::new();
    let mut later_count = 0u32;
    for row in &rows {
        let due_dt = row.due_at.as_deref().and_then(parse_dt);
        // A row with an unparseable/missing due date is treated as due now
        // rather than lost.
        let is_due = due_dt.map(|d| d <= now).unwrap_or(true);
        if is_due {
            let overdue_days = due_dt.map(|d| (now - d).num_days().max(0)).unwrap_or(0);
            due.push(ReviewItem {
                problem_id: row.problem_id.clone(),
                unit_id: store
                    .unit_of_problem(&row.problem_id)
                    .unwrap_or_default()
                    .to_string(),
                state: card_state_from_wire(&row.state),
                due_at: row.due_at.clone().unwrap_or_else(|| now.to_rfc3339()),
                last_reviewed_at: row.last_reviewed_at.clone(),
                lapses: row.lapses.max(0) as u32,
                overdue_days,
            });
        } else {
            later_count += 1;
        }
    }

    let due = interleave(due);
    let habit = habit(db, now, today, due.len() as u32, &rows);
    Ok(ReviewQueue {
        due,
        later_count,
        habit,
    })
}

/// The honest habit header (COURSE_BLUEPRINT.md §7). Streak = calendar days with
/// a passing submit, forgiving one missed day via a freeze; due/reviewed counts
/// come from the schedule. No XP, no leaderboards — just "did you show up, and
/// what's waiting."
fn habit(
    db: &Db,
    now: DateTime<Utc>,
    today: NaiveDate,
    due_today: u32,
    rows: &[review::ReviewRow],
) -> HabitState {
    let pass_days: Vec<NaiveDate> = attempts::pass_days(db)
        .unwrap_or_default()
        .iter()
        .filter_map(|(day, _)| NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
        .collect();
    let streak = progress::streak_with_freezes(&pass_days, today);

    let today_utc = now.format("%Y-%m-%d").to_string();
    let reviewed_today = rows
        .iter()
        .filter(|r| {
            r.last_reviewed_at
                .as_deref()
                .is_some_and(|s| s.starts_with(&today_utc))
        })
        .count() as u32;

    HabitState {
        current_streak: streak.current,
        best_streak: streak.best,
        freeze_active: streak.freeze_active,
        due_today,
        reviewed_today,
    }
}

/// Round-robins due items across their units so consecutive re-solves favour
/// *different* patterns — the interleaving the research calls for
/// (LESSON_COURSE_DESIGN.md §8). Groups keep first-appearance order (which is
/// due-order, since `list_all` sorts by due date), then one is taken from each
/// group per pass.
fn interleave(items: Vec<ReviewItem>) -> Vec<ReviewItem> {
    let mut groups: Vec<(String, VecDeque<ReviewItem>)> = Vec::new();
    for item in items {
        match groups.iter_mut().find(|(k, _)| *k == item.unit_id) {
            Some((_, q)) => q.push_back(item),
            None => {
                let key = item.unit_id.clone();
                let mut q = VecDeque::new();
                q.push_back(item);
                groups.push((key, q));
            }
        }
    }

    let mut out = Vec::new();
    let mut drained = false;
    while !drained {
        drained = true;
        for (_, q) in groups.iter_mut() {
            if let Some(item) = q.pop_front() {
                out.push(item);
                drained = false;
            }
        }
    }
    out
}

/// Reconstructs an FSRS card from its stored row. `elapsed_days`/`reps` are left
/// at 0: the scheduler recomputes elapsed from `now - last_review`, and `reps`
/// only seeds the (disabled) fuzz. A `new` card carries 0 stability/difficulty —
/// the scheduler initialises them from the first rating.
fn card_from_row(row: &review::ReviewRow, now: DateTime<Utc>) -> Card {
    Card {
        due: row.due_at.as_deref().and_then(parse_dt).unwrap_or(now),
        stability: row.stability.unwrap_or(0.0),
        difficulty: row.difficulty.unwrap_or(0.0),
        elapsed_days: 0,
        scheduled_days: 0,
        reps: 0,
        lapses: row.lapses.max(0) as i32,
        state: state_from_wire(&row.state),
        last_review: row
            .last_reviewed_at
            .as_deref()
            .and_then(parse_dt)
            .unwrap_or(now),
    }
}

fn parse_dt(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

fn to_fsrs_rating(r: ReviewRating) -> Rating {
    match r {
        ReviewRating::Again => Rating::Again,
        ReviewRating::Hard => Rating::Hard,
        ReviewRating::Good => Rating::Good,
        ReviewRating::Easy => Rating::Easy,
    }
}

fn state_to_wire(s: State) -> &'static str {
    match s {
        State::New => "new",
        State::Learning => "learning",
        State::Review => "review",
        State::Relearning => "relearning",
    }
}

fn state_from_wire(s: &str) -> State {
    match s {
        "learning" => State::Learning,
        "review" => State::Review,
        "relearning" => State::Relearning,
        _ => State::New,
    }
}

fn to_card_state(s: State) -> ReviewCardState {
    match s {
        State::New => ReviewCardState::New,
        State::Learning => ReviewCardState::Learning,
        State::Review => ReviewCardState::Review,
        State::Relearning => ReviewCardState::Relearning,
    }
}

fn card_state_from_wire(s: &str) -> ReviewCardState {
    to_card_state(state_from_wire(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    use crate::services::db::attempts::{record_attempt, AttemptRecord};
    use crate::services::pack_store::PackStore;

    fn real_resources() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    fn fixture() -> (tempfile::TempDir, CurriculumStore, Db) {
        let packs = PackStore::new(real_resources().join("test-packs.json.gz"));
        let store = CurriculumStore::load(&real_resources(), &packs).expect("curriculum loads");
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, store, db)
    }

    fn t(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn only_course_problems_enter_the_queue() {
        let (_dir, store, db) = fixture();
        // two-sum is a worked example of arrays-hashing.
        assert!(enqueue(&store, &db, "two-sum", t("2026-07-10T09:00:00+00:00")).unwrap());
        // A wider-catalog slug the course never references is ignored.
        assert!(!enqueue(
            &store,
            &db,
            "not-a-course-problem-xyz",
            t("2026-07-10T09:00:00+00:00")
        )
        .unwrap());
        assert!(review::get(&db, "two-sum").unwrap().is_some());
        assert!(review::get(&db, "not-a-course-problem-xyz")
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_solved_problem_becomes_due_immediately() {
        let (_dir, store, db) = fixture();
        let now = t("2026-07-10T09:00:00+00:00");
        enqueue(&store, &db, "two-sum", now).unwrap();
        let q = queue(&store, &db, now, now.date_naive()).unwrap();
        assert_eq!(q.due.len(), 1);
        assert_eq!(q.due[0].problem_id, "two-sum");
        assert_eq!(q.due[0].unit_id, "arrays-hashing");
        assert_eq!(q.due[0].state, ReviewCardState::New);
        assert_eq!(q.later_count, 0);
    }

    #[test]
    fn a_good_resolve_spaces_the_card_out_by_days() {
        let (_dir, store, db) = fixture();
        let now = t("2026-07-10T09:00:00+00:00");
        enqueue(&store, &db, "two-sum", now).unwrap();

        let out = record(&db, "two-sum", ReviewRating::Good, now).unwrap();
        assert_eq!(out.state, ReviewCardState::Review);
        assert!(
            out.interval_days >= 1,
            "good interval was {}",
            out.interval_days
        );
        assert!(!out.demoted);
        assert_eq!(out.lapses, 0);
        // No longer due today — it's scheduled into the future.
        assert!(parse_dt(&out.due_at).unwrap() > now);
        let q = queue(&store, &db, now, now.date_naive()).unwrap();
        assert!(q.due.is_empty());
        assert_eq!(q.later_count, 1);
    }

    #[test]
    fn failing_a_review_demotes_it() {
        let (_dir, store, db) = fixture();
        let now = t("2026-07-10T09:00:00+00:00");
        enqueue(&store, &db, "two-sum", now).unwrap();
        // Graduate it once so the card is in `review`, then fail the re-solve.
        let good = record(&db, "two-sum", ReviewRating::Good, now).unwrap();
        let again = record(&db, "two-sum", ReviewRating::Again, now).unwrap();

        assert!(again.demoted);
        assert_eq!(again.lapses, 1, "a failed review bumps the lapse counter");
        // Demotion pulls it back in far sooner than the good interval had.
        assert!(
            again.interval_days < good.interval_days,
            "again {} should be sooner than good {}",
            again.interval_days,
            good.interval_days
        );
        // And it's back in rotation (due again very soon).
        let soon = now + chrono::Duration::days(again.interval_days + 1);
        let q = queue(&store, &db, soon, soon.date_naive()).unwrap();
        assert_eq!(q.due.len(), 1);
        assert_eq!(q.due[0].lapses, 1);
    }

    #[test]
    fn recording_an_unqueued_problem_is_rejected() {
        let (_dir, _store, db) = fixture();
        let err = record(
            &db,
            "two-sum",
            ReviewRating::Good,
            t("2026-07-10T09:00:00+00:00"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("not in the review queue"), "{err}");
    }

    #[test]
    fn due_queue_interleaves_across_patterns() {
        // Two arrays-hashing cards and one two-pointers card, all due — the queue
        // must not front-load both arrays cards before the two-pointers one.
        let mk = |slug: &str, unit: &str| ReviewItem {
            problem_id: slug.into(),
            unit_id: unit.into(),
            state: ReviewCardState::Review,
            due_at: "2026-07-10T09:00:00+00:00".into(),
            last_reviewed_at: None,
            lapses: 0,
            overdue_days: 0,
        };
        let out = interleave(vec![
            mk("two-sum", "arrays-hashing"),
            mk("group-anagrams", "arrays-hashing"),
            mk("valid-palindrome", "two-pointers"),
        ]);
        let units: Vec<_> = out.iter().map(|i| i.unit_id.as_str()).collect();
        assert_eq!(
            units,
            vec!["arrays-hashing", "two-pointers", "arrays-hashing"]
        );
    }

    #[test]
    fn habit_reports_a_streak_from_passing_submits() {
        let (_dir, store, db) = fixture();
        // Two consecutive practice days ending today.
        for day in ["2026-07-09T09:00:00.000", "2026-07-10T09:00:00.000"] {
            record_attempt(
                &db,
                &AttemptRecord {
                    problem_id: "two-sum",
                    language: "python",
                    kind: "submit",
                    status: "pass",
                    runtime_ms: Some(10),
                    code: "x",
                    attempted_at: day,
                },
            )
            .unwrap();
        }
        let now = t("2026-07-10T09:00:00+00:00");
        let q = queue(&store, &db, now, now.date_naive()).unwrap();
        assert_eq!(q.habit.current_streak, 2);
        assert!(!q.habit.freeze_active);
    }
}
