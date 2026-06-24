//! Python runner integration tests (task 0004). Run against the real
//! interpreter; every test is skipped with a visible message when `python`
//! is not on PATH so CI machines without it stay green.

mod common;

use std::time::Instant;

use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::runner;
use common::fixture_problem;

#[test]
fn correct_solution_passes_all_cases() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    return a + b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
    assert_eq!(result.passed, 3);
    assert_eq!(result.total, 3);
    assert!(result.runtime_ms.is_some());
}

#[test]
fn run_mode_excludes_hidden_cases() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    return a + b",
        false,
    )
    .unwrap();
    assert_eq!(result.total, 2);
    assert!(result.cases.iter().all(|c| !c.hidden));
}

#[test]
fn wrong_solution_fails_with_per_case_detail() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    return a - b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Fail);
    assert!(result.passed < result.total);
    let first = &result.cases[0];
    assert_eq!(first.input.as_deref(), Some("a=1, b=2"));
    assert_eq!(first.expected.as_deref(), Some("3"));
    assert_eq!(first.output.as_deref(), Some("-1"));
}

#[test]
fn raise_maps_to_error_with_traceback() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    raise ValueError('boom')",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert!(result.cases.is_empty());
    let err = result.error.unwrap();
    assert!(err.contains("ValueError: boom"), "traceback was: {err}");
    assert!(err.contains("solution.py"), "traceback was: {err}");
    assert!(
        !err.contains("harness.py"),
        "harness frames should be stripped: {err}"
    );
}

#[test]
fn infinite_loop_hits_the_timeout() {
    require_runtime!("python");
    let started = Instant::now();
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    while True:\n        pass",
        true,
    )
    .unwrap();
    let elapsed = started.elapsed();
    assert_eq!(result.status, RunStatus::Timeout);
    assert_eq!(
        result.error.as_deref(),
        Some("Time limit exceeded — execution stopped after 3000 ms.")
    );
    assert!(
        elapsed.as_millis() < 4500,
        "took {elapsed:?}, expected ≤ ~3.5s"
    );
}

#[test]
fn hidden_case_values_never_cross_ipc() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    return a + b",
        true,
    )
    .unwrap();
    let hidden = result
        .cases
        .iter()
        .find(|c| c.hidden)
        .expect("hidden case ran");
    assert!(hidden.input.is_none() && hidden.output.is_none() && hidden.expected.is_none());
    // belt and braces: the serialized payload contains no hidden values at all
    let payload = serde_json::to_string(&result).unwrap();
    assert!(!payload.contains("-7"), "hidden input leaked: {payload}");
}

#[test]
fn user_print_noise_does_not_break_parsing() {
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    print('debugging', a, b)\n    print('@@almost-sentinel')\n    return a + b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
}

#[test]
fn missing_entry_function_reports_a_friendly_error() {
    require_runtime!("python");
    let result = runner::execute(&fixture_problem(), Language::Python, "x = 41", true).unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert!(result.error.unwrap().contains("No function found"));
}
