//! Problem types — the IPC contract for everything the library/workspace
//! render. Field names and casing mirror `src/lib/types.ts` exactly: problem
//! fields are `snake_case` (`description_md`, `test_cases`), summary fields
//! are camelCase (`lastAttempted`), and the hyphenated enum values
//! (`"built-in"`, `"in-progress"`, `"needs-review"`) are explicit renames.
//! Do not add fields the TypeScript side doesn't have.

use serde::{Deserialize, Serialize};

/// The 15 patterns problems are organized by (types.ts `PATTERNS`).
pub const PATTERNS: [&str; 15] = [
    "Arrays & Hashing",
    "Two Pointers",
    "Sliding Window",
    "Stack",
    "Binary Search",
    "Linked List",
    "Trees",
    "Heap / Priority Queue",
    "Backtracking",
    "Graphs",
    "1-D DP",
    "2-D DP",
    "Greedy",
    "Intervals",
    "Bit Manipulation",
];

/// Validated-string newtype rather than an enum so pattern churn doesn't
/// ripple through serde; `is_known()` is enforced by the loader/validators.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pattern(pub String);

impl Pattern {
    pub fn is_known(&self) -> bool {
        PATTERNS.contains(&self.0.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// How the runner compares a solution's output to the expected value.
/// `Exact` is strict deep equality (the default). `Unordered` treats the
/// top-level array as a set — for problems whose statement allows the answer
/// "in any order" (subsets, permutations, grouped anagrams), so a correct
/// solution isn't failed for ordering. Real judges use special checkers for
/// exactly this; this is our lightweight version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Checker {
    #[default]
    Exact,
    Unordered,
}

pub(crate) fn checker_is_exact(c: &Checker) -> bool {
    matches!(c, Checker::Exact)
}

/// Full judge taxonomy (CONTENT_DESIGN.md §4). `Checker` covers the original
/// two modes and stays on disk for existing problems; `Judge` adds the modes
/// imported/pack-backed problems need. A problem carries at most one `judge`;
/// when absent it is derived from `checker` (see `Problem::effective_judge`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Judge {
    Exact,
    Unordered,
    /// Recursive numeric compare with tolerance (e.g. median problems).
    Float {
        epsilon: f64,
    },
    /// Void-return problems judged on a mutated argument (e.g. rotate array).
    InPlace {
        arg_index: usize,
    },
    /// Multiple valid answers — a pack-shipped validator (our code, never the
    /// user's) decides `(input, output) -> bool` instead of value comparison.
    AnyValid {
        validator_python: String,
        validator_javascript: String,
    },
    /// Ops-sequence problems (LRU Cache): input is `[ops, argLists]`,
    /// expected is the per-op output array — LeetCode's wire format.
    /// `design_io` node-types the constructor/method call boundary so packs
    /// like binary-search-tree-iterator can hand a real `TreeNode` to the
    /// constructor; absent ⇒ raw JSON args (all pre-existing design packs).
    Design {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        design_io: Option<DesignIo>,
    },
}

/// Node I/O for one `design` method: param types positionally, plus the
/// return type when it needs serializing (absent ⇒ plain JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodIo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<IoType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<IoType>,
}

/// Per-op I/O map for `design` packs: constructor param types plus a
/// method-name → `MethodIo` table. Methods absent from the map run all-JSON,
/// so authors only declare the node-typed ops.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignIo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ctor: Vec<IoType>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub methods: std::collections::BTreeMap<String, MethodIo>,
}

/// Which callable the harness invokes, per language. `"Solution.twoSum"`
/// means instantiate the class and call the method; a bare name is a
/// top-level function. Absent on a problem ⇒ the legacy `solve` convention.
/// Also serves as the import-time match fingerprint (CONTENT_DESIGN.md §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPoint {
    pub python: String,
    pub javascript: String,
    pub arity: u32,
    /// Node I/O types (task 0003): when present, the harness (de)serializes
    /// `ListNode`/`TreeNode` params + return at the call boundary so a LeetCode
    /// stub runs unmodified. Absent ⇒ all-JSON (today's behavior, unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_types: Option<IoTypes>,
}

