//! Deterministic complexity feedback (Phase 5).
//!
//! We already run the learner's code — so instead of guessing, we *measure*.
//! The runner executes the solution on a ladder of growing inputs under a
//! line-event counter (`sys.settrace`, Python only) and reports (n, ops)
//! samples. This module classifies that growth into a Big-O class by
//! best-fit against a fixed set of candidate curves, and compares it to the
//! pack's known-optimal time complexity — no AI, fully offline.
//!
//! The counter sees Python-level line events *plus* a size-weighted charge for
//! sequence-consuming built-ins (`sorted`, `sum`, `min`, `max`, `any`, `all`),
//! so an O(n log n) sort or an O(n) reduction shows up in the growth instead of
//! reading flat; container methods (`.sort()`, `.index()`, `.insert()`, …) are
//! charged via `sys.setprofile`. Work no hook can see (`x in list`, slicing)
//! is caught by cross-checking untraced wall time: when time grows clearly
//! steeper than the op count ([`hidden_work`]), the timing class wins.
//!
//! Space is measured too: tracemalloc's peak minus the newly allocated return
//! value (auxiliary heap), plus the max recursion depth (call-stack space) —
//! see [`classify_space`].

use serde::{Deserialize, Serialize};

/// One measurement: `ops` Python-level operations executed at input size `n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexitySample {
    pub n: u64,
    pub ops: u64,
}

/// How the measured class compares to the pack's optimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityVerdict {
    /// Measured class matches the optimal.
    Optimal,
    /// Measured class grows faster than optimal (the teachable moment).
    Slower,
    /// Measured class looks better than the stated optimal — usually a
    /// measurement artifact (work hidden in built-ins), flagged as such.
    Faster,
    /// Couldn't line the two up (no optimal on the pack, or unclassifiable).
    Unknown,
}

/// Everything one probe run measured, one sample per input size (`n` is the
/// size; the second field is the measured quantity in each series).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProbeSamples {
    /// Python line events + size-weighted built-in charges.
    pub ops: Vec<ComplexitySample>,
    /// Fastest untraced wall time, in nanoseconds.
    pub time_ns: Vec<ComplexitySample>,
    /// Auxiliary heap: tracemalloc peak minus the newly allocated return value.
    pub heap_bytes: Vec<ComplexitySample>,
    /// Max recursion depth of the learner's frames (call-stack space).
    pub depth: Vec<ComplexitySample>,
}

/// The full result of a complexity probe, mirrored in `src/lib/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityReport {
    /// `false` when a probe couldn't run (non-Python, no pack/generator,
    /// the code errored or was too slow) — `note` explains why.
    pub available: bool,
    /// Measured class, e.g. `"O(n^2)"`. `None` when unavailable.
    pub measured: Option<String>,
    /// Pack-declared optimal, e.g. `"O(n)"`. `None` when the pack omits it.
    pub optimal: Option<String>,
    pub verdict: ComplexityVerdict,
    /// One-line learner-facing message ("You wrote ~O(n²); optimal is O(n)…").
    pub note: String,
    pub samples: Vec<ComplexitySample>,
    /// `true` when running time grew clearly faster than the operation count —
    /// the extra work is inside C built-ins (`x in list`, slicing, `s += c`).
    /// `measured` then reflects the timing-based class.
    #[serde(default)]
    pub hidden_work: bool,
    /// Measured auxiliary space class (heap + recursion depth), e.g. `"O(n)"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_measured: Option<String>,
    /// Pack-declared optimal space, e.g. `"O(1)"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_optimal: Option<String>,
    #[serde(default = "unknown_verdict")]
    pub space_verdict: ComplexityVerdict,
    /// One-line learner-facing message about space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_note: Option<String>,
}

fn unknown_verdict() -> ComplexityVerdict {
    ComplexityVerdict::Unknown
}

impl ComplexityReport {
    /// A probe that couldn't produce a measurement, with the reason shown.
    pub fn unavailable(note: impl Into<String>) -> Self {
        Self {
            available: false,
            measured: None,
            optimal: None,
            verdict: ComplexityVerdict::Unknown,
            note: note.into(),
            samples: Vec::new(),
            hidden_work: false,
            space_measured: None,
            space_optimal: None,
            space_verdict: ComplexityVerdict::Unknown,
            space_note: None,
        }
    }
}

