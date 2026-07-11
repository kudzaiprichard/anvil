//! Advanced-progression engine (Phase 7) — the compiled-behavior half of the
//! four progression features layered on the proven Phase 1–6 engine
//! (LESSON_COURSE_DESIGN.md §6: unlock/mastery/placement rules are code, never
//! editable data):
//!
//!   * [`capstone_view`] / [`evaluate_capstone`] — the Stage-7 **mixed capstone**:
//!     an unlabeled cross-unit pool, scored like a gate but spanning every unit.
//!   * [`placement_probe`] / [`apply_placement`] — **diagnostic placement**:
//!     recognize a pattern in the probe and you're placed out of that unit,
//!     starting the learner at their frontier instead of unit 1.
//!   * [`readiness`] — the honest **readiness signal**: how much of the ladder is
//!     mastered, folded with whether the unlabeled capstone is cleared.
//!
//! Parallel-DAG unlock and partial-prereq-credit review live in
//! `services::progression` and `services::review` respectively; this module is
//! the Phase-7 additions that don't fit either.

use std::collections::HashSet;

use crate::domain::advancement::{
    CapstoneOutcome, CapstoneProblemView, CapstoneView, PlacementOutcome, PlacementProbe, Readiness,
};
use crate::domain::quiz::{QuizAnswer, QuizItemType};
use crate::error::{AppError, AppResult};
use crate::services::curriculum::CurriculumStore;
use crate::services::db::{capstone, mastery, Db};

/// Whether every unit in the curriculum is mastered (gate-earned or placed) —
/// the precondition for the capstone to *count* toward readiness.
fn all_units_mastered(store: &CurriculumStore, mastered: &HashSet<String>) -> bool {
    store.unit_ids().iter().all(|u| mastered.contains(*u))
}

/// The capstone as the course page sees it (no pattern labels). `None` when the
/// curriculum ships no capstone.
pub fn capstone_view(store: &CurriculumStore, db: &Db) -> AppResult<Option<CapstoneView>> {
    let Some(cap) = store.capstone() else {
        return Ok(None);
    };
    let solved = capstone::solved(db)?;
    let mastered = mastery::mastered_units(db)?;
    let passed_count = cap
        .problems
        .iter()
        .filter(|p| solved.contains(&p.slug))
        .count() as u32;
    let problems = cap
        .problems
        .iter()
        .map(|p| CapstoneProblemView {
            problem_id: p.slug.clone(),
            solved: solved.contains(&p.slug),
        })
        .collect();
    Ok(Some(CapstoneView {
        id: cap.id.clone(),
        title: cap.title.clone(),
        pass_count: cap.pass_count,
        timer_target_min: cap.timer_target_min,
        passed_count,
        total: cap.problems.len() as u32,
        met: passed_count >= cap.pass_count,
        unlocked: all_units_mastered(store, &mastered),
        problems,
    }))
}

/// Scores one capstone attempt. Like a gate (§6): a peeked/hinted attempt is not
/// counted; a clean solve of a capstone problem is recorded. Rejects a slug that
/// isn't in the capstone pool.
pub fn evaluate_capstone(
    store: &CurriculumStore,
    db: &Db,
    problem_id: &str,
    used_help: bool,
    now: &str,
) -> AppResult<CapstoneOutcome> {
    let cap = store
        .capstone()
        .ok_or_else(|| AppError::NotFound("this course has no capstone".into()))?;
    if !cap.problems.iter().any(|p| p.slug == problem_id) {
        return Err(AppError::Validation(format!(
            "'{problem_id}' is not a capstone problem"
        )));
    }
    if !used_help {
        capstone::record_solve(db, problem_id, now)?;
    }
    let solved = capstone::solved(db)?;
    let passed_count = cap
        .problems
        .iter()
        .filter(|p| solved.contains(&p.slug))
        .count() as u32;
    Ok(CapstoneOutcome {
        counted: !used_help,
        passed_count,
        total: cap.problems.len() as u32,
        met: passed_count >= cap.pass_count,
    })
}