/// Per-call I/O shape (task 0003 + closing-the-48 Phase B). Leaves serialize
/// as `"json"`/`"linked_list"`/`"tree"`/`"cyclic_list"`/… ; composites as
/// `{"list_of": …}` / `{"ctx_only": …}` / `{"node_ref": {"param": i}}` (and
/// the other param-referencing forms). Every variant here must be understood
/// by BOTH harnesses' deserialize/serialize and by `tools/build_packs.py`'s
/// `_ok_io_type` — the bundle parse is strict, so an unknown type would empty
/// the whole pack store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoType {
    Json,
    LinkedList,
    Tree,
    /// `{"values": [...], "pos": k|null}` — tail links back to node #k.
    CyclicList,
    /// `[[val, randomIdx|null], ...]` — LeetCode's copy-random-pointer form.
    RandomList,
    /// Adjacency list; node i (0-based) carries val i+1.
    Graph,
    /// LeetCode level order with null group separators.
    NAryTree,
    /// Level order of `[isLeaf, val]` pairs with nulls (tl/tr/bl/br).
    QuadTree,
    /// In: plain level order; out: serialized by FOLLOWING the next pointers.
    NextTree,
    /// Segments with a global-index parent: `[{"values": [...], "parent": …}]`.
    MultilevelList,
    ListOf(Box<IoType>),
    /// Built into the judging context but never passed to the solution
    /// (e.g. delete-node's hidden list head).
    CtxOnly(Box<IoType>),
    /// Wire value is a node VALUE; the harness resolves the real node object
    /// inside the already-built param `param`. As a return type: the node's
    /// value, after verifying identity membership in that structure.
    NodeRef { param: usize },
    /// A structure-preserving deep copy of the already-built param `param`.
    CloneOf { param: usize },
    /// `{"values": [...], "attach": idx|null}` — a fresh list whose last node
    /// links into param `param`'s chain (intersection-of-two-linked-lists).
    TailOf { param: usize },
    /// Return-only: the returned node's index in param `param`'s chain.
    NodeIndexOf { param: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoTypes {
    pub params: Vec<IoType>,
    pub returns: IoType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProblemSource {
    #[serde(rename = "built-in")]
    BuiltIn,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "imported")]
    Imported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub input: String,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation_md: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    /// Positional arguments passed to the solve function (decision §9.1).
    pub input: Vec<serde_json::Value>,
    pub expected: serde_json::Value,
    pub hidden: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub python: String,
    pub javascript: String,
    /// Starter stubs for languages beyond the runnable Python/JS pair, captured
    /// verbatim from an imported catalog (`cpp`, `java`, `go`, …). Display-only
    /// for now — the runner still executes only Python/JS — but preserved so
    /// re-importing a catalog that adds languages never drops them, and adding
    /// a runnable language later needs no schema change. Omitted when empty.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub extra: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Complexity {
    pub time: String,
    pub space: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceSolution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub javascript: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<Complexity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    /// Display number used in lists ("23. …"). App-level, not part of §8.3.
    pub number: u32,
    pub title: String,
    pub pattern: Pattern,
    pub difficulty: Difficulty,
    pub source: ProblemSource,
    pub description_md: String,
    /// Sanitized HTML statement, set only for imported problems so the panel
    /// can render with full fidelity (superscripts, `<pre>` examples);
    /// `description_md` (from `body_text`) stays the search/fallback source
    /// (CONTENT_DESIGN.md §8). Absent ⇒ render `description_md` as markdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,
    pub constraints: Vec<String>,
    pub examples: Vec<Example>,
    pub function_signature: FunctionSignature,
    pub test_cases: Vec<TestCase>,
    /// Output comparison mode; omitted (defaults to `exact`) for the common
    /// case so existing problem files need no change.
    #[serde(default, skip_serializing_if = "checker_is_exact")]
    pub checker: Checker,
    /// Full judge mode; absent ⇒ derived from `checker`. Only pack-backed
    /// (imported) problems set this today.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge: Option<Judge>,
    /// Harness entry point; absent ⇒ legacy top-level `solve` convention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<EntryPoint>,
    pub hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_solution: Option<ReferenceSolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation_md: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
    pub license: String,
    pub author: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemStatus {
    Todo,
    InProgress,
    Solved,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemSummary {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub pattern: Pattern,
    pub difficulty: Difficulty,
    /// Library badge ("imported" / "user"); built-ins render no badge.
    pub source: ProblemSource,
    pub status: ProblemStatus,
    /// ISO date of the last attempt; the UI formats it relatively.
    #[serde(rename = "lastAttempted", skip_serializing_if = "Option::is_none")]
    pub last_attempted: Option<String>,
}

impl Problem {
    /// The judge the runner must use: explicit `judge` when present,
    /// otherwise derived from the legacy `checker` field.
    pub fn effective_judge(&self) -> Judge {
        match &self.judge {
            Some(j) => j.clone(),
            None => match self.checker {
                Checker::Exact => Judge::Exact,
                Checker::Unordered => Judge::Unordered,
            },
        }
    }

    /// Structural rules every stored problem must satisfy (PROBLEMS.md
    /// quality bar) — enforced on built-ins at startup and on imported
    /// files before they touch the library.
    pub fn validate_structure(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("id is empty".into());
        }
        if self.title.trim().is_empty() {
            return Err("title is empty".into());
        }
        if self.description_md.trim().is_empty() {
            return Err("description_md is empty".into());
        }
        if !self.pattern.is_known() {
            return Err(format!("unknown pattern '{}'", self.pattern.0));
        }
        if self.function_signature.python.trim().is_empty() {
            return Err("python function signature is empty".into());
        }
        if self.function_signature.javascript.trim().is_empty() {
            return Err("javascript function signature is empty".into());
        }
        let visible = self.test_cases.iter().filter(|tc| !tc.hidden).count();
        let hidden = self.test_cases.len() - visible;
        if visible < 1 {
            return Err("needs at least 1 visible test case".into());
        }
        if hidden < 1 {
            return Err("needs at least 1 hidden test case".into());
        }
        Ok(())
    }

    /// Privacy pass for IPC (task 0006): built-in problems cross the bridge
    /// with hidden test case values blanked — the runner resolves hidden
    /// cases Rust-side by problem id, so the WebView never needs them.
    /// User/imported problems return in full: the content is already the
    /// user's own, and the create page needs it to edit (`/create?id=`).
    pub fn sanitized_for_ipc(&self) -> Problem {
        let mut p = self.clone();
        if p.source == ProblemSource::BuiltIn {
            for tc in p.test_cases.iter_mut().filter(|tc| tc.hidden) {
                tc.input = Vec::new();
                tc.expected = serde_json::Value::Null;
            }
        }
        p
    }

    pub fn summary(&self, status: ProblemStatus, last_attempted: Option<String>) -> ProblemSummary {
        ProblemSummary {
            id: self.id.clone(),
            number: self.number,
            title: self.title.clone(),
            pattern: self.pattern.clone(),
            difficulty: self.difficulty,
            source: self.source,
            status,
            last_attempted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Round-trips `value` through `T` and asserts the JSON is unchanged —
    /// proves both field names and optional-field omission match types.ts.
    fn round_trip<T: Serialize + for<'de> Deserialize<'de>>(value: serde_json::Value) {
        let parsed: T = serde_json::from_value(value.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&parsed).expect("serialize"), value);
    }

    #[test]
    fn enums_use_the_exact_types_ts_strings() {
        assert_eq!(
            serde_json::to_value(Difficulty::Easy).unwrap(),
            json!("Easy")
        );
        assert_eq!(
            serde_json::to_value(ProblemSource::BuiltIn).unwrap(),
            json!("built-in")
        );
        assert_eq!(
            serde_json::to_value(ProblemStatus::InProgress).unwrap(),
            json!("in-progress")
        );
        assert_eq!(
            serde_json::to_value(ProblemStatus::NeedsReview).unwrap(),
            json!("needs-review")
        );
        assert_eq!(
            serde_json::to_value(ProblemStatus::Todo).unwrap(),
            json!("todo")
        );
        assert_eq!(
            serde_json::to_value(Pattern("Heap / Priority Queue".into())).unwrap(),
            json!("Heap / Priority Queue")
        );
    }

    #[test]
    fn test_case_round_trips_with_positional_input() {
        round_trip::<TestCase>(json!({
            "input": [[2, 7, 11, 15], 9],
            "expected": [0, 1],
            "hidden": false
        }));
    }

    #[test]
    fn problem_summary_uses_camel_case_last_attempted() {
        round_trip::<ProblemSummary>(json!({
            "id": "pair-with-target-sum",
            "number": 1,
            "title": "Pair With Target Sum",
            "pattern": "Arrays & Hashing",
            "difficulty": "Easy",
            "source": "built-in",
            "status": "in-progress",
            "lastAttempted": "2026-06-09"
        }));
        // omitted when never attempted, not null
        let s = ProblemSummary {
            id: "x".into(),
            number: 1,
            title: "X".into(),
            pattern: Pattern("Stack".into()),
            difficulty: Difficulty::Easy,
            source: ProblemSource::BuiltIn,
            status: ProblemStatus::Todo,
            last_attempted: None,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert!(v.get("lastAttempted").is_none());
    }

    #[test]
    fn full_spec_shaped_problem_round_trips() {
        round_trip::<Problem>(json!({
            "id": "fixture-pair-sum",
            "number": 1,
            "title": "Pair With Target Sum",
            "pattern": "Arrays & Hashing",
            "difficulty": "Easy",
            "source": "built-in",
            "description_md": "Given an array `nums` and an integer `target`…",
            "constraints": ["2 <= nums.length <= 10^4"],
            "examples": [
                { "input": "nums = [2,7,11,15], target = 9", "output": "[0,1]",
                  "explanation_md": "Because nums[0] + nums[1] == 9." },
                { "input": "nums = [3,2,4], target = 6", "output": "[1,2]" }
            ],
            "function_signature": {
                "python": "def solve(nums, target):\n    pass",
                "javascript": "function solve(nums, target) {}"
            },
            "test_cases": [
                { "input": [[2, 7, 11, 15], 9], "expected": [0, 1], "hidden": false },
                { "input": [[3, 3], 6], "expected": [0, 1], "hidden": true }
            ],
            "hints": ["Hash map gives O(1) lookups."],
            "reference_solution": {
                "python": "def solve(nums, target): ...",
                "javascript": "function solve(nums, target) {}",
                "complexity": { "time": "O(n)", "space": "O(n)" }
            },
            "explanation_md": "Scan once with a hash map.",
            "follow_up": "Can you do it in one pass?",
            "license": "project-default",
            "author": "built-in"
        }));
    }

    #[test]
    fn sanitized_builtin_problem_carries_no_hidden_values_anywhere() {
        let p: Problem = serde_json::from_value(json!({
            "id": "privacy-fixture",
            "number": 9,
            "title": "Privacy Fixture",
            "pattern": "Stack",
            "difficulty": "Easy",
            "source": "built-in",
            "description_md": "d",
            "constraints": [],
            "examples": [],
            "function_signature": { "python": "def solve(x):", "javascript": "function solve(x) {}" },
            "test_cases": [
                { "input": ["visible-input"], "expected": "visible-expected", "hidden": false },
                { "input": ["secret-input-77631"], "expected": "secret-expected-77631", "hidden": true }
            ],
            "hints": [],
            "license": "project-default",
            "author": "built-in"
        }))
        .unwrap();

        let payload = serde_json::to_string(&p.sanitized_for_ipc()).unwrap();
        assert!(
            !payload.contains("77631"),
            "hidden values leaked: {payload}"
        );
        assert!(payload.contains("visible-input"));
        // hidden case still present (flag + count), just blanked
        assert_eq!(p.sanitized_for_ipc().test_cases.len(), 2);

        // user problems are NOT stripped — the create page edits them
        let mut user = p.clone();
        user.source = ProblemSource::User;
        let user_payload = serde_json::to_string(&user.sanitized_for_ipc()).unwrap();
        assert!(user_payload.contains("secret-input-77631"));
    }

    #[test]
    fn io_type_variants_round_trip_their_wire_names() {
        // The names both harnesses and tools/build_packs.py dispatch on —
        // a rename here silently breaks the whole bundle, so pin them.
        for (value, expect) in [
            (json!("json"), IoType::Json),
            (json!("linked_list"), IoType::LinkedList),
            (json!("tree"), IoType::Tree),
            (json!("cyclic_list"), IoType::CyclicList),
            (json!("random_list"), IoType::RandomList),
            (json!("graph"), IoType::Graph),
            (json!("n_ary_tree"), IoType::NAryTree),
            (json!("quad_tree"), IoType::QuadTree),
            (json!("next_tree"), IoType::NextTree),
            (json!("multilevel_list"), IoType::MultilevelList),
            (
                json!({ "list_of": "tree" }),
                IoType::ListOf(Box::new(IoType::Tree)),
            ),
            (
                json!({ "ctx_only": "linked_list" }),
                IoType::CtxOnly(Box::new(IoType::LinkedList)),
            ),
            (
                json!({ "node_ref": { "param": 0 } }),
                IoType::NodeRef { param: 0 },
            ),
            (
                json!({ "clone_of": { "param": 0 } }),
                IoType::CloneOf { param: 0 },
            ),
            (
                json!({ "tail_of": { "param": 0 } }),
                IoType::TailOf { param: 0 },
            ),
            (
                json!({ "node_index_of": { "param": 0 } }),
                IoType::NodeIndexOf { param: 0 },
            ),
        ] {
            let parsed: IoType = serde_json::from_value(value.clone()).expect("deserialize");
            assert_eq!(parsed, expect);
            assert_eq!(serde_json::to_value(&parsed).unwrap(), value);
        }
    }

    #[test]
    fn judge_and_entry_point_round_trip_on_a_problem() {
        round_trip::<Problem>(json!({
            "id": "imported-two-sum",
            "number": 1,
            "title": "Two Sum",
            "pattern": "Arrays & Hashing",
            "difficulty": "Easy",
            "source": "imported",
            "description_md": "d",
            "constraints": [],
            "examples": [],
            "function_signature": {
                "python": "class Solution:\n    def twoSum(self, nums, target): ...",
                "javascript": "var twoSum = function(nums, target) {};"
            },
            "test_cases": [
                { "input": [[2, 7], 9], "expected": [0, 1], "hidden": false }
            ],
            "judge": { "type": "in_place", "arg_index": 0 },
            "entry_point": { "python": "Solution.twoSum", "javascript": "twoSum", "arity": 2 },
            "hints": [],
            "license": "user-import",
            "author": "imported"
        }));
    }

    #[test]
    fn effective_judge_falls_back_to_checker() {
        let mut p: Problem = serde_json::from_value(json!({
            "id": "minimal", "number": 2, "title": "Minimal", "pattern": "Stack",
            "difficulty": "Medium", "source": "user", "description_md": "d",
            "constraints": [], "examples": [],
            "function_signature": { "python": "def solve():", "javascript": "function solve() {}" },
            "test_cases": [], "hints": [],
            "license": "user-original", "author": "user"
        }))
        .unwrap();
        assert_eq!(p.effective_judge(), Judge::Exact);
        p.checker = Checker::Unordered;
        assert_eq!(p.effective_judge(), Judge::Unordered);
        p.judge = Some(Judge::Float { epsilon: 1e-5 });
        assert_eq!(p.effective_judge(), Judge::Float { epsilon: 1e-5 });
    }

    #[test]
    fn existing_bundled_problem_files_parse_unchanged() {
        // Regression for the schema extension: a built-in-shape problem (no
        // judge / entry_point / body_html) must deserialize and re-serialize to
        // the same JSON — no new fields invented, none dropped. Built-ins are no
        // longer bundled (the catalog loads from the local scrape), so this
        // guards the schema with an inline fixture instead of a shipped file.
        let original = json!({
            "id": "pair-with-target-sum",
            "number": 1,
            "title": "Pair With Target Sum",
            "pattern": "Arrays & Hashing",
            "difficulty": "Easy",
            "source": "built-in",
            "description_md": "Given `nums` and `target`, return the two indices that sum to target.",
            "constraints": ["`2 <= nums.length <= 10^4`", "Only one valid answer exists."],
            "examples": [
                { "input": "nums = [2,7,11,15], target = 9", "output": "[0,1]", "explanation_md": "2 + 7 == 9." },
                { "input": "nums = [3,2,4], target = 6", "output": "[1,2]" }
            ],
            "function_signature": {
                "python": "def solve(nums, target):\n    pass",
                "javascript": "function solve(nums, target) {}"
            },
            "test_cases": [
                { "input": [[2, 7, 11, 15], 9], "expected": [0, 1], "hidden": false },
                { "input": [[3, 2, 4], 6], "expected": [1, 2], "hidden": false },
                { "input": [[3, 3], 6], "expected": [0, 1], "hidden": true }
            ],
            "hints": [
                "A brute-force pass over every pair is O(n^2).",
                "Store each value's index in a hash map as you scan.",
                "For each element, check whether target - value is already in the map."
            ],
            "reference_solution": {
                "python": "def solve(nums, target):\n    return []",
                "javascript": "function solve(nums, target) { return []; }",
                "complexity": { "time": "O(n)", "space": "O(n)" }
            },
            "explanation_md": "Scan once, remembering each visited value's index in a hash map.",
            "follow_up": "Can you devise an algorithm that runs in O(n) time?",
            "license": "project-default",
            "author": "built-in"
        });
        let parsed: Problem = serde_json::from_value(original.clone()).expect("parses as Problem");
        assert!(parsed.judge.is_none());
        assert!(parsed.entry_point.is_none());
        assert_eq!(serde_json::to_value(&parsed).unwrap(), original);
        parsed.validate_structure().expect("still validates");
    }

    #[test]
    fn optional_problem_fields_are_omitted_when_absent() {
        let p: Problem = serde_json::from_value(json!({
            "id": "minimal",
            "number": 2,
            "title": "Minimal",
            "pattern": "Stack",
            "difficulty": "Medium",
            "source": "user",
            "description_md": "d",
            "constraints": [],
            "examples": [],
            "function_signature": { "python": "def solve():", "javascript": "function solve() {}" },
            "test_cases": [],
            "hints": [],
            "license": "user-original",
            "author": "user"
        }))
        .unwrap();
        let v = serde_json::to_value(&p).unwrap();
        for key in [
            "reference_solution",
            "explanation_md",
            "follow_up",
            "judge",
            "entry_point",
        ] {
            assert!(v.get(key).is_none(), "{key} should be omitted");
        }
    }
}