/// Least-squares slope of ln(value) against ln(n): ~1 for linear growth, ~2
/// for quadratic. `None` with fewer than 3 positive points.
pub fn growth_slope(samples: &[ComplexitySample]) -> Option<f64> {
    let pts: Vec<(f64, f64)> = samples
        .iter()
        .filter(|s| s.n >= 2 && s.ops > 0)
        .map(|s| ((s.n as f64).ln(), (s.ops as f64).ln()))
        .collect();
    if pts.len() < 3 {
        return None;
    }
    let k = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / k;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / k;
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    Some(sxy / sxx)
}

/// Minimum wall time at the largest size before timing is trusted (below
/// this, interpreter overhead and timer noise dominate).
const HIDDEN_WORK_MIN_NS: u64 = 500_000;
/// How much steeper (in log-log slope) time must grow than the op count. A
/// whole power of n is ~1.0; 0.6 leaves room for cache/allocator drift on a
/// genuinely linear solution while catching an O(n) scan hidden per element.
const HIDDEN_WORK_SLOPE_GAP: f64 = 0.6;
/// How much steeper the learner's time must grow than the reference
/// solution's on the same inputs. Smaller than the op-count gap: this guard
/// only has to cancel input-inherent growth (big integers, long strings),
/// where the reference grows just as steeply (gap ≈ 0) — while a hidden scan
/// that often exits early (random answer positions) still shows ~0.5.
const REFERENCE_SLOPE_GAP: f64 = 0.4;

/// Whether wall time grew clearly faster than the counted operations — i.e.
/// real work is happening inside built-ins the op counter can't see.
///
/// `reference_time` is the pack's verified reference solution timed on the
/// same inputs. When present, the learner's time must also outgrow *it*:
/// some generators produce inputs where even the optimal solution's time
/// legitimately outpaces its op count (big-integer products, long strings),
/// and that's the input's cost, not a hidden-work mistake.
pub fn hidden_work(
    ops: &[ComplexitySample],
    time_ns: &[ComplexitySample],
    reference_time: Option<&[ComplexitySample]>,
) -> bool {
    let Some(top) = time_ns.iter().max_by_key(|s| s.n) else {
        return false;
    };
    if top.ops < HIDDEN_WORK_MIN_NS {
        return false;
    }
    let (Some(so), Some(st)) = (growth_slope(ops), growth_slope(time_ns)) else {
        return false;
    };
    if st - so < HIDDEN_WORK_SLOPE_GAP {
        return false;
    }
    match reference_time.and_then(growth_slope) {
        Some(sr) => st - sr >= REFERENCE_SLOPE_GAP,
        None => true,
    }
}

/// Auxiliary heap under this many bytes *per input element* (at the largest
/// size) is constant: measured on CPython 3.12, building the returned answer
/// leaves ~3.5–4.2 B/element of list over-allocation slack, while the smallest
/// genuine O(n) temporary — a copied/sorted list of references — is ~8 B.
const CONSTANT_HEAP_PER_ELEM: f64 = 6.0;
/// With deep recursion, CPython's frame data-stack chunks show up in the heap
/// trace at ~22 B/frame. Heap under this per element is that stack overhead,
/// already accounted for by the depth series.
const STACK_HEAP_PER_ELEM: f64 = 46.0;
/// Recursion this shallow at every size is constant stack.
const CONSTANT_DEPTH: u64 = 3;

fn per_elem_at_top(s: &[ComplexitySample]) -> f64 {
    s.iter()
        .max_by_key(|x| x.n)
        .map_or(0.0, |x| x.ops as f64 / x.n.max(1) as f64)
}

