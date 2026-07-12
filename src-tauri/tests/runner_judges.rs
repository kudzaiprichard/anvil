//! Judge-mode integration tests (task 0003): float / in_place / any_valid /
//! design produce correct pass AND fail verdicts on real interpreter runs.
//! Runtime-gated like all sandbox tests.

mod common;

use app_lib::domain::problem::{DesignIo, EntryPoint, IoType, IoTypes, Judge, TestCase};
use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::runner;
use common::fixture_problem;
use serde_json::json;

fn problem_with(
    judge: Judge,
    entry_point: Option<EntryPoint>,
    cases: Vec<TestCase>,
) -> app_lib::domain::problem::Problem {
    let mut p = fixture_problem();
    p.judge = Some(judge);
    p.entry_point = entry_point;
    p.test_cases = cases;
    p
}

fn case(input: serde_json::Value, expected: serde_json::Value) -> TestCase {
    TestCase {
        input: input.as_array().unwrap().clone(),
        expected,
        hidden: false,
    }
}

// ---------- float ----------

#[test]
fn float_judge_passes_within_epsilon_and_fails_outside() {
    require_runtime!("python");
    let p = problem_with(
        Judge::Float { epsilon: 1e-5 },
        None,
        vec![case(json!([4]), json!(2.0))],
    );
    let near = runner::execute(
        &p,
        Language::Python,
        "def solve(x):\n    return x ** 0.5 + 1e-7",
        true,
    )
    .unwrap();
    assert_eq!(near.status, RunStatus::Pass, "{:?}", near.error);

    let far = runner::execute(
        &p,
        Language::Python,
        "def solve(x):\n    return x ** 0.5 + 0.01",
        true,
    )
    .unwrap();
    assert_eq!(far.status, RunStatus::Fail);
}

// ---------- in_place ----------

fn rotate_problem() -> app_lib::domain::problem::Problem {
    problem_with(
        Judge::InPlace { arg_index: 0 },
        Some(EntryPoint {
            python: "Solution.rotate".into(),
            javascript: "rotate".into(),
            arity: 2,
            io_types: None,
        }),
        vec![case(json!([[1, 2, 3, 4, 5], 2]), json!([4, 5, 1, 2, 3]))],
    )
}

