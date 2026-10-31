//! Complexity-probe integration tests (Phase 5). Run the real interpreter to
//! op-count a solution on a ladder of generated inputs and confirm the growth
//! classifies as expected. Skipped (visibly) when `python` isn't on PATH so CI
//! machines without it stay green.

mod common;

use app_lib::domain::complexity::{classify, classify_space, hidden_work, ProbeSamples};
use app_lib::services::runner::{count_ops, ProbeOutcome};

/// A generator that just hands the solution `list(range(size))` — the size *is*
/// n, and neither solution below short-circuits, so the op count is a clean
/// function of n.
const GEN: &str = "def gen(rng, size):\n    return [list(range(size))]\n";
const SIZES: &[u64] = &[100, 200, 400, 800];

/// Runs the probe, or `None` when the CI sandbox's thread/process cap breaks
/// the harness's worker-thread spawn (`RuntimeError: can't start new thread`)
/// — an environment limit, not a probe/classification bug (same class as
/// `common::stress_skipped_by_sandbox`). Any other failure still panics, so a
/// genuine regression is never masked.
fn probe(code: &str) -> Option<Vec<app_lib::domain::complexity::ComplexitySample>> {
    probe_all(code).map(|s| s.ops)
}

/// Like [`probe`], but every measured series (ops, time, heap, depth).
fn probe_all(code: &str) -> Option<ProbeSamples> {
    match count_ops(code, "solve", None, GEN, 0, SIZES, "python").expect("probe ran") {
        ProbeOutcome::Samples(s) => Some(s),
        ProbeOutcome::Failed(e) if common::is_sandbox_thread_limit(&e) => {
            eprintln!("SKIPPED: sandbox could not launch the complexity probe here: {e}");
            None
        }
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
    let Some(samples) =
        probe("def solve(nums):\n    s = 0\n    for x in nums:\n        s += x\n    return s\n")
    else {
        return;
    };
    assert_eq!(samples.len(), SIZES.len());
    // ops grow ~linearly with n.
    assert!(samples[0].ops < samples[3].ops);
    assert_eq!(classify(&samples), Some("O(n)"));
}

#[test]
fn nested_loop_measures_as_quadratic() {
    require_runtime!("python");
    let Some(samples) = probe(
        "def solve(nums):\n    c = 0\n    for i in range(len(nums)):\n        for j in range(len(nums)):\n            c += 1\n    return c\n",
    ) else {
        return;
    };
    assert_eq!(samples.len(), SIZES.len());
    assert_eq!(classify(&samples), Some("O(n^2)"));
}

#[test]
fn builtin_sort_cost_is_charged_not_read_as_flat() {
    require_runtime!("python");
    // `sorted` runs in C, so a pure line-count would read this as ~O(1). The
    // probe charges its n·log n work, so it correctly measures as super-linear.
    let Some(samples) = probe("def solve(nums):\n    s = sorted(nums)\n    return s[-1]\n") else {
        return;
    };
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
        ProbeOutcome::Samples(s) => assert!(s.ops.is_empty(), "expected no samples, got {s:?}"),
        ProbeOutcome::Failed(_) => {}
        ProbeOutcome::TooSlow => panic!("unexpected timeout"),
    }
}

#[test]
fn membership_in_a_list_is_caught_as_hidden_work() {
    require_runtime!("python");
    // One Python line per element (ops read O(n)), but `x in seen` scans a
    // list in C — the real cost is O(n^2). Timing must expose it.
    let code = "def solve(nums):
    seen = []
    for x in nums:
        if x in seen:
            return True
        seen.append(x)
    return False
";
    let Some(s) = probe_all(code) else { return };
    assert_eq!(
        s.time_ns.len(),
        SIZES.len(),
        "untraced timing recorded per size"
    );
    assert!(
        hidden_work(&s.ops, &s.time_ns, None),
        "time should outgrow ops: ops={:?} time={:?}",
        s.ops,
        s.time_ns
    );
}

#[test]
fn a_set_based_scan_has_no_hidden_work() {
    require_runtime!("python");
    let code = "def solve(nums):
    seen = set()
    for x in nums:
        if x in seen:
            return True
        seen.add(x)
    return False
";
    let Some(s) = probe_all(code) else { return };
    assert!(
        !hidden_work(&s.ops, &s.time_ns, None),
        "time={:?}",
        s.time_ns
    );
    // ...but the set itself is O(n) auxiliary space.
    assert_eq!(
        classify_space(&s.heap_bytes, &s.depth),
        Some("O(n)"),
        "heap={:?}",
        s.heap_bytes
    );
}

#[test]
fn constant_space_scan_measures_o1_space() {
    require_runtime!("python");
    let code = "def solve(nums):
    best = 0
    for x in nums:
        best = max(best, x)
    return best
";
    let Some(s) = probe_all(code) else { return };
    assert_eq!(
        classify_space(&s.heap_bytes, &s.depth),
        Some("O(1)"),
        "heap={:?}",
        s.heap_bytes
    );
}

#[test]
fn returned_output_is_not_counted_as_extra_space() {
    require_runtime!("python");
    // Building the O(n) answer itself isn't auxiliary space.
    let code = "def solve(nums):
    out = []
    for x in nums:
        out.append(x * 2)
    return out
";
    let Some(s) = probe_all(code) else { return };
    assert_eq!(
        classify_space(&s.heap_bytes, &s.depth),
        Some("O(1)"),
        "heap={:?}",
        s.heap_bytes
    );
}

#[test]
fn recursion_depth_counts_as_space() {
    require_runtime!("python");
    // O(1) locals per frame, but n frames deep → O(n) stack.
    let code = "def solve(nums):
    def go(i):
        if i == len(nums):
            return 0
        return nums[i] + go(i + 1)
    return go(0)
";
    let Some(s) = probe_all(code) else { return };
    assert!(s.depth.iter().all(|d| d.ops >= d.n), "depth={:?}", s.depth);
    assert_eq!(classify_space(&s.heap_bytes, &s.depth), Some("O(n)"));
}

#[test]
fn list_sort_method_is_charged() {
    require_runtime!("python");
    let code = "def solve(nums):
    nums.sort()
    return nums[0]
";
    let Some(ops) = probe(code) else { return };
    assert_ne!(classify(&ops), Some("O(1)"), "ops={ops:?}");
}
