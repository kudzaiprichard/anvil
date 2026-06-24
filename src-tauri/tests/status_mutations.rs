//! Status-mutation rules (task 0009) at the db-service level: bookmark
//! toggling, mastered flag persistence, review transitions.

use app_lib::services::db::{attempts, problem_state, Db};
use tempfile::tempdir;

fn solve(db: &Db, problem_id: &str) {
    attempts::record_attempt(
        db,
        &attempts::AttemptRecord {
            problem_id,
            language: "python",
            kind: "submit",
            status: "pass",
            runtime_ms: Some(5),
            code: "def solve(): pass",
            attempted_at: "2026-06-12T10:00:00",
        },
    )
    .unwrap();
}

#[test]
fn bookmark_toggles_and_persists_across_reopen() {
    let dir = tempdir().unwrap();
    {
        let db = Db::open(dir.path()).unwrap();
        assert!(problem_state::toggle_bookmark(&db, "p1").unwrap());
        assert!(!problem_state::toggle_bookmark(&db, "p1").unwrap());
        assert!(problem_state::toggle_bookmark(&db, "p1").unwrap());
    }
    let db = Db::open(dir.path()).unwrap();
    assert!(problem_state::get(&db, "p1").unwrap().unwrap().bookmarked);
}

#[test]
fn mastered_survives_later_attempts() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    solve(&db, "p1");
    problem_state::set_mastered(&db, "p1", true).unwrap();

    // a later failing submit keeps both solved and mastered
    attempts::record_attempt(
        &db,
        &attempts::AttemptRecord {
            problem_id: "p1",
            language: "python",
            kind: "submit",
            status: "fail",
            runtime_ms: Some(5),
            code: "def solve(): pass",
            attempted_at: "2026-06-12T11:00:00",
        },
    )
    .unwrap();
    let row = problem_state::get(&db, "p1").unwrap().unwrap();
    assert_eq!(row.status, "solved");
    assert!(row.mastered);
}

#[test]
fn review_round_trip() {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path()).unwrap();
    solve(&db, "p1");
    problem_state::set_status(&db, "p1", "needs-review").unwrap();
    assert_eq!(
        problem_state::get(&db, "p1").unwrap().unwrap().status,
        "needs-review"
    );
    problem_state::set_status(&db, "p1", "solved").unwrap();
    assert_eq!(
        problem_state::get(&db, "p1").unwrap().unwrap().status,
        "solved"
    );
}
