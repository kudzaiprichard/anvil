//! Runner orchestration (BACKEND_PLAN §6): fresh temp dir → write solution +
//! harness + cases.json → spawn the runtime with piped stdio → enforce the
//! whole-run timeout → parse sentinel lines → compare against expected
//! values → assemble `RunResult` in exactly the shape the UI renders.
//!
//! Privacy invariant: display strings (`input`/`expected`/`output`) are
//! populated for VISIBLE cases only — hidden cases cross IPC as pass/fail
//! flags and nothing else. Never log user code or hidden expectations.

pub mod node;
pub mod python;
pub mod sandbox;

use std::process::Command;
use std::time::Duration;

use serde::Deserialize;

use crate::domain::problem::{Judge, Problem, TestCase};
use crate::domain::run::{CaseResult, Language, RunResult, RunStatus};
use crate::error::{AppError, AppResult};
use sandbox::{Guards, WaitOutcome};

/// Harness lines start with this so user print noise never breaks parsing.
const SENTINEL: &str = "@@ANVIL@@";

/// Everything that differs between runtimes. Adding a language = one new
/// spec + one harness file (task 0005's abstraction; no traits needed).
pub struct LanguageSpec {
    pub solution_filename: &'static str,
    pub harness_filename: &'static str,
    pub harness_source: &'static str,
    pub program: &'static str,
    pub args: &'static [&'static str],
    /// Appended to the user's file on disk (editor content untouched);
    /// lets the JS harness find an unexported `function solve`.
    pub solution_suffix: &'static str,
}

fn spec_for(language: Language) -> &'static LanguageSpec {
    match language {
        Language::Python => &python::SPEC,
        Language::Javascript => &node::SPEC,
    }
}

/// One parsed sentinel line from the harness.
#[derive(Deserialize)]
struct HarnessLine {
    index: u32,
    ok: bool,
    #[serde(default)]
    output: serde_json::Value,
    #[serde(default, rename = "timeMs")]
    time_ms: Option<f64>,
    #[serde(default)]
    traceback: Option<String>,
    /// `any_valid` mode only: the pack validator's verdict for this case.
    #[serde(default)]
    valid: Option<bool>,
}

/// Builds the optional `meta.json` the harness reads next to `cases.json`
/// (CONTENT_DESIGN.md §4–5). `None` means legacy behavior — no file is
/// written, so built-in problems run exactly as before.
fn harness_meta(problem: &Problem, language: Language, judge: &Judge) -> Option<serde_json::Value> {
    let mut meta = serde_json::Map::new();
    if let Some(ep) = &problem.entry_point {
        let name = match language {
            Language::Python => &ep.python,
            Language::Javascript => &ep.javascript,
        };
        meta.insert("entry_point".into(), serde_json::json!(name));
        if let Some(io) = &ep.io_types {
            meta.insert("io_types".into(), serde_json::json!(io));
        }
    }
    match judge {
        Judge::InPlace { arg_index } => {
            meta.insert("mode".into(), serde_json::json!("in_place"));
            meta.insert("arg_index".into(), serde_json::json!(arg_index));
        }
        Judge::Design => {
            meta.insert("mode".into(), serde_json::json!("design"));
        }
        Judge::AnyValid { .. } => {
            meta.insert("mode".into(), serde_json::json!("any_valid"));
            let file = match language {
                Language::Python => "validator.py",
                Language::Javascript => "validator.js",
            };
            meta.insert("validator_file".into(), serde_json::json!(file));
        }
        Judge::Exact | Judge::Unordered | Judge::Float { .. } => {}
    }
    if meta.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(meta))
    }
}

/// Runs with the spec's bare program name (PATH lookup) — used by tests and
/// as the fallback when detection hasn't resolved a path.
pub fn execute(
    problem: &Problem,
    language: Language,
    code: &str,
    include_hidden: bool,
) -> AppResult<RunResult> {
    let program = spec_for(language).program;
    execute_with_program(problem, language, code, include_hidden, program)
}

