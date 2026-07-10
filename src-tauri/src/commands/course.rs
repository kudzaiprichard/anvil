//! IPC for the course/curriculum layer. Reads (`get_curriculum`/`get_unit`/
//! `get_lesson`) are thin lookups into the validated `CurriculumStore`;
//! writes (`record_lesson_progress`) land in the `lesson_progress` table,
//! keeping the content/state split (bundled resources vs. per-user SQLite)
//! that mirrors the catalog vs. `problem_state` design.

use tauri::State;

use crate::domain::curriculum::Curriculum;
use crate::domain::lesson::Lesson;
use crate::domain::mastery::{GateOutcome, UnitProgress};
use crate::domain::progress::{LessonProgress, LessonStatus};
use crate::domain::unit::Unit;
use crate::error::{AppError, AppResult};
use crate::services::db::{self, lesson_progress};
use crate::services::progression;
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

/// Records that the user opened (`in-progress`) or finished (`complete`) a
/// lesson. The unit is derived from the validated lesson content, not trusted
/// from the caller — an unknown lesson id is rejected.
#[tauri::command]
pub fn record_lesson_progress(
    state: State<AppState>,
    lesson_id: String,
    status: LessonStatus,
) -> AppResult<()> {
    log::debug!("record_lesson_progress: {lesson_id} {status:?}");
    let lesson = state
        .curriculum
        .get_lesson(&lesson_id)
        .ok_or_else(|| AppError::NotFound(format!("Lesson not found: {lesson_id}")))?;
    lesson_progress::record(
        &state.db,
        &lesson_id,
        &lesson.unit,
        status,
        &db::now_local_iso(),
    )
}

/// Every recorded lesson's progress — the `/learn` view badges lessons from
/// this.
#[tauri::command]
pub fn get_lesson_progress(state: State<AppState>) -> AppResult<Vec<LessonProgress>> {
    log::debug!("get_lesson_progress");
    lesson_progress::list(&state.db)
}

/// Every unit's progression snapshot (locked/unlocked/mastered + lesson and
/// gate progress), in stage order — the course overview and unit views read
/// this to gate navigation and draw progress. Engine logic, not stored data
/// (LESSON_COURSE_DESIGN.md §6): lock state is derived from the prereq DAG and
/// which units have passed their gate.
#[tauri::command]
pub fn get_progression(state: State<AppState>) -> AppResult<Vec<UnitProgress>> {
    log::debug!("get_progression");
    progression::progression(&state.curriculum, &state.db)
}

/// Evaluates a passing gate attempt (COURSE_BLUEPRINT.md §6). The unit is
/// derived from validated content — a caller can't gate a non-gate problem or a
/// locked unit. `used_help` is trusted from the workspace, which sets it when
/// the learner reveals a hint or the solution: such an attempt is recorded but
/// never counts toward mastery. On a counted pass that meets the threshold, the
/// unit is mastered and the next unit(s) unlock.
#[tauri::command]
pub fn evaluate_gate(
    state: State<AppState>,
    unit_id: String,
    problem_id: String,
    used_help: bool,
) -> AppResult<GateOutcome> {
    log::debug!("evaluate_gate: {unit_id} / {problem_id} (used_help={used_help})");
    progression::evaluate_gate(
        &state.curriculum,
        &state.db,
        &unit_id,
        &problem_id,
        used_help,
        &db::now_local_iso(),
    )
}
