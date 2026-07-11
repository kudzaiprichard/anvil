//! IPC for the Library and Workspace screens (`listProblems` / `getProblem`
//! in `src/lib/api/tauri.ts`). Thin layer only: look up in the store, join
//! progress state, serialize. Filtering/sorting stays client-side
//! (`use-problem-filters.ts`), so `list_problems` always returns the full
//! annotated list.

use tauri::State;

use crate::domain::problem::{Problem, ProblemStatus, ProblemSummary};
use crate::error::AppResult;
use crate::services::db::problem_state;
use crate::state::AppState;

fn status_from_db(status: &str) -> ProblemStatus {
    match status {
        "in-progress" => ProblemStatus::InProgress,
        "solved" => ProblemStatus::Solved,
        "needs-review" => ProblemStatus::NeedsReview,
        _ => ProblemStatus::Todo,
    }
}

/// Status + lastAttempted joined from `problem_state`. `lastAttempted` is a
/// local ISO timestamp; the seam (`tauri.ts`) formats it relatively for
/// display.
#[tauri::command]
pub fn list_problems(state: State<AppState>) -> AppResult<Vec<ProblemSummary>> {
    log::debug!("list_problems");
    let states = problem_state::get_all(&state.db)?;
    Ok(state
        .problems
        .all()
        .iter()
        .map(|p| {
            let row = states.get(&p.id);
            p.summary(
                row.map_or(ProblemStatus::Todo, |r| status_from_db(&r.status)),
                row.and_then(|r| r.last_attempted_at.clone()),
            )
        })
        .collect())
}

/// Hidden test case values are stripped for built-ins before crossing IPC
/// (`Problem::sanitized_for_ipc`) — the runner resolves hidden cases
/// Rust-side by id, so the WebView never sees them.
#[tauri::command]
pub fn get_problem(state: State<AppState>, id: String) -> AppResult<Option<Problem>> {
    log::debug!("get_problem: {id}");
    Ok(state.problems.get(&id).map(|p| p.sanitized_for_ipc()))
}
