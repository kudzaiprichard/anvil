//! Per-problem state: status, mastered/bookmarked flags, last attempt
//! details. Transition rules live in `apply_attempt`; status mutations from
//! the UI (mark mastered, bookmark) land in task 0009.

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use super::Db;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct ProblemStateRow {
    pub problem_id: String,
    /// `"todo" | "in-progress" | "solved" | "needs-review"`.
    pub status: String,
    pub mastered: bool,
    pub bookmarked: bool,
    pub last_attempted_at: Option<String>,
    pub last_code: Option<String>,
    pub last_language: Option<String>,
}

/// Transition rules (BACKEND_PLAN 0008 §4): a passing submit promotes to
/// `solved`; any other run/submit on a not-yet-solved problem marks
/// `in-progress`; `solved`/`needs-review` are never downgraded by a failed
/// attempt, and `mastered` survives everything. Takes a `Connection` so
/// `attempts::record_attempt` can call it inside its transaction.
pub fn apply_attempt(
    conn: &Connection,
    problem_id: &str,
    passing_submit: bool,
    code: &str,
    language: &str,
    attempted_at: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO problem_state (problem_id, status, last_attempted_at, last_code, last_language)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(problem_id) DO UPDATE SET
           status = CASE
             WHEN excluded.status = 'solved' THEN 'solved'
             WHEN problem_state.status IN ('solved', 'needs-review') THEN problem_state.status
             ELSE 'in-progress'
           END,
           last_attempted_at = excluded.last_attempted_at,
           last_code = excluded.last_code,
           last_language = excluded.last_language",
        params![
            problem_id,
            if passing_submit { "solved" } else { "in-progress" },
            attempted_at,
            code,
            language
        ],
    )?;
    Ok(())
}

pub fn set_mastered(db: &Db, problem_id: &str, mastered: bool) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO problem_state (problem_id, mastered) VALUES (?1, ?2)
         ON CONFLICT(problem_id) DO UPDATE SET mastered = excluded.mastered",
        params![problem_id, mastered as i64],
    )?;
    Ok(())
}

pub fn set_status(db: &Db, problem_id: &str, status: &str) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO problem_state (problem_id, status) VALUES (?1, ?2)
         ON CONFLICT(problem_id) DO UPDATE SET status = excluded.status",
        params![problem_id, status],
    )?;
    Ok(())
}

/// Flips the bookmark flag and returns the new value.
pub fn toggle_bookmark(db: &Db, problem_id: &str) -> AppResult<bool> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO problem_state (problem_id, bookmarked) VALUES (?1, 1)
         ON CONFLICT(problem_id) DO UPDATE SET bookmarked = 1 - problem_state.bookmarked",
        params![problem_id],
    )?;
    let value: i64 = conn.query_row(
        "SELECT bookmarked FROM problem_state WHERE problem_id = ?1",
        [problem_id],
        |row| row.get(0),
    )?;
    Ok(value != 0)
}

pub fn get(db: &Db, problem_id: &str) -> AppResult<Option<ProblemStateRow>> {
    let conn = db.lock()?;
    let row = conn
        .query_row(
            "SELECT problem_id, status, mastered, bookmarked,
                    last_attempted_at, last_code, last_language
             FROM problem_state WHERE problem_id = ?1",
            [problem_id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

/// All rows keyed by problem id — the `list_problems` join.
pub fn get_all(db: &Db) -> AppResult<HashMap<String, ProblemStateRow>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT problem_id, status, mastered, bookmarked,
                last_attempted_at, last_code, last_language
         FROM problem_state",
    )?;
    let rows = stmt
        .query_map([], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .map(|row| (row.problem_id.clone(), row))
        .collect())
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ProblemStateRow> {
    Ok(ProblemStateRow {
        problem_id: row.get(0)?,
        status: row.get(1)?,
        mastered: row.get::<_, i64>(2)? != 0,
        bookmarked: row.get::<_, i64>(3)? != 0,
        last_attempted_at: row.get(4)?,
        last_code: row.get(5)?,
        last_language: row.get(6)?,
    })
}