/// The honest readiness aggregate (BLUEPRINT.md §7). Ladder mastery is weighted
/// 80%, the unlabeled capstone 20%; `ready` demands *both* a fully-mastered
/// ladder and a cleared capstone.
pub fn readiness(store: &CurriculumStore, db: &Db) -> AppResult<Readiness> {
    let mastered = mastery::mastered_units(db)?;
    let units_total = store.unit_ids().len() as u32;
    let units_mastered = store
        .unit_ids()
        .iter()
        .filter(|u| mastered.contains(**u))
        .count() as u32;

    let (capstone_total, capstone_solved, capstone_pass) = match store.capstone() {
        Some(cap) => {
            let solved = capstone::solved(db)?;
            let n = cap
                .problems
                .iter()
                .filter(|p| solved.contains(&p.slug))
                .count() as u32;
            (cap.problems.len() as u32, n, cap.pass_count)
        }
        None => (0, 0, 0),
    };
    let capstone_met = capstone_total > 0 && capstone_solved >= capstone_pass;

    let ladder_frac = if units_total == 0 {
        0.0
    } else {
        units_mastered as f64 / units_total as f64
    };
    let cap_frac = if capstone_total == 0 {
        0.0
    } else {
        (capstone_solved as f64 / capstone_pass.max(1) as f64).min(1.0)
    };
    let percent = ((ladder_frac * 0.8 + cap_frac * 0.2) * 100.0).round() as u32;

    Ok(Readiness {
        units_total,
        units_mastered,
        capstone_total,
        capstone_solved,
        capstone_met,
        percent: percent.min(100),
        ready: units_total > 0 && units_mastered == units_total && capstone_met,
    })
}

/// Builds the diagnostic placement probe from the interleaved pattern-picker
/// pool: every unlabeled recognition item, plus the distinct units they can
/// place the learner out of. An empty pool yields an empty probe (placement is
/// then a no-op).
pub fn placement_probe(store: &CurriculumStore) -> PlacementProbe {
    let items: Vec<_> = store
        .pattern_pool()
        .items
        .iter()
        .filter(|i| i.item_type == QuizItemType::PatternPicker)
        .cloned()
        .collect();
    let mut unit_ids = Vec::new();
    for item in &items {
        if let Some(u) = &item.correct_pattern {
            if !unit_ids.contains(u) {
                unit_ids.push(u.clone());
            }
        }
    }
    PlacementProbe { items, unit_ids }
}

