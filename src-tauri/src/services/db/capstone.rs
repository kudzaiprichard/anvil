//! Stage-7 mixed-capstone persistence (Phase 7). One row per capstone problem
//! cleared hint-free. The connection never leaves this module tree (see
//! `mod.rs`): the engine in `services::advancement` calls these typed helpers.

use std::collections::HashSet;

use rusqlite::params;

use super::Db;
use crate::error::AppResult;

/// Records a capstone problem cleared hint-free & no-peek. Idempotent: re-solving
/// keeps the first `solved_at` and never double-counts.
pub fn record_solve(db: &Db, problem_id: &str, now: &str) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO capstone_solve (problem_id, solved_at)
         VALUES (?1, ?2)
         ON CONFLICT(problem_id) DO NOTHING",
        params![problem_id, now],
    )?;
    Ok(())
}

/// The set of capstone problems cleared hint-free.
pub fn solved(db: &Db) -> AppResult<HashSet<String>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT problem_id FROM capstone_solve")?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<HashSet<_>, _>>()?;
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

    #[test]
    fn capstone_solve_is_idempotent() {
        let (_dir, db) = temp_db();
        record_solve(&db, "two-sum", "T1").unwrap();
        record_solve(&db, "two-sum", "T2").unwrap();
        record_solve(&db, "number-of-islands", "T3").unwrap();
        assert_eq!(solved(&db).unwrap().len(), 2);
        assert!(solved(&db).unwrap().contains("two-sum"));
    }
}
