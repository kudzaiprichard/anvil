//! Entry-point resolution integration tests (task 0002): verbatim
//! LeetCode-style stubs must run unmodified, driven by `entry_point`;
//! problems without one keep the legacy `solve` behavior. Runtime-gated
//! like all sandbox tests.

mod common;

use app_lib::domain::problem::EntryPoint;
use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::runner;
use common::fixture_problem;

/// The add-two fixture re-shaped as a LeetCode-style problem: same cases,
/// but solved by `Solution.addTwo` (Python) / `addTwo` (JavaScript).
fn entry_point_problem() -> app_lib::domain::problem::Problem {
    let mut p = fixture_problem();
    p.entry_point = Some(EntryPoint {
        python: "Solution.addTwo".into(),
        javascript: "addTwo".into(),
        arity: 2,
        io_types: None,
    });
    p
}

#[test]
fn python_class_solution_stub_runs_unmodified() {
    require_runtime!("python");
    // Verbatim LeetCode stub shape, filled in — no rewriting.
    let code =
        "class Solution:\n    def addTwo(self, a: int, b: int) -> int:\n        return a + b\n";
    let result = runner::execute(&entry_point_problem(), Language::Python, code, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
    assert_eq!(result.passed, 3);
}

#[test]
fn javascript_var_function_stub_runs_unmodified() {
    require_runtime!("node");
    let code = "/**\n * @param {number} a\n * @param {number} b\n * @return {number}\n */\nvar addTwo = function(a, b) {\n    return a + b;\n};";
    let result = runner::execute(&entry_point_problem(), Language::Javascript, code, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
    assert_eq!(result.passed, 3);
}

#[test]
fn javascript_const_arrow_and_function_decl_forms_resolve() {
    require_runtime!("node");
    for code in [
        "const addTwo = (a, b) => a + b;",
        "function addTwo(a, b) { return a + b; }",
        "let addTwo = function(a, b) { return a + b; };",
    ] {
        let result =
            runner::execute(&entry_point_problem(), Language::Javascript, code, true).unwrap();
        let Some(result) = common::skip_if_node_unavailable(result) else {
            continue;
        };
        assert_eq!(result.status, RunStatus::Pass, "{code}: {:?}", result.error);
    }
}

#[test]
fn python_bare_function_entry_point_resolves() {
    require_runtime!("python");
    let mut p = fixture_problem();
    p.entry_point = Some(EntryPoint {
        python: "add_two".into(),
        javascript: "addTwo".into(),
        arity: 2,
        io_types: None,
    });
    let result = runner::execute(
        &p,
        Language::Python,
        "def add_two(a, b):\n    return a + b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

#[test]
fn legacy_solve_convention_is_untouched() {
    require_runtime!("python");
    // No entry point on the problem ⇒ `solve` resolution as before, even
    // when a class is also present.
    let code = "class Unrelated:\n    pass\n\ndef solve(a, b):\n    return a + b";
    let result = runner::execute(&fixture_problem(), Language::Python, code, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

#[test]
fn missing_python_entry_point_reports_what_was_looked_for() {
    require_runtime!("python");
    // Class present but the method is missing.
    let code = "class Solution:\n    def somethingElse(self):\n        return 0";
    let result = runner::execute(&entry_point_problem(), Language::Python, code, true).unwrap();
    assert_eq!(result.status, RunStatus::Error);
    let err = result.error.unwrap();
    assert!(err.contains("Solution.addTwo"), "error was: {err}");
    assert!(err.contains("addTwo"), "error was: {err}");
    assert!(!err.contains("Traceback"), "traceback soup: {err}");

    // Class itself missing.
    let result = runner::execute(
        &entry_point_problem(),
        Language::Python,
        "def unrelated():\n    return 0",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert!(result.error.unwrap().contains("Solution"));
}

#[test]
fn missing_javascript_entry_point_reports_what_was_looked_for() {
    require_runtime!("node");
    let result = runner::execute(
        &entry_point_problem(),
        Language::Javascript,
        "var somethingElse = function(a, b) { return a + b; };",
        true,
    )
    .unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Error);
    let err = result.error.unwrap();
    assert!(err.contains("addTwo"), "error was: {err}");
}
