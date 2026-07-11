//! SQLite persistence tests (task 0008): migration idempotence, attempt →
//! state transitions, and the recorded history queries — all against a
//! temp-dir database.

use app_lib::services::db::{attempts, problem_state, Db};
use tempfile::tempdir;

fn record(
    db: &Db,
    problem_id: &str,
    kind: &str,
    status: &str,
    attempted_at: &str,
) -> Result<(), app_lib::error::AppError> {
    attempts::record_attempt(
        db,
        &attempts::AttemptRecord {
            problem_id,
            language: "python",
            kind,
            status,
            runtime_ms: Some(12),
            code: "def solve(): pass",
            attempted_at,
        },
    )
}

#[test]
fn migrations_are_idempotent_across_reopens() {
    let dir = tempdir().unwrap();
    {
        let db = Db::open(dir.path()).unwrap();
        record(&db, "p1", "submit", "pass", "2026-06-10T10:00:00").unwrap();
    }
    // second open re-runs migrate() against the same file — must not fail
    // or wipe data
    let db = Db::open(dir.path()).unwrap();
    let days = attempts::pass_days(&db).unwrap();
    assert_eq!(days, vec![("2026-06-10".to_string(), 1)]);
}

#[test]
fn attempt_state_transitions_follow_the_rules() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();

    // fresh problem + failing run → in-progress
    record(&db, "p1", "run", "fail", "2026-06-10T10:00:00").unwrap();
    let row = problem_state::get(&db, "p1").unwrap().unwrap();
    assert_eq!(row.status, "in-progress");
    assert_eq!(
        row.last_attempted_at.as_deref(),
        Some("2026-06-10T10:00:00")
    );
    assert_eq!(row.last_language.as_deref(), Some("python"));

    // passing RUN does not solve — only submits count
    record(&db, "p1", "run", "pass", "2026-06-10T10:05:00").unwrap();
    assert_eq!(
        problem_state::get(&db, "p1").unwrap().unwrap().status,
        "in-progress"
    );

    // passing submit → solved
    record(&db, "p1", "submit", "pass", "2026-06-10T10:10:00").unwrap();
    assert_eq!(
        problem_state::get(&db, "p1").unwrap().unwrap().status,
        "solved"
    );

    // later failing submit never downgrades solved
    record(&db, "p1", "submit", "fail", "2026-06-11T09:00:00").unwrap();
    let row = problem_state::get(&db, "p1").unwrap().unwrap();
    assert_eq!(row.status, "solved");
    assert_eq!(
        row.last_attempted_at.as_deref(),
        Some("2026-06-11T09:00:00")
    );
}

#[test]
fn erroring_and_timeout_submits_mark_in_progress_not_solved() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    record(&db, "p2", "submit", "error", "2026-06-10T10:00:00").unwrap();
    record(&db, "p2", "submit", "timeout", "2026-06-10T11:00:00").unwrap();
    assert_eq!(
        problem_state::get(&db, "p2").unwrap().unwrap().status,
        "in-progress"
    );
}

#[test]
fn pass_days_groups_by_calendar_day() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    record(&db, "p1", "submit", "pass", "2026-06-10T10:00:00").unwrap();
    record(&db, "p2", "submit", "pass", "2026-06-10T18:00:00").unwrap();
    record(&db, "p3", "submit", "pass", "2026-06-12T08:00:00").unwrap();
    record(&db, "p4", "submit", "fail", "2026-06-11T08:00:00").unwrap(); // fails don't count
    record(&db, "p5", "run", "pass", "2026-06-11T08:00:00").unwrap(); // runs don't count
    assert_eq!(
        attempts::pass_days(&db).unwrap(),
        vec![("2026-06-10".to_string(), 2), ("2026-06-12".to_string(), 1)]
    );
}

#[test]
fn first_solve_days_take_each_problems_earliest_pass() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    record(&db, "p1", "submit", "pass", "2026-06-10T10:00:00").unwrap();
    record(&db, "p1", "submit", "pass", "2026-06-12T10:00:00").unwrap(); // re-solve ignored
    record(&db, "p2", "submit", "pass", "2026-06-11T10:00:00").unwrap();
    assert_eq!(
        attempts::first_solve_days(&db).unwrap(),
        vec!["2026-06-10".to_string(), "2026-06-11".to_string()]
    );
}

#[test]
fn get_all_returns_every_state_row() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    record(&db, "p1", "submit", "pass", "2026-06-10T10:00:00").unwrap();
    record(&db, "p2", "run", "fail", "2026-06-11T10:00:00").unwrap();
    let all = problem_state::get_all(&db).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all["p1"].status, "solved");
    assert_eq!(all["p2"].status, "in-progress");
}
