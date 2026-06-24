"""Dev helper: verify preset slug candidates against the dev scrape catalog.

Usage: python tools/check_preset_slugs.py
Reads the candidate lists below (and, once they exist, the shipped preset
files under src-tauri/resources/presets/) and reports any slug not present
in .docs/my_questions.json.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

NEETCODE_150 = {
    "Arrays & Hashing": [
        "contains-duplicate", "valid-anagram", "two-sum", "group-anagrams",
        "top-k-frequent-elements", "encode-and-decode-strings",
        "product-of-array-except-self", "valid-sudoku",
        "longest-consecutive-sequence",
    ],
    "Two Pointers": [
        "valid-palindrome", "two-sum-ii-input-array-is-sorted", "3sum",
        "container-with-most-water", "trapping-rain-water",
    ],
    "Sliding Window": [
        "best-time-to-buy-and-sell-stock",
        "longest-substring-without-repeating-characters",
        "longest-repeating-character-replacement", "permutation-in-string",
        "minimum-window-substring", "sliding-window-maximum",
    ],
    "Stack": [
        "valid-parentheses", "min-stack", "evaluate-reverse-polish-notation",
        "generate-parentheses", "daily-temperatures", "car-fleet",
        "largest-rectangle-in-histogram",
    ],
    "Binary Search": [
        "binary-search", "search-a-2d-matrix", "koko-eating-bananas",
        "find-minimum-in-rotated-sorted-array", "search-in-rotated-sorted-array",
        "time-based-key-value-store", "median-of-two-sorted-arrays",
    ],
    "Linked List": [
        "reverse-linked-list", "merge-two-sorted-lists", "linked-list-cycle",
        "reorder-list", "remove-nth-node-from-end-of-list",
        "copy-list-with-random-pointer", "add-two-numbers",
        "find-the-duplicate-number", "lru-cache", "merge-k-sorted-lists",
        "reverse-nodes-in-k-group",
    ],
    "Trees": [
        "invert-binary-tree", "maximum-depth-of-binary-tree",
        "diameter-of-binary-tree", "balanced-binary-tree", "same-tree",
        "subtree-of-another-tree",
        "lowest-common-ancestor-of-a-binary-search-tree",
        "binary-tree-level-order-traversal", "binary-tree-right-side-view",
        "count-good-nodes-in-binary-tree", "validate-binary-search-tree",
        "kth-smallest-element-in-a-bst",
        "construct-binary-tree-from-preorder-and-inorder-traversal",
        "binary-tree-maximum-path-sum", "serialize-and-deserialize-binary-tree",
        # Tries (mapped onto Trees in Anvil's 15 patterns)
        "implement-trie-prefix-tree", "design-add-and-search-words-data-structure",
        "word-search-ii",
    ],
    "Heap / Priority Queue": [
        "kth-largest-element-in-a-stream", "last-stone-weight",
        "k-closest-points-to-origin", "kth-largest-element-in-an-array",
        "task-scheduler", "design-twitter", "find-median-from-data-stream",
    ],
    "Backtracking": [
        "subsets", "combination-sum", "permutations", "subsets-ii",
        "combination-sum-ii", "word-search", "palindrome-partitioning",
        "letter-combinations-of-a-phone-number", "n-queens",
    ],
    "Graphs": [
        "number-of-islands", "clone-graph", "max-area-of-island",
        "pacific-atlantic-water-flow", "surrounded-regions", "rotting-oranges",
        "walls-and-gates", "course-schedule", "course-schedule-ii",
        "redundant-connection",
        "number-of-connected-components-in-an-undirected-graph",
        "graph-valid-tree", "word-ladder",
        # Advanced graphs (same Anvil bucket)
        "reconstruct-itinerary", "min-cost-to-connect-all-points",
        "network-delay-time", "swim-in-rising-water", "alien-dictionary",
        "cheapest-flights-within-k-stops",
    ],
    "1-D DP": [
        "climbing-stairs", "min-cost-climbing-stairs", "house-robber",
        "house-robber-ii", "longest-palindromic-substring",
        "palindromic-substrings", "decode-ways", "coin-change",
        "maximum-product-subarray", "word-break",
        "longest-increasing-subsequence", "partition-equal-subset-sum",
    ],
    "2-D DP": [
        "unique-paths", "longest-common-subsequence",
        "best-time-to-buy-and-sell-stock-with-cooldown", "coin-change-ii",
        "target-sum", "interleaving-string",
        "longest-increasing-path-in-a-matrix", "distinct-subsequences",
        "edit-distance", "burst-balloons", "regular-expression-matching",
    ],
    "Greedy": [
        "maximum-subarray", "jump-game", "jump-game-ii", "gas-station",
        "hand-of-straights", "merge-triplets-to-form-target-triplet",
        "partition-labels", "valid-parenthesis-string",
    ],
    "Intervals": [
        "insert-interval", "merge-intervals", "non-overlapping-intervals",
        "meeting-rooms", "meeting-rooms-ii",
        "minimum-interval-to-include-each-query",
    ],
    "Arrays & Hashing (Math & Geometry)": [
        "rotate-image", "spiral-matrix", "set-matrix-zeroes", "happy-number",
        "plus-one", "powx-n", "multiply-strings", "detect-squares",
    ],
    "Bit Manipulation": [
        "single-number", "number-of-1-bits", "counting-bits", "reverse-bits",
        "missing-number", "sum-of-two-integers", "reverse-integer",
    ],
}

BLIND_75 = {
    "Arrays & Hashing": [
        "two-sum", "best-time-to-buy-and-sell-stock", "contains-duplicate",
        "product-of-array-except-self", "valid-anagram", "group-anagrams",
        "top-k-frequent-elements", "encode-and-decode-strings",
        "longest-consecutive-sequence",
    ],
    "Two Pointers": ["valid-palindrome", "3sum", "container-with-most-water"],
    "Sliding Window": [
        "longest-substring-without-repeating-characters",
        "longest-repeating-character-replacement", "minimum-window-substring",
    ],
    "Stack": ["valid-parentheses"],
    "Binary Search": [
        "find-minimum-in-rotated-sorted-array", "search-in-rotated-sorted-array",
    ],
    "Linked List": [
        "reverse-linked-list", "linked-list-cycle", "merge-two-sorted-lists",
        "merge-k-sorted-lists", "remove-nth-node-from-end-of-list",
        "reorder-list",
    ],
    "Trees": [
        "maximum-depth-of-binary-tree", "same-tree", "invert-binary-tree",
        "binary-tree-maximum-path-sum", "binary-tree-level-order-traversal",
        "serialize-and-deserialize-binary-tree", "subtree-of-another-tree",
        "construct-binary-tree-from-preorder-and-inorder-traversal",
        "validate-binary-search-tree", "kth-smallest-element-in-a-bst",
        "lowest-common-ancestor-of-a-binary-search-tree",
        "implement-trie-prefix-tree",
        "design-add-and-search-words-data-structure", "word-search-ii",
    ],
    "Heap / Priority Queue": ["find-median-from-data-stream"],
    "Graphs": [
        "clone-graph", "course-schedule", "pacific-atlantic-water-flow",
        "number-of-islands", "alien-dictionary", "graph-valid-tree",
        "number-of-connected-components-in-an-undirected-graph",
    ],
    "1-D DP": [
        "climbing-stairs", "coin-change", "longest-increasing-subsequence",
        "word-break", "combination-sum-iv", "house-robber", "house-robber-ii",
        "decode-ways", "maximum-product-subarray",
        "longest-palindromic-substring", "palindromic-substrings",
    ],
    "2-D DP": ["longest-common-subsequence", "unique-paths"],
    "Greedy": ["maximum-subarray", "jump-game"],
    "Intervals": [
        "insert-interval", "merge-intervals", "non-overlapping-intervals",
        "meeting-rooms", "meeting-rooms-ii",
    ],
    "Arrays & Hashing (Matrix)": [
        "set-matrix-zeroes", "spiral-matrix", "rotate-image", "word-search",
    ],
    "Bit Manipulation": [
        "sum-of-two-integers", "number-of-1-bits", "counting-bits",
        "missing-number", "reverse-bits",
    ],
}


# LeetCode-premium problems: real catalog slugs, but absent from any
# free-account scrape. Flagged in the preset files so the importer can say
# "premium — not in your export" instead of silently matching nothing.
PREMIUM = {
    "encode-and-decode-strings", "walls-and-gates",
    "number-of-connected-components-in-an-undirected-graph",
    "graph-valid-tree", "alien-dictionary", "meeting-rooms",
    "meeting-rooms-ii",
}

# Anvil's 15 patterns (mirror of src/lib/types.ts PATTERNS).
PATTERNS = [
    "Arrays & Hashing", "Two Pointers", "Sliding Window", "Stack",
    "Binary Search", "Linked List", "Trees", "Heap / Priority Queue",
    "Backtracking", "Graphs", "1-D DP", "2-D DP", "Greedy", "Intervals",
    "Bit Manipulation",
]


def flatten(groups):
    return [slug for slugs in groups.values() for slug in slugs]


def merged_groups(groups):
    """Merge helper labels like 'Arrays & Hashing (Matrix)' into the base
    pattern and order groups by the canonical PATTERNS order."""
    merged = {}
    for label, slugs in groups.items():
        base = label.split(" (")[0]
        assert base in PATTERNS, base
        merged.setdefault(base, []).extend(slugs)
    return [
        {"pattern": p, "slugs": merged[p]} for p in PATTERNS if p in merged
    ]


def write_presets():
    out_dir = ROOT / "src-tauri" / "resources" / "presets"
    out_dir.mkdir(parents=True, exist_ok=True)
    for preset_id, name, groups in [
        ("blind75", "Blind 75", BLIND_75),
        ("neetcode150", "NeetCode 150", NEETCODE_150),
    ]:
        slugs = set(flatten(groups))
        payload = {
            "id": preset_id,
            "name": name,
            "groups": merged_groups(groups),
            "premium": sorted(slugs & PREMIUM),
        }
        path = out_dir / f"{preset_id}.json"
        path.write_text(
            json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        print(f"wrote {path}")


def main():
    catalog_path = ROOT / ".docs" / "my_questions.json"
    data = json.loads(catalog_path.read_text(encoding="utf-8"))
    catalog = {q["slug"] for q in data["questions"]}
    print(f"catalog: {len(catalog)} slugs (free account — premium absent)")

    ok = True
    for name, groups in [("neetcode150", NEETCODE_150), ("blind75", BLIND_75)]:
        slugs = flatten(groups)
        dupes = {s for s in slugs if slugs.count(s) > 1}
        missing = [s for s in slugs if s not in catalog and s not in PREMIUM]
        print(f"{name}: {len(slugs)} slugs, {len(set(slugs))} unique")
        if dupes:
            ok = False
            print(f"  DUPES: {sorted(dupes)}")
        if missing:
            ok = False
            print(f"  MISSING FROM CATALOG: {missing}")

    b75 = set(flatten(BLIND_75))
    n150 = set(flatten(NEETCODE_150))
    print(f"blind75 - neetcode150 = {sorted(b75 - n150)}")
    if not ok:
        sys.exit(1)
    write_presets()


if __name__ == "__main__":
    main()
