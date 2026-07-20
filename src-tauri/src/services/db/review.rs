//! FSRS review-schedule persistence (Phase 6).
//! One row per problem that has entered the spaced-review queue (a solved or
//! gated Stage-1 problem). The connection never leaves this module tree (see
//! `mod.rs`): the scheduler in `services::review` reconstructs an `rs_fsrs::Card`
//! from a [`ReviewRow`], computes the next state, and writes it back here. This
//! is per-user state — the same content/state split as `problem_state` vs. the
//! catalog; the curriculum *content* stays bundled resource data.
//!
//! Timestamps in this table are RFC3339 UTC (e.g. `2026-07-10T09:00:00+00:00`),
//! not the local-ISO stamps the rest of the DB uses: FSRS math is timezone-
//! agnostic day arithmetic, and a single canonical zone keeps `due_at`
//! comparisons and lexical ordering correct.

use rusqlite::{params, OptionalExtension};

use super::Db;
use crate::error::AppResult;

/// A raw `review_schedule` row. `stability`/`difficulty` are `None` only for a
/// freshly enqueued `new` card that has never been reviewed; `due_at` is always
/// set (a new card is due immediately).
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewRow {
    pub problem_id: String,
    /// `"new" | "learning" | "review" | "relearning"` (mirrors the CHECK).
    pub state: String,
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    pub due_at: Option<String>,
    pub last_reviewed_at: Option<String>,
    pub lapses: i64,
}

/// Enqueues a problem as a fresh `new` card due immediately. Idempotent: a
/// problem already in the queue keeps its existing schedule (we never reset a
/// card's spacing just because it was solved again). Returns `true` when a new
/// row was actually inserted.
pub fn enqueue_new(db: &Db, problem_id: &str, due_at: &str) -> AppResult<bool> {
    let conn = db.lock()?;
    let changed = conn.execute(
        "INSERT INTO review_schedule (problem_id, state, due_at, lapses)
         VALUES (?1, 'new', ?2, 0)
         ON CONFLICT(problem_id) DO NOTHING",
        params![problem_id, due_at],
    )?;
    Ok(changed > 0)
}

/// Writes a card's post-review scheduler state (`services::review` computed it
/// via FSRS). Upsert so the first review of a `new` card and every later review
/// use the same path.
pub fn upsert(db: &Db, row: &ReviewRow) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO review_schedule
           (problem_id, state, stability, difficulty, due_at, last_reviewed_at, lapses)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(problem_id) DO UPDATE SET
           state = excluded.state,
           stability = excluded.stability,
           difficulty = excluded.difficulty,
           due_at = excluded.due_at,
           last_reviewed_at = excluded.last_reviewed_at,
           lapses = excluded.lapses",
        params![
            row.problem_id,
            row.state,
            row.stability,
            row.difficulty,
            row.due_at,
            row.last_reviewed_at,
            row.lapses
        ],
    )?;
    Ok(())
}

/// One card by problem id, or `None` when it has never entered the queue.
pub fn get(db: &Db, problem_id: &str) -> AppResult<Option<ReviewRow>> {
    let conn = db.lock()?;
    let row = conn
        .query_row(
            "SELECT problem_id, state, stability, difficulty, due_at, last_reviewed_at, lapses
             FROM review_schedule WHERE problem_id = ?1",
            [problem_id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

/// Every scheduled card, earliest due first — `services::review` splits this
/// into "due now" vs. "later" and interleaves the due half.
pub fn list_all(db: &Db) -> AppResult<Vec<ReviewRow>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT problem_id, state, stability, difficulty, due_at, last_reviewed_at, lapses
         FROM review_schedule
         ORDER BY due_at IS NULL, due_at, problem_id",
    )?;
    let rows = stmt
        .query_map([], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ReviewRow> {
    Ok(ReviewRow {
        problem_id: row.get(0)?,
        state: row.get(1)?,
        stability: row.get(2)?,
        difficulty: row.get(3)?,
        due_at: row.get(4)?,
        last_reviewed_at: row.get(5)?,
        lapses: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, db)
    }

    #[test]
    fn enqueue_is_idempotent_and_keeps_the_original_schedule() {
        let (_dir, db) = temp_db();
        assert!(enqueue_new(&db, "two-sum", "2026-07-10T09:00:00+00:00").unwrap());
        // A second enqueue (e.g. the learner solved it again) must not reset it.
        assert!(!enqueue_new(&db, "two-sum", "2026-07-20T09:00:00+00:00").unwrap());
        let row = get(&db, "two-sum").unwrap().unwrap();
        assert_eq!(row.state, "new");
        assert_eq!(row.due_at.as_deref(), Some("2026-07-10T09:00:00+00:00"));
        assert_eq!(row.lapses, 0);
        assert!(row.stability.is_none());
    }

    #[test]
    fn upsert_persists_scheduler_fields() {
        let (_dir, db) = temp_db();
        enqueue_new(&db, "two-sum", "2026-07-10T09:00:00+00:00").unwrap();
        upsert(
            &db,
            &ReviewRow {
                problem_id: "two-sum".into(),
                state: "review".into(),
                stability: Some(3.12),
                difficulty: Some(5.0),
                due_at: Some("2026-07-13T09:00:00+00:00".into()),
                last_reviewed_at: Some("2026-07-10T09:00:00+00:00".into()),
                lapses: 0,
            },
        )
        .unwrap();
        let row = get(&db, "two-sum").unwrap().unwrap();
        assert_eq!(row.state, "review");
        assert_eq!(row.stability, Some(3.12));
        assert_eq!(row.due_at.as_deref(), Some("2026-07-13T09:00:00+00:00"));
    }

    #[test]
    fn list_all_orders_by_due_date() {
        let (_dir, db) = temp_db();
        enqueue_new(&db, "b", "2026-07-12T00:00:00+00:00").unwrap();
        enqueue_new(&db, "a", "2026-07-10T00:00:00+00:00").unwrap();
        let ids: Vec<_> = list_all(&db)
            .unwrap()
            .into_iter()
            .map(|r| r.problem_id)
            .collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[test]
    fn get_is_none_for_an_unscheduled_problem() {
        let (_dir, db) = temp_db();
        assert!(get(&db, "nope").unwrap().is_none());
    }
}