/// Runs with an explicit interpreter path (task 0007: the detection cache).
pub fn execute_with_program(
    problem: &Problem,
    language: Language,
    code: &str,
    include_hidden: bool,
    program: &str,
) -> AppResult<RunResult> {
    let spec = spec_for(language);
    let selected: Vec<&TestCase> = problem
        .test_cases
        .iter()
        .filter(|tc| include_hidden || !tc.hidden)
        .collect();
    let total = selected.len() as u32;

    // RAII guard: the temp dir is removed on every exit path, kill included.
    let dir = tempfile::tempdir()?;
    let mut solution = code.to_string();
    solution.push_str(spec.solution_suffix);
    std::fs::write(dir.path().join(spec.solution_filename), solution)?;
    std::fs::write(dir.path().join(spec.harness_filename), spec.harness_source)?;
    let judge = problem.effective_judge();
    if let Some(meta) = harness_meta(problem, language, &judge) {
        std::fs::write(dir.path().join("meta.json"), meta.to_string())?;
    }
    if let Judge::AnyValid {
        validator_python,
        validator_javascript,
    } = &judge
    {
        // Pack-shipped validator — our code, never anything from the
        // imported file (CONTENT_DESIGN.md §4).
        let (file, mut source) = match language {
            Language::Python => ("validator.py", validator_python.clone()),
            Language::Javascript => ("validator.js", validator_javascript.clone()),
        };
        if language == Language::Javascript {
            // Same shim trick as solution.js: a bare `function validate`
            // needs no explicit export.
            source.push_str(
                "\nmodule.exports = typeof validate !== \"undefined\" ? validate : module.exports;\n",
            );
        }
        std::fs::write(dir.path().join(file), source)?;
    }
    let cases_json: Vec<serde_json::Value> = selected
        .iter()
        .enumerate()
        .map(|(i, tc)| serde_json::json!({ "index": i + 1, "args": tc.input }))
        .collect();
    std::fs::write(
        dir.path().join("cases.json"),
        serde_json::to_string(&cases_json)
            .map_err(|e| AppError::Runner(format!("failed to encode cases: {e}")))?,
    )?;

    let guards = Guards::default();
    let mut cmd = Command::new(program);
    cmd.args(spec.args)
        .arg(spec.harness_filename)
        .current_dir(dir.path());
    let child = sandbox::spawn_guarded(cmd, &guards)?;

    let (exit_status, stdout, stderr, peak_memory_bytes) =
        match child.wait_with_timeout(Duration::from_millis(guards.timeout_ms))? {
            WaitOutcome::TimedOut => {
                return Ok(RunResult {
                    status: RunStatus::Timeout,
                    cases: vec![],
                    passed: 0,
                    total,
                    runtime_ms: None,
                    memory_mb: None,
                    error: Some(format!(
                        "Time limit exceeded — execution stopped after {} ms.",
                        guards.timeout_ms
                    )),
                });
            }
            WaitOutcome::MemoryKilled => {
                return Ok(error_result(total, memory_limit_message(&guards)));
            }
            WaitOutcome::Exited {
                status,
                stdout,
                stderr,
                peak_memory_bytes,
            } => (status, stdout, stderr, peak_memory_bytes),
        };

    let lines: Vec<HarnessLine> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(SENTINEL))
        .filter_map(|rest| match serde_json::from_str::<HarnessLine>(rest) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                log::warn!("unparseable harness line: {e}");
                None
            }
        })
        .collect();

    // Structured runtime error from the harness (exception, bad entry fn…).
    if let Some(err_line) = lines.iter().find(|l| !l.ok) {
        let traceback = err_line
            .traceback
            .clone()
            .unwrap_or_else(|| "Unknown runtime error.".into());
        // A MemoryError traceback is the memory cap manifesting inside the
        // interpreter — surface the friendly guard message instead.
        let message = if sandbox::is_memory_abort(&traceback) {
            memory_limit_message(&guards)
        } else {
            traceback
        };
        return Ok(error_result(total, message));
    }
    // The interpreter died without a structured error (startup crash, OOM…).
    if !exit_status.success() || lines.len() != selected.len() {
        let message = if sandbox::is_memory_abort(&stderr) {
            memory_limit_message(&guards)
        } else if stderr.trim().is_empty() {
            format!("{program} exited unexpectedly ({exit_status})")
        } else {
            stderr.trim().to_string()
        };
        return Ok(error_result(total, message));
    }

    let params = python_param_names(&problem.function_signature.python);
    let mut cases = Vec::with_capacity(selected.len());
    let mut passed = 0u32;
    let mut runtime_ms = 0f64;
    for (i, tc) in selected.iter().enumerate() {
        let index = (i + 1) as u32;
        let line = match lines.iter().find(|l| l.index == index) {
            Some(line) => line,
            None => {
                return Ok(error_result(
                    total,
                    format!("missing result for case {index}"),
                ))
            }
        };
        let case_passed = case_passes(&judge, line, &tc.expected);
        runtime_ms += line.time_ms.unwrap_or(0.0);
        let mut case = CaseResult {
            index,
            hidden: tc.hidden,
            passed: case_passed,
            input: None,
            output: None,
            expected: None,
            error: None,
        };
        if !tc.hidden {
            case.input = Some(fmt_input(&params, &tc.input));
            case.expected = Some(fmt_value(&tc.expected));
            case.output = Some(fmt_value(&line.output));
        }
        if case_passed {
            passed += 1;
        }
        cases.push(case);
    }

    Ok(RunResult {
        status: if passed == total {
            RunStatus::Pass
        } else {
            RunStatus::Fail
        },
        cases,
        passed,
        total,
        runtime_ms: Some(runtime_ms.round() as u64),
        // Job Object peak accounting on Windows; None on Unix in v1.
        memory_mb: peak_memory_bytes.map(|b| (b as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0),
        error: None,
    })
}

