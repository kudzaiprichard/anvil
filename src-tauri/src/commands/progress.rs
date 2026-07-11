//! IPC for the Dashboard (`getProgress` / `getDashboard`) and the workspace
//! status buttons (`setProblemStatus`, `toggleBookmark`,
//! `getProblemUserState`) in `src/lib/api/tauri.ts`.

use tauri::State;

use crate::domain::problem::ProblemStatus;
use crate::domain::progress::{DashboardData, ProblemUserState, Progress, StatusAction};
use crate::error::{AppError, AppResult};
use crate::services::db::{attempts, problem_state};
use crate::services::progress;
use crate::state::AppState;

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

#[tauri::command]
pub fn get_progress(state: State<AppState>) -> AppResult<Progress> {
    log::debug!("get_progress");
    let states: Vec<_> = problem_state::get_all(&state.db)?.into_values().collect();
    let pass_days: Vec<_> = attempts::pass_days(&state.db)?
        .iter()
        .filter_map(|(day, _)| parse_date(day))
        .collect();
    Ok(progress::compute_progress(
        state.problems.all().len() as u32,
        &states,
        &pass_days,
        today(),
    ))
}

#[tauri::command]
pub fn get_dashboard(state: State<AppState>) -> AppResult<DashboardData> {
    log::debug!("get_dashboard");
    let today = today();
    let states = problem_state::get_all(&state.db)?;
    let pass_day_counts: Vec<_> = attempts::pass_days(&state.db)?
        .iter()
        .filter_map(|(day, count)| parse_date(day).map(|d| (d, *count)))
        .collect();
    let first_solves: Vec<_> = attempts::first_solve_days(&state.db)?
        .iter()
        .filter_map(|day| parse_date(day))
        .collect();

    let state_rows: Vec<_> = states.values().cloned().collect();
    let pass_days: Vec<_> = pass_day_counts.iter().map(|(d, _)| *d).collect();
    let progress_counts = progress::compute_progress(
        state.problems.all().len() as u32,
        &state_rows,
        &pass_days,
        today,
    );

    let per_problem: Vec<_> = state
        .problems
        .all()
        .iter()
        .map(|p| {
            let solved = states.get(&p.id).is_some_and(|r| r.status == "solved");
            (p.pattern.clone(), solved)
        })
        .collect();
    let pattern_stats = progress::pattern_stats(&per_problem);
    let (focus, strong) = progress::focus_and_strong(&pattern_stats);

    // most recently touched in-progress problem
    let continue_problem = states
        .values()
        .filter(|r| r.status == "in-progress")
        .max_by(|a, b| a.last_attempted_at.cmp(&b.last_attempted_at))
        .and_then(|r| {
            state
                .problems
                .get(&r.problem_id)
                .map(|p| p.summary(ProblemStatus::InProgress, r.last_attempted_at.clone()))
        });

    Ok(DashboardData {
        progress: progress_counts,
        activity: progress::activity(&pass_day_counts, today),
        cumulative: progress::cumulative_weekly(&first_solves, today),
        axis_labels: progress::axis_labels(today),
        focus,
        strong,
        continue_problem,
        pattern_stats,
    })
}

/// "Mark mastered" and review-flag mutations. Commands don't trust callers:
/// mastering requires the problem to actually be solved.
#[tauri::command]
pub fn set_problem_status(
    state: State<AppState>,
    problem_id: String,
    action: StatusAction,
) -> AppResult<()> {
    log::debug!("set_problem_status: {problem_id} {action:?}");
    if state.problems.get(&problem_id).is_none() {
        return Err(AppError::NotFound(format!(
            "Problem not found: {problem_id}"
        )));
    }
    let row = problem_state::get(&state.db, &problem_id)?;
    let status = row.as_ref().map(|r| r.status.as_str()).unwrap_or("todo");
    match action {
        StatusAction::MarkMastered => {
            if status != "solved" {
                return Err(AppError::Validation(
                    "Solve the problem before marking it mastered.".into(),
                ));
            }
            problem_state::set_mastered(&state.db, &problem_id, true)
        }
        StatusAction::UnmarkMastered => problem_state::set_mastered(&state.db, &problem_id, false),
        StatusAction::NeedsReview => {
            problem_state::set_status(&state.db, &problem_id, "needs-review")
        }
        StatusAction::ClearReview => {
            if status == "needs-review" {
                problem_state::set_status(&state.db, &problem_id, "solved")
            } else {
                Ok(())
            }
        }
    }
}

#[tauri::command]
pub fn toggle_bookmark(state: State<AppState>, problem_id: String) -> AppResult<bool> {
    log::debug!("toggle_bookmark: {problem_id}");
    problem_state::toggle_bookmark(&state.db, &problem_id)
}

/// Workspace boot: bookmark icon, mastered flag, and the last run/submit
/// code snapshot for one problem (the editor restores it).
#[tauri::command]
pub fn get_problem_user_state(
    state: State<AppState>,
    problem_id: String,
) -> AppResult<ProblemUserState> {
    let row = problem_state::get(&state.db, &problem_id)?;
    Ok(match row {
        Some(r) => ProblemUserState {
            status: match r.status.as_str() {
                "in-progress" => ProblemStatus::InProgress,
                "solved" => ProblemStatus::Solved,
                "needs-review" => ProblemStatus::NeedsReview,
                _ => ProblemStatus::Todo,
            },
            bookmarked: r.bookmarked,
            mastered: r.mastered,
            last_code: r.last_code,
            last_language: match r.last_language.as_deref() {
                Some("python") => Some(crate::domain::run::Language::Python),
                Some("javascript") => Some(crate::domain::run::Language::Javascript),
                _ => None,
            },
        },
        None => ProblemUserState {
            status: ProblemStatus::Todo,
            bookmarked: false,
            mastered: false,
            last_code: None,
            last_language: None,
        },
    })
}
