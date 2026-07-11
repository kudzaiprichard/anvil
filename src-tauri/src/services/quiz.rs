//! Quiz grading + review-signal recording (Phase 4). The "hard-coded" half of
//! the formative-check feature (LESSON_COURSE_DESIGN.md §6.1: quiz *items* are
//! data, the *runner + grading logic* is engine). A submission names its
//! source — a lesson id, or the interleaved `pattern-pool` — which resolves to
//! a validated `Quiz` in the `CurriculumStore`; grading is pure
//! (`Quiz::grade`), and the outcome is recorded to feed the review signal.
//!
//! Quizzes are **never a gate** (§3.4): grading records recognition evidence
//! and returns per-item feedback, and nothing here touches unlock/mastery state.

use crate::domain::quiz::{QuizAnswer, QuizGrade};
use crate::error::{AppError, AppResult};
use crate::services::curriculum::CurriculumStore;
use crate::services::db::{quiz_result, Db};

/// The interleaved cross-unit pattern pool's reserved source id (not a lesson).
pub const PATTERN_POOL_SOURCE: &str = "pattern-pool";

/// Grades a formative submission and records the outcome (review signal).
/// `source` is a lesson id or [`PATTERN_POOL_SOURCE`]; an unknown source is
/// rejected loudly rather than silently graded against an empty quiz. Never
/// blocks progression — the returned `QuizGrade` is feedback only.
pub fn submit(
    store: &CurriculumStore,
    db: &Db,
    source: &str,
    answers: &[QuizAnswer],
    now: &str,
) -> AppResult<QuizGrade> {
    let quiz = if source == PATTERN_POOL_SOURCE {
        store.pattern_pool()
    } else {
        store
            .get_quiz(source)
            .ok_or_else(|| AppError::NotFound(format!("Quiz not found for source: {source}")))?
    };

    let grade = quiz.grade(answers);
    quiz_result::record(db, source, &grade.results, now)?;
    Ok(grade)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    use crate::services::pack_store::PackStore;

    fn real_resources() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    fn fixture() -> (tempfile::TempDir, CurriculumStore, Db) {
        let packs = PackStore::new(real_resources().join("test-packs.json.gz"));
        let store = CurriculumStore::load(&real_resources(), &packs).expect("curriculum loads");
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, store, db)
    }

    #[test]
    fn grades_a_lesson_quiz_and_records_the_signal() {
        let (_dir, store, db) = fixture();
        let quiz = store
            .get_quiz("01-hashmap-lookup")
            .expect("lesson quiz")
            .clone();
        // Answer every item correctly using the stored answers.
        let answers: Vec<QuizAnswer> = quiz
            .items
            .iter()
            .map(|it| QuizAnswer {
                item_id: it.id.clone(),
                selected: it.answer.clone(),
            })
            .collect();

        let grade = submit(&store, &db, "01-hashmap-lookup", &answers, "T1").unwrap();
        assert_eq!(grade.correct_count, grade.total);

        // The submission fed the review signal.
        let stats = quiz_result::stats(&db, "01-hashmap-lookup").unwrap();
        assert_eq!(stats.answered, grade.total);
        assert_eq!(stats.correct, grade.total);
    }

    #[test]
    fn grades_the_interleaved_pattern_pool() {
        let (_dir, store, db) = fixture();
        let pool = store.pattern_pool().clone();
        assert!(
            !pool.items.is_empty(),
            "shipped pattern pool should be non-empty"
        );
        // Answer just the first pool item, correctly — only answered items count.
        let first = &pool.items[0];
        let grade = submit(
            &store,
            &db,
            PATTERN_POOL_SOURCE,
            &[QuizAnswer {
                item_id: first.id.clone(),
                selected: first.answer.clone(),
            }],
            "T1",
        )
        .unwrap();
        assert_eq!(grade.total, 1);
        assert_eq!(grade.correct_count, 1);
        let stats = quiz_result::stats(&db, PATTERN_POOL_SOURCE).unwrap();
        assert_eq!(stats.answered, 1);
        assert_eq!(stats.correct, 1);
    }

    #[test]
    fn unknown_source_is_rejected() {
        let (_dir, store, db) = fixture();
        let err = submit(&store, &db, "no-such-lesson", &[], "T1").unwrap_err();
        assert!(err.to_string().contains("Quiz not found"), "{err}");
    }
}
