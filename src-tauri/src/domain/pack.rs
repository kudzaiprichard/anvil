//! Test-pack types — the shape of one entry in the shipped `test-packs.json`
//! bundle, keyed by LeetCode slug. Packs are entirely
//! our own content (tests, solutions, hints), merged with a user's imported
//! question at import time. Field names mirror `src/lib/types.ts` exactly.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::problem::{Complexity, EntryPoint, Judge};

/// What a literal pack test is probing; surfaced in "reveal failing case".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackTestKind {
    Edge,
    Boundary,
    Trap,
}

/// A literal verified test: positional args (same convention as
/// `TestCase.input`) and an expected value computed by executing the
/// reference solutions — never authored by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackTest {
    pub kind: PackTestKind,
    pub description: String,
    pub input: Vec<Value>,
    pub expected: Value,
}

/// A deterministic stress-input generator. Large inputs never ship as
/// literals; the importer materializes these once through the sandbox
/// into ordinary hidden test cases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressSpec {
    pub description: String,
    pub seed: u64,
    pub size: u64,
    /// Our generator source: `def gen(rng, size)` returning the args tuple.
    pub generator_python: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Machine-usable summary of one parameter's constraints, extracted from the
/// statement by the pipeline; drives boundary/stress input generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintSpec {
    pub param: String,
    /// Coarse shape tag, e.g. `"int"`, `"int[]"`, `"string"`.
    pub kind: String,
    /// `[min, max]` length bounds for sequence params.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<(i64, i64)>,
    /// `[min, max]` value bounds for numeric params/elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<(i64, i64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackSolutions {
    /// Python is the expected-value source and is always present.
    pub python: String,
    /// Absent for single-language packs (closing-the-48: the concurrency set
    /// has no JavaScript — JS has no shared-memory threads). The build skips
    /// the cross-language check for these; the workspace disables the toggle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub javascript: Option<String>,
    /// The naive oracle used for differential verification, kept for
    /// re-verification runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brute_force_python: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<Complexity>,
}

/// One shipped test pack. `verified` is set by the generation pipeline only
/// after the full execution cross-check; unverified packs are quarantined
/// and never reach the bundle, so loaders may treat `verified: false` as a
/// data error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestPack {
    pub slug: String,
    pub qid: String,
    pub schema_version: u32,
    pub entry_point: EntryPoint,
    pub judge: Judge,
    /// One-sentence pattern explanation (what this problem teaches).
    pub pattern: String,
    /// Three progressive hints: nudge → approach → near-answer.
    pub hints: Vec<String>,
    pub constraints: Vec<ConstraintSpec>,
    pub tests: Vec<PackTest>,
    pub stress: Vec<StressSpec>,
    pub solutions: PackSolutions,
    /// Whether the statement's own examples are usable as visible test cases
    /// under this pack's wire format (closing-the-48). True exactly when the
    /// build anchored the pack against the statement examples; false for
    /// `no_anchor_ok` packs whose wire encoding differs from the statement
    /// (cyclic lists, multilevel lists, shim args, …) — the import then shows
    /// the pack's own tests instead. Absent on pre-existing packs ⇒ true,
    /// which is correct: they were all anchor-verified or their examples
    /// never parsed in the first place.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub examples_ok: bool,
    pub verified: bool,
    pub generated_at: String,
}

