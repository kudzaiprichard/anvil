//! Mastery-gate persistence (Phase 3). Two tables back the progression engine:
//! `gate_solve` (0004) is the fine-grained evidence — one row per gate problem
//! cleared hint-free — and `unit_mastery` (0003) holds the derived per-unit
//! verdict (`mastered` + attempt counter). The connection never leaves this
//! module tree (see `mod.rs`): the engine in `services::progression` calls
//! these typed helpers and only ever sees domain data.

use std::collections::HashSet;

use rusqlite::params;

use super::Db;
use crate::error::AppResult;

/// Records a gate problem cleared hint-free & no-peek. Idempotent: re-solving
/// the same problem keeps the first `solved_at` and never double-counts, so the
/// engine's `passed_count` tally stays honest. Only called for solves that
/// actually count (a peeked attempt is discarded before it reaches here).
pub fn record_gate_solve(
    db: &Db,
    unit_id: &str,
    problem_id: &str,
    novel: bool,
    now: &str,
) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO gate_solve (unit_id, problem_id, novel, solved_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(unit_id, problem_id) DO NOTHING",
        params![unit_id, problem_id, novel as i64, now],
    )?;
    Ok(())
}

/// Distinct gate problems solved hint-free for a unit, as `(problem_id, novel)`.
pub fn gate_solves(db: &Db, unit_id: &str) -> AppResult<Vec<(String, bool)>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT problem_id, novel FROM gate_solve WHERE unit_id = ?1 ORDER BY solved_at, problem_id",
    )?;
    let rows = stmt
        .query_map([unit_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Bumps a unit's lifetime gate-attempt counter (every evaluate_gate call,
/// counted or not). Creates the `unit_mastery` row on first touch.
pub fn bump_gate_attempts(db: &Db, unit_id: &str) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO unit_mastery (unit_id, status, gate_attempts)
         VALUES (?1, 'unlocked', 1)
         ON CONFLICT(unit_id) DO UPDATE SET gate_attempts = unit_mastery.gate_attempts + 1",
        params![unit_id],
    )?;
    Ok(())
}

/// Marks a unit mastered — the durable record that its gate was passed. Stamps
/// `mastered_at` once and mirrors the final gate tally into the counters.
pub fn mark_mastered(
    db: &Db,
    unit_id: &str,
    passed_count: u32,
    passed_novel: u32,
    now: &str,
) -> AppResult<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO unit_mastery
           (unit_id, status, gate_passed_count, gate_passed_novel, mastered_at)
         VALUES (?1, 'mastered', ?2, ?3, ?4)
         ON CONFLICT(unit_id) DO UPDATE SET
           status = 'mastered',
           gate_passed_count = excluded.gate_passed_count,
           gate_passed_novel = excluded.gate_passed_novel,
           mastered_at = COALESCE(unit_mastery.mastered_at, excluded.mastered_at)",
        params![unit_id, passed_count, passed_novel, now],
    )?;
    Ok(())
}

/// The set of units whose gate has been passed — drives unlocking (a unit is
/// unlocked once all its prereqs are in this set).
pub fn mastered_units(db: &Db) -> AppResult<HashSet<String>> {
    let conn = db.lock()?;
    let mut stmt =
        conn.prepare("SELECT unit_id FROM unit_mastery WHERE status = 'mastered'")?;
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
    fn gate_solve_is_idempotent_per_problem() {
        let (_dir, db) = temp_db();
        record_gate_solve(&db, "u1", "p1", true, "T1").unwrap();
        // Re-solving the same problem must not add a second row.
        record_gate_solve(&db, "u1", "p1", true, "T2").unwrap();
        record_gate_solve(&db, "u1", "p2", false, "T3").unwrap();

        let mut solves = gate_solves(&db, "u1").unwrap();
        solves.sort();
        assert_eq!(
            solves,
            vec![("p1".to_string(), true), ("p2".to_string(), false)]
        );
    }

    #[test]
    fn mastered_units_reflects_mark_mastered() {
        let (_dir, db) = temp_db();
        assert!(mastered_units(&db).unwrap().is_empty());
        bump_gate_attempts(&db, "u1").unwrap();
        mark_mastered(&db, "u1", 2, 1, "T1").unwrap();
        let set = mastered_units(&db).unwrap();
        assert!(set.contains("u1"));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn mastered_at_is_stamped_once() {
        let (_dir, db) = temp_db();
        mark_mastered(&db, "u1", 1, 1, "T1").unwrap();
        mark_mastered(&db, "u1", 2, 1, "T2").unwrap();
        let conn = db.lock().unwrap();
        let at: String = conn
            .query_row(
                "SELECT mastered_at FROM unit_mastery WHERE unit_id = 'u1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(at, "T1");
    }
}
