//! Per-lesson user progress (LESSON_COURSE_DESIGN.md §6.4). Curriculum/unit/
//! lesson *content* is bundled resource data (validated by
//! `services::curriculum`); this table holds only what the user *does* with a
//! lesson — the same content/state split as `problem_state` vs. the catalog.
//! Phase 2 writes `in-progress` (on open) and `complete` (on the learner's
//! mark-complete); later phases layer mastery/review on top.

use rusqlite::params;

use super::Db;
use crate::domain::progress::{LessonProgress, LessonStatus};
use crate::error::{AppError, AppResult};

/// Upserts a lesson's progress row. Monotonic: `in-progress` never downgrades
/// an already-`complete` lesson, and `started_at` is stamped once and kept.
/// `NotStarted` is the absence of a row, not a recordable transition — it is
/// rejected loudly rather than written.
pub fn record(
    db: &Db,
    lesson_id: &str,
    unit_id: &str,
    status: LessonStatus,
    now: &str,
) -> AppResult<()> {
    let conn = db.lock()?;
    match status {
        LessonStatus::InProgress => conn.execute(
            "INSERT INTO lesson_progress (lesson_id, unit_id, status, started_at)
             VALUES (?1, ?2, 'in-progress', ?3)
             ON CONFLICT(lesson_id) DO UPDATE SET
               status = CASE WHEN lesson_progress.status = 'complete'
                             THEN 'complete' ELSE 'in-progress' END,
               started_at = COALESCE(lesson_progress.started_at, excluded.started_at)",
            params![lesson_id, unit_id, now],
        )?,
        LessonStatus::Complete => conn.execute(
            "INSERT INTO lesson_progress (lesson_id, unit_id, status, started_at, completed_at)
             VALUES (?1, ?2, 'complete', ?3, ?3)
             ON CONFLICT(lesson_id) DO UPDATE SET
               status = 'complete',
               started_at = COALESCE(lesson_progress.started_at, excluded.started_at),
               completed_at = excluded.completed_at",
            params![lesson_id, unit_id, now],
        )?,
        LessonStatus::NotStarted => {
            return Err(AppError::Validation(
                "cannot record lesson progress as 'not-started'".into(),
            ))
        }
    };
    Ok(())
}

/// Every recorded lesson's progress — the `/learn` view reads this to badge
/// lessons in-progress/complete.
pub fn list(db: &Db) -> AppResult<Vec<LessonProgress>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT lesson_id, unit_id, status, started_at, completed_at
         FROM lesson_progress",
    )?;
    let rows = stmt
        .query_map([], |row| {
            let status: String = row.get(2)?;
            Ok(LessonProgress {
                lesson_id: row.get(0)?,
                unit_id: row.get(1)?,
                status: LessonStatus::from_wire(&status),
                started_at: row.get(3)?,
                completed_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, db)
    }

    fn one(db: &Db) -> LessonProgress {
        let rows = list(db).unwrap();
        assert_eq!(rows.len(), 1);
        rows.into_iter().next().unwrap()
    }

    #[test]
    fn in_progress_then_complete_is_monotonic() {
        let (_dir, db) = temp_db();
        record(&db, "l1", "u1", LessonStatus::InProgress, "T1").unwrap();
        let row = one(&db);
        assert_eq!(row.status, LessonStatus::InProgress);
        assert_eq!(row.started_at.as_deref(), Some("T1"));
        assert_eq!(row.completed_at, None);

        // Completing keeps the original started_at and stamps completed_at.
        record(&db, "l1", "u1", LessonStatus::Complete, "T2").unwrap();
        let row = one(&db);
        assert_eq!(row.status, LessonStatus::Complete);
        assert_eq!(row.started_at.as_deref(), Some("T1"));
        assert_eq!(row.completed_at.as_deref(), Some("T2"));

        // Re-opening a completed lesson must NOT downgrade it back to progress.
        record(&db, "l1", "u1", LessonStatus::InProgress, "T3").unwrap();
        let row = one(&db);
        assert_eq!(row.status, LessonStatus::Complete);
        assert_eq!(row.completed_at.as_deref(), Some("T2"));
    }

    #[test]
    fn not_started_is_rejected() {
        let (_dir, db) = temp_db();
        let err = record(&db, "l1", "u1", LessonStatus::NotStarted, "T1").unwrap_err();
        assert!(err.to_string().contains("not-started"), "{err}");
        assert!(list(&db).unwrap().is_empty());
    }
}
