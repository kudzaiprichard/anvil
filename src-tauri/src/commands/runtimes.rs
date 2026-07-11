//! IPC for the Settings → Runtime pane (`detectRuntimes` in
//! `src/lib/api/tauri.ts`). Re-probes PATH on every call (the pane's
//! "Re-detect" button is the refresh trigger) and updates the cache the
//! runner reads.

use tauri::State;

use crate::error::{AppError, AppResult};
use crate::services::runtime_detect::{self, RuntimeInfo};
use crate::state::AppState;

#[tauri::command]
pub async fn detect_runtimes(state: State<'_, AppState>) -> AppResult<Vec<RuntimeInfo>> {
    log::debug!("detect_runtimes");
    let detected = tauri::async_runtime::spawn_blocking(runtime_detect::detect)
        .await
        .map_err(|e| AppError::Runner(format!("detection task panicked: {e}")))?;
    state.set_runtimes(detected.clone());
    Ok(detected)
}
