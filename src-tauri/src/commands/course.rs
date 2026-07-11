//! IPC for the course/curriculum layer. Reads (`get_curriculum`/`get_unit`/
//! `get_lesson`) are thin lookups into the validated `CurriculumStore`;
//! writes (`record_lesson_progress`) land in the `lesson_progress` table,
//! keeping the content/state split (bundled resources vs. per-user SQLite)
//! that mirrors the catalog vs. `problem_state` design.

use tauri::State;

use crate::domain::advancement::{
    CapstoneOutcome, CapstoneView, PlacementOutcome, PlacementProbe, Readiness,
};
use crate::domain::curriculum::Curriculum;
use crate::domain::lesson::Lesson;
use crate::domain::mastery::{GateOutcome, UnitProgress};
use crate::domain::progress::{LessonProgress, LessonStatus};
use crate::domain::quiz::{Quiz, QuizAnswer, QuizGrade};
use crate::domain::review::{ReviewOutcome, ReviewQueue, ReviewRating};
use crate::domain::unit::Unit;
use crate::error::{AppError, AppResult};
use crate::services::db::{self, lesson_progress};
use crate::services::{advancement, progression, quiz, review};
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

/// A lesson's formative quiz (concept-check + pattern-picker + complexity
/// items). Thin lookup into the validated content — `None` for an unknown
/// lesson id, mirroring `get_lesson`.
#[tauri::command]
pub fn get_quiz(state: State<AppState>, lesson_id: String) -> AppResult<Option<Quiz>> {
    log::debug!("get_quiz: {lesson_id}");
    Ok(state.curriculum.get_quiz(&lesson_id).cloned())
}

/// The interleaved, cross-unit pattern-picker pool (unlabeled recognition
/// drills). Empty when no pool ships.
#[tauri::command]
pub fn get_pattern_pool(state: State<AppState>) -> AppResult<Quiz> {
    log::debug!("get_pattern_pool");
    Ok(state.curriculum.pattern_pool().clone())
}

/// Grades a formative quiz submission and records the outcome to feed the
/// review signal (LESSON_COURSE_DESIGN.md §3.4). `source` is a lesson id or the
/// reserved `pattern-pool` id. Grading is server-side against the validated
/// content, so the answer key never has to be trusted from the caller. Quizzes
/// **never gate progression** — the result is feedback only.
#[tauri::command]
pub fn submit_quiz(
    state: State<AppState>,
    source: String,
    answers: Vec<QuizAnswer>,
) -> AppResult<QuizGrade> {
    log::debug!("submit_quiz: {source} ({} answer(s))", answers.len());
    quiz::submit(
        &state.curriculum,
        &state.db,
        &source,
        &answers,
        &db::now_local_iso(),
    )
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

/// The Stage-7 mixed capstone as the course page shows it (Phase 7): the
/// unlabeled cross-unit pool + how far through it the learner is. `None` when no
/// capstone ships. The pattern each problem belongs to is deliberately absent —
/// the capstone is the unlabeled recognition exam (BLUEPRINT.md §4).
#[tauri::command]
pub fn get_capstone(state: State<AppState>) -> AppResult<Option<CapstoneView>> {
    log::debug!("get_capstone");
    advancement::capstone_view(&state.curriculum, &state.db)
}

/// Scores one capstone attempt (Phase 7). Like a gate: a peeked/hinted attempt
/// (`used_help`) never counts. Rejects a slug that isn't in the capstone pool.
#[tauri::command]
pub fn evaluate_capstone(
    state: State<AppState>,
    problem_id: String,
    used_help: bool,
) -> AppResult<CapstoneOutcome> {
    log::debug!("evaluate_capstone: {problem_id} (used_help={used_help})");
    advancement::evaluate_capstone(
        &state.curriculum,
        &state.db,
        &problem_id,
        used_help,
        &db::now_local_iso(),
    )
}

/// The diagnostic placement probe (Phase 7): unlabeled pattern-picker items the
/// learner answers to be *placed out* of units they already recognize, starting
/// them at their frontier.
#[tauri::command]
pub fn get_placement(state: State<AppState>) -> AppResult<PlacementProbe> {
    log::debug!("get_placement");
    Ok(advancement::placement_probe(&state.curriculum))
}

/// Applies a submitted placement probe (Phase 7): recognized units whose prereqs
/// are also cleared are marked mastered-via-placement, unlocking the learner's
/// frontier. Returns which units were placed and the newly-unlocked frontier.
#[tauri::command]
pub fn apply_placement(
    state: State<AppState>,
    answers: Vec<QuizAnswer>,
) -> AppResult<PlacementOutcome> {
    log::debug!("apply_placement: {} answer(s)", answers.len());
    advancement::apply_placement(&state.curriculum, &state.db, &answers, &db::now_local_iso())
}

/// The honest course-readiness signal (Phase 7, BLUEPRINT.md §7): how much of
/// the ladder is mastered, folded with whether the unlabeled capstone is cleared.
#[tauri::command]
pub fn get_readiness(state: State<AppState>) -> AppResult<Readiness> {
    log::debug!("get_readiness");
    advancement::readiness(&state.curriculum, &state.db)
}

/// The spaced-review queue (Phase 6, COURSE_BLUEPRINT.md §7): the Stage-1
/// problems due to be re-solved *cold* right now — interleaved across patterns —
/// plus the honest habit header (streak-with-freezes, counts). Pure engine
/// output derived from the FSRS `review_schedule`; content stays bundled data.
#[tauri::command]
pub fn get_review_queue(state: State<AppState>) -> AppResult<ReviewQueue> {
    log::debug!("get_review_queue");
    review::queue(
        &state.curriculum,
        &state.db,
        chrono::Utc::now(),
        chrono::Local::now().date_naive(),
    )
}

/// Records a cold re-solve and reschedules the card via FSRS (COURSE_BLUEPRINT.md
/// §7). `rating` is the learner's self-assessed recall after re-solving; an
/// `again` demotes the card (interval collapses, lapse counter bumps). Rejects a
/// problem that never entered the queue.
#[tauri::command]
pub fn record_review(
    state: State<AppState>,
    problem_id: String,
    rating: ReviewRating,
) -> AppResult<ReviewOutcome> {
    log::debug!("record_review: {problem_id} {rating:?}");
    review::record(&state.db, &problem_id, rating, chrono::Utc::now())
}