/// Auxiliary-space class from heap peaks and recursion depths: the larger of
/// the two classes (an O(n)-deep recursion is O(n) space even with O(1)
/// locals). `None` when neither series has enough data.
pub fn classify_space(
    heap: &[ComplexitySample],
    depth: &[ComplexitySample],
) -> Option<&'static str> {
    let d = if depth.len() < 3 {
        None
    } else if depth.iter().all(|x| x.ops <= CONSTANT_DEPTH) {
        Some("O(1)")
    } else {
        classify(depth)
    };
    let recursive = d.is_some_and(|c| c != "O(1)");
    let floor = if recursive {
        STACK_HEAP_PER_ELEM
    } else {
        CONSTANT_HEAP_PER_ELEM
    };
    // Heap growth snaps to the classes space answers actually take, by
    // log-log slope: containers resize in steps (dicts/sets double), which a
    // curve fit can misread as n log n.
    let h = if heap.len() < 3 {
        None
    } else if per_elem_at_top(heap) < floor {
        Some("O(1)")
    } else {
        growth_slope(heap).map(|s| match s {
            s if s < 0.35 => "O(1)",
            s if s < 1.5 => "O(n)",
            s if s < 2.5 => "O(n^2)",
            _ => "O(n^3)",
        })
    };
    match (h, d) {
        (Some(a), Some(b)) => Some(max_label(a, b)),
        (a, b) => a.or(b),
    }
}

/// The lower-ranked of two measured labels.
pub fn min_label(a: &'static str, b: &'static str) -> &'static str {
    if label_rank(b) < label_rank(a) {
        b
    } else {
        a
    }
}

/// The higher-ranked of two measured labels.
pub fn max_label(a: &'static str, b: &'static str) -> &'static str {
    if label_rank(b) > label_rank(a) {
        b
    } else {
        a
    }
}

/// `true` when a declared complexity is expressed in `n` alone (e.g. "O(n)",
/// "O(n log n)", "O(1)"). Strings in other variables ("O(k)", "O(m*n)",
/// "O(h)") can't be lined up against a single size ladder.
pub fn only_in_n(s: &str) -> bool {
    let lower = s.to_lowercase();
    let stripped = lower.replace("log", "").replace("sqrt", "");
    let inner = stripped
        .trim()
        .trim_start_matches('o')
        .trim_start_matches('(')
        .trim_end_matches(')');
    !inner.chars().any(|c| c.is_ascii_alphabetic() && c != 'n')
}

/// Candidate growth curves, in ascending order — the label is what the UI
/// shows, `rank` orders them for the optimal comparison, and `f` is the
/// curve fitted against the samples.
struct Model {
    label: &'static str,
    rank: u8,
    f: fn(f64) -> f64,
}

const MODELS: &[Model] = &[
    Model {
        label: "O(1)",
        rank: 0,
        f: |_| 1.0,
    },
    Model {
        label: "O(log n)",
        rank: 1,
        f: |n| n.max(2.0).log2(),
    },
    Model {
        label: "O(n)",
        rank: 2,
        f: |n| n,
    },
    Model {
        label: "O(n log n)",
        rank: 3,
        f: |n| n * n.max(2.0).log2(),
    },
    Model {
        label: "O(n^2)",
        rank: 4,
        f: |n| n * n,
    },
    Model {
        label: "O(n^3)",
        rank: 5,
        f: |n| n * n * n,
    },
];

/// The rank of a measured label (produced by `classify`) — always known.
fn label_rank(label: &str) -> Option<u8> {
    MODELS.iter().find(|m| m.label == label).map(|m| m.rank)
}

/// Best-fit Big-O class for a set of (n, ops) samples, or `None` when the
/// data is too thin/degenerate to classify. Method: for each candidate curve
/// `f`, the ratios `ops_i / f(n_i)` should be ~constant for the true class;
/// we pick the curve whose ratios have the lowest coefficient of variation.
pub fn classify(samples: &[ComplexitySample]) -> Option<&'static str> {
    // Need enough distinct, non-trivial points to fit a curve.
    let usable: Vec<(f64, f64)> = samples
        .iter()
        .filter(|s| s.n >= 2)
        .map(|s| (s.n as f64, s.ops as f64))
        .collect();
    if usable.len() < 3 {
        return None;
    }
    // All-zero op counts can't be classified (nothing ran).
    if usable.iter().all(|&(_, ops)| ops <= 0.0) {
        return None;
    }

    let mut best: Option<(&'static str, f64)> = None;
    for model in MODELS {
        let ratios: Vec<f64> = usable.iter().map(|&(n, ops)| ops / (model.f)(n)).collect();
        let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
        if mean <= 0.0 {
            continue;
        }
        let variance = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / ratios.len() as f64;
        let cov = variance.sqrt() / mean; // coefficient of variation
        let improves = match best {
            None => true,
            Some((_, b)) => cov < b,
        };
        if improves {
            best = Some((model.label, cov));
        }
    }
    best.map(|(label, _)| label)
}