#[test]
fn in_place_python_judges_the_mutated_argument() {
    require_runtime!("python");
    // Void return; only the mutation counts.
    let good = "class Solution:\n    def rotate(self, nums, k):\n        k %= len(nums)\n        nums[:] = nums[-k:] + nums[:-k]\n";
    let result = runner::execute(&rotate_problem(), Language::Python, good, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Returning the right answer WITHOUT mutating must fail.
    let no_mutation =
        "class Solution:\n    def rotate(self, nums, k):\n        return nums[-k:] + nums[:-k]\n";
    let result = runner::execute(&rotate_problem(), Language::Python, no_mutation, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn in_place_javascript_mutates_the_real_array_not_a_copy() {
    require_runtime!("node");
    // Mutating through the parameter alias must be observed.
    let good = "var rotate = function(nums, k) {\n  k %= nums.length;\n  nums.unshift(...nums.splice(nums.length - k));\n};";
    let result = runner::execute(&rotate_problem(), Language::Javascript, good, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Reassigning the parameter (no aliasing mutation) must fail.
    let rebound =
        "var rotate = function(nums, k) {\n  nums = nums.slice(-k).concat(nums.slice(0, -k));\n};";
    let result = runner::execute(&rotate_problem(), Language::Javascript, rebound, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Fail);
}

// ---------- design ----------

fn counter_problem() -> app_lib::domain::problem::Problem {
    // Ops sequence in LeetCode wire format: a Counter with add/total.
    problem_with(
        Judge::Design { design_io: None },
        Some(EntryPoint {
            python: "Counter".into(),
            javascript: "Counter".into(),
            arity: 1,
            io_types: None,
        }),
        vec![case(
            json!([["Counter", "add", "add", "total"], [[10], [5], [7], []]]),
            json!([null, null, null, 22]),
        )],
    )
}

const COUNTER_PY: &str = "class Counter:\n    def __init__(self, start):\n        self.value = start\n    def add(self, n):\n        self.value += n\n    def total(self):\n        return self.value\n";

const COUNTER_JS: &str = "class Counter {\n  constructor(start) { this.value = start; }\n  add(n) { this.value += n; }\n  total() { return this.value; }\n}";

#[test]
fn design_python_collects_per_op_outputs() {
    require_runtime!("python");
    let result = runner::execute(&counter_problem(), Language::Python, COUNTER_PY, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // A wrong total fails the case (outputs compared exactly).
    let wrong = COUNTER_PY.replace("self.value += n", "self.value += n + 1");
    let result = runner::execute(&counter_problem(), Language::Python, &wrong, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn design_javascript_prototype_style_class_works() {
    require_runtime!("node");
    // ES6 class form.
    let result =
        runner::execute(&counter_problem(), Language::Javascript, COUNTER_JS, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // LeetCode's prototype style form.
    let proto = "var Counter = function(start) { this.value = start; };\nCounter.prototype.add = function(n) { this.value += n; };\nCounter.prototype.total = function() { return this.value; };";
    let result = runner::execute(&counter_problem(), Language::Javascript, proto, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

#[test]
fn design_exception_mid_sequence_reports_the_op_index() {
    require_runtime!("python");
    let raising = "class Counter:\n    def __init__(self, start):\n        self.value = start\n    def add(self, n):\n        raise ValueError('boom')\n    def total(self):\n        return self.value\n";
    let result = runner::execute(&counter_problem(), Language::Python, raising, true).unwrap();
    assert_eq!(result.status, RunStatus::Error);
    let err = result.error.unwrap();
    assert!(err.contains("op 1 (add)"), "error was: {err}");
    assert!(err.contains("boom"), "error was: {err}");
}

// ---------- design + design_io (closing-the-48 Phase A) ----------

fn bst_iterator_problem() -> app_lib::domain::problem::Problem {
    // The constructor receives a REAL TreeNode (deserialized from level-order)
    // — the exact gap that kept binary-search-tree-iterator deferred.
    problem_with(
        Judge::Design {
            design_io: Some(DesignIo {
                ctor: vec![IoType::Tree],
                methods: std::collections::BTreeMap::new(),
            }),
        },
        Some(EntryPoint {
            python: "BSTIterator".into(),
            javascript: "BSTIterator".into(),
            arity: 1,
            io_types: None,
        }),
        vec![case(
            json!([
                ["BSTIterator", "next", "next", "hasNext", "next", "hasNext"],
                [[[7, 3, 15, null, null, 9, 20]], [], [], [], [], []]
            ]),
            json!([null, 3, 7, true, 9, true]),
        )],
    )
}

const BST_ITER_PY: &str = "class BSTIterator:\n    def __init__(self, root):\n        self.vals = []\n        def dfs(n):\n            if not n:\n                return\n            dfs(n.left)\n            self.vals.append(n.val)\n            dfs(n.right)\n        dfs(root)\n        self.i = 0\n    def next(self):\n        v = self.vals[self.i]\n        self.i += 1\n        return v\n    def hasNext(self):\n        return self.i < len(self.vals)\n";

const BST_ITER_JS: &str = "class BSTIterator {\n  constructor(root) {\n    this.vals = [];\n    const dfs = (n) => {\n      if (!n) return;\n      dfs(n.left);\n      this.vals.push(n.val);\n      dfs(n.right);\n    };\n    dfs(root);\n    this.i = 0;\n  }\n  next() { return this.vals[this.i++]; }\n  hasNext() { return this.i < this.vals.length; }\n}";

#[test]
fn design_io_hands_python_constructor_a_real_tree_node() {
    require_runtime!("python");
    let result =
        runner::execute(&bst_iterator_problem(), Language::Python, BST_ITER_PY, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Wrong traversal order still fails on exact per-op comparison.
    let wrong = BST_ITER_PY.replace("dfs(n.left)", "dfs(n.right)").replace(
        "self.vals.append(n.val)\n            dfs(n.right)",
        "self.vals.append(n.val)\n            dfs(n.left)",
    );
    let result =
        runner::execute(&bst_iterator_problem(), Language::Python, &wrong, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn design_io_hands_javascript_constructor_a_real_tree_node() {
    require_runtime!("node");
    let result =
        runner::execute(&bst_iterator_problem(), Language::Javascript, BST_ITER_JS, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

// ---------- Phase B wire types: node_ref / graph freshness ----------

fn lca_problem() -> app_lib::domain::problem::Problem {
    // p and q arrive as REAL nodes inside root's tree; the return value is
    // identity-checked against that tree and serialized as its value.
    problem_with(
        Judge::Exact,
        Some(EntryPoint {
            python: "Solution.lowestCommonAncestor".into(),
            javascript: "lowestCommonAncestor".into(),
            arity: 3,
            io_types: Some(IoTypes {
                params: vec![
                    IoType::Tree,
                    IoType::NodeRef { param: 0 },
                    IoType::NodeRef { param: 0 },
                ],
                returns: IoType::NodeRef { param: 0 },
            }),
        }),
        vec![case(
            json!([[6, 2, 8, 0, 4, 7, 9, null, null, 3, 5], 2, 8]),
            json!(6),
        )],
    )
}

#[test]
fn node_ref_params_reach_python_as_real_nodes() {
    require_runtime!("python");
    let code = "class Solution:\n    def lowestCommonAncestor(self, root, p, q):\n        node = root\n        while node:\n            if p.val < node.val and q.val < node.val:\n                node = node.left\n            elif p.val > node.val and q.val > node.val:\n                node = node.right\n            else:\n                return node\n";
    let result = runner::execute(&lca_problem(), Language::Python, code, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Returning a freshly built node (not part of the input tree) must not pass.
    let foreign = "class Solution:\n    def lowestCommonAncestor(self, root, p, q):\n        return TreeNode(6)\n";
    let result = runner::execute(&lca_problem(), Language::Python, foreign, true).unwrap();
    assert_ne!(result.status, RunStatus::Pass);
}

fn clone_graph_problem() -> app_lib::domain::problem::Problem {
    problem_with(
        Judge::Exact,
        Some(EntryPoint {
            python: "Solution.cloneGraph".into(),
            javascript: "cloneGraph".into(),
            arity: 1,
            io_types: Some(IoTypes {
                params: vec![IoType::Graph],
                returns: IoType::Graph,
            }),
        }),
        vec![case(
            json!([[[2, 4], [1, 3], [2, 4], [1, 3]]]),
            json!([[2, 4], [1, 3], [2, 4], [1, 3]]),
        )],
    )
}

#[test]
fn graph_io_type_judges_a_clone_and_rejects_returning_the_input() {
    require_runtime!("python");
    let clone = "class Solution:\n    def cloneGraph(self, node):\n        if not node: return None\n        memo = {}\n        def dfs(n):\n            if id(n) in memo: return memo[id(n)]\n            c = Node(n.val)\n            memo[id(n)] = c\n            c.neighbors = [dfs(x) for x in n.neighbors]\n            return c\n        return dfs(node)\n";
    let result = runner::execute(&clone_graph_problem(), Language::Python, clone, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // The classic cheat — returning the input graph — must not pass.
    let cheat = "class Solution:\n    def cloneGraph(self, node):\n        return node\n";
    let result = runner::execute(&clone_graph_problem(), Language::Python, cheat, true).unwrap();
    assert_ne!(result.status, RunStatus::Pass);
}

#[test]
fn node_ref_params_reach_javascript_as_real_nodes() {
    require_runtime!("node");
    let code = "var lowestCommonAncestor = function(root, p, q) {\n  let node = root;\n  while (node) {\n    if (p.val < node.val && q.val < node.val) node = node.left;\n    else if (p.val > node.val && q.val > node.val) node = node.right;\n    else return node;\n  }\n};";
    let result = runner::execute(&lca_problem(), Language::Javascript, code, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

// ---------- any_valid ----------

fn any_pair_problem() -> app_lib::domain::problem::Problem {
    // "Return any pair of distinct indices whose values sum to the target"
    // — multiple valid answers, so a validator decides.
    let validator_py = "def validate(args, output):\n    nums, target = args\n    if not isinstance(output, list) or len(output) != 2:\n        return False\n    i, j = output\n    if i == j or not (0 <= i < len(nums)) or not (0 <= j < len(nums)):\n        return False\n    return nums[i] + nums[j] == target\n";
    let validator_js = "function validate(args, output) {\n  const [nums, target] = args;\n  if (!Array.isArray(output) || output.length !== 2) return false;\n  const [i, j] = output;\n  if (i === j || i < 0 || j < 0 || i >= nums.length || j >= nums.length) return false;\n  return nums[i] + nums[j] === target;\n}";
    problem_with(
        Judge::AnyValid {
            validator_python: validator_py.into(),
            validator_javascript: validator_js.into(),
        },
        None,
        vec![case(json!([[1, 9, 5, 5], 10]), json!([0, 1]))],
    )
}

#[test]
fn any_valid_python_accepts_a_different_but_valid_answer() {
    require_runtime!("python");
    // Returns [2,3] (5+5) — not the canonical [0,1], still valid.
    let alt = "def solve(nums, target):\n    return [2, 3]";
    let result = runner::execute(&any_pair_problem(), Language::Python, alt, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Well-formed but wrong (1+5 != 10) must fail.
    let wrong = "def solve(nums, target):\n    return [0, 2]";
    let result = runner::execute(&any_pair_problem(), Language::Python, wrong, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn any_valid_javascript_runs_the_pack_validator() {
    require_runtime!("node");
    let alt = "function solve(nums, target) { return [3, 2]; }";
    let result = runner::execute(&any_pair_problem(), Language::Javascript, alt, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    let wrong = "function solve(nums, target) { return [0, 0]; }";
    let result = runner::execute(&any_pair_problem(), Language::Javascript, wrong, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Fail);
}

// ---------- injected shims (closing-the-48 Phase D) ----------

fn first_bad_version_problem() -> app_lib::domain::problem::Problem {
    // The oracle's hidden value rides in the second (never-passed) param slot;
    // python gets a global isBadVersion, javascript a curried entry.
    problem_with(
        Judge::Exact,
        Some(EntryPoint {
            python: "Solution.firstBadVersion".into(),
            javascript: "solution".into(),
            arity: 1,
            io_types: Some(IoTypes {
                params: vec![
                    IoType::Json,
                    IoType::Shim(app_lib::domain::problem::ShimSpec {
                        kind: "is_bad_version".into(),
                        curry_js: true,
                    }),
                ],
                returns: IoType::Json,
            }),
        }),
        vec![case(json!([5, 4]), json!(4))],
    )
}

#[test]
fn is_bad_version_shim_reaches_python_as_a_global() {
    require_runtime!("python");
    let code = "class Solution:\n    def firstBadVersion(self, n):\n        lo, hi = 1, n\n        while lo < hi:\n            mid = (lo + hi) // 2\n            if isBadVersion(mid):\n                hi = mid\n            else:\n                lo = mid + 1\n        return lo\n";
    let result =
        runner::execute(&first_bad_version_problem(), Language::Python, code, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

#[test]
fn is_bad_version_shim_curries_the_javascript_entry() {
    require_runtime!("node");
    let code = "var solution = function(isBadVersion) {\n  return function(n) {\n    let lo = 1, hi = n;\n    while (lo < hi) {\n      const mid = Math.floor((lo + hi) / 2);\n      if (isBadVersion(mid)) hi = mid; else lo = mid + 1;\n    }\n    return lo;\n  };\n};";
    let result =
        runner::execute(&first_bad_version_problem(), Language::Javascript, code, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

#[test]
fn mountain_array_shim_enforces_the_get_budget() {
    require_runtime!("python");
    let p = problem_with(
        Judge::Exact,
        Some(EntryPoint {
            python: "Solution.findInMountainArray".into(),
            javascript: "findInMountainArray".into(),
            arity: 2,
            io_types: Some(IoTypes {
                params: vec![
                    IoType::Json,
                    IoType::Shim(app_lib::domain::problem::ShimSpec {
                        kind: "mountain_array".into(),
                        curry_js: false,
                    }),
                ],
                returns: IoType::Json,
            }),
        }),
        vec![case(
            json!([99, {"arr": [1, 2, 3, 4, 5, 3, 1], "budget": 4}]),
            json!(-1),
        )],
    );
    // A linear scan burns more than 4 gets and must be stopped by the budget.
    let greedy = "class Solution:\n    def findInMountainArray(self, target, mountain_arr):\n        for i in range(mountain_arr.length()):\n            if mountain_arr.get(i) == target:\n                return i\n        return -1\n";
    let result = runner::execute(&p, Language::Python, greedy, true).unwrap();
    assert_ne!(result.status, RunStatus::Pass);
    let err = format!("{:?}", result.error);
    assert!(err.contains("budget"), "error was: {err}");
}

// ---------- round_trip + property (closing-the-48 Phase C) ----------

fn tree_codec_problem() -> app_lib::domain::problem::Problem {
    problem_with(
        Judge::RoundTrip {
            io: IoType::Tree,
            encode: "serialize".into(),
            decode: "deserialize".into(),
        },
        Some(EntryPoint {
            python: "Codec.serialize".into(),
            javascript: "serialize".into(),
            arity: 1,
            io_types: None,
        }),
        vec![case(
            json!([[1, 2, 3, null, null, 4, 5]]),
            json!([1, 2, 3, null, null, 4, 5]),
        )],
    )
}

const CODEC_PY: &str = "class Codec:\n    def serialize(self, root):\n        vals = []\n        def dfs(n):\n            if not n:\n                vals.append('#')\n                return\n            vals.append(str(n.val))\n            dfs(n.left)\n            dfs(n.right)\n        dfs(root)\n        return ','.join(vals)\n    def deserialize(self, data):\n        it = iter(data.split(','))\n        def build():\n            v = next(it)\n            if v == '#':\n                return None\n            n = TreeNode(int(v))\n            n.left = build()\n            n.right = build()\n            return n\n        return build()\n";

#[test]
fn round_trip_judge_accepts_any_working_codec_and_rejects_a_broken_one() {
    require_runtime!("python");
    let result = runner::execute(&tree_codec_problem(), Language::Python, CODEC_PY, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // Encode preorder but decode as if it were mirrored — halves disagree.
    let broken = CODEC_PY.replacen(
        "vals.append(str(n.val))\n            dfs(n.left)\n            dfs(n.right)",
        "vals.append(str(n.val))\n            dfs(n.right)\n            dfs(n.left)",
        1,
    );
    let result = runner::execute(&tree_codec_problem(), Language::Python, &broken, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

fn randomized_set_problem() -> app_lib::domain::problem::Problem {
    let validator_py = "def validate(args, outputs):\n    ops, arg_lists = args\n    present = set()\n    for i in range(1, len(ops)):\n        op, a, out = ops[i], arg_lists[i], outputs[i]\n        if op == 'insert':\n            ok = a[0] not in present\n            present.add(a[0])\n            if out != ok: return False\n        elif op == 'remove':\n            ok = a[0] in present\n            present.discard(a[0])\n            if out != ok: return False\n        elif op == 'getRandom':\n            if out not in present: return False\n    return True\n";
    let validator_js = "function validate(args, outputs) {\n  const [ops, argLists] = args;\n  const present = new Set();\n  for (let i = 1; i < ops.length; i++) {\n    const op = ops[i], a = argLists[i], out = outputs[i];\n    if (op === 'insert') {\n      const ok = !present.has(a[0]);\n      present.add(a[0]);\n      if (out !== ok) return false;\n    } else if (op === 'remove') {\n      const ok = present.has(a[0]);\n      present.delete(a[0]);\n      if (out !== ok) return false;\n    } else if (op === 'getRandom') {\n      if (!present.has(out)) return false;\n    }\n  }\n  return true;\n}";
    problem_with(
        Judge::Property {
            validator_python: validator_py.into(),
            validator_javascript: validator_js.into(),
            exec: app_lib::domain::problem::PropertyExec::Design,
            design_io: None,
        },
        Some(EntryPoint {
            python: "RandomizedSet".into(),
            javascript: "RandomizedSet".into(),
            arity: 0,
            io_types: None,
        }),
        vec![case(
            json!([
                ["RandomizedSet", "insert", "insert", "getRandom", "remove", "getRandom"],
                [[], [1], [2], [], [1], []]
            ]),
            // A sample run's outputs — display only; the validator judges.
            json!([null, true, true, 1, true, 2]),
        )],
    )
}

const RSET_PY: &str = "import random\nclass RandomizedSet:\n    def __init__(self):\n        self.items = []\n        self.pos = {}\n    def insert(self, v):\n        if v in self.pos: return False\n        self.pos[v] = len(self.items)\n        self.items.append(v)\n        return True\n    def remove(self, v):\n        if v not in self.pos: return False\n        i = self.pos.pop(v)\n        last = self.items.pop()\n        if i < len(self.items):\n            self.items[i] = last\n            self.pos[last] = i\n        return True\n    def getRandom(self):\n        return random.choice(self.items)\n";

#[test]
fn property_judge_validates_random_outputs_per_call() {
    require_runtime!("python");
    let result =
        runner::execute(&randomized_set_problem(), Language::Python, RSET_PY, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // getRandom returning something not in the set must fail even though
    // every run's outputs legitimately differ.
    let bad = RSET_PY.replace("return random.choice(self.items)", "return -999999");
    let result =
        runner::execute(&randomized_set_problem(), Language::Python, &bad, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn property_judge_runs_the_javascript_validator() {
    require_runtime!("node");
    let sol = "class RandomizedSet {\n  constructor() { this.items = []; this.pos = new Map(); }\n  insert(v) {\n    if (this.pos.has(v)) return false;\n    this.pos.set(v, this.items.length);\n    this.items.push(v);\n    return true;\n  }\n  remove(v) {\n    if (!this.pos.has(v)) return false;\n    const i = this.pos.get(v);\n    this.pos.delete(v);\n    const last = this.items.pop();\n    if (i < this.items.length) { this.items[i] = last; this.pos.set(last, i); }\n    return true;\n  }\n  getRandom() { return this.items[Math.floor(Math.random() * this.items.length)]; }\n}";
    let result =
        runner::execute(&randomized_set_problem(), Language::Javascript, sol, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

// ---------- regression: legacy judges byte-identical ----------

#[test]
fn exact_and_unordered_problems_run_exactly_as_before() {
    require_runtime!("python");
    // The plain fixture (no judge, no entry point) — legacy path end to end.
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    return a + b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass);
    assert_eq!(result.passed, 3);

    // checker: unordered, still via the legacy field
    let mut p = fixture_problem();
    p.checker = app_lib::domain::problem::Checker::Unordered;
    p.test_cases = vec![case(json!([[3, 1, 2]]), json!([1, 2, 3]))];
    let result = runner::execute(
        &p,
        Language::Python,
        "def solve(nums):\n    return nums",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}

// ---------- node I/O adapter (task 0003) ----------

fn entry(python: &str, javascript: &str, arity: u32, io: IoTypes) -> Option<EntryPoint> {
    Some(EntryPoint {
        python: python.into(),
        javascript: javascript.into(),
        arity,
        io_types: Some(io),
    })
}

#[test]
fn linked_list_python_stub_runs_unmodified_and_judges() {
    require_runtime!("python");
    let p = problem_with(
        Judge::Exact,
        entry(
            "Solution.reverseList",
            "reverseList",
            1,
            IoTypes {
                params: vec![IoType::LinkedList],
                returns: IoType::LinkedList,
            },
        ),
        vec![case(json!([[1, 2, 3, 4, 5]]), json!([5, 4, 3, 2, 1]))],
    );
    // The annotated LeetCode stub (referencing ListNode + Optional) must run as
    // pasted — proving the harness injects the node classes.
    let good = "from typing import Optional\nclass Solution:\n    def reverseList(self, head: Optional[ListNode]) -> Optional[ListNode]:\n        prev = None\n        while head:\n            nxt = head.next\n            head.next = prev\n            prev = head\n            head = nxt\n        return prev\n";
    let result = runner::execute(&p, Language::Python, good, true).unwrap();
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    // An identity (non-reversing) solution fails the case.
    let wrong = "class Solution:\n    def reverseList(self, head):\n        return head\n";
    let result = runner::execute(&p, Language::Python, wrong, true).unwrap();
    assert_eq!(result.status, RunStatus::Fail);
}

#[test]
fn binary_tree_javascript_stub_runs_unmodified_and_judges() {
    require_runtime!("node");
    let p = problem_with(
        Judge::Exact,
        entry(
            "Solution.maxDepth",
            "maxDepth",
            1,
            IoTypes {
                params: vec![IoType::Tree],
                returns: IoType::Json,
            },
        ),
        vec![case(json!([[3, 9, 20, null, null, 15, 7]]), json!(3))],
    );
    let good = "var maxDepth = function(root) {\n  if (!root) return 0;\n  return 1 + Math.max(maxDepth(root.left), maxDepth(root.right));\n};";
    let result = runner::execute(&p, Language::Javascript, good, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);

    let wrong = "var maxDepth = function(root) { return root ? 1 : 0; };";
    let result = runner::execute(&p, Language::Javascript, wrong, true).unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Fail);
}
