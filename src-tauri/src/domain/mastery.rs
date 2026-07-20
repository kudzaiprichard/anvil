//! Progression + mastery-gate types (Phase 3). These are *per-user state*
//! shapes (camelCase JSON, like `progress::LessonProgress`), distinct from the
//! bundled content schemas in `unit`/`curriculum` (snake_case). The engine that
//! fills them lives in `services::progression`; the enforcement rules
//! (locked→unlocked→mastered, "pass = N incl. >=1 novel, hint-free").

use serde::{Deserialize, Serialize};

/// Where a unit sits for this user. `locked` until every prereq is `mastered`;
/// `mastered` once its gate is passed. Mirrors the `unit_mastery.status` CHECK
/// and the TS `UnitStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitStatus {
    Locked,
    Unlocked,
    Mastered,
}

/// How close the learner is to clearing a unit's mastery gate. Only hint-free,
/// no-peek solves of the unit's `role:gate` problems are tallied here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitGateState {
    /// How many gate problems must be solved hint-free (from the unit's gate
    /// config).
    pub pass_count: u32,
    /// Whether at least one of those solves must be a `novel` problem.
    pub require_novel: bool,
    /// Soft per-problem target in minutes — shown, never enforced (§6).
    pub timer_target_min: u32,
    /// Distinct gate problems solved hint-free so far.
    pub passed_count: u32,
    /// Of those, how many were tagged `novel`.
    pub passed_novel: u32,
    /// The gate problem slugs the learner has cleared hint-free.
    pub solved_slugs: Vec<String>,
    /// Total gate problems available in this unit's pool.
    pub total: u32,
    /// `true` once `passed_count >= pass_count` and the novel requirement is met.
    pub met: bool,
}

/// A unit's full progression snapshot for the course/unit views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitProgress {
    pub unit_id: String,
    pub status: UnitStatus,
    pub lessons_total: u32,
    pub lessons_complete: u32,
    pub gate: UnitGateState,
    /// Prereq unit ids not yet mastered — what the learner must clear to
    /// unlock this unit (empty when already unlocked/mastered).
    pub blocked_by: Vec<String>,
}

/// The result of a single gate attempt (`evaluate_gate`). Tells the UI whether
/// the solve counted, whether it tipped the unit to mastered, and which units
/// that just unlocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateOutcome {
    /// `false` when the learner used a hint/solution — a peeked attempt never
    /// counts toward mastery (§6).
    pub counted: bool,
    /// This solve is what pushed the unit over its gate threshold.
    pub unit_mastered: bool,
    /// The unit was already mastered before this attempt.
    pub already_mastered: bool,
    /// The updated gate tally after this attempt.
    pub gate: UnitGateState,
    /// Unit ids that transitioned locked→unlocked because of this pass.
    pub unlocked: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unit_status_uses_kebab_case() {
        assert_eq!(
            serde_json::to_value(UnitStatus::Locked).unwrap(),
            json!("locked")
        );
        assert_eq!(
            serde_json::to_value(UnitStatus::Unlocked).unwrap(),
            json!("unlocked")
        );
        assert_eq!(
            serde_json::to_value(UnitStatus::Mastered).unwrap(),
            json!("mastered")
        );
    }

    #[test]
    fn unit_progress_round_trips_camel_case() {
        let value = json!({
            "unitId": "arrays-hashing",
            "status": "unlocked",
            "lessonsTotal": 1,
            "lessonsComplete": 0,
            "gate": {
                "passCount": 2,
                "requireNovel": true,
                "timerTargetMin": 25,
                "passedCount": 1,
                "passedNovel": 1,
                "solvedSlugs": ["longest-consecutive-sequence"],
                "total": 2,
                "met": false
            },
            "blockedBy": []
        });
        let parsed: UnitProgress = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }

    #[test]
    fn gate_outcome_round_trips_camel_case() {
        let value = json!({
            "counted": true,
            "unitMastered": true,
            "alreadyMastered": false,
            "gate": {
                "passCount": 1,
                "requireNovel": true,
                "timerTargetMin": 25,
                "passedCount": 1,
                "passedNovel": 1,
                "solvedSlugs": ["trapping-rain-water"],
                "total": 1,
                "met": true
            },
            "unlocked": ["sliding-window"]
        });
        let parsed: GateOutcome = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }
}