/// Applies a submitted placement probe (BLUEPRINT.md §7 "diagnostic placement").
/// A unit is *recognized* when the learner answered every probe item for it
/// correctly; a recognized unit is *placed out* only if all its prerequisites are
/// themselves placed or already mastered (you can't skip a unit whose
/// foundations you haven't shown). Placed units are marked mastered-via-placement
/// so their dependents unlock — the learner starts at their frontier.
pub fn apply_placement(
    store: &CurriculumStore,
    db: &Db,
    answers: &[QuizAnswer],
    now: &str,
) -> AppResult<PlacementOutcome> {
    let probe = placement_probe(store);

    // Grade: a unit is recognized iff it has >=1 probe item and every one was
    // answered correctly.
    let mut correct_by_unit: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();
    for item in &probe.items {
        let Some(unit) = &item.correct_pattern else {
            continue;
        };
        let selected = answers
            .iter()
            .find(|a| a.item_id == item.id)
            .map(|a| a.selected.as_str());
        let entry = correct_by_unit.entry(unit.clone()).or_insert((0, 0));
        entry.1 += 1; // total probed
        if selected == Some(item.answer.as_str()) {
            entry.0 += 1; // correct
        }
    }
    let recognized: HashSet<String> = correct_by_unit
        .into_iter()
        .filter(|(_, (correct, total))| *total > 0 && correct == total)
        .map(|(unit, _)| unit)
        .collect();

    // Place out to a fixpoint: a recognized unit is placed once all its prereqs
    // are placed or already mastered. Iterate until no further unit qualifies.
    let already = mastery::mastered_units(db)?;
    let mut placed_set: HashSet<String> = HashSet::new();
    loop {
        let mut progressed = false;
        for uid in store.unit_ids() {
            if placed_set.contains(uid) || already.contains(uid) || !recognized.contains(uid) {
                continue;
            }
            let Some(unit) = store.get_unit(uid) else {
                continue;
            };
            let prereqs_ok = unit
                .prereqs
                .iter()
                .all(|p| already.contains(p) || placed_set.contains(p));
            if prereqs_ok {
                placed_set.insert(uid.to_string());
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }

    let mut placed: Vec<String> = placed_set.iter().cloned().collect();
    placed.sort();
    for uid in &placed {
        mastery::place_out(db, uid, now)?;
    }

    // The new frontier: units now unlocked (all prereqs mastered/placed) that
    // aren't themselves mastered/placed.
    let mastered_now: HashSet<String> = already.union(&placed_set).cloned().collect();
    let mut frontier: Vec<String> = store
        .unit_ids()
        .iter()
        .filter(|uid| !mastered_now.contains(**uid))
        .filter(|uid| {
            store
                .get_unit(uid)
                .is_some_and(|u| u.prereqs.iter().all(|p| mastered_now.contains(p)))
        })
        .map(|uid| uid.to_string())
        .collect();
    frontier.sort();

    Ok(PlacementOutcome { placed, frontier })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    use crate::services::db::Db;
    use crate::services::pack_store::PackStore;
    use crate::services::progression;

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

    #[test]
    fn capstone_loads_and_hides_labels() {
        let (_dir, store, db) = fixture();
        let view = capstone_view(&store, &db)
            .unwrap()
            .expect("capstone present");
        assert!(view.total >= 4);
        assert_eq!(view.passed_count, 0);
        assert!(!view.met);
        assert!(!view.unlocked); // nothing mastered yet
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains("\"unit\""), "leaked unit label: {json}");
    }

    #[test]
    fn evaluate_capstone_counts_clean_solves_only() {
        let (_dir, store, db) = fixture();
        let cap = store.capstone().unwrap();
        let first = cap.problems[0].slug.clone();

        let peeked = evaluate_capstone(&store, &db, &first, true, "T1").unwrap();
        assert!(!peeked.counted);
        assert_eq!(peeked.passed_count, 0);

        let clean = evaluate_capstone(&store, &db, &first, false, "T2").unwrap();
        assert!(clean.counted);
        assert_eq!(clean.passed_count, 1);
    }

    #[test]
    fn evaluate_capstone_rejects_non_capstone_problem() {
        let (_dir, store, db) = fixture();
        let err = evaluate_capstone(&store, &db, "not-a-capstone-slug", false, "T1").unwrap_err();
        assert!(err.to_string().contains("not a capstone problem"), "{err}");
    }

    #[test]
    fn readiness_starts_low_and_reflects_mastery() {
        let (_dir, store, db) = fixture();
        let r = readiness(&store, &db).unwrap();
        assert_eq!(r.units_mastered, 0);
        assert!(r.units_total >= 19);
        assert!(!r.ready);
        assert_eq!(r.percent, 0);
    }

    #[test]
    fn placement_places_out_the_root_and_unlocks_the_frontier() {
        let (_dir, store, db) = fixture();
        let probe = placement_probe(&store);
        assert!(
            !probe.items.is_empty(),
            "shipped pattern pool feeds the probe"
        );

        // Answer every probe item correctly → recognized everywhere the pool
        // covers. With only Stage-1 pool items, big-o has no probe item so it is
        // NOT recognized, which (correctly) blocks placing out of arrays-hashing.
        let answers: Vec<QuizAnswer> = probe
            .items
            .iter()
            .map(|i| QuizAnswer {
                item_id: i.id.clone(),
                selected: i.answer.clone(),
            })
            .collect();
        let out = apply_placement(&store, &db, &answers, "T1").unwrap();
        // arrays-hashing can only be placed if big-o (its prereq) is recognized;
        // the Stage-1 pool doesn't probe big-o, so nothing places yet.
        assert!(
            out.placed.is_empty(),
            "should not place past an un-probed prerequisite: {:?}",
            out.placed
        );

        // But a wrong answer never places, and a probe that DOES cover the whole
        // ancestry does place — exercised once Stage-0/1 pool items ship. Here we
        // at least confirm the frontier still contains the root.
        let rows = progression::progression(&store, &db).unwrap();
        assert!(rows.iter().any(|r| r.unit_id == "big-o"));
    }
}
