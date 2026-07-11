//! Progress/dashboard aggregation — pure functions over query results so
//! every rule unit-tests with fixture rows (no DB, no clock). Dashboard
//! aggregations are extended in task 0009.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::domain::problem::{Pattern, PATTERNS};
use crate::domain::progress::{ActivityDay, PatternStat, Progress};
use crate::services::db::problem_state::ProblemStateRow;

/// The dashboard window: 26 weeks ending today.
const WINDOW_DAYS: u64 = 26 * 7;

/// Streak rules (0008 §6): consecutive calendar days (local time) with ≥1
/// passing submit. Current streak counts back from today — or yesterday, so
/// a streak isn't "broken" before the user practiced today. Returns
/// `(current, best)`.
pub fn streaks(pass_days: &[NaiveDate], today: NaiveDate) -> (u32, u32) {
    let mut days = pass_days.to_vec();
    days.sort();
    days.dedup();

    let mut best = 0u32;
    let mut run = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for day in &days {
        run = match prev {
            Some(p) if *day == p + chrono::Days::new(1) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(*day);
    }

    let current = match days.last() {
        Some(&last) if last == today || last == today - chrono::Days::new(1) => run,
        _ => 0,
    };
    (current, best)
}

/// A streak that forgives one isolated missed day — the honest habit layer's
/// "never miss twice" (COURSE_BLUEPRINT.md §7). `freeze_active` means the streak
/// is currently held together by a freeze (yesterday was missed) and will break
/// on a second consecutive miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreezeStreak {
    pub current: u32,
    pub best: u32,
    pub freeze_active: bool,
}

/// Streak with freezes (COURSE_BLUEPRINT.md §7 habit layer): consecutive
/// calendar days with ≥1 passing submit, where a *single* missed day between two
/// practice days is bridged by a freeze — but two missed days in a row break it.
/// Each freeze forgives at most the one gap it spans. Unlike [`streaks`] (strict
/// consecutive days), this is the number the review page headlines. `current`
/// counts only real practice days; the bridged day doesn't inflate it.
pub fn streak_with_freezes(pass_days: &[NaiveDate], today: NaiveDate) -> FreezeStreak {
    let mut days = pass_days.to_vec();
    days.sort();
    days.dedup();

    let mut best = 0u32;
    let mut run = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for day in &days {
        let gap = prev.map(|p| (*day - p).num_days());
        run = match gap {
            // Consecutive, or one forgiven missed day between practice days.
            Some(1) | Some(2) => run + 1,
            // ≥2 missed days in a row (gap ≥ 3) — or the first day.
            _ => 1,
        };
        best = best.max(run);
        prev = Some(*day);
    }

    let (current, freeze_active) = match days.last() {
        // Alive if the last practice day is today, yesterday, or (freeze) two
        // days ago. Three-plus days idle means a second consecutive miss → dead.
        Some(&last) => {
            let idle = (today - last).num_days();
            if (0..=2).contains(&idle) {
                (run, idle == 2)
            } else {
                (0, false)
            }
        }
        None => (0, false),
    };
    FreezeStreak {
        current,
        best,
        freeze_active,
    }
}

/// Counts for `get_progress` from the state table + the solve-day history.
pub fn compute_progress(
    total: u32,
    states: &[ProblemStateRow],
    pass_days: &[NaiveDate],
    today: NaiveDate,
) -> Progress {
    let solved = states.iter().filter(|s| s.status == "solved").count();
    // attempted = every problem touched at least once
    let attempted = states
        .iter()
        .filter(|s| s.last_attempted_at.is_some())
        .count();
    let mastered = states.iter().filter(|s| s.mastered).count();
    let needs_review = states.iter().filter(|s| s.status == "needs-review").count();
    let (streak_days, best_streak_days) = streaks(pass_days, today);
    Progress {
        solved: solved as u32,
        total,
        attempted: attempted as u32,
        streak_days,
        best_streak_days,
        mastered: mastered as u32,
        needs_review: needs_review as u32,
    }
}

/// Zero-filled daily passing-submit counts for the heatmap, oldest first.
pub fn activity(day_counts: &[(NaiveDate, u32)], today: NaiveDate) -> Vec<ActivityDay> {
    let start = today - chrono::Days::new(WINDOW_DAYS - 1);
    let by_day: HashMap<NaiveDate, u32> = day_counts.iter().cloned().collect();
    (0..WINDOW_DAYS)
        .map(|i| {
            let day = start + chrono::Days::new(i);
            ActivityDay {
                date: day.format("%Y-%m-%d").to_string(),
                count: by_day.get(&day).copied().unwrap_or(0),
            }
        })
        .collect()
}

