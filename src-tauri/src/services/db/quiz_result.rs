//! Formative quiz-result persistence (Phase 4).
//! Quizzes never gate progression; these rows only *feed the review signal* —
//! the recognition evidence later phases (FSRS scheduling, readiness) build on.
//! The connection never leaves this module tree (see `mod.rs`): the grading
//! service in `services::quiz` calls these typed helpers and sees only domain
//! data. Content lives in bundled resources; this is per-user state, the same
//! split as `problem_state` vs. the catalog.

use rusqlite::params;

use super::Db;
use crate::domain::quiz::QuizItemResult;
use crate::error::AppResult;

/// Aggregate recognition performance for one quiz source (a lesson id, or
/// `pattern-pool`): how many item answers were recorded and how many correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuizStats {
    pub answered: u32,
    pub correct: u32,
}

/// Records one graded submission — one row per item (append-only, so re-takes
/// accumulate as history, which *is* the signal). Called only after grading;
/// takes the already-graded results so the stored `correct` matches what the
/// learner saw.
pub fn record(db: &Db, source: &str, results: &[QuizItemResult], now: &str) -> AppResult<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    for r in results {
        tx.execute(
            "INSERT INTO quiz_result (source, item_id, item_type, correct, answered_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                source,
                r.item_id,
                item_type_wire(r.item_type),
                r.correct as i64,
                now
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Recognition stats for one source, across all recorded attempts.
pub fn stats(db: &Db, source: &str) -> AppResult<QuizStats> {
    let conn = db.lock()?;
    let (answered, correct) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(correct), 0) FROM quiz_result WHERE source = ?1",
        [source],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?;
    Ok(QuizStats {
        answered: answered as u32,
        correct: correct as u32,
    })
}

/// The `QuizItemType` wire string, matching the migration's CHECK constraint
/// and the serde `kebab-case` rename on the enum.
fn item_type_wire(ty: crate::domain::quiz::QuizItemType) -> &'static str {
    use crate::domain::quiz::QuizItemType::*;
    match ty {
        ConceptCheck => "concept-check",
        PatternPicker => "pattern-picker",
        Complexity => "complexity",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::quiz::{QuizItemResult, QuizItemType};

    fn temp_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, db)
    }

    fn result(id: &str, ty: QuizItemType, correct: bool) -> QuizItemResult {
        QuizItemResult {
            item_id: id.into(),
            item_type: ty,
            correct,
            selected: "x".into(),
            answer: "x".into(),
            explanation_md: "e".into(),
            correct_pattern: None,
        }
    }

    #[test]
    fn records_and_aggregates_stats() {
        let (_dir, db) = temp_db();
        record(
            &db,
            "01-hashmap-lookup",
            &[
                result("q1", QuizItemType::ConceptCheck, true),
                result("q2", QuizItemType::PatternPicker, false),
            ],
            "T1",
        )
        .unwrap();
        let s = stats(&db, "01-hashmap-lookup").unwrap();
        assert_eq!(s.answered, 2);
        assert_eq!(s.correct, 1);
    }

    #[test]
    fn retakes_accumulate_as_history() {
        let (_dir, db) = temp_db();
        record(
            &db,
            "pattern-pool",
            &[result("p1", QuizItemType::PatternPicker, false)],
            "T1",
        )
        .unwrap();
        record(
            &db,
            "pattern-pool",
            &[result("p1", QuizItemType::PatternPicker, true)],
            "T2",
        )
        .unwrap();
        let s = stats(&db, "pattern-pool").unwrap();
        assert_eq!(s.answered, 2);
        assert_eq!(s.correct, 1);
    }

    #[test]
    fn stats_for_unknown_source_is_zero() {
        let (_dir, db) = temp_db();
        let s = stats(&db, "nope").unwrap();
        assert_eq!(s.answered, 0);
        assert_eq!(s.correct, 0);
    }
}
