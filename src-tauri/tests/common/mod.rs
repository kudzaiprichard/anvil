//! Shared helpers for runner integration tests. Each test target compiles
//! this independently, so unused items in one target are expected.
#![allow(dead_code)]

use app_lib::domain::problem::{
    Difficulty, FunctionSignature, Pattern, Problem, ProblemSource, TestCase,
};

pub fn runtime_available(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// True when a stress-materialization skip list is non-empty and *every* skip
/// is a sandbox execution failure (`AppError::Runner`, which Displays as
/// `"runner error: ..."` — e.g. a CI thread/process cap surfacing as
/// "can't start new thread"), rather than a genuine generator/materialization
/// bug. Lets the sandbox-dependent stress tests skip on an environment limit
/// without masking a real regression (a non-runner skip returns `false`, so the
/// caller's `assert!` still fires).
pub fn stress_skipped_by_sandbox(skipped: &[String]) -> bool {
    !skipped.is_empty() && skipped.iter().all(|s| s.contains("runner error"))
}

/// True when an error message is the CI sandbox's thread/process cap
/// surfacing as a Python `RuntimeError: can't start new thread` — the same
/// environment limit as [`stress_skipped_by_sandbox`], but matched directly
/// against one error string (e.g. a `ProbeOutcome::Failed` payload) rather
/// than a skip list.
pub fn is_sandbox_thread_limit(msg: &str) -> bool {
    msg.contains("can't start new thread")
}

#[macro_export]
macro_rules! require_runtime {
    ($program:literal) => {
        if !common::runtime_available($program) {
            eprintln!(concat!("SKIPPED: ", $program, " not found on PATH"));
            return;
        }
    };
}

/// Two visible cases + one hidden case of an "add two ints" fixture.
pub fn fixture_problem() -> Problem {
    Problem {
        id: "fixture-add-two".into(),
        number: 1,
        title: "Add Two Fixture".into(),
        pattern: Pattern("Greedy".into()),
        difficulty: Difficulty::Easy,
        source: ProblemSource::BuiltIn,
        description_md: "Test fixture: return the sum of two integers.".into(),
        body_html: None,
        constraints: vec![],
        examples: vec![],
        function_signature: FunctionSignature {
            python: "def solve(a, b):\n    pass".into(),
            javascript: "function solve(a, b) {}".into(),
            extra: Default::default(),
        },
        checker: app_lib::domain::problem::Checker::Exact,
        judge: None,
        entry_point: None,
        test_cases: vec![
            TestCase {
                input: vec![1.into(), 2.into()],
                expected: 3.into(),
                hidden: false,
            },
            TestCase {
                input: vec![5.into(), 5.into()],
                expected: 10.into(),
                hidden: false,
            },
            TestCase {
                input: vec![(-7).into(), 7.into()],
                expected: 0.into(),
                hidden: true,
            },
        ],
        hints: vec![],
        reference_solution: None,
        explanation_md: None,
        follow_up: None,
        license: "project-default".into(),
        author: "built-in".into(),
    }
}