/// Coarse rank of a pack's declared complexity string ("O(n log n)",
/// "O(n*k*log k)", "O(2^n)", …). Heuristic but stable; used only to order
/// measured-vs-optimal, never shown to the learner verbatim.
pub fn rank_of_optimal(s: &str) -> Option<u8> {
    let inner: String = s
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if inner.is_empty() {
        return None;
    }
    if inner.contains("2^") || inner.contains("2**") || inner.contains('!') {
        return Some(6); // exponential / factorial — worse than cubic
    }
    if inner.contains("n^3") || inner.contains("n**3") || inner.contains("n*n*n") {
        return Some(5);
    }
    if inner.contains("n^2")
        || inner.contains("n**2")
        || inner.contains("n*n")
        || inner.contains('²')
    {
        return Some(4);
    }
    if inner.contains("log") {
        // Strip the log and the `n` it consumes; a surviving `n` means there's
        // a linear factor outside the log → n log n, else pure log n.
        let reduced = inner
            .replace("log2n", "")
            .replace("log(n)", "")
            .replace("logn", "")
            .replace("log", "");
        return Some(if reduced.contains('n') { 3 } else { 1 });
    }
    if inner.contains('n') {
        return Some(2);
    }
    Some(0)
}

/// Compares a measured class against a pack-declared optimal string.
pub fn verdict(measured: &str, optimal: Option<&str>) -> ComplexityVerdict {
    let (Some(m), Some(o)) = (label_rank(measured), optimal.and_then(rank_of_optimal)) else {
        return ComplexityVerdict::Unknown;
    };
    match m.cmp(&o) {
        std::cmp::Ordering::Greater => ComplexityVerdict::Slower,
        std::cmp::Ordering::Equal => ComplexityVerdict::Optimal,
        std::cmp::Ordering::Less => ComplexityVerdict::Faster,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples(pairs: &[(u64, u64)]) -> Vec<ComplexitySample> {
        pairs
            .iter()
            .map(|&(n, ops)| ComplexitySample { n, ops })
            .collect()
    }

    #[test]
    fn classifies_linear_growth() {
        // ops ≈ 3n
        assert_eq!(
            classify(&samples(&[
                (100, 300),
                (200, 600),
                (400, 1200),
                (800, 2400)
            ])),
            Some("O(n)")
        );
    }

    #[test]
    fn classifies_quadratic_growth() {
        // ops ≈ n²/2 — a nested scan
        assert_eq!(
            classify(&samples(&[
                (100, 5_000),
                (200, 20_000),
                (400, 80_000),
                (800, 320_000)
            ])),
            Some("O(n^2)")
        );
    }

    #[test]
    fn classifies_constant_and_nlogn() {
        assert_eq!(
            classify(&samples(&[(100, 7), (200, 7), (400, 8), (800, 7)])),
            Some("O(1)")
        );
        // ops ≈ n·log2(n)
        let nlogn = samples(&[
            (128, (128.0 * 7.0) as u64),
            (256, (256.0 * 8.0) as u64),
            (512, (512.0 * 9.0) as u64),
            (1024, (1024.0 * 10.0) as u64),
        ]);
        assert_eq!(classify(&nlogn), Some("O(n log n)"));
    }

    #[test]
    fn refuses_to_classify_thin_data() {
        assert_eq!(classify(&samples(&[(100, 100), (200, 200)])), None);
        assert_eq!(classify(&samples(&[(100, 0), (200, 0), (400, 0)])), None);
    }

    #[test]
    fn ranks_optimal_strings() {
        assert_eq!(rank_of_optimal("O(1)"), Some(0));
        assert_eq!(rank_of_optimal("O(log n)"), Some(1));
        assert_eq!(rank_of_optimal("O(n)"), Some(2));
        assert_eq!(rank_of_optimal("O(n log n)"), Some(3));
        assert_eq!(rank_of_optimal("O(n*k*log k)"), Some(3));
        assert_eq!(rank_of_optimal("O(n^2)"), Some(4));
        assert_eq!(rank_of_optimal("O(n²)"), Some(4));
        assert_eq!(rank_of_optimal("O(2^n)"), Some(6));
    }

    #[test]
    fn verdict_compares_measured_to_optimal() {
        assert_eq!(verdict("O(n^2)", Some("O(n)")), ComplexityVerdict::Slower);
        assert_eq!(verdict("O(n)", Some("O(n)")), ComplexityVerdict::Optimal);
        assert_eq!(verdict("O(n)", Some("O(n^2)")), ComplexityVerdict::Faster);
        assert_eq!(verdict("O(n)", None), ComplexityVerdict::Unknown);
    }

    fn series(points: &[(u64, u64)]) -> Vec<ComplexitySample> {
        points
            .iter()
            .map(|&(n, ops)| ComplexitySample { n, ops })
            .collect()
    }

    #[test]
    fn growth_slope_reads_the_power_of_n() {
        let lin = series(&[(100, 1000), (200, 2000), (400, 4000), (800, 8000)]);
        let quad = series(&[(100, 10_000), (200, 40_000), (400, 160_000), (800, 640_000)]);
        assert!((growth_slope(&lin).unwrap() - 1.0).abs() < 1e-9);
        assert!((growth_slope(&quad).unwrap() - 2.0).abs() < 1e-9);
        assert!(growth_slope(&lin[..2]).is_none());
    }

    #[test]
    fn hidden_work_needs_a_steeper_time_curve_and_real_signal() {
        let ops = series(&[(100, 200), (200, 400), (400, 800), (800, 1600)]);
        // Quadratic wall time behind linear ops, well above the noise floor.
        let slow = series(&[
            (100, 40_000),
            (200, 150_000),
            (400, 600_000),
            (800, 2_400_000),
        ]);
        assert!(hidden_work(&ops, &slow, None));
        // ...unless the reference solution's time grows just as steeply on
        // these inputs (e.g. big-integer arithmetic): that's the input's cost.
        let ref_steep = series(&[
            (100, 30_000),
            (200, 120_000),
            (400, 480_000),
            (800, 1_900_000),
        ]);
        assert!(!hidden_work(&ops, &slow, Some(&ref_steep)));
        let ref_lin = series(&[(100, 20_000), (200, 40_000), (400, 80_000), (800, 160_000)]);
        assert!(hidden_work(&ops, &slow, Some(&ref_lin)));
        // Linear time: nothing hidden.
        let fair = series(&[
            (100, 70_000),
            (200, 140_000),
            (400, 290_000),
            (800, 600_000),
        ]);
        assert!(!hidden_work(&ops, &fair, None));
        // Steep but microscopic timings are noise, not evidence.
        let tiny = series(&[(100, 1_000), (200, 4_000), (400, 16_000), (800, 64_000)]);
        assert!(!hidden_work(&ops, &tiny, None));
    }

    #[test]
    fn space_classes_from_heap_and_depth() {
        let flat_depth = series(&[(100, 1), (200, 1), (400, 1), (800, 1)]);
        let deep = series(&[(100, 101), (200, 201), (400, 401), (800, 801)]);
        // ~4 B/element of output-building slack reads as constant.
        let slack = series(&[(100, 550), (200, 950), (400, 1750), (800, 3350)]);
        assert_eq!(classify_space(&slack, &flat_depth), Some("O(1)"));
        // A copied list (~8 B/element) is linear.
        let copy = series(&[(100, 930), (200, 1730), (400, 3360), (800, 6560)]);
        assert_eq!(classify_space(&copy, &flat_depth), Some("O(n)"));
        // Deep recursion is O(n) stack even when the heap only shows frame chunks.
        let chunks = series(&[(100, 380), (200, 380), (400, 4952), (800, 17_752)]);
        assert_eq!(classify_space(&chunks, &deep), Some("O(n)"));
        assert_eq!(classify_space(&[], &[]), None);
        // Dict/set capacity doubles in steps; still linear, not n log n.
        let steps = series(&[(100, 3_400), (200, 6_800), (400, 14_000), (800, 67_000)]);
        assert_eq!(classify_space(&steps, &flat_depth), Some("O(n)"));
    }

    #[test]
    fn only_in_n_rejects_other_variables() {
        assert!(only_in_n("O(n)"));
        assert!(only_in_n("O(1)"));
        assert!(only_in_n("O(n log n)"));
        assert!(only_in_n("O(log n)"));
        assert!(!only_in_n("O(k)"));
        assert!(!only_in_n("O(m*n)"));
        assert!(!only_in_n("O(h)"));
    }
}
