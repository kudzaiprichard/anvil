//! Node runner integration tests (task 0005): the 0004 matrix run through
//! `node`, the no-export starter shim, the undefined→null rule, and the
//! cross-language parity guarantee. Skipped when `node` is absent.

mod common;

use std::time::Instant;

use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::runner;
use common::fixture_problem;

#[test]
fn correct_solution_passes_all_cases() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  return a + b;\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
    assert_eq!(result.passed, 3);
}

#[test]
fn starter_without_export_runs_via_the_shim() {
    require_runtime!("node");
    // exactly what the editor ships: a bare function, no module.exports
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  return a + b;\n}",
        false,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
    assert_eq!(result.total, 2);
}

#[test]
fn explicit_module_exports_also_works() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "module.exports = (a, b) => a + b;",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
}

#[test]
fn wrong_solution_fails_with_per_case_detail() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  return a - b;\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Fail);
    let first = &result.cases[0];
    assert_eq!(first.input.as_deref(), Some("a=1, b=2"));
    assert_eq!(first.output.as_deref(), Some("-1"));
}

#[test]
fn throw_maps_to_error_with_stack() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  throw new TypeError(\"boom\");\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert!(result.cases.is_empty());
    let err = result.error.unwrap();
    assert!(err.contains("TypeError: boom"), "stack was: {err}");
    assert!(
        !err.contains("harness.js"),
        "harness frames should be stripped: {err}"
    );
}

#[test]
fn infinite_loop_hits_the_timeout() {
    require_runtime!("node");
    let started = Instant::now();
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  while (true) {}\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Timeout);
    assert!(started.elapsed().as_millis() < 4500);
}

#[test]
fn undefined_return_serializes_as_null() {
    require_runtime!("node");
    // solve returns undefined → compared as null → fails against expected 3
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {}",
        false,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Fail);
    assert_eq!(result.cases[0].output.as_deref(), Some("null"));
}

#[test]
fn hidden_case_values_never_cross_ipc() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  return a + b;\n}",
        true,
    )
    .unwrap();
    let hidden = result
        .cases
        .iter()
        .find(|c| c.hidden)
        .expect("hidden case ran");
    assert!(hidden.input.is_none() && hidden.output.is_none() && hidden.expected.is_none());
    let payload = serde_json::to_string(&result).unwrap();
    assert!(!payload.contains("-7"), "hidden input leaked: {payload}");
}

#[test]
fn console_log_noise_does_not_break_parsing() {
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  console.log(\"debug\", a, b);\n  console.log(\"@@almost-sentinel\");\n  return a + b;\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
}

/// One problem through both runtimes must produce identical `RunResult`
/// JSON apart from the per-run measurements (`runtimeMs`, `memoryMb`).
#[test]
fn cross_language_parity() {
    require_runtime!("node");
    require_runtime!("python");
    let problem = fixture_problem();
    let py = runner::execute(
        &problem,
        Language::Python,
        "def solve(a, b):\n    return a + b",
        true,
    )
    .unwrap();
    let js = runner::execute(
        &problem,
        Language::Javascript,
        "function solve(a, b) {\n  return a + b;\n}",
        true,
    )
    .unwrap();
    let mut py_json = serde_json::to_value(&py).unwrap();
    let mut js_json = serde_json::to_value(&js).unwrap();
    for json in [&mut py_json, &mut js_json] {
        let obj = json.as_object_mut().unwrap();
        obj.remove("runtimeMs");
        obj.remove("memoryMb");
    }
    assert_eq!(py_json, js_json);
}
