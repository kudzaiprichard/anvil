# Pack authoring guide (task 0002)

How to hand-author one verified test pack. Read this fully before writing a
`tools/packs/<slug>.json`. The build (`tools/build_packs.py`) is the gate; this
guide is how you pass it the first time and, more importantly, how you avoid
*freezing a subtly-wrong pack* (the verifier catches wrong expecteds and
cross-language disagreement, but it does **not** catch a wrong **judge type** —
that is on you, see §4).

## 0. The one rule

**Never type an expected value.** The source file has no place for one. Every
`expected` is computed by executing your python reference through the real
sandbox harness. You write *inputs* and *solutions*; the build computes outputs
and cross-checks python vs javascript vs the brute-force oracle.

## 1. Source file shape

```jsonc
{
  "slug": "exact-match-the-filename",
  "judge": "exact",                       // string, or an object (see §4)
  "pattern": "one original sentence on what this teaches",
  "hints": ["nudge", "approach", "near-answer"],   // exactly 3, progressive
  "constraints": [                        // numeric facts only (uncopyrightable)
    { "param": "nums", "kind": "int[]", "len": [1, 100000], "value": [-1000000000, 1000000000] }
  ],
  "edge_inputs": [                        // INPUTS ONLY. bare arg-list, or rich entry:
    { "kind": "edge",     "description": "…", "input": [ /* positional args */ ] },
    { "kind": "boundary", "description": "…", "input": [ … ] },
    { "kind": "trap",     "description": "…", "input": [ … ] }
  ],
  "stress": [                             // >= 1 deterministic generator
    { "description": "…", "seed": 1, "size": 10000,
      "generator_python": "def gen(rng, size):\n    ...\n    return [arg1, arg2]\n",
      "note": "optional" }
  ],
  "oracle_python": "class Solution:\n    def method(self, ...): ...  # naive but obviously correct",
  "solutions": {
    "python":     "class Solution:\n    def method(self, ...): ...",
    "javascript": "var method = function(...) { ... };"
  },
  "complexity": { "time": "O(n)", "space": "O(1)" }   // optional
}
```

- **No** `expected`, `output`, `tests` keys anywhere — the build rejects them.
- `entry_point` is **derived from the scraped stub** automatically; do not write it.
- `oracle_python` is optional but **strongly** wanted: a second, independent,
  obviously-correct implementation is the differential check that catches a clever
  bug in your optimal one. Make it genuinely different (brute force), not a copy.

## 2. Input conventions (match the harness exactly)

- A test `input` is the **positional argument list** of the entry function, in
  order. `twoSum(nums, target)` with `nums=[2,7,11,15], target=9` ⇒
  `"input": [[2,7,11,15], 9]`. A 1-arg function still takes a list: `isValid(s)`
  with `s="()"` ⇒ `"input": ["()"]`.
- **Python** solution must match the scraped stub: `class Solution:` with the
  method named as in the stub (the build derives `Solution.method`).
- **JavaScript** solution must define a top-level binding named exactly as the JS
  stub's function (e.g. `var twoSum = function(nums, target) { … };`). Arrow/`function`
  forms work too; the name must match.
- Both languages must **return** the answer (never print). Use only plain JSON
  values (numbers, strings, booleans, arrays, objects, null). No `ListNode`,
  `TreeNode`, or other object graphs — those problems are out of scope this phase
  (no runtime node adapter) and ship basic-mode.
- Anchors: the build parses the statement's own `Example` blocks and requires your
  python optimal to **reproduce every one**. That is what proves you solved *this*
  problem. If the statement examples cannot be parsed, OR they parse but misalign (the parser
  cross-matches multiple example blocks, pairing an input with the wrong output — e.g. count-and-say),
  OR the judge is design / in_place (§4), set `"no_anchor_ok": true` and supply the example(s) as
  explicit `edge_inputs`. `no_anchor_ok` drops statement anchoring entirely; correctness then rests
  on your `edge_inputs` + the python/js/oracle agreement (+ the validator for any_valid), so make
  those thorough.

## 3. Inputs to include

