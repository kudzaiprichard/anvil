//! Closing-the-48 end-to-end: for one problem per new mechanism, drive the
//! EXACT path the app takes — real catalog row → pack fingerprint → stress
//! materialization → `merge_question` → `runner::execute` — and assert the
//! merged problem judges a correct solution Pass and a cheating/wrong one
//! not-Pass, at Full tier. Runtime-gated like all sandbox tests; skipped
//! when the local dev catalog (`resources/catalog/catalog_leetcode.json`,
//! untracked) is absent, e.g. on CI.

mod common;

use app_lib::domain::lc_import::{ScrapeFile, ScrapeQuestion};
use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::lc_import::{merge_question, verify_fingerprint};
use app_lib::services::pack_store::{materialize_stress, PackStore};
use app_lib::services::runner;
use std::path::PathBuf;

fn shipped_packs() -> PackStore {
    PackStore::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test-packs.json.gz"),
    )
}

/// The local dev scrape, or None when it isn't on this machine (CI).
fn load_catalog() -> Option<ScrapeFile> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("catalog")
        .join("catalog_leetcode.json");
    if !path.exists() {
        eprintln!("SKIPPED: dev catalog not present at {}", path.display());
        return None;
    }
    let raw = std::fs::read_to_string(path).expect("catalog readable");
    Some(serde_json::from_str(&raw).expect("catalog parses as ScrapeFile"))
}

fn question<'a>(catalog: &'a ScrapeFile, slug: &str) -> &'a ScrapeQuestion {
    catalog
        .questions
        .iter()
        .find(|q| q.slug == slug)
        .unwrap_or_else(|| panic!("{slug} missing from catalog"))
}

/// One mechanism's scenario: the pack's own python reference must Pass and
/// the given wrong/cheating solution must not.
struct Scenario {
    slug: &'static str,
    mechanism: &'static str,
    wrong_python: &'static str,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        slug: "lowest-common-ancestor-of-a-binary-search-tree",
        mechanism: "node_ref params + identity-checked node return",
        // Always answering p is wrong whenever the ancestor differs.
        wrong_python: "class Solution:\n    def lowestCommonAncestor(self, root, p, q):\n        return p\n",
    },
    Scenario {
        slug: "linked-list-cycle-ii",
        mechanism: "cyclic_list wire form + node_index_of return",
        wrong_python: "class Solution:\n    def detectCycle(self, head):\n        return None\n",
    },
    Scenario {
        slug: "clone-graph",
        mechanism: "graph io type + returned-copy freshness check",
        // The classic cheat: hand back the input graph itself.
        wrong_python: "class Solution:\n    def cloneGraph(self, node):\n        return node\n",
    },
    Scenario {
        slug: "serialize-and-deserialize-binary-tree",
        mechanism: "round_trip judge (codec canonicalization)",
        // Encode preorder, decode mirrored — halves disagree, tree comes back wrong.
        wrong_python: "class Codec:\n    def serialize(self, root):\n        vals = []\n        def dfs(n):\n            if not n:\n                vals.append('#')\n                return\n            vals.append(str(n.val))\n            dfs(n.left)\n            dfs(n.right)\n        dfs(root)\n        return ','.join(vals)\n    def deserialize(self, data):\n        it = iter(data.split(','))\n        def build():\n            v = next(it)\n            if v == '#':\n                return None\n            n = TreeNode(int(v))\n            n.right = build()\n            n.left = build()\n            return n\n        return build()\n",
    },
    Scenario {
        slug: "first-bad-version",
        mechanism: "is_bad_version global shim",
        wrong_python: "class Solution:\n    def firstBadVersion(self, n):\n        return n\n",
    },
    Scenario {
        slug: "insert-delete-getrandom-o1",
        mechanism: "property judge (op-replay validator)",
        wrong_python: "import random\nclass RandomizedSet:\n    def __init__(self):\n        self.items = []\n        self.pos = {}\n    def insert(self, val):\n        if val in self.pos: return False\n        self.pos[val] = len(self.items)\n        self.items.append(val)\n        return True\n    def remove(self, val):\n        if val not in self.pos: return False\n        i = self.pos.pop(val)\n        last = self.items.pop()\n        if i < len(self.items):\n            self.items[i] = last\n            self.pos[last] = i\n        return True\n    def getRandom(self):\n        return -999999\n",
    },
    Scenario {
        slug: "binary-search-tree-iterator",
        mechanism: "design_io tree constructor",
        wrong_python: "class BSTIterator:\n    def __init__(self, root):\n        pass\n    def next(self):\n        return 0\n    def hasNext(self):\n        return False\n",
    },
    Scenario {
        slug: "guess-the-word",
        mechanism: "master_guess shim verdict + call budget",
        // One wrong guess, then gives up — the Master's verdict stays false.
        wrong_python: "class Solution:\n    def findSecretWord(self, words, master):\n        master.guess(words[-1])\n",
    },
    Scenario {
        slug: "flatten-a-multilevel-doubly-linked-list",
        mechanism: "multilevel_list wire form",
        wrong_python: "class Solution:\n    def flatten(self, head):\n        return head\n",
    },
    Scenario {
        slug: "maximum-path-intersection-sum-in-a-grid",
        mechanism: "column-interval DP (the resolved 48.md section-4.8 defer)",
        wrong_python: "class Solution:\n    def maxScore(self, grid):\n        return 0\n",
    },
    Scenario {
        slug: "print-in-order",
        mechanism: "concurrency judge (amplified thread scheduling)",
        wrong_python: "class Foo:\n    def __init__(self):\n        pass\n    def first(self, printFirst):\n        printFirst()\n    def second(self, printSecond):\n        printSecond()\n    def third(self, printThird):\n        printThird()\n",
    },
    Scenario {
        slug: "the-dining-philosophers",
        mechanism: "concurrency judge (fork-exclusivity validator)",
        // Eats without ever picking up the right fork — always invalid.
        wrong_python: "import threading\nclass DiningPhilosophers:\n    def __init__(self):\n        self.table = threading.Lock()\n    def wantsToEat(self, philosopher, pickLeftFork, pickRightFork, eat, putLeftFork, putRightFork):\n        with self.table:\n            pickLeftFork()\n            eat()\n            putLeftFork()\n",
    },
];