/// Executes `code` against raw positional-arg inputs and returns each
/// case's raw JSON output in order — no comparison, no display formatting.
/// Used at import time to materialize stress generators and compute
/// expected values by execution (task 0008, CONTENT_DESIGN.md §7); it only
/// ever runs our own pack-shipped code, never user code. `AnyValid` is
/// executed as a plain call (no validator — there is nothing to judge when
/// the point is to *produce* the reference output).
pub fn compute_outputs(
    language: Language,
    code: &str,
    entry_point: Option<&str>,
    judge: &Judge,
    inputs: &[Vec<serde_json::Value>],
    program: &str,
    io_types: Option<&crate::domain::problem::IoTypes>,
) -> AppResult<Vec<serde_json::Value>> {
    let spec = spec_for(language);
    let dir = tempfile::tempdir()?;
    let mut solution = code.to_string();
    solution.push_str(spec.solution_suffix);
    std::fs::write(dir.path().join(spec.solution_filename), solution)?;
    std::fs::write(dir.path().join(spec.harness_filename), spec.harness_source)?;

    let mut meta = serde_json::Map::new();
    if let Some(name) = entry_point {
        meta.insert("entry_point".into(), serde_json::json!(name));
    }
    if let Some(io) = io_types {
        meta.insert("io_types".into(), serde_json::json!(io));
    }
    match judge {
        Judge::InPlace { arg_index } => {
            meta.insert("mode".into(), serde_json::json!("in_place"));
            meta.insert("arg_index".into(), serde_json::json!(arg_index));
        }
        Judge::Design => {
            meta.insert("mode".into(), serde_json::json!("design"));
        }
        _ => {}
    }
    if !meta.is_empty() {
        std::fs::write(
            dir.path().join("meta.json"),
            serde_json::Value::Object(meta).to_string(),
        )?;
    }

    let cases_json: Vec<serde_json::Value> = inputs
        .iter()
        .enumerate()
        .map(|(i, args)| serde_json::json!({ "index": i + 1, "args": args }))
        .collect();
    std::fs::write(
        dir.path().join("cases.json"),
        serde_json::to_string(&cases_json)
            .map_err(|e| AppError::Runner(format!("failed to encode cases: {e}")))?,
    )?;

    let guards = Guards::default();
    let mut cmd = Command::new(program);
    cmd.args(spec.args)
        .arg(spec.harness_filename)
        .current_dir(dir.path());
    let child = sandbox::spawn_guarded(cmd, &guards)?;

    let (exit_status, stdout, stderr) = match child
        .wait_with_timeout(Duration::from_millis(guards.timeout_ms))?
    {
        WaitOutcome::TimedOut => {
            return Err(AppError::Runner(format!(
                "time limit exceeded after {} ms",
                guards.timeout_ms
            )))
        }
        WaitOutcome::MemoryKilled => return Err(AppError::Runner(memory_limit_message(&guards))),
        WaitOutcome::Exited {
            status,
            stdout,
            stderr,
            ..
        } => (status, stdout, stderr),
    };

    let lines: Vec<HarnessLine> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(SENTINEL))
        .filter_map(|rest| serde_json::from_str::<HarnessLine>(rest).ok())
        .collect();
    if let Some(err_line) = lines.iter().find(|l| !l.ok) {
        return Err(AppError::Runner(
            err_line
                .traceback
                .clone()
                .unwrap_or_else(|| "unknown runtime error".into()),
        ));
    }
    if !exit_status.success() || lines.len() != inputs.len() {
        let detail = if stderr.trim().is_empty() {
            format!("{program} exited unexpectedly ({exit_status})")
        } else {
            stderr.trim().to_string()
        };
        return Err(AppError::Runner(detail));
    }

    (1..=inputs.len() as u32)
        .map(|index| {
            lines
                .iter()
                .find(|l| l.index == index)
                .map(|l| l.output.clone())
                .ok_or_else(|| AppError::Runner(format!("missing result for case {index}")))
        })
        .collect()
}

