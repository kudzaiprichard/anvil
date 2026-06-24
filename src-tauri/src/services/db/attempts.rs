//! Attempt history: every run/submit lands here. Read side feeds progress
//! (streaks) and the dashboard (heatmap, cumulative series). The combined
//! attempt + problem-state write for a finished run is `record_attempt` —
//! one transaction, so history and state can never disagree.

use rusqlite::params;

use super::{problem_state, Db};
use crate::error::AppResult;

/// Everything recorded when a run/submit finishes.
pub struct AttemptRecord<'a> {
    pub problem_id: &'a str,
    pub language: &'a str,
    /// `"run"` or `"submit"`.
    pub kind: &'a str,
    /// `"pass" | "fail" | "error" | "timeout"`.
    pub status: &'a str,
    pub runtime_ms: Option<u64>,
    pub code: &'a str,
    /// Local ISO timestamp (`db::now_local_iso()`); injected for tests.
    pub attempted_at: &'a str,
}

/// Inserts the attempt row and applies the state transition atomically.
pub fn record_attempt(db: &Db, rec: &AttemptRecord) -> AppResult<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO attempts (problem_id, language, kind, status, runtime_ms, attempted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            rec.problem_id,
            rec.language,
            rec.kind,
            rec.status,
            rec.runtime_ms,
            rec.attempted_at
        ],
    )?;
    problem_state::apply_attempt(
        &tx,
        rec.problem_id,
        rec.kind == "submit" && rec.status == "pass",
        rec.code,
        rec.language,
        rec.attempted_at,
    )?;
    tx.commit()?;
    Ok(())
}

/// Distinct calendar days with ≥1 passing submit, with counts, ascending —
/// drives streaks (0008) and the heatmap (0009).
pub fn pass_days(db: &Db) -> AppResult<Vec<(String, u32)>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT substr(attempted_at, 1, 10) AS day, COUNT(*)
         FROM attempts
         WHERE kind = 'submit' AND status = 'pass'
         GROUP BY day ORDER BY day",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Day of each problem's FIRST passing submit, ascending — the cumulative
/// solve curve (0009).
pub fn first_solve_days(db: &Db) -> AppResult<Vec<String>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT MIN(substr(attempted_at, 1, 10)) AS day
         FROM attempts
         WHERE kind = 'submit' AND status = 'pass'
         GROUP BY problem_id ORDER BY day",
    )?;
    let rows = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
