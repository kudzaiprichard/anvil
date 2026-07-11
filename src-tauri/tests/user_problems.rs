//! User-problem persistence + store-merge tests (task 0010): save →
//! reload-from-db → listed, solvable by the runner, id/number stable across
//! edits; draft CRUD round-trip.

mod common;

use app_lib::domain::draft::{self, UserProblemDraft};
use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::db::{drafts, user_problems, Db};
use app_lib::services::runner;
use tempfile::tempdir;

fn sum_draft() -> UserProblemDraft {
    serde_json::from_value(serde_json::json!({
        "title": "Sum Of Two",
        "pattern": "Greedy",
        "difficulty": "Easy",
        "description_md": "Return the sum of two integers.",
        "constraints": ["-100 <= a, b <= 100"],
        "examples": [{ "input": "a = 1, b = 2", "output": "3" }],
        "function_signature": {
            "python": "def solve(a, b):\n    pass",
            "javascript": "function solve(a, b) {}"
        },
        "test_cases": [
            { "input": "[1, 2]", "expected": "3", "hidden": false },
            { "input": "[-4, 4]", "expected": "0", "hidden": true }
        ],
        "hints": [],
        "reference_solution": { "python": "def solve(a, b):\n    return a + b" },
        "originalityWarranty": true
    }))
    .unwrap()
}

#[test]
fn saved_problem_survives_a_store_reload_with_stable_identity() {
    let dir = tempdir().unwrap();
    let problem = draft::build_problem(&sum_draft(), "sum-of-two-42", 13).unwrap();
    {
        let db = Db::open(dir.path()).unwrap();
        user_problems::upsert(&db, &problem, "2026-06-12T10:00:00").unwrap();
    }

    // "restart": fresh connection
    let db = Db::open(dir.path()).unwrap();
    let listed = user_problems::list(&db).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], problem);

    // edit keeps id + number
    let mut edited_draft = sum_draft();
    edited_draft.title = "Sum Of Two Integers".into();
    let edited = draft::build_problem(&edited_draft, "sum-of-two-42", 13).unwrap();
    user_problems::upsert(&db, &edited, "2026-06-12T11:00:00").unwrap();
    let listed = user_problems::list(&db).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "sum-of-two-42");
    assert_eq!(listed[0].number, 13);
    assert_eq!(listed[0].title, "Sum Of Two Integers");
}

#[test]
fn built_problem_is_solvable_by_the_runner() {
    require_runtime!("python");
    let problem = draft::build_problem(&sum_draft(), "sum-of-two-42", 13).unwrap();
    // the reference solution passes all cases — exactly what
    // validate_user_problem runs
    let result = runner::execute(
        &problem,
        Language::Python,
        problem
            .reference_solution
            .as_ref()
            .unwrap()
            .python
            .as_deref()
            .unwrap(),
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
    assert_eq!(result.total, 2);

    // and a wrong user solution fails like on any built-in problem
    let wrong = runner::execute(
        &problem,
        Language::Python,
        "def solve(a, b):\n    return a - b",
        true,
    )
    .unwrap();
    assert_eq!(wrong.status, RunStatus::Fail);
}

#[test]
fn draft_crud_round_trip_survives_reopen() {
    let dir = tempdir().unwrap();
    let payload = serde_json::to_string(&sum_draft()).unwrap();
    {
        let db = Db::open(dir.path()).unwrap();
        drafts::save(&db, "draft-1", &payload, "2026-06-12T10:00:00").unwrap();
        drafts::save(&db, "draft-2", &payload, "2026-06-12T11:00:00").unwrap();
        // overwrite keeps one row per id
        drafts::save(&db, "draft-1", &payload, "2026-06-12T12:00:00").unwrap();
    }

    let db = Db::open(dir.path()).unwrap();
    let listed = drafts::list(&db).unwrap();
    assert_eq!(listed.len(), 2);
    // most recently updated first
    assert_eq!(listed[0].0, "draft-1");
    assert_eq!(listed[0].2, "2026-06-12T12:00:00");

    let loaded = drafts::get(&db, "draft-2").unwrap().unwrap();
    let parsed: UserProblemDraft = serde_json::from_str(&loaded).unwrap();
    assert_eq!(parsed.title, "Sum Of Two");

    drafts::delete(&db, "draft-1").unwrap();
    drafts::delete(&db, "draft-2").unwrap();
    assert!(drafts::list(&db).unwrap().is_empty());
    assert!(drafts::get(&db, "draft-1").unwrap().is_none());
}
