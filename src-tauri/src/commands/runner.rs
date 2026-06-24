//! IPC for the workspace Run/Submit buttons (`runCode` / `submitCode` in
//! `src/lib/api/tauri.ts`). Async + `spawn_blocking` so the up-to-3-second
//! subprocess never blocks the main thread. Both commands record history +
//! state (task 0008); only passing SUBMITS count toward solves/streaks.

use tauri::State;

use crate::domain::run::{Language, RunRequest, RunResult, RunStatus};
use crate::error::{AppError, AppResult};
use crate::services::db::{self, attempts};
use crate::services::runner;
use crate::state::AppState;

async fn run(
    state: State<'_, AppState>,
    req: RunRequest,
    include_hidden: bool,
) -> AppResult<RunResult> {
    let problem = state
        .problems
        .get(&req.id)
        .ok_or_else(|| AppError::NotFound(format!("Problem not found: {}", req.id)))?;
    let program = state.runtime_path(req.language).ok_or_else(|| {
        AppError::Runner(match req.language {
            Language::Python => "Python 3.10+ not found — see Settings → Runtime".into(),
            Language::Javascript => "Node.js 18+ not found — see Settings → Runtime".into(),
        })
    })?;
    let (language, code) = (req.language, req.code.clone());
    let result = tauri::async_runtime::spawn_blocking(move || {
        runner::execute_with_program(&problem, language, &code, include_hidden, &program)
    })
    .await
    .map_err(|e| AppError::Runner(format!("runner task panicked: {e}")))??;

    // Record history + state. A DB failure must never eat the run result —
    // the user's feedback matters more than the stat; log and move on.
    let record = attempts::AttemptRecord {
        problem_id: &req.id,
        language: match req.language {
            Language::Python => "python",
            Language::Javascript => "javascript",
        },
        kind: if include_hidden { "submit" } else { "run" },
        status: match result.status {
            RunStatus::Pass => "pass",
            RunStatus::Fail => "fail",
            RunStatus::Error => "error",
            RunStatus::Timeout => "timeout",
        },
        runtime_ms: result.runtime_ms,
        code: &req.code,
        attempted_at: &db::now_local_iso(),
    };
    if let Err(e) = attempts::record_attempt(&state.db, &record) {
        log::error!("failed to record attempt for {}: {e}", req.id);
    }

    Ok(result)
}

/// Run against VISIBLE test cases only.
#[tauri::command]
pub async fn run_code(state: State<'_, AppState>, req: RunRequest) -> AppResult<RunResult> {
    log::debug!("run_code: {} ({:?})", req.id, req.language);
    run(state, req, false).await
}

/// Run against ALL test cases (visible + hidden).
#[tauri::command]
pub async fn submit_code(state: State<'_, AppState>, req: RunRequest) -> AppResult<RunResult> {
    log::debug!("submit_code: {} ({:?})", req.id, req.language);
    run(state, req, true).await
}
