//! Progression + mastery-gate engine (Phase 3). This is the *behavior* half of
//! the course — the "hard-coded" side of LESSON_COURSE_DESIGN.md §6: unlock
//! rules and mastery evaluation live in compiled Rust, never in editable data.
//!
//! Two responsibilities:
//!   * **Progression view** — for every unit, is it `locked`/`unlocked`/
//!     `mastered`, how far through its lessons, and how close to its gate.
//!   * **Gate evaluation** — a learner submitted a passing gate solve; decide
//!     whether it counts (COURSE_BLUEPRINT.md §6: hint-free & no-peek, pass = N
//!     incl. >=1 novel), persist it, and unlock the next unit(s) on mastery.
//!
//! The linear Stage-1 chain falls out of the prereq DAG: a unit is `unlocked`
//! only once *all* its prereqs are `mastered`, so Two Pointers stays locked
//! until Arrays & Hashing's gate is passed, and Sliding Window until both.

use std::collections::HashSet;

use crate::domain::mastery::{GateOutcome, UnitGateState, UnitProgress, UnitStatus};
use crate::domain::unit::{ProblemRole, Unit};
use crate::error::{AppError, AppResult};
use crate::services::curriculum::CurriculumStore;
use crate::services::db::{self, lesson_progress, mastery, Db};

/// Tallies a unit's gate progress from the manifest + the set of gate slugs the
/// learner has cleared hint-free. Novelty is read from the manifest (the source
/// of truth), not the stored row, so re-tagging a problem re-derives correctly.
fn gate_state(unit: &Unit, solved: &HashSet<String>) -> UnitGateState {
    let gate_problems: Vec<&_> = unit
        .problems
        .iter()
        .filter(|p| p.role == ProblemRole::Gate)
        .collect();

    let mut passed_count = 0;
    let mut passed_novel = 0;
    let mut solved_slugs = Vec::new();
    for gp in &gate_problems {
        if solved.contains(&gp.slug) {
            passed_count += 1;
            if gp.novel {
                passed_novel += 1;
            }
            solved_slugs.push(gp.slug.clone());
        }
    }

    let met =
        passed_count >= unit.gate.pass_count && (!unit.gate.require_novel || passed_novel >= 1);

    UnitGateState {
        pass_count: unit.gate.pass_count,
        require_novel: unit.gate.require_novel,
        timer_target_min: unit.gate.timer_target_min,
        passed_count,
        passed_novel,
        solved_slugs,
        total: gate_problems.len() as u32,
        met,
    }
}

/// A unit's lock state + what still blocks it, given the set of mastered units.
/// A unit is `unlocked` only once every prereq is mastered (linear chain for
/// Stage 1; general DAG for later stages).
fn status_for(unit: &Unit, mastered: &HashSet<String>) -> (UnitStatus, Vec<String>) {
    if mastered.contains(&unit.id) {
        return (UnitStatus::Mastered, Vec::new());
    }
    let blocked_by: Vec<String> = unit
        .prereqs
        .iter()
        .filter(|p| !mastered.contains(*p))
        .cloned()
        .collect();
    if blocked_by.is_empty() {
        (UnitStatus::Unlocked, Vec::new())
    } else {
        (UnitStatus::Locked, blocked_by)
    }
}

/// Builds one unit's progression snapshot from the already-fetched mastered set,
/// its hint-free gate solves, and its lesson-completion counts.
fn unit_progress(
    unit: &Unit,
    mastered: &HashSet<String>,
    solved: &HashSet<String>,
    lessons_complete: u32,
) -> UnitProgress {
    let (status, blocked_by) = status_for(unit, mastered);
    UnitProgress {
        unit_id: unit.id.clone(),
        status,
        lessons_total: unit.lessons.len() as u32,
        lessons_complete,
        gate: gate_state(unit, solved),
        blocked_by,
    }
}

/// Progression snapshot for every unit, in curriculum (stage) order — the
/// course overview + unit views read this to lock/badge units.
pub fn progression(store: &CurriculumStore, db: &Db) -> AppResult<Vec<UnitProgress>> {
    let mastered = mastery::mastered_units(db)?;
    let progress = lesson_progress::list(db)?;

    let mut out = Vec::new();
    for uid in store.curriculum().unit_ids() {
        let Some(unit) = store.get_unit(uid) else {
            continue; // loader already guarantees this, but stay defensive
        };
        let solved: HashSet<String> = mastery::gate_solves(db, &unit.id)?
            .into_iter()
            .map(|(slug, _)| slug)
            .collect();
        let lessons_complete = unit
            .lessons
            .iter()
            .filter(|lid| {
                progress.iter().any(|p| {
                    &p.lesson_id == *lid
                        && p.status == crate::domain::progress::LessonStatus::Complete
                })
            })
            .count() as u32;
        out.push(unit_progress(unit, &mastered, &solved, lessons_complete));
    }
    Ok(out)
}