fn default_true() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_true(v: &bool) -> bool {
    *v
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn round_trip<T: Serialize + for<'de> Deserialize<'de>>(value: serde_json::Value) {
        let parsed: T = serde_json::from_value(value.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&parsed).expect("serialize"), value);
    }

    #[test]
    fn judge_round_trips_all_variants() {
        round_trip::<Judge>(json!({ "type": "exact" }));
        round_trip::<Judge>(json!({ "type": "unordered" }));
        round_trip::<Judge>(json!({ "type": "float", "epsilon": 1e-5 }));
        round_trip::<Judge>(json!({ "type": "in_place", "arg_index": 0 }));
        round_trip::<Judge>(json!({
            "type": "any_valid",
            "validator_python": "def validate(args, out): return True",
            "validator_javascript": "function validate(args, out) { return true; }"
        }));
        round_trip::<Judge>(json!({ "type": "design" }));
        // design_io (closing-the-48 Phase A): node-typed ctor/method boundary.
        round_trip::<Judge>(json!({
            "type": "design",
            "design_io": {
                "ctor": ["tree"],
                "methods": { "next": { "returns": "json" }, "insert": { "params": ["tree"] } }
            }
        }));
        // Phase C judges: codec round-trip and property (randomized) packs.
        round_trip::<Judge>(json!({
            "type": "round_trip",
            "io": "tree",
            "encode": "serialize",
            "decode": "deserialize"
        }));
        round_trip::<Judge>(json!({
            "type": "property",
            "validator_python": "def validate(args, outputs): return True",
            "validator_javascript": "function validate(args, outputs) { return true; }"
        }));
        round_trip::<Judge>(json!({
            "type": "property",
            "validator_python": "def validate(args, out): return True",
            "validator_javascript": "function validate(args, out) { return true; }",
            "exec": "call",
            "design_io": { "ctor": ["linked_list"] }
        }));
        // Concurrency (python-only): driver + validator + amplified runs.
        round_trip::<Judge>(json!({
            "type": "concurrency",
            "driver_python": "def drive(cls, args, record): ...",
            "validator_python": "def validate(args, events): return True",
            "runs": 6
        }));
    }

    #[test]
    fn unknown_judge_type_is_a_clear_error() {
        let err = serde_json::from_value::<Judge>(json!({ "type": "telepathy" }))
            .expect_err("should reject unknown judge type");
        assert!(err.to_string().contains("telepathy"), "{err}");
    }

    #[test]
    fn test_pack_round_trips_the_design_doc_example() {
        // The two-sum example, verbatim in shape.
        round_trip::<TestPack>(json!({
            "slug": "two-sum",
            "qid": "1",
            "schema_version": 1,
            "entry_point": { "python": "Solution.twoSum", "javascript": "twoSum", "arity": 2 },
            "judge": { "type": "exact" },
            "pattern": "Hash map stores complements — trades space for O(n) time.",
            "hints": ["nudge", "approach", "near-answer"],
            "constraints": [
                { "param": "nums", "kind": "int[]", "len": [2, 10000], "value": [-1000000000, 1000000000] },
                { "param": "target", "kind": "int", "value": [-1000000000, 1000000000] }
            ],
            "tests": [
                { "kind": "edge", "description": "duplicate values sum to target",
                  "input": [[3, 3], 6], "expected": [0, 1] },
                { "kind": "boundary", "description": "max magnitude values",
                  "input": [[1000000000, -1000000000], 0], "expected": [0, 1] },
                { "kind": "trap", "description": "same element may not be reused",
                  "input": [[1, 3, 4, 2], 6], "expected": [2, 3] }
            ],
            "stress": [
                { "description": "10k elements — exposes O(n^2)", "seed": 42, "size": 10000,
                  "generator_python": "def gen(rng, size): ...", "note": "< 100ms expected" }
            ],
            "solutions": {
                "python": "class Solution:\n    def twoSum(self, nums, target): ...",
                "javascript": "var twoSum = function(nums, target) {};",
                "brute_force_python": "class Solution:\n    def twoSum(self, nums, target): ...",
                "complexity": { "time": "O(n)", "space": "O(n)" }
            },
            "verified": true,
            "generated_at": "2026-06-12T00:00:00Z"
        }));
    }

    #[test]
    fn optional_pack_fields_are_omitted_when_absent() {
        let spec: StressSpec = serde_json::from_value(json!({
            "description": "d", "seed": 1, "size": 10, "generator_python": "def gen(rng, size): ..."
        }))
        .unwrap();
        let v = serde_json::to_value(&spec).unwrap();
        assert!(v.get("note").is_none());

        let c: ConstraintSpec =
            serde_json::from_value(json!({ "param": "target", "kind": "int" })).unwrap();
        let v = serde_json::to_value(&c).unwrap();
        assert!(v.get("len").is_none());
        assert!(v.get("value").is_none());
    }
}
