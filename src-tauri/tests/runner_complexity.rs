//! Complexity-probe integration tests (Phase 5). Run the real interpreter to
//! op-count a solution on a ladder of generated inputs and confirm the growth
//! classifies as expected. Skipped (visibly) when `python` isn't on PATH so CI
//! machines without it stay green.

mod common;

use app_lib::domain::complexity::classify;
use app_lib::services::runner::{count_ops, ProbeOutcome};

/// A generator that just hands the solution `list(range(size))` — the size *is*
/// n, and neither solution below short-circuits, so the op count is a clean
/// function of n.
const GEN: &str = "def gen(rng, size):\n    return [list(range(size))]\n";
const SIZES: &[u64] = &[100, 200, 400, 800];

fn probe(code: &str) -> Vec<app_lib::domain::complexity::ComplexitySample> {
    match count_ops(code, "solve", None, GEN, 0, SIZES, "python").expect("probe ran") {
        ProbeOutcome::Samples(s) => s,
        other => panic!(
            "probe did not produce samples: {}",
            match other {
                ProbeOutcome::TooSlow => "too slow".to_string(),
                ProbeOutcome::Failed(e) => e,
                ProbeOutcome::Samples(_) => unreachable!(),
            }
        ),
    }
}

#[test]
fn linear_solution_measures_as_linear() {
    require_runtime!("python");
    let samples =
        probe("def solve(nums):\n    s = 0\n    for x in nums:\n        s += x\n    return s\n");
    assert_eq!(samples.len(), SIZES.len());
    // ops grow ~linearly with n.
    assert!(samples[0].ops < samples[3].ops);
    assert_eq!(classify(&samples), Some("O(n)"));
}

#[test]
fn nested_loop_measures_as_quadratic() {
    require_runtime!("python");
    let samples = probe(
        "def solve(nums):\n    c = 0\n    for i in range(len(nums)):\n        for j in range(len(nums)):\n            c += 1\n    return c\n",
    );
    assert_eq!(samples.len(), SIZES.len());
    assert_eq!(classify(&samples), Some("O(n^2)"));
}

#[test]
fn builtin_sort_cost_is_charged_not_read_as_flat() {
    require_runtime!("python");
    // `sorted` runs in C, so a pure line-count would read this as ~O(1). The
    // probe charges its n·log n work, so it correctly measures as super-linear.
    let samples = probe("def solve(nums):\n    s = sorted(nums)\n    return s[-1]\n");
    assert_eq!(samples.len(), SIZES.len());
    let measured = classify(&samples).expect("classified");
    assert!(
        measured == "O(n log n)" || measured == "O(n)",
        "sort-based solution should measure as at least linear, got {measured}"
    );
    assert_ne!(measured, "O(1)", "C built-in cost must not read as flat");
    // super-linear: doubling n more than doubles the ops.
    assert!(samples[3].ops > 2 * samples[1].ops);
}

#[test]
fn a_solution_that_errors_is_reported_not_panicked() {
    require_runtime!("python");
    match count_ops(
        "def solve(nums):\n    raise ValueError('boom')\n",
        "solve",
        None,
        GEN,
        0,
        SIZES,
        "python",
    )
    .expect("probe ran")
    {
        // Each size fails independently, so no usable samples survive.
        ProbeOutcome::Samples(s) => assert!(s.is_empty(), "expected no samples, got {s:?}"),
        ProbeOutcome::Failed(_) => {}
        ProbeOutcome::TooSlow => panic!("unexpected timeout"),
    }
}