/// Evaluates a passing gate solve (COURSE_BLUEPRINT.md §6). Rejects a solve for
/// a problem that isn't one of the unit's `role:gate` problems, and refuses to
/// grade a unit that is still locked. A peeked/hinted attempt (`used_help`) is
/// recorded as an attempt but never counts toward mastery. When the counted
/// solve tips the unit over its threshold, the unit is marked mastered and any
/// units it just unblocked are reported back.
pub fn evaluate_gate(
    store: &CurriculumStore,
    db: &Db,
    unit_id: &str,
    problem_id: &str,
    used_help: bool,
    now: &str,
) -> AppResult<GateOutcome> {
    let unit = store
        .get_unit(unit_id)
        .ok_or_else(|| AppError::NotFound(format!("Unit not found: {unit_id}")))?;

    let gate_problem = unit
        .problems
        .iter()
        .find(|p| p.slug == problem_id && p.role == ProblemRole::Gate)
        .ok_or_else(|| {
            AppError::Validation(format!(
                "'{problem_id}' is not a gate problem of unit '{unit_id}'"
            ))
        })?;

    let mastered = mastery::mastered_units(db)?;
    let already_mastered = mastered.contains(unit_id);
    let (status, _) = status_for(unit, &mastered);
    if status == UnitStatus::Locked {
        return Err(AppError::Validation(format!(
            "unit '{unit_id}' is locked — pass its prerequisites first"
        )));
    }

    // Every attempt bumps the lifetime counter, counted or not.
    mastery::bump_gate_attempts(db, unit_id)?;

    // A peeked/hinted attempt never counts (§6). Report the unchanged tally.
    if used_help {
        let solved: HashSet<String> = mastery::gate_solves(db, unit_id)?
            .into_iter()
            .map(|(slug, _)| slug)
            .collect();
        return Ok(GateOutcome {
            counted: false,
            unit_mastered: false,
            already_mastered,
            gate: gate_state(unit, &solved),
            unlocked: Vec::new(),
        });
    }

    mastery::record_gate_solve(db, unit_id, problem_id, gate_problem.novel, now)?;
    let solved: HashSet<String> = mastery::gate_solves(db, unit_id)?
        .into_iter()
        .map(|(slug, _)| slug)
        .collect();
    let gate = gate_state(unit, &solved);

    let mut unit_mastered = false;
    let mut unlocked = Vec::new();
    if gate.met && !already_mastered {
        mastery::mark_mastered(db, unit_id, gate.passed_count, gate.passed_novel, now)?;
        unit_mastered = true;

        // Which units did mastering this one just unblock? A unit newly unlocks
        // if all its prereqs are mastered *now* but weren't before.
        let mut after = mastered.clone();
        after.insert(unit_id.to_string());
        for uid in store.curriculum().unit_ids() {
            let Some(candidate) = store.get_unit(uid) else {
                continue;
            };
            if after.contains(&candidate.id) {
                continue;
            }
            let now_unlocked = candidate.prereqs.iter().all(|p| after.contains(p));
            let was_unlocked = candidate.prereqs.iter().all(|p| mastered.contains(p));
            if now_unlocked && !was_unlocked {
                unlocked.push(candidate.id.clone());
            }
        }
    }

    Ok(GateOutcome {
        counted: true,
        unit_mastered,
        already_mastered,
        gate,
        unlocked,
    })
}