fn memory_limit_message(guards: &Guards) -> String {
    format!(
        "Memory limit exceeded ({} MB) — execution stopped.",
        guards.memory_bytes / (1024 * 1024)
    )
}

/// Per-case verdict honoring the problem's judge (CONTENT_DESIGN.md §4).
/// `in_place` and `design` compare exactly — the harness already emitted
/// the mutated argument / collected op outputs as the case output.
/// `any_valid` trusts the pack validator's verdict from the harness line.
fn case_passes(judge: &Judge, line: &HarnessLine, expected: &serde_json::Value) -> bool {
    match judge {
        Judge::Exact | Judge::InPlace { .. } | Judge::Design => line.output == *expected,
        Judge::Unordered => unordered_match(&line.output, expected),
        Judge::Float { epsilon } => float_match(&line.output, expected, *epsilon),
        Judge::AnyValid { .. } => line.valid == Some(true),
    }
}

/// `Unordered` treats the top-level array as a set (canonical-JSON sort) so
/// an "any order" answer isn't failed for ordering — nested structure still
/// matches exactly.
fn unordered_match(output: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (output, expected) {
        (serde_json::Value::Array(out), serde_json::Value::Array(exp)) => {
            if out.len() != exp.len() {
                return false;
            }
            let mut a: Vec<String> = out.iter().map(canonical_json).collect();
            let mut b: Vec<String> = exp.iter().map(canonical_json).collect();
            a.sort();
            b.sort();
            a == b
        }
        _ => output == expected,
    }
}

