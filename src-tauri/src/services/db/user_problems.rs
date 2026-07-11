//! User-authored problems (spec §8.2: just another problem record). The
//! full `Problem` is stored as JSON; `number` is duplicated as a column for
//! cheap max-number queries.

use rusqlite::params;

use super::Db;
use crate::domain::problem::Problem;
use crate::error::{AppError, AppResult};

pub fn upsert(db: &Db, problem: &Problem, now: &str) -> AppResult<()> {
    let json = serde_json::to_string(problem)
        .map_err(|e| AppError::Storage(format!("failed to encode problem: {e}")))?;
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO user_problems (id, number, json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(id) DO UPDATE SET
           json = excluded.json,
           updated_at = excluded.updated_at",
        params![problem.id, problem.number, json, now],
    )?;
    Ok(())
}

pub fn list(db: &Db) -> AppResult<Vec<Problem>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT json FROM user_problems ORDER BY number")?;
    let rows: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    rows.iter()
        .map(|json| {
            serde_json::from_str(json)
                .map_err(|e| AppError::Storage(format!("corrupt user problem record: {e}")))
        })
        .collect()
}