/// Every closing-the-48 slug. The reference-passes/tier check runs for all of
/// them; the SCENARIOS above additionally prove a wrong solution fails.
const ALL_48: &[&str] = &[
    "lowest-common-ancestor-of-a-binary-search-tree",
    "linked-list-cycle",
    "clone-graph",
    "serialize-and-deserialize-binary-tree",
    "first-bad-version",
    "insert-delete-getrandom-o1",
    "binary-search-tree-iterator",
    "lowest-common-ancestor-of-a-binary-tree",
    "all-nodes-distance-k-in-binary-tree",
    "find-a-corresponding-node-of-a-binary-tree-in-a-clone-of-that-tree",
    "intersection-of-two-linked-lists",
    "delete-node-in-a-linked-list",
    "linked-list-cycle-ii",
    "copy-list-with-random-pointer",
    "serialize-and-deserialize-bst",
    "encode-and-decode-tinyurl",
    "populating-next-right-pointers-in-each-node",
    "populating-next-right-pointers-in-each-node-ii",
    "construct-quad-tree",
    "logical-or-of-two-binary-grids-represented-as-quad-trees",
    "flatten-a-multilevel-doubly-linked-list",
    "n-ary-tree-level-order-traversal",
    "n-ary-tree-preorder-traversal",
    "n-ary-tree-postorder-traversal",
    "maximum-depth-of-n-ary-tree",
    "guess-number-higher-or-lower",
    "find-in-mountain-array",
    "guess-the-word",
    "find-positive-integer-solution-for-a-given-equation",
    "peeking-iterator",
    "flatten-nested-list-iterator",
    "implement-rand10-using-rand7",
    "insert-delete-getrandom-o1-duplicates-allowed",
    "linked-list-random-node",
    "shuffle-an-array",
    "random-pick-index",
    "random-pick-with-weight",
    "random-pick-with-blacklist",
    "random-flip-matrix",
    "generate-random-point-in-a-circle",
    "random-point-in-non-overlapping-rectangles",
    "maximum-path-intersection-sum-in-a-grid",
    // The final six: the concurrency judge (python-only packs).
    "print-in-order",
    "print-foobar-alternately",
    "print-zero-even-odd",
    "building-h2o",
    "fizz-buzz-multithreaded",
    "the-dining-philosophers",
];