/// Recursive compare for `float` judges: numbers within `epsilon`, all
/// other structure (array shape, object keys, strings, bools) exact.
fn float_match(output: &serde_json::Value, expected: &serde_json::Value, epsilon: f64) -> bool {
    use serde_json::Value;
    match (output, expected) {
        (Value::Number(a), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(a), Some(b)) => (a - b).abs() <= epsilon,
            _ => a == b,
        },
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(x, y)| float_match(x, y, epsilon))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|w| float_match(v, w, epsilon)))
        }
        _ => output == expected,
    }
}

fn canonical_json(v: &serde_json::Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

fn error_result(total: u32, message: String) -> RunResult {
    RunResult {
        status: RunStatus::Error,
        cases: vec![],
        passed: 0,
        total,
        runtime_ms: None,
        memory_mb: None,
        error: Some(message),
    }
}

/// Display formatting — mirrors `fmtValue` in the mock seam: compact JSON
/// with ", " separators.
fn fmt_value(value: &serde_json::Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "null".into())
        .replace(',', ", ")
}

/// Mirrors the mock's `fmtInput`: `name=value` pairs joined with ", ",
/// names parsed from the python signature.
fn fmt_input(params: &[String], args: &[serde_json::Value]) -> String {
    args.iter()
        .enumerate()
        .map(|(i, arg)| {
            let name = params.get(i).cloned().unwrap_or_else(|| format!("arg{i}"));
            let json = serde_json::to_string(arg).unwrap_or_else(|_| "null".into());
            format!("{name}={json}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Parameter names from a `def solve(a, b: int)` style signature — the same
/// rule as `paramNames` in the frontend seam.
fn python_param_names(signature: &str) -> Vec<String> {
    let Some(open) = signature.find('(') else {
        return Vec::new();
    };
    let Some(close) = signature[open..].find(')').map(|i| open + i) else {
        return Vec::new();
    };
    signature[open + 1..close]
        .split(',')
        .filter_map(|part| {
            let name = part.split(':').next().unwrap_or("").trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn param_names_parse_typed_and_untyped_signatures() {
        assert_eq!(
            python_param_names("def solve(nums: list[int], target: int) -> list[int]:"),
            vec!["nums", "target"]
        );
        assert_eq!(python_param_names("def solve(s):\n    pass"), vec!["s"]);
        assert_eq!(python_param_names("def solve():"), Vec::<String>::new());
        assert_eq!(python_param_names("no signature"), Vec::<String>::new());
    }

    fn line(output: serde_json::Value) -> HarnessLine {
        HarnessLine {
            index: 1,
            ok: true,
            output,
            time_ms: None,
            traceback: None,
            valid: None,
        }
    }

    #[test]
    fn unordered_checker_accepts_any_top_level_order() {
        let out = json!([[], [1], [1, 2], [2]]);
        let exp = json!([[], [1], [2], [1, 2]]);
        // strict equality rejects the reordering; unordered accepts it
        assert!(!case_passes(&Judge::Exact, &line(out.clone()), &exp));
        assert!(case_passes(&Judge::Unordered, &line(out), &exp));
        // but a wrong element set still fails, and inner order still matters
        assert!(!unordered_match(&json!([[1], [2]]), &json!([[1], [3]])));
        assert!(!unordered_match(&json!([[2, 1]]), &json!([[1, 2]])));
        // length mismatch fails
        assert!(!unordered_match(&json!([1, 2]), &json!([1, 2, 2])));
    }

    #[test]
    fn float_judge_compares_numbers_with_tolerance() {
        let judge = Judge::Float { epsilon: 1e-5 };
        assert!(case_passes(&judge, &line(json!(2.0000001)), &json!(2.0)));
        assert!(!case_passes(&judge, &line(json!(2.1)), &json!(2.0)));
        // recursion through arrays and objects
        assert!(float_match(
            &json!([1.0, [2.4999999999]]),
            &json!([1, [2.5]]),
            1e-5
        ));
        assert!(!float_match(&json!([1.0, 2.0]), &json!([1.0]), 1e-5));
        assert!(float_match(
            &json!({ "a": 0.30000000000000004 }),
            &json!({ "a": 0.3 }),
            1e-5
        ));
        // non-numeric structure stays exact
        assert!(!float_match(&json!("2.0"), &json!(2.0), 1e-5));
        assert!(float_match(&json!(null), &json!(null), 1e-5));
    }

    #[test]
    fn any_valid_judge_trusts_only_the_validator_verdict() {
        let judge = Judge::AnyValid {
            validator_python: String::new(),
            validator_javascript: String::new(),
        };
        let mut l = line(json!([1, 0]));
        // matching output without a validator verdict is NOT a pass
        assert!(!case_passes(&judge, &l, &json!([1, 0])));
        l.valid = Some(true);
        assert!(case_passes(&judge, &l, &json!([0, 1])));
        l.valid = Some(false);
        assert!(!case_passes(&judge, &l, &json!([1, 0])));
    }

    #[test]
    fn meta_json_is_absent_for_legacy_problems() {
        let problem = crate::domain::problem::Problem {
            id: "legacy".into(),
            number: 1,
            title: "Legacy".into(),
            pattern: crate::domain::problem::Pattern("Stack".into()),
            difficulty: crate::domain::problem::Difficulty::Easy,
            source: crate::domain::problem::ProblemSource::BuiltIn,
            description_md: "d".into(),
            body_html: None,
            constraints: vec![],
            examples: vec![],
            function_signature: crate::domain::problem::FunctionSignature {
                python: "def solve(x):".into(),
                javascript: "function solve(x) {}".into(),
            },
            test_cases: vec![],
            checker: crate::domain::problem::Checker::Exact,
            judge: None,
            entry_point: None,
            hints: vec![],
            reference_solution: None,
            explanation_md: None,
            follow_up: None,
            license: "project-default".into(),
            author: "built-in".into(),
        };
        let judge = problem.effective_judge();
        assert!(harness_meta(&problem, Language::Python, &judge).is_none());

        // an entry point alone produces meta with the per-language name
        let mut with_ep = problem.clone();
        with_ep.entry_point = Some(crate::domain::problem::EntryPoint {
            python: "Solution.twoSum".into(),
            javascript: "twoSum".into(),
            arity: 2,
            io_types: None,
        });
        let meta = harness_meta(&with_ep, Language::Python, &judge).unwrap();
        assert_eq!(meta["entry_point"], "Solution.twoSum");
        assert!(meta.get("mode").is_none());
        let meta = harness_meta(&with_ep, Language::Javascript, &judge).unwrap();
        assert_eq!(meta["entry_point"], "twoSum");

        // judge modes the harness must know about land in meta
        let meta =
            harness_meta(&with_ep, Language::Python, &Judge::InPlace { arg_index: 1 }).unwrap();
        assert_eq!(meta["mode"], "in_place");
        assert_eq!(meta["arg_index"], 1);
        let meta = harness_meta(&problem, Language::Javascript, &Judge::Design).unwrap();
        assert_eq!(meta["mode"], "design");
        // float is judged Rust-side — no meta needed
        assert!(
            harness_meta(&problem, Language::Python, &Judge::Float { epsilon: 1e-5 }).is_none()
        );
    }

    #[test]
    fn display_formatting_matches_the_mock() {
        assert_eq!(fmt_value(&json!([0, 1])), "[0, 1]");
        assert_eq!(fmt_value(&json!(true)), "true");
        assert_eq!(
            fmt_input(
                &["nums".to_string(), "target".to_string()],
                &[json!([2, 7, 11, 15]), json!(9)]
            ),
            "nums=[2,7,11,15], target=9"
        );
        // unnamed args fall back to argN
        assert_eq!(fmt_input(&[], &[json!(1)]), "arg0=1");
    }
}
