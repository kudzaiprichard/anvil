//! Create-page drafts: the raw `UserProblemDraft` JSON, keyed by a local
//! draft id. Nothing here is a problem yet — saving a draft does NOT touch
//! the library.

use rusqlite::{params, OptionalExtension};

use super::Db;
use crate::error::AppResult;

pub fn save(db: &Db, id: &str, json: &str, now: &str) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO drafts (id, json, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET json = excluded.json, updated_at = excluded.updated_at",
        params![id, json, now],
    )?;
    Ok(())
}

/// `(id, json, updated_at)`, most recently updated first.
pub fn list(db: &Db) -> AppResult<Vec<(String, String, String)>> {
    let conn = db.lock()?;
    let mut stmt =
        conn.prepare("SELECT id, json, updated_at FROM drafts ORDER BY updated_at DESC")?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<String>> {
    let conn = db.lock()?;
    let row = conn
        .query_row("SELECT json FROM drafts WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()?;
    Ok(row)
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM drafts WHERE id = ?1", [id])?;
    Ok(())
}
