//! Advanced-progression wire types (Phase 7). These are the *per-user state*
//! shapes (camelCase JSON, like `mastery`/`review`) for the four Phase-7
//! progression features layered on the proven engine:
//!   * **Diagnostic placement** — start the learner at their frontier by placing
//!     them out of units they already recognize ([`PlacementProbe`] /
//!     [`PlacementOutcome`]).
//!   * **Mixed capstone** — the Stage-7 unlabeled pool ([`CapstoneView`] /
//!     [`CapstoneOutcome`]); the pattern each problem belongs to is deliberately
//!     absent from the wire shape so the UI can never leak the label.
//!   * **Readiness signal** — one honest "are you interview-ready?" aggregate
//!     ([`Readiness`]).
//!
//! The engine that fills them lives in `services::advancement`.

use serde::{Deserialize, Serialize};

use crate::domain::quiz::QuizItem;

/// One capstone problem as the workspace sees it: a slug and whether it's been
/// cleared — **no pattern/unit label** (BLUEPRINT.md §4: the capstone is the
/// unlabeled recognition exam).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapstoneProblemView {
    pub problem_id: String,
    pub solved: bool,
}

/// The capstone as shown on the course page: its config, the unlabeled problem
/// pool, and how far through it the learner is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapstoneView {
    pub id: String,
    pub title: String,
    pub pass_count: u32,
    pub timer_target_min: u32,
    pub passed_count: u32,
    pub total: u32,
    /// `true` once `passed_count >= pass_count`.
    pub met: bool,
    /// `true` once every unit is mastered — the capstone only *counts* toward
    /// readiness when the whole ladder is climbed, though it can be attempted
    /// early for practice.
    pub unlocked: bool,
    pub problems: Vec<CapstoneProblemView>,
}

/// The result of one capstone attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapstoneOutcome {
    /// `false` when the learner peeked (hint/solution) — a peeked attempt never
    /// counts, exactly like a gate problem (§6).
    pub counted: bool,
    pub passed_count: u32,
    pub total: u32,
    pub met: bool,
}

/// The diagnostic placement probe: a handful of *unlabeled* pattern-picker items
/// (drawn from the interleaved pool) whose correct answers place the learner out
/// of units they already recognize. `unit_ids` names, in probe order, which
/// units the items test — the UI shows only the prompts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementProbe {
    pub items: Vec<QuizItem>,
    /// The distinct units this probe can place the learner out of.
    pub unit_ids: Vec<String>,
}

/// The result of submitting the placement probe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementOutcome {
    /// Units the learner was placed out of (recognized, and all prereqs also
    /// recognized) — marked mastered-via-placement so their dependents unlock.
    pub placed: Vec<String>,
    /// Units now unlocked (the learner's new frontier) as a result.
    pub frontier: Vec<String>,
}

/// The honest course-readiness aggregate (BLUEPRINT.md §7 "readiness signal").
/// No single number is gospel — it combines how much of the ladder is mastered
/// with whether the unlabeled capstone has been cleared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub units_total: u32,
    pub units_mastered: u32,
    pub capstone_total: u32,
    pub capstone_solved: u32,
    pub capstone_met: bool,
    /// 0–100 overall completion: ladder mastery weighted with capstone clears.
    pub percent: u32,
    /// `true` only when every unit is mastered *and* the capstone is met — the
    /// operational "can solve unfamiliar problems alone."
    pub ready: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn readiness_round_trips_camel_case() {
        let value = json!({
            "unitsTotal": 19,
            "unitsMastered": 3,
            "capstoneTotal": 16,
            "capstoneSolved": 0,
            "capstoneMet": false,
            "percent": 15,
            "ready": false
        });
        let parsed: Readiness = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }

    #[test]
    fn capstone_view_hides_the_pattern() {
        // The wire shape carries no `unit`/`pattern` key by construction.
        let v = CapstoneView {
            id: "mixed-capstone".into(),
            title: "Mixed Capstone".into(),
            pass_count: 4,
            timer_target_min: 40,
            passed_count: 0,
            total: 1,
            met: false,
            unlocked: false,
            problems: vec![CapstoneProblemView {
                problem_id: "two-sum".into(),
                solved: false,
            }],
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(
            !json.contains("unit"),
            "capstone wire leaked a unit label: {json}"
        );
        assert!(
            !json.contains("pattern"),
            "capstone wire leaked a pattern: {json}"
        );
    }
}
