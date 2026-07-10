//! Deterministic complexity feedback (COURSE_BLUEPRINT.md §7, Phase 5).
//!
//! We already run the learner's code — so instead of guessing, we *measure*.
//! The runner executes the solution on a ladder of growing inputs under a
//! line-event counter (`sys.settrace`, Python only) and reports (n, ops)
//! samples. This module classifies that growth into a Big-O class by
//! best-fit against a fixed set of candidate curves, and compares it to the
//! pack's known-optimal time complexity — no AI, fully offline.
//!
//! Honesty note (COURSE_BLUEPRINT.md §8): the counter sees *Python-level*
//! operations, so work hidden in C built-ins (`sorted`, `set`, `Counter`)
//! is under-counted. That's the right lens for the thing this teaches —
//! spotting an O(n²) nested scan you could have made O(n) — and the UI says
//! so plainly.

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
        }
    }
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
}
