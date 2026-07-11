//! Progress + dashboard types — camelCase like `src/lib/types.ts`
//! (`streakDays`, `needsReview`, …). Dashboard types are filled by task
//! 0009; `Progress` ships with 0008.

use serde::{Deserialize, Serialize};

use crate::domain::problem::{Pattern, ProblemStatus, ProblemSummary};

/// UI status mutations (`set_problem_status`). Commands don't trust
/// callers: `MarkMastered` is rejected unless the problem is solved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusAction {
    MarkMastered,
    UnmarkMastered,
    NeedsReview,
    ClearReview,
}

/// Where a lesson sits for this user (LESSON_COURSE_DESIGN.md §6.4). Mirrors
/// the `lesson_progress.status` CHECK constraint and the TS `LessonStatus`.
/// `NotStarted` is the absence of a stored row, surfaced by the UI only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LessonStatus {
    NotStarted,
    InProgress,
    Complete,
}

impl LessonStatus {
    /// Reads the stored `status` text back into the enum; an unrecognized
    /// value (only possible via out-of-band DB edits) reads as `NotStarted`.
    pub fn from_wire(s: &str) -> Self {
        match s {
            "in-progress" => Self::InProgress,
            "complete" => Self::Complete,
            _ => Self::NotStarted,
        }
    }
}

/// One lesson's stored progress row (`record_lesson_progress` /
/// `get_lesson_progress`). Timestamps are local-time ISO-8601 strings, like
/// every other stamp in the DB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonProgress {
    pub lesson_id: String,
    pub unit_id: String,
    pub status: LessonStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Per-problem user state for the workspace: bookmark icon, mastered flag,
/// and the code snapshot from the most recent run/submit so the editor can
/// restore where the user left off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemUserState {
    pub status: ProblemStatus,
    pub bookmarked: bool,
    pub mastered: bool,
    #[serde(rename = "lastCode", skip_serializing_if = "Option::is_none")]
    pub last_code: Option<String>,
    #[serde(rename = "lastLanguage", skip_serializing_if = "Option::is_none")]
    pub last_language: Option<crate::domain::run::Language>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub solved: u32,
    pub total: u32,
    pub attempted: u32,
    pub streak_days: u32,
    pub best_streak_days: u32,
    pub mastered: u32,
    pub needs_review: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDay {
    /// ISO date (yyyy-mm-dd).
    pub date: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternStat {
    pub pattern: Pattern,
    pub solved: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub progress: Progress,
    /// Daily solve counts for the heatmap, oldest first (~26 weeks).
    pub activity: Vec<ActivityDay>,
    /// Cumulative solved counts for the progress line, oldest first.
    pub cumulative: Vec<u32>,
    /// Month labels for the line chart axis: [start, mid].
    pub axis_labels: [String; 2],
    pub focus: Vec<PatternStat>,
    pub strong: Vec<PatternStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_problem: Option<ProblemSummary>,
    pub pattern_stats: Vec<PatternStat>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn progress_round_trips_with_camel_case_fields() {
        let value = json!({
            "solved": 3, "total": 12, "attempted": 5,
            "streakDays": 2, "bestStreakDays": 4,
            "mastered": 1, "needsReview": 1
        });
        let parsed: Progress = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }

    #[test]
    fn problem_user_state_round_trips_and_omits_absent_snapshot() {
        let value = json!({
            "status": "solved",
            "bookmarked": true,
            "mastered": false,
            "lastCode": "def solve(a, b):\n    return a + b",
            "lastLanguage": "python"
        });
        let parsed: ProblemUserState = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(&parsed).unwrap(), value);

        let fresh = ProblemUserState {
            status: ProblemStatus::Todo,
            bookmarked: false,
            mastered: false,
            last_code: None,
            last_language: None,
        };
        let v = serde_json::to_value(&fresh).unwrap();
        assert!(v.get("lastCode").is_none());
        assert!(v.get("lastLanguage").is_none());
    }

    #[test]
    fn dashboard_data_round_trips() {
        let value = json!({
            "progress": {
                "solved": 1, "total": 12, "attempted": 1,
                "streakDays": 1, "bestStreakDays": 1,
                "mastered": 0, "needsReview": 0
            },
            "activity": [{ "date": "2026-06-12", "count": 1 }],
            "cumulative": [0, 1],
            "axisLabels": ["Dec", "Mar"],
            "focus": [{ "pattern": "Stack", "solved": 0, "total": 2 }],
            "strong": [{ "pattern": "Graphs", "solved": 1, "total": 2 }],
            "continueProblem": {
                "id": "x", "number": 1, "title": "X", "pattern": "Stack",
                "difficulty": "Easy", "source": "built-in", "status": "in-progress"
            },
            "patternStats": [{ "pattern": "Stack", "solved": 0, "total": 2 }]
        });
        let parsed: DashboardData = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }
}
