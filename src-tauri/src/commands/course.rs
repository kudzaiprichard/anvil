//! IPC stubs for the course/curriculum data layer (Phase 1 — no user-facing
//! UI yet; these exist so `app/learn/` has a contract to build against in
//! Phase 2). Thin layer only: look up in `CurriculumStore`, serialize.

use tauri::State;

use crate::domain::curriculum::Curriculum;
use crate::domain::lesson::Lesson;
use crate::domain::unit::Unit;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub fn get_curriculum(state: State<AppState>) -> AppResult<Curriculum> {
    log::debug!("get_curriculum");
    Ok(state.curriculum.curriculum().clone())
}

#[tauri::command]
pub fn get_unit(state: State<AppState>, id: String) -> AppResult<Option<Unit>> {
    log::debug!("get_unit: {id}");
    Ok(state.curriculum.get_unit(&id).cloned())
}

#[tauri::command]
pub fn get_lesson(state: State<AppState>, id: String) -> AppResult<Option<Lesson>> {
    log::debug!("get_lesson: {id}");
    Ok(state.curriculum.get_lesson(&id).cloned())
}