Aim for ~5–9 literal `edge_inputs` plus ≥1 stress generator:
- **edge**: small hand-picked cases that exercise the core idea and corner cases
  (empty-ish, single element, all-same, negatives, duplicates).
- **boundary**: the constraint extremes (min/max length, min/max value).
- **trap**: the cases a *plausible wrong* solution gets wrong (off-by-one, "must
  not reuse element", overflow, already-sorted, etc.). These give "reveal failing
  case" its teeth.
- **stress**: `def gen(rng, size)` returning the positional args list, seeded and
  deterministic. It must produce inputs your reference solves correctly and
  quickly (well under ~10s). For problems with a *unique* answer, **construct** the
  input so the answer stays unique (don't rely on chance).

## 4. Judge types — pick the RIGHT one (the verifier won't)

The cross-check forces python, javascript, and the oracle to agree **structurally**
on every input. So all three of your implementations must produce the *same*
canonical output. The judge type then decides how a *user's* answer is compared at
runtime. Choosing wrong silently ships a pack that fails correct user code.

| `judge` | Use when | Source form |
|---|---|---|
| `"exact"` | exactly one correct output (scalars, fixed-order arrays, matrices, booleans, strings) | `"judge": "exact"` |
| `"unordered"` | output is a **flat collection whose order is irrelevant** and whose **elements are themselves order-invariant or scalar** (e.g. "return the values in any order"). NOT for list-of-lists where inner order also varies. | `"judge": "unordered"` |
| `"float"` | numeric answer compared with tolerance (returns a double / "within 1e-5") | `{ "type": "float", "epsilon": 1e-5 }` |
| `"in_place"` | the function returns void/None (or a length) and the **answer is a mutated argument** (sort in place, move zeroes) | `{ "type": "in_place", "arg_index": 0 }` (index of the mutated arg) |
| `"any_valid"` | **many distinct valid outputs** and you cannot reduce to a canonical order — list-of-lists (3sum, subsets, permutations, combinations, partitions), "any valid ordering", group-anagrams. | `{ "type": "any_valid", "validator_python": "...", "validator_javascript": "..." }` |
| `"design"` | class with an **operations sequence** (LRUCache, MinStack, Trie). Input is `[ops, argLists]`. | `"judge": "design"`, `"no_anchor_ok": true` |
| `"round_trip"` | **codec problems** where the solver invents the format (serialize-and-deserialize-*, tinyurl): the harness runs `decode(encode(x))` and emits the canonical serialization, so the exact compare judges it. | `{ "type": "round_trip", "io": "tree", "encode": "serialize", "decode": "deserialize" }` |
| `"property"` | **randomized output** (getRandom, shuffle, pick): no single correct answer, so a pack validator replays the op sequence and checks each output belongs to the legal set. Default execution is the design ops shape; `"exec": "call"` for a plain function (rand10). | `{ "type": "property", "validator_python": "…", "validator_javascript": "…" }`, `"no_anchor_ok": true` |
| `"concurrency"` | **multithreading problems** (print-in-order, dining-philosophers). Python-only: set `"languages": ["python"]` and ship an `oracle_python` (it replaces the JS differential). A pack DRIVER spawns the real threads and records events; a pack VALIDATOR judges each recorded sequence. The harness amplifies races (tiny GIL switch interval, jitter inside every record, barrier starts, `runs` repetitions) and a watchdog turns deadlocks into a clear failure. Drivers: daemon threads + join deadline; validators judge the LOGICAL sequence only so a correct solution can never flake. | `{ "type": "concurrency", "driver_python": "…", "validator_python": "…", "runs": 6 }`, `"no_anchor_ok": true` |

### `unordered` vs `any_valid` (the most common mistake)

`unordered` does a **top-level multiset** compare where each element is matched by
its canonical JSON. So `[[1,2],[3,4]]` ≠ `[[2,1],[4,3]]` under `unordered` — inner
order matters. Therefore:
- "return the array, order doesn't matter, elements are scalars" → **`unordered`**.
- list-of-lists / list-of-strings where **inner order also varies**, or where the
  same set can be expressed differently → **`any_valid`** with a validator.

### CRITICAL: `any_valid` cross-check needs identical-ordered references

The build's cross-check compares python vs javascript vs oracle **structurally**
for `any_valid` (it cannot know your problem's equivalence, so it demands exact
equality). The *runtime* validator is what accepts any valid ordering — but to get
*past the build*, your three reference implementations must emit **byte-identical
output**: the same elements in the same order, with the same inner order. So
**canonicalize the output in all three**: sort each inner list, then sort the outer
list with one fixed comparator. Use this comparator so python and JS agree:

```python
res.sort()                      # python: lexicographic, shorter-prefix first
```
```js
res.sort((a, b) => {            // JS: match python's list ordering exactly
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return a.length - b.length;
});
```

(For `unordered` this is NOT needed — that judge already compares as a multiset, so
any order passes the cross-check. The canonical-order rule is specific to
`any_valid`.) Keep `any_valid` inputs **small** when your oracle is exponential.

### Writing an `any_valid` validator

Ship `validate(args, output) -> bool` in **both** python and javascript. It must
**accept every valid output and reject every invalid one** — both directions
matter. The robust pattern: canonicalize, then set-compare against a freshly
recomputed correct answer (or verify structural validity directly).

```python
# validator_python for, e.g., 3sum
def validate(args, output):
    nums = args[0]
    # 1. structural: each triplet sums to 0, uses available elements, distinct set
    from collections import Counter
    avail = Counter(nums)
    seen = set()
    for t in output:
        if len(t) != 3 or sum(t) != 0:
            return False
        key = tuple(sorted(t))
        if key in seen:
            return False            # no duplicate triplets
        seen.add(key)
        c = Counter(t)
        for v, k in c.items():
            if avail[v] < k:
                return False        # can't use more copies than exist
    # 2. completeness: it found ALL triplets (compare against a recomputed count)
    correct = set()
    s = sorted(nums)
    # ... compute the canonical set of triplets ...
    return seen == correct
```

Always include **completeness** (found *all* answers), not just validity (each
answer is fine) — otherwise returning `[]` passes. Mirror the logic exactly in JS.

**You must manually test every `any_valid` validator** before returning the pack
(see §6): it must accept the reference output, accept a *re-ordered/alternative*
valid output, and reject at least one wrong output (missing one, extra one, invalid
one). Report that evidence.

## 4b. Node I/O types — linked lists & binary trees (task 0003)

Problems whose params/return are `ListNode` / `TreeNode` are authored with an
`io_types` field; the harness then (de)serializes at the call boundary so the
**user's unmodified LeetCode stub runs**. You write the reference solution using
`ListNode`/`TreeNode` exactly as on leetcode.com (the harness injects those
classes — do not define them yourself).

```jsonc
"io_types": { "params": ["linked_list", "linked_list"], "returns": "linked_list" }
// types: "json" (default/non-node) | "linked_list" | "tree" | {"list_of": <type>}
```

- **Wire form** (what `edge_inputs` and the statement examples use): linked list =
  `[v0, v1, …]` (empty = `[]`); binary tree = BFS level-order with `null` for
  missing children, e.g. `[3,9,20,null,null,15,7]` (empty = `[]`).
- `params` length must equal the stub arity. `returns` is the type of the value
  the function returns; for `in_place` node problems it is taken from the mutated
  param's type automatically.
- Composite examples: `merge-k-sorted-lists` → `{"params": [{"list_of":"linked_list"}], "returns":"linked_list"}`;
  `convert-sorted-list-to-bst` → `{"params":["linked_list"], "returns":"tree"}`;
  `unique-binary-search-trees-ii` → `{"params":["json"], "returns":{"list_of":"tree"}}`.
- Solutions reference the nodes normally (`head.next`, `root.left`, `ListNode(v)`,
  `TreeNode(v)`); the oracle should be an independent construction. Statement
  examples anchor as usual (the harness serializes the reference's node output
  back to an array). Do **not** set `io_types` for plain-value problems.

### 4c. Closing-the-48 wire types (Phase B/D of the `48` branch)

The full leaf set is now `json | linked_list | tree | cyclic_list | random_list |
graph | n_ary_tree | quad_tree | next_tree | multilevel_list`, plus these composite
forms (see `.docs/48_PLAN.md` and the pilot packs for worked examples):

- `{"node_ref": {"param": i}}` — the wire value is a node **value**; the harness
  resolves the real node object inside already-built param `i` (args build left to
  right). As a *return* type it emits the node's value after verifying the returned
  object really lives in that structure. Referenced values must be unique.
- `{"clone_of": {"param": i}}` — a structure-preserving deep copy of param `i`
  (wire value: `null`).
- `{"tail_of": {"param": i}}` — `{"values": [...], "attach": idx|null}`: a fresh
  list whose last node links into param `i`'s chain (intersection-of-two-linked-lists).
- `{"node_index_of": {"param": i}}` — return-only: the returned node's index in
  param `i`'s chain.
- `{"ctx_only": <type>}` — built for judging but **not passed** to the solution
  (delete-node's hidden head; pair with `in_place`'s `arg_index`). ctx_only params
  don't count toward the stub arity.
- `{"shim": {"kind": "<kind>", "curry_js"?: true}}` — an injected object/callback
  built from per-case hidden data. Kinds: `iterator`, `nested_integer`,
  `custom_function` (function_id 1–9), `master_guess` (budget-enforced, the case
  output becomes the guessed-it verdict), `mountain_array` (budget-enforced),
  and the global-installed `is_bad_version` / `guess_oracle` / `rand7` (their param
  slot carries only the hidden value and is not passed; `curry_js` handles
  LeetCode's curried JS stubs). `rand7` is a deterministic seeded LCG, identical
  across languages.

Wire forms: `cyclic_list` = `{"values": [...], "pos": k|null}`; `random_list` =
`[[val, randomIdx|null], ...]`; `graph` = adjacency list (node i is val i+1, return
freshness-checked); `n_ary_tree` = LeetCode null-separated level order; `quad_tree`
= level order of `[isLeaf, val]` 1/0 pairs; `next_tree` = plain level order in,
serialized **by following the `next` pointers** out; `multilevel_list` = segments
`[{"values": [...], "parent": null|globalNodeIndex}, ...]`.

`design` packs may node-type the constructor/method boundary with
`"judge": {"type": "design", "design_io": {"ctor": ["tree"], "methods": {"insert":
{"params": ["tree"], "returns": "tree"}}}}` — undeclared methods stay all-JSON. A
`property` judge accepts the same `design_io` key for node-typed constructors.

## 5. Originality (non-negotiable)

Author **all** prose yourself. Never copy LeetCode statement text, examples, or
editorial into `pattern`, `hints`, `description`, or comments. `constraints` are
numeric facts and are fine. The concept/name ("3Sum", "LRU Cache") is reusable;
the *expression* is not. The pack contains no statement text by design — keep it
that way.

## 6. Self-check before you finish (do NOT skip)

From the repo root, verify only your slugs — this writes nothing and is safe to run
in parallel with other authors:

```bash
python tools/build_packs.py --check --only slug-a,slug-b,slug-c
```

Every one of your slugs must print `OK`. A `-- slug: <reason>` is a failure — fix
the source and re-run until clean. Common failures and fixes:
- *"optimal solution disagrees with statement example …"* → your python is wrong or
  solves a different variant; fix it (this is the anchor doing its job).
- *"javascript disagrees on …"* → py/js differ; make them produce identical output.
- *"brute force disagrees on …"* → one of optimal/oracle is wrong on that input.
- *"optimal solution crashed on a generated input"* → your stress generator made an
  input your solution can't handle (or violates a constraint). Fix the generator.
- *"could not derive entry point"* → the stub isn't `class Solution` / parseable.

For `any_valid`, additionally run a quick manual python snippet (using
`tools/generate_test_packs.py` helpers `run_harness` / `compute_expected`) proving
the validator accepts a reordered valid answer and rejects a wrong one. Paste the
result in your report.

Return: the list of slugs that print `OK`, the judge type chosen for each, and any
slug you could not confidently complete (with the reason) so it can be escalated or
recorded basic-mode. **Never** return a slug as done if `--check` does not pass it.