#[test]
fn all_48_closing_the_48_packs_import_and_reference_passes() {
    require_runtime!("python");
    let Some(catalog) = load_catalog() else {
        return;
    };
    let store = shipped_packs();

    for slug in ALL_48 {
        let pack = store
            .get(slug)
            .unwrap_or_else(|| panic!("{slug}: pack missing from shipped bundle"));
        let q = question(&catalog, slug);
        assert!(
            verify_fingerprint(&pack.entry_point, &q.code_stubs.python),
            "{slug}: stub fingerprint failed"
        );

        let stress = materialize_stress(pack, "python");
        if common::stress_skipped_by_sandbox(&stress.skipped) {
            eprintln!("SKIPPED mid-run: sandbox cannot exec generators here");
            return;
        }
        assert!(
            stress.skipped.is_empty(),
            "{slug}: stress skipped: {:?}",
            stress.skipped
        );

        let merged = merge_question(q, Some(pack), stress.cases, &[], 1);
        assert_eq!(merged.problem.experience_tier(), "full", "{slug}");
        assert!(
            merged.problem.test_cases.iter().any(|c| !c.hidden),
            "{slug}: no visible cases after merge"
        );
        assert!(
            merged.problem.test_cases.iter().any(|c| c.hidden),
            "{slug}: no hidden cases after merge"
        );

        let result = runner::execute(
            &merged.problem,
            Language::Python,
            &pack.solutions.python,
            true,
        )
        .unwrap();
        assert_eq!(
            result.status,
            RunStatus::Pass,
            "{slug}: reference failed: {:?}",
            result.error
        );
        eprintln!("OK import+run {slug} ({} cases)", result.total);
    }
}

#[test]
fn every_new_mechanism_runs_full_tier_end_to_end() {
    require_runtime!("python");
    let Some(catalog) = load_catalog() else {
        return;
    };
    let store = shipped_packs();

    for s in SCENARIOS {
        let pack = store
            .get(s.slug)
            .unwrap_or_else(|| panic!("{}: pack missing from shipped bundle", s.slug));
        let q = question(&catalog, s.slug);

        // 1. The real catalog stub must fingerprint against the pack — this is
        // the gate the app applies before granting Full tier.
        assert!(
            verify_fingerprint(&pack.entry_point, &q.code_stubs.python),
            "{}: stub fingerprint failed",
            s.slug
        );

        // 2. Stress generators materialize into hidden cases via the sandbox.
        let stress = materialize_stress(pack, "python");
        if common::stress_skipped_by_sandbox(&stress.skipped) {
            eprintln!("SKIPPED mid-run: sandbox cannot exec generators here");
            return;
        }
        assert!(
            stress.skipped.is_empty(),
            "{}: stress skipped: {:?}",
            s.slug,
            stress.skipped
        );

        // 3. Merge exactly as the import flow does.
        let merged = merge_question(q, Some(pack), stress.cases, &[], 1);
        assert!(
            merged.problem.test_cases.iter().any(|c| c.hidden),
            "{}: no hidden cases after merge",
            s.slug
        );
        assert_eq!(
            merged.problem.experience_tier(),
            "full",
            "{}: merged problem is not full tier",
            s.slug
        );

        // 4. The pack's own reference passes through the real runner…
        let result = runner::execute(
            &merged.problem,
            Language::Python,
            &pack.solutions.python,
            true,
        )
        .unwrap();
        assert_eq!(
            result.status,
            RunStatus::Pass,
            "{} [{}]: reference failed: {:?}",
            s.slug,
            s.mechanism,
            result.error
        );

        // 5. …and the cheat/wrong solution must not.
        let result =
            runner::execute(&merged.problem, Language::Python, s.wrong_python, true).unwrap();
        assert_ne!(
            result.status,
            RunStatus::Pass,
            "{} [{}]: the wrong solution passed",
            s.slug,
            s.mechanism
        );
        eprintln!("OK e2e {} [{}]", s.slug, s.mechanism);
    }
}

#[test]
fn curried_javascript_shim_stub_runs_full_tier_end_to_end() {
    require_runtime!("python");
    require_runtime!("node");
    let Some(catalog) = load_catalog() else {
        return;
    };
    let store = shipped_packs();
    let pack = store.get("first-bad-version").expect("pack present");
    let q = question(&catalog, "first-bad-version");

    let stress = materialize_stress(pack, "python");
    if common::stress_skipped_by_sandbox(&stress.skipped) {
        eprintln!("SKIPPED: sandbox cannot exec generators here");
        return;
    }
    let merged = merge_question(q, Some(pack), stress.cases, &[], 1);

    // LeetCode's curried JS stub shape must run against the same pack.
    let result = runner::execute(
        &merged.problem,
        Language::Javascript,
        pack.solutions
            .javascript
            .as_deref()
            .expect("first-bad-version ships a JS reference"),
        true,
    )
    .unwrap();
    let Some(result) = common::skip_if_node_unavailable(result) else {
        return;
    };
    assert_eq!(result.status, RunStatus::Pass, "{:?}", result.error);
}