/// All-time cumulative solve count sampled weekly across the window
/// (27 points), oldest first — monotonic by construction.
pub fn cumulative_weekly(first_solves: &[NaiveDate], today: NaiveDate) -> Vec<u32> {
    let start = today - chrono::Days::new(WINDOW_DAYS - 1);
    (0..=26u64)
        .map(|week| {
            let cutoff = start + chrono::Days::new((week * 7).min(WINDOW_DAYS - 1));
            first_solves.iter().filter(|d| **d <= cutoff).count() as u32
        })
        .collect()
}

/// Month labels for the chart axis: [window start, window middle].
pub fn axis_labels(today: NaiveDate) -> [String; 2] {
    let start = today - chrono::Days::new(WINDOW_DAYS);
    let mid = today - chrono::Days::new(WINDOW_DAYS / 2);
    [start.format("%b").to_string(), mid.format("%b").to_string()]
}

/// Per-pattern solved/total in the canonical `PATTERNS` order; input is one
/// `(pattern, solved?)` entry per problem.
pub fn pattern_stats(problems: &[(Pattern, bool)]) -> Vec<PatternStat> {
    PATTERNS
        .iter()
        .map(|name| {
            let total = problems.iter().filter(|(p, _)| p.0 == *name).count() as u32;
            let solved = problems.iter().filter(|(p, s)| p.0 == *name && *s).count() as u32;
            PatternStat {
                pattern: Pattern((*name).to_string()),
                solved,
                total,
            }
        })
        .collect()
}

/// Mock semantics (`buildDashboard`): rank patterns by solve ratio; focus =
/// 3 weakest, strong = 3 strongest that have ≥1 solve. Patterns with no
/// problems are excluded from both.
pub fn focus_and_strong(stats: &[PatternStat]) -> (Vec<PatternStat>, Vec<PatternStat>) {
    let mut ranked: Vec<PatternStat> = stats.iter().filter(|s| s.total > 0).cloned().collect();
    ranked.sort_by(|a, b| ratio(a).total_cmp(&ratio(b)));
    let focus = ranked.iter().take(3).cloned().collect();
    let strong = ranked
        .iter()
        .rev()
        .filter(|s| s.solved > 0)
        .take(3)
        .cloned()
        .collect();
    (focus, strong)
}