/// Convenience for callers that only have a `&Db` and want a local timestamp.
pub fn now() -> String {
    db::now_local_iso()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

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

    fn status_of<'a>(rows: &'a [UnitProgress], id: &str) -> &'a UnitProgress {
        rows.iter().find(|r| r.unit_id == id).expect("unit present")
    }

    /// Masters Big-O (the stage-0 root), which every stage-1 unit now sits
    /// behind. Its gate needs one novel solve — `maximum-subarray`.
    fn master_big_o(store: &CurriculumStore, db: &Db) {
        evaluate_gate(store, db, "big-o", "maximum-subarray", false, "B1").unwrap();
    }

    #[test]
    fn fresh_progression_locks_everything_but_the_root() {
        let (_dir, store, db) = fixture();
        let rows = progression(&store, &db).unwrap();
        // Big-O is the single stage-0 root; everything else waits on it.
        assert_eq!(status_of(&rows, "big-o").status, UnitStatus::Unlocked);
        assert_eq!(
            status_of(&rows, "arrays-hashing").status,
            UnitStatus::Locked
        );
        assert_eq!(status_of(&rows, "two-pointers").status, UnitStatus::Locked);
        assert_eq!(status_of(&rows, "arrays-hashing").blocked_by, vec!["big-o"]);
    }

    #[test]
    fn locked_unit_cannot_be_gated() {
        let (_dir, store, db) = fixture();
        // two-pointers is locked until arrays-hashing is mastered.
        let err = evaluate_gate(
            &store,
            &db,
            "two-pointers",
            "trapping-rain-water",
            false,
            "T1",
        )
        .unwrap_err();
        assert!(err.to_string().contains("locked"), "{err}");
    }

    #[test]
    fn non_gate_problem_is_rejected() {
        let (_dir, store, db) = fixture();
        // two-sum is a worked example of arrays-hashing, not a gate problem.
        let err = evaluate_gate(&store, &db, "arrays-hashing", "two-sum", false, "T1").unwrap_err();
        assert!(err.to_string().contains("not a gate problem"), "{err}");
    }

    #[test]
    fn peeked_attempt_does_not_count() {
        let (_dir, store, db) = fixture();
        master_big_o(&store, &db);
        let out = evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "longest-consecutive-sequence",
            true, // used help
            "T1",
        )
        .unwrap();
        assert!(!out.counted);
        assert_eq!(out.gate.passed_count, 0);
        assert!(!out.unit_mastered);
        // Nothing persisted → the unit is still just unlocked.
        let rows = progression(&store, &db).unwrap();
        assert_eq!(
            status_of(&rows, "arrays-hashing").status,
            UnitStatus::Unlocked
        );
        assert_eq!(status_of(&rows, "two-pointers").status, UnitStatus::Locked);
    }

    #[test]
    fn passing_the_gate_masters_the_unit_and_unlocks_parallel_branches() {
        let (_dir, store, db) = fixture();
        master_big_o(&store, &db);
        // arrays-hashing needs pass_count=2 incl. >=1 novel; both gate problems
        // are novel. One solve isn't enough.
        let out = evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "longest-consecutive-sequence",
            false,
            "T1",
        )
        .unwrap();
        assert!(out.counted);
        assert_eq!(out.gate.passed_count, 1);
        assert!(!out.gate.met);
        assert!(!out.unit_mastered);
        assert!(out.unlocked.is_empty());

        // Second distinct gate solve tips it over → mastered. Mastering
        // arrays-hashing unlocks EVERY unit whose prereqs are now all met — this
        // is DAG parallel unlock across branches: two-pointers (s1), stack (s2),
        // and binary-search (s2, since big-o is already mastered) all open at once.
        let out = evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "subarray-sum-equals-k",
            false,
            "T2",
        )
        .unwrap();
        assert!(out.counted);
        assert!(out.gate.met);
        assert!(out.unit_mastered);
        assert_eq!(out.unlocked, vec!["two-pointers", "stack", "binary-search"]);

        let rows = progression(&store, &db).unwrap();
        assert_eq!(
            status_of(&rows, "arrays-hashing").status,
            UnitStatus::Mastered
        );
        assert_eq!(
            status_of(&rows, "two-pointers").status,
            UnitStatus::Unlocked
        );
        assert_eq!(status_of(&rows, "stack").status, UnitStatus::Unlocked);
        assert_eq!(
            status_of(&rows, "binary-search").status,
            UnitStatus::Unlocked
        );
        // sliding-window needs BOTH arrays-hashing and two-pointers.
        assert_eq!(
            status_of(&rows, "sliding-window").status,
            UnitStatus::Locked
        );
        // linked-list needs arrays-hashing AND two-pointers — still locked.
        assert_eq!(status_of(&rows, "linked-list").status, UnitStatus::Locked);
    }

    #[test]
    fn resolving_the_same_gate_problem_twice_does_not_double_count() {
        let (_dir, store, db) = fixture();
        master_big_o(&store, &db);
        evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "longest-consecutive-sequence",
            false,
            "T1",
        )
        .unwrap();
        let out = evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "longest-consecutive-sequence",
            false,
            "T2",
        )
        .unwrap();
        assert_eq!(out.gate.passed_count, 1);
        assert!(!out.gate.met);
    }

    #[test]
    fn stage_one_chain_unlocks_in_order() {
        let (_dir, store, db) = fixture();
        master_big_o(&store, &db);
        // Master arrays-hashing.
        evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "longest-consecutive-sequence",
            false,
            "T1",
        )
        .unwrap();
        evaluate_gate(
            &store,
            &db,
            "arrays-hashing",
            "subarray-sum-equals-k",
            false,
            "T2",
        )
        .unwrap();
        // Now two-pointers is gateable; its gate needs one novel solve. Mastering
        // it opens sliding-window (s1) and linked-list (s2) together.
        let out = evaluate_gate(
            &store,
            &db,
            "two-pointers",
            "trapping-rain-water",
            false,
            "T3",
        )
        .unwrap();
        assert!(out.unit_mastered);
        assert_eq!(out.unlocked, vec!["sliding-window", "linked-list"]);
        let rows = progression(&store, &db).unwrap();
        assert_eq!(
            status_of(&rows, "sliding-window").status,
            UnitStatus::Unlocked
        );
    }
}
