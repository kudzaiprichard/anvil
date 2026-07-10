//! Retention / spaced-review types (Phase 6). These are *per-user state*
//! shapes (camelCase JSON, like `progress::LessonProgress`), the front end of
//! the FSRS review queue and the honest habit layer (COURSE_BLUEPRINT.md §7,
//! LESSON_COURSE_DESIGN.md §6). The scheduling engine that fills them wraps the
//! pure-Rust `rs-fsrs` crate in `services::review`; the domain layer stays free
//! of the algorithm crate so it unit-tests as plain serde.
//!
//! A solved Stage-1 problem enters `review_schedule` as a `new` card and comes
//! back on a widening, interleaved schedule; a failed re-solve (`again`) demotes
//! it — its interval collapses and its lapse counter bumps.

use serde::{Deserialize, Serialize};

/// The learner's self-assessed recall after a *cold* re-solve — the four FSRS
/// grades (COURSE_BLUEPRINT.md §8: retrieval practice, re-solve don't re-read).
/// Maps to `rs_fsrs::Rating` in `services::review`. `Again` is the failure grade
/// that demotes the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewRating {
    Again,
    Hard,
    Good,
    Easy,
}

/// Where a card sits in the FSRS state machine. Mirrors the
/// `review_schedule.state` CHECK constraint and `rs_fsrs::State`. The long-term
/// scheduler this course uses moves cards `new → review` and keeps them in
/// `review` (a lapse shortens the interval rather than dropping to `relearning`),
/// but the full set is modelled so the contract matches the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewCardState {
    New,
    Learning,
    Review,
    Relearning,
}

/// One problem that is due to be re-solved now, in the interleaved queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewItem {
    /// The LeetCode slug — the workspace opens this problem for a cold re-solve.
    pub problem_id: String,
    /// The unit whose pattern this problem belongs to — drives interleaving and
    /// the queue's pattern label.
    pub unit_id: String,
    pub state: ReviewCardState,
    /// When the card became due (RFC3339 UTC).
    pub due_at: String,
    /// Last time it was reviewed, or `None` for a card that has never been
    /// re-solved since it entered the queue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reviewed_at: Option<String>,
    /// How many times this problem has been failed (`again`) — the demotion
    /// counter.
    pub lapses: u32,
    /// Whole days the card is overdue (0 when it just came due).
    pub overdue_days: i64,
}

/// The honest habit layer (COURSE_BLUEPRINT.md §7): a streak that survives one
/// missed day via a freeze ("never miss twice"), never XP or leaderboards. The
/// streak counts calendar days with ≥1 passing submit; a single gap is bridged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitState {
    /// Consecutive practice days, forgiving one isolated missed day.
    pub current_streak: u32,
    /// Longest such run ever.
    pub best_streak: u32,
    /// A freeze is currently holding the streak together — the learner missed
    /// yesterday but the streak is still alive today. Miss again and it breaks.
    pub freeze_active: bool,
    /// Cards due to re-solve right now.
    pub due_today: u32,
    /// Cards already re-solved today.
    pub reviewed_today: u32,
}

/// The review page's payload: what's due now (interleaved), how many more are
/// scheduled for later, and the habit header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewQueue {
    /// Due cards, interleaved so consecutive items favour different patterns.
    pub due: Vec<ReviewItem>,
    /// Cards scheduled but not yet due — the "coming up" count.
    pub later_count: u32,
    pub habit: HabitState,
}

/// The result of recording one re-solve (`record_review`): the card's new state,
/// when it's next due, the fresh interval, and whether this counted as a
/// demotion (a failed re-solve).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutcome {
    pub problem_id: String,
    pub state: ReviewCardState,
    /// When the card is next due (RFC3339 UTC).
    pub due_at: String,
    /// Days until it's next due — the spacing interval FSRS just chose.
    pub interval_days: i64,
    pub lapses: u32,
    /// `true` when the learner failed the re-solve (`again`): the interval
    /// collapsed and the lapse counter bumped.
    pub demoted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rating_uses_lowercase_wire_values() {
        assert_eq!(serde_json::to_value(ReviewRating::Again).unwrap(), json!("again"));
        assert_eq!(serde_json::to_value(ReviewRating::Good).unwrap(), json!("good"));
        assert_eq!(
            serde_json::from_value::<ReviewRating>(json!("easy")).unwrap(),
            ReviewRating::Easy
        );
    }

    #[test]
    fn card_state_uses_kebab_case() {
        assert_eq!(serde_json::to_value(ReviewCardState::New).unwrap(), json!("new"));
        assert_eq!(
            serde_json::to_value(ReviewCardState::Relearning).unwrap(),
            json!("relearning")
        );
    }

    #[test]
    fn review_item_round_trips_and_omits_absent_last_reviewed() {
        let value = json!({
            "problemId": "two-sum",
            "unitId": "arrays-hashing",
            "state": "review",
            "dueAt": "2026-07-13T00:00:00+00:00",
            "lapses": 1,
            "overdueDays": 2
        });
        let parsed: ReviewItem = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(parsed.last_reviewed_at, None);
        assert_eq!(serde_json::to_value(&parsed).unwrap(), value);
    }

    #[test]
    fn review_queue_round_trips_camel_case() {
        let value = json!({
            "due": [{
                "problemId": "valid-palindrome",
                "unitId": "two-pointers",
                "state": "new",
                "dueAt": "2026-07-10T09:00:00+00:00",
                "lastReviewedAt": "2026-07-09T09:00:00+00:00",
                "lapses": 0,
                "overdueDays": 0
            }],
            "laterCount": 3,
            "habit": {
                "currentStreak": 4,
                "bestStreak": 9,
                "freezeActive": true,
                "dueToday": 1,
                "reviewedToday": 2
            }
        });
        let parsed: ReviewQueue = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }

    #[test]
    fn review_outcome_round_trips_camel_case() {
        let value = json!({
            "problemId": "two-sum",
            "state": "review",
            "dueAt": "2026-07-13T00:00:00+00:00",
            "intervalDays": 3,
            "lapses": 0,
            "demoted": false
        });
        let parsed: ReviewOutcome = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }
}