fn ratio(s: &PatternStat) -> f64 {
    s.solved as f64 / s.total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn streak_counts_back_from_today() {
        let days = [d("2026-06-10"), d("2026-06-11"), d("2026-06-12")];
        assert_eq!(streaks(&days, d("2026-06-12")), (3, 3));
    }

    #[test]
    fn streak_survives_a_not_yet_practiced_today() {
        let days = [d("2026-06-10"), d("2026-06-11")];
        assert_eq!(streaks(&days, d("2026-06-12")), (2, 2));
    }

    #[test]
    fn streak_breaks_after_a_full_missed_day() {
        let days = [d("2026-06-09"), d("2026-06-10")];
        assert_eq!(streaks(&days, d("2026-06-12")), (0, 2));
    }

    #[test]
    fn best_streak_tracks_the_longest_historic_run() {
        let days = [
            d("2026-05-01"),
            d("2026-05-02"),
            d("2026-05-03"),
            d("2026-05-04"),
            d("2026-06-11"),
            d("2026-06-12"),
        ];
        assert_eq!(streaks(&days, d("2026-06-12")), (2, 4));
    }

    #[test]
    fn empty_history_means_zero_streaks() {
        assert_eq!(streaks(&[], d("2026-06-12")), (0, 0));
    }

    #[test]
    fn duplicate_days_count_once() {
        let days = [d("2026-06-12"), d("2026-06-12"), d("2026-06-11")];
        assert_eq!(streaks(&days, d("2026-06-12")), (2, 2));
    }

    #[test]
    fn freeze_streak_survives_one_missed_day() {
        // Practiced Mon, Tue, then missed Wed, practiced Thu. The freeze bridges
        // Wed → the streak is 3 practice days and still alive today (Thu).
        let days = [d("2026-06-08"), d("2026-06-09"), d("2026-06-11")];
        let s = streak_with_freezes(&days, d("2026-06-11"));
        assert_eq!(s.current, 3);
        assert_eq!(s.best, 3);
        assert!(!s.freeze_active); // practiced today, no freeze needed right now
    }

    #[test]
    fn freeze_streak_breaks_on_two_missed_days() {
        // Practiced Mon, Tue; missed Wed AND Thu. By Fri the streak is dead.
        let days = [d("2026-06-08"), d("2026-06-09")];
        let s = streak_with_freezes(&days, d("2026-06-12"));
        assert_eq!(s.current, 0);
        assert_eq!(s.best, 2);
        assert!(!s.freeze_active);
    }

    #[test]
    fn freeze_is_active_when_yesterday_was_missed() {
        // Last practiced two days ago (missed yesterday) — the streak is alive
        // today only because a freeze is holding it; one more miss ends it.
        let days = [d("2026-06-09"), d("2026-06-10")];
        let s = streak_with_freezes(&days, d("2026-06-12"));
        assert_eq!(s.current, 2);
        assert!(s.freeze_active);
    }

    #[test]
    fn freeze_streak_matches_strict_streak_with_no_gaps() {
        let days = [d("2026-06-10"), d("2026-06-11"), d("2026-06-12")];
        let s = streak_with_freezes(&days, d("2026-06-12"));
        assert_eq!((s.current, s.best), (3, 3));
        assert!(!s.freeze_active);
    }

    #[test]
    fn freeze_forgives_only_one_gap_at_a_time() {
        // Gaps of exactly one missed day each are all bridged: a long staircase
        // of every-other-day practice is one continuous frozen streak.
        let days = [d("2026-06-02"), d("2026-06-04"), d("2026-06-06")];
        let s = streak_with_freezes(&days, d("2026-06-06"));
        assert_eq!(s.current, 3);
        assert_eq!(s.best, 3);
    }

    #[test]
    fn empty_history_has_no_freeze_streak() {
        let s = streak_with_freezes(&[], d("2026-06-12"));
        assert_eq!((s.current, s.best, s.freeze_active), (0, 0, false));
    }

    #[test]
    fn activity_zero_fills_the_whole_window_across_month_boundaries() {
        let counts = [(d("2026-06-10"), 2), (d("2026-03-01"), 1)];
        let days = activity(&counts, d("2026-06-12"));
        assert_eq!(days.len(), 182);
        assert_eq!(days.last().unwrap().date, "2026-06-12");
        assert_eq!(days.first().unwrap().date, "2025-12-13");
        assert_eq!(
            days.iter().find(|a| a.date == "2026-06-10").unwrap().count,
            2
        );
        assert_eq!(
            days.iter().find(|a| a.date == "2026-03-01").unwrap().count,
            1
        );
        // everything else zero-filled
        assert_eq!(days.iter().map(|a| a.count).sum::<u32>(), 3);
    }

    #[test]
    fn cumulative_is_monotonic_and_counts_pre_window_history() {
        let solves = [d("2020-01-01"), d("2026-06-01"), d("2026-06-11")];
        let series = cumulative_weekly(&solves, d("2026-06-12"));
        assert_eq!(series.len(), 27);
        assert_eq!(series[0], 1); // the 2020 solve is already counted
        assert_eq!(*series.last().unwrap(), 3);
        assert!(series.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn focus_picks_weakest_and_strong_requires_a_solve() {
        let stat = |p: &str, solved: u32, total: u32| PatternStat {
            pattern: Pattern(p.into()),
            solved,
            total,
        };
        let stats = vec![
            stat("Stack", 0, 4),
            stat("Graphs", 4, 4),
            stat("Two Pointers", 1, 4),
            stat("Trees", 0, 0), // no problems: excluded everywhere
        ];
        let (focus, strong) = focus_and_strong(&stats);
        assert_eq!(focus[0].pattern.0, "Stack");
        assert_eq!(focus.len(), 3);
        assert_eq!(strong[0].pattern.0, "Graphs");
        // Stack has zero solves → never "strong"
        assert!(strong.iter().all(|s| s.pattern.0 != "Stack"));
    }

    #[test]
    fn pattern_stats_cover_all_15_in_order() {
        let problems = vec![
            (Pattern("Stack".into()), true),
            (Pattern("Stack".into()), false),
            (Pattern("Graphs".into()), true),
        ];
        let stats = pattern_stats(&problems);
        assert_eq!(stats.len(), 15);
        assert_eq!(stats[0].pattern.0, "Arrays & Hashing");
        let stack = stats.iter().find(|s| s.pattern.0 == "Stack").unwrap();
        assert_eq!((stack.solved, stack.total), (1, 2));
    }
}
