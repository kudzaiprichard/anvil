# WAVE_PLAN.md — remaining test-pack authoring, wave by wave

> Generated 2026-07-05 from `.docs/my_questions.json` + `tools/packs/index.json` +
> `tools/packs/*.json` on disk. Operating procedure: `tools/packs/ORCHESTRATION.md`
> (single session, Sonnet subagents, throttled waves, freeze between waves).
> Regenerate the numbers any time with `python tools/progress_summary.py`.

## Where things stand

| | count |
|---|---|
| Catalog questions (statement + Python stub) | **3,026** |
| Frozen verified packs (`index.json`) | **1,801** |
| Authored on disk, **not yet frozen** | **8** |
| No pack at all | **1,217** |
| — of which authorable queue (heuristic) | **1180** (282 E / 580 M / 318 H) |
| — of which design-shaped (≥3 methods) | **28** (dedicated chunks at the end) |
| — of which likely basic-mode defers (§5 shapes) | **37** (confirmation wave at the end) |

Accounting: 1,801 frozen + 8 unfrozen + 1,180 authorable + 37 defer-candidates = 3,026. ✔

## Sizing — why these waves won't exhaust mid-wave

- **5 Sonnet subagents per wave, 12 slugs each (~60 packs/wave)** — slightly
  under ORCHESTRATION.md's ~65 ceiling, because the previous run DID hit the limit;
  the redo list from any dropped agent folds into the next wave, so a smaller wave is
  pure upside.
- **Freeze after every wave** (`--bundle` + `cargo test` + `npm run build`) so progress
  banks before any limit can bite. Never dispatch wave N+1 on a red gate.
- **Per session window: dispatch at most 3 waves, then stop and check.** A wave is
  ~10–18 min wall-clock but the token budget is the real constraint; 3 waves ≈ 180
  packs is a comfortable session. If all 3 finished cleanly and the session still
  feels responsive, a 4th is fine — but never start a wave you aren't sure you can
  finish; a half-finished wave costs a sweep pass.
- **If the limit hits mid-wave anyway:** trust the disk, not the agent reports —
  sweep `tools/packs/` for the wave's slugs, `--check` what landed, delete
  quarantined/half-written files, fold the rest into the next wave's redo list.
- 20 waves total → roughly 7 session windows.

## Hard rules (same as ORCHESTRATION.md — repeat in every subagent brief)

- **NEVER run `git`** — `src-tauri/src/` and `tools/packs/` are untracked; a stray git op deletes them.
- Only create/edit files under `tools/packs/`. Stdlib-only solutions. Never hand-type expected values.
- Every constraint needs a `kind`. Self-check with `python tools/build_packs.py --check --only <slugs>` before reporting.
- Hitting a §5 shape (cyclic list, injected object, random, SQL…) → defer with a recorded reason, don't force it.

## Wave 0 — freeze what's already on disk (no subagents, ~2 min)

These 8 packs are authored but not in the frozen manifest. `--check` them, delete any
that fail, then `--bundle` + gates:

- `count-numbers-with-unique-digits`
- `largest-multiple-of-three`
- `minimum-falling-path-sum`
- `nth-magical-number`
- `number-of-digit-one`
- `queue-reconstruction-by-height`
- `sort-array-by-parity`
- `the-k-weakest-rows-in-a-matrix`

```bash
python tools/build_packs.py --check --only count-numbers-with-unique-digits largest-multiple-of-three minimum-falling-path-sum nth-magical-number number-of-digit-one queue-reconstruction-by-height sort-array-by-parity the-k-weakest-rows-in-a-matrix
python tools/build_packs.py --bundle && (cd src-tauri && cargo test) && npm run build
```

## Authoring waves

Each chunk below is one Sonnet subagent (`subagent_type: general-purpose`,
`model: "sonnet"`) pointed at `tools/PACK_AUTHORING_GUIDE.md` + its slug list.
Fold the previous wave's redo list into the next wave before dispatching.

### Wave 1 — 60 slugs (scrape batches 1, 2, 3, 4, 6, 13)

> ⚠ These are the leftovers of the *mostly-done* batches — earlier passes skipped
> them, often because they're hard shapes (n-ary/quad trees, node-reference args,
> codecs, multilevel lists). Expect a higher-than-usual defer rate in this wave;
> record the reason per slug instead of forcing a pack.

**1A** (4E/8M/0H): `intersection-of-two-linked-lists` `delete-node-in-a-linked-list` `construct-quad-tree` `n-ary-tree-level-order-traversal` `flatten-a-multilevel-doubly-linked-list` `implement-rand10-using-rand7` `encode-and-decode-tinyurl` `logical-or-of-two-binary-grids-represented-as-quad-trees` `maximum-depth-of-n-ary-tree` `n-ary-tree-preorder-traversal` `n-ary-tree-postorder-traversal` `all-nodes-distance-k-in-binary-tree`

**1B** (5E/6M/1H): `smallest-subtree-with-all-the-deepest-nodes` `leaf-similar-trees` `construct-binary-tree-from-preorder-and-postorder-traversal` `all-possible-full-binary-trees` `increasing-order-search-tree` `number-of-recent-calls` `range-sum-of-bst` `valid-mountain-array` `all-elements-in-two-binary-search-trees` `apply-discount-every-n-orders` `longest-zigzag-path-in-a-binary-tree` `maximum-sum-bst-in-binary-tree`

**1C** (1E/5M/6H): `find-a-corresponding-node-of-a-binary-tree-in-a-clone-of-that-tree` `balance-a-binary-search-tree` `pseudo-palindromic-paths-in-a-binary-tree` `minimize-or-of-remaining-elements-using-operations` `find-products-of-elements-of-big-array` `taking-maximum-energy-from-the-mystic-dungeon` `find-number-of-ways-to-reach-the-k-th-stair` `block-placement-queries` `construct-string-with-minimum-cost` `minimum-cost-for-cutting-cake-ii` `minimum-length-of-string-after-operations` `minimum-array-changes-to-make-differences-equal`

**1D** (1E/5M/6H): `maximum-score-from-grid-operations` `maximum-number-of-operations-to-move-ones-to-the-end` `minimum-operations-to-make-array-equal-to-target` `count-the-number-of-substrings-with-dominant-ones` `check-if-the-rectangle-corner-is-reachable` `minimum-number-of-flips-to-make-binary-grid-palindromic-i` `minimum-number-of-flips-to-make-binary-grid-palindromic-ii` `time-taken-to-mark-all-nodes` `shortest-distance-after-road-addition-queries-i` `shortest-distance-after-road-addition-queries-ii` `alternating-groups-iii` `snake-in-matrix`

**1E** (2E/5M/5H): `count-the-number-of-good-nodes` `find-the-count-of-monotonic-pairs-i` `find-the-power-of-k-size-subarrays-i` `find-the-power-of-k-size-subarrays-ii` `maximum-value-sum-by-placing-three-rooks-i` `maximum-value-sum-by-placing-three-rooks-ii` `count-substrings-that-satisfy-k-constraint-i` `maximum-energy-boost-from-two-drinks` `find-the-largest-palindrome-divisible-by-k` `count-substrings-that-satisfy-k-constraint-ii` `final-array-state-after-k-multiplication-operations-i` `count-almost-equal-pairs-i`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 2 — 60 slugs (scrape batches 7, 13)

**2A** (2E/5M/5H): `final-array-state-after-k-multiplication-operations-ii` `count-almost-equal-pairs-ii` `find-two-non-overlapping-sub-arrays-each-with-target-sum` `allocate-mailboxes` `running-sum-of-1d-array` `least-number-of-unique-integers-after-k-removals` `minimum-number-of-days-to-make-m-bouquets` `kth-ancestor-of-a-tree-node` `xor-operation-in-an-array` `making-file-names-unique` `avoid-flood-in-the-city` `find-critical-and-pseudo-critical-edges-in-minimum-spanning-tree`

**2B** (3E/6M/3H): `average-salary-excluding-the-minimum-and-maximum-salary` `the-kth-factor-of-n` `longest-subarray-of-1s-after-deleting-one-element` `parallel-courses-ii` `path-crossing` `check-if-array-pairs-are-divisible-by-k` `number-of-subsequences-that-satisfy-the-given-sum-condition` `max-value-of-equation` `can-make-arithmetic-progression-from-sequence` `last-moment-before-all-ants-fall-out-of-a-plank` `count-submatrices-with-all-ones` `minimum-possible-integer-after-at-most-k-adjacent-swaps-on-digits`

**2C** (3E/5M/4H): `reformat-date` `range-sum-of-sorted-subarray-sums` `minimum-difference-between-largest-and-smallest-value-in-three-moves` `stone-game-iv` `number-of-good-pairs` `number-of-substrings-with-only-1s` `path-with-maximum-probability` `best-position-for-a-service-centre` `water-bottles` `number-of-nodes-in-the-sub-tree-with-the-same-label` `maximum-number-of-non-overlapping-substrings` `find-a-value-of-a-mysterious-function-closest-to-target`

**2D** (3E/6M/3H): `count-odd-numbers-in-an-interval-range` `number-of-sub-arrays-with-odd-sum` `number-of-good-ways-to-split-a-string` `minimum-number-of-increments-on-subarrays-to-form-a-target-array` `minimum-suffix-flips` `number-of-good-leaf-nodes-pairs` `string-compression-ii` `count-good-triplets` `find-the-winner-of-an-array-game` `minimum-swaps-to-arrange-a-binary-grid` `get-the-maximum-score` `kth-missing-positive-number`

**2E** (3E/6M/3H): `can-convert-string-in-k-moves` `minimum-insertions-to-balance-a-parentheses-string` `find-longest-awesome-substring` `make-the-string-great` `find-kth-bit-in-nth-binary-string` `maximum-number-of-non-overlapping-subarrays-with-sum-equals-target` `minimum-cost-to-cut-a-stick` `three-consecutive-odds` `minimum-operations-to-make-array-equal` `magnetic-force-between-two-balls` `minimum-number-of-days-to-eat-n-oranges` `thousand-separator`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 3 — 60 slugs (scrape batches 7)

**3A** (3E/6M/3H): `minimum-number-of-vertices-to-reach-all-nodes` `minimum-numbers-of-function-calls-to-make-target-array` `detect-cycles-in-2d-grid` `most-visited-sector-in-a-circular-track` `maximum-number-of-coins-you-can-get` `find-latest-group-of-size-m` `stone-game-v` `detect-pattern-of-length-m-repeated-k-or-more-times` `maximum-length-of-subarray-with-positive-product` `minimum-number-of-days-to-disconnect-island` `number-of-ways-to-reorder-array-to-get-same-bst` `matrix-diagonal-sum`

**3B** (3E/6M/3H): `number-of-ways-to-split-a-string` `shortest-subarray-to-be-removed-to-make-array-sorted` `count-all-possible-routes` `replace-all-s-to-avoid-consecutive-repeating-characters` `number-of-ways-where-square-of-number-is-equal-to-product-of-two-numbers` `minimum-time-to-make-rope-colorful` `remove-max-number-of-edges-to-keep-graph-fully-traversable` `special-positions-in-a-binary-matrix` `count-unhappy-friends` `check-if-string-is-transformable-with-substring-sort-operations` `sum-of-all-odd-length-subarrays` `maximum-sum-obtained-of-any-permutation`

**3C** (3E/6M/3H): `make-sum-divisible-by-p` `strange-printer-ii` `rearrange-spaces-between-words` `split-a-string-into-the-max-number-of-unique-substrings` `maximum-non-negative-product-in-a-matrix` `minimum-cost-to-connect-two-groups-of-points` `crawler-log-folder` `maximum-profit-of-operating-a-centennial-wheel` `maximum-number-of-achievable-transfer-requests` `design-parking-system` `alert-using-same-key-card-three-or-more-times-in-a-one-hour-period` `find-valid-matrix-given-row-and-column-sums`

**3D** (3E/5M/4H): `find-servers-that-handled-most-number-of-requests` `special-array-with-x-elements-greater-than-or-equal-x` `even-odd-tree` `maximum-number-of-visible-points` `minimum-one-bit-operations-to-make-integers-zero` `maximum-nesting-depth-of-the-parentheses` `maximal-network-rank` `split-two-strings-to-make-palindrome` `count-subtrees-with-max-distance-between-cities` `mean-of-array-after-removing-some-elements` `coordinate-with-maximum-network-quality` `number-of-sets-of-k-non-overlapping-line-segments`

**3E** (4E/5M/3H): `largest-substring-between-two-equal-characters` `lexicographically-smallest-string-after-applying-operations` `best-team-with-no-conflicts` `graph-connectivity-with-threshold` `slowest-key` `arithmetic-subarrays` `path-with-minimum-effort` `rank-transform-of-a-matrix` `sort-array-by-increasing-frequency` `widest-vertical-area-between-two-points-containing-no-points` `count-substrings-that-differ-by-one-character` `number-of-ways-to-form-a-target-string-given-a-dictionary`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 4 — 60 slugs (scrape batches 7)

**4A** (3E/6M/3H): `check-array-formation-through-concatenation` `count-sorted-vowel-strings` `furthest-building-you-can-reach` `kth-smallest-instructions` `get-maximum-in-generated-array` `minimum-deletions-to-make-character-frequencies-unique` `sell-diminishing-valued-colored-balls` `create-sorted-array-through-instructions` `defuse-the-bomb` `minimum-deletions-to-make-string-balanced` `minimum-jumps-to-reach-home` `distribute-repeating-integers`

**4B** (4E/5M/3H): `design-an-ordered-stream` `determine-if-two-strings-are-close` `minimum-operations-to-reduce-x-to-zero` `maximize-grid-happiness` `check-if-two-string-arrays-are-equivalent` `smallest-string-with-a-given-numeric-value` `ways-to-make-a-fair-array` `minimum-initial-energy-to-finish-tasks` `maximum-repeating-substring` `merge-in-between-linked-lists` `minimum-number-of-removals-to-make-mountain-array` `richest-customer-wealth`

**4C** (3E/6M/3H): `find-the-most-competitive-subsequence` `minimum-moves-to-make-array-complementary` `minimize-deviation-in-array` `goal-parser-interpretation` `max-number-of-k-sum-pairs` `concatenation-of-consecutive-binary-numbers` `minimum-incompatibility` `count-the-number-of-consistent-strings` `sum-of-absolute-differences-in-a-sorted-array` `stone-game-vi` `delivering-boxes-from-storage-to-ports` `count-of-matches-in-tournament`

**4D** (3E/6M/3H): `partitioning-into-minimum-number-of-deci-binary-numbers` `stone-game-vii` `maximum-height-by-stacking-cuboids` `reformat-phone-number` `maximum-erasure-value` `jump-game-vi` `checking-existence-of-edge-length-limited-paths` `number-of-students-unable-to-eat-lunch` `average-waiting-time` `maximum-binary-string-after-change` `minimum-adjacent-swaps-for-k-consecutive-ones` `determine-if-string-halves-are-alike`

**4E** (3E/6M/3H): `maximum-number-of-eaten-apples` `where-will-the-ball-fall` `maximum-xor-with-an-element-from-array` `maximum-units-on-a-truck` `count-good-meals` `ways-to-split-array-into-three-subarrays` `minimum-operations-to-make-a-subsequence` `calculate-money-in-leetcode-bank` `maximum-score-from-removing-substrings` `construct-the-lexicographically-largest-valid-sequence` `number-of-ways-to-reconstruct-a-tree` `decode-xored-array`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 5 — 60 slugs (scrape batches 7, 9)

**5A** (3E/7M/2H): `swapping-nodes-in-a-linked-list` `minimize-hamming-distance-after-swap-operations` `find-minimum-time-to-finish-all-jobs` `number-of-rectangles-that-can-form-the-largest-square` `tuple-with-same-product` `largest-submatrix-with-rearrangements` `find-greatest-common-divisor-of-array` `find-unique-binary-string` `minimize-the-difference-between-target-and-chosen-elements` `find-array-given-subset-sums` `minimum-difference-between-highest-and-lowest-of-k-scores` `find-the-kth-largest-integer-in-the-array`

**5B** (3E/6M/3H): `minimum-number-of-work-sessions-to-finish-the-tasks` `number-of-unique-good-subsequences` `find-the-middle-index-in-array` `find-all-groups-of-farmland` `the-number-of-good-subsets` `count-special-quadruplets` `the-number-of-weak-characters-in-the-game` `first-day-where-you-have-been-in-all-the-rooms` `gcd-sort-of-an-array` `reverse-prefix-of-word` `number-of-pairs-of-interchangeable-rectangles` `maximum-product-of-the-length-of-two-palindromic-subsequences`

**5C** (3E/5M/4H): `smallest-missing-genetic-value-in-each-subtree` `count-number-of-pairs-with-absolute-difference-k` `find-original-array-from-doubled-array` `maximum-earnings-from-taxi` `minimum-number-of-operations-to-make-array-continuous` `final-value-of-variable-after-performing-operations` `sum-of-beauty-in-the-array` `longest-subsequence-repeated-k-times` `maximum-difference-between-increasing-elements` `grid-game` `check-if-word-can-be-placed-in-crossword` `the-score-of-students-solving-math-expression`

**5D** (4E/5M/3H): `convert-1d-array-into-2d-array` `number-of-pairs-of-strings-with-concatenation-equal-to-target` `maximize-the-confusion-of-an-exam` `maximum-number-of-ways-to-partition-an-array` `minimum-moves-to-convert-string` `find-missing-observations` `stone-game-ix` `smallest-k-length-subsequence-with-occurrences-of-a-letter` `two-out-of-three` `minimum-operations-to-make-a-uni-value-grid` `partition-array-into-two-arrays-to-minimize-sum-difference` `minimum-number-of-moves-to-seat-everyone`

**5E** (3E/6M/3H): `remove-colored-pieces-if-both-neighbors-are-the-same-color` `the-time-when-the-network-becomes-idle` `kth-smallest-product-of-two-sorted-arrays` `check-if-numbers-are-ascending-in-a-sentence` `count-number-of-maximum-bitwise-or-subsets` `second-minimum-time-to-reach-destination` `number-of-valid-words-in-a-sentence` `next-greater-numerically-balanced-number` `count-nodes-with-the-highest-score` `parallel-courses-iii` `kth-distinct-string-in-an-array` `two-best-non-overlapping-events`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 6 — 60 slugs (scrape batches 9)

**6A** (3E/6M/3H): `plates-between-candles` `number-of-valid-move-combinations-on-chessboard` `smallest-index-with-equal-value` `find-the-minimum-and-maximum-number-of-nodes-between-critical-points` `minimum-operations-to-convert-number` `check-if-an-original-string-exists-given-two-encoded-strings` `count-vowel-substrings-of-a-string` `vowels-of-all-substrings` `minimized-maximum-of-products-distributed-to-any-store` `maximum-path-quality-of-a-graph` `check-whether-two-strings-are-almost-equivalent` `most-beautiful-item-for-each-query`

**6B** (3E/6M/3H): `maximum-number-of-tasks-you-can-assign` `time-needed-to-buy-tickets` `reverse-nodes-in-even-length-groups` `decode-the-slanted-ciphertext` `process-restricted-friend-requests` `two-furthest-houses-with-different-colors` `watering-plants` `range-frequency-queries` `sum-of-k-mirror-numbers` `count-common-words-with-one-occurrence` `minimum-number-of-food-buckets-to-feed-the-hamsters` `minimum-cost-homecoming-of-a-robot-in-a-grid`

**6C** (3E/6M/3H): `count-fertile-pyramids-in-a-land` `find-target-indices-after-sorting-array` `k-radius-subarray-averages` `removing-minimum-and-maximum-from-array` `find-all-people-with-secret` `finding-3-digit-even-numbers` `delete-the-middle-node-of-a-linked-list` `step-by-step-directions-from-a-binary-tree-node-to-another` `valid-arrangement-of-pairs` `find-subsequence-of-length-k-with-the-largest-sum` `find-good-days-to-rob-the-bank` `detonate-the-maximum-bombs`

**6D** (3E/6M/3H): `rings-and-rods` `sum-of-subarray-ranges` `watering-plants-ii` `maximum-fruits-harvested-after-at-most-k-steps` `find-first-palindromic-string-in-the-array` `adding-spaces-to-a-string` `number-of-smooth-descent-periods-of-a-stock` `minimum-operations-to-make-the-array-k-increasing` `maximum-number-of-words-found-in-sentences` `find-all-possible-recipes-from-given-supplies` `check-if-a-parentheses-string-can-be-valid` `abbreviating-the-product-of-a-range`

**6E** (3E/6M/3H): `a-number-after-a-double-reversal` `execution-of-all-suffix-instructions-staying-in-a-grid` `intervals-between-identical-elements` `recover-the-original-array` `check-if-all-as-appears-before-all-bs` `number-of-laser-beams-in-a-bank` `destroying-asteroids` `maximum-employees-to-be-invited-to-a-meeting` `capitalize-the-title` `maximum-twin-sum-of-a-linked-list` `longest-palindrome-by-concatenating-two-letter-words` `stamping-the-grid`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 7 — 60 slugs (scrape batches 9)

**7A** (3E/6M/3H): `check-if-every-row-and-column-contains-all-numbers` `minimum-swaps-to-group-all-1s-together-ii` `count-words-obtained-after-adding-a-letter` `earliest-possible-day-of-full-bloom` `divide-a-string-into-groups-of-size-k` `minimum-moves-to-reach-target-score` `solving-questions-with-brainpower` `maximum-running-time-of-n-computers` `minimum-cost-of-buying-candies-with-discount` `count-the-hidden-sequences` `k-highest-ranked-items-within-a-price-range` `number-of-ways-to-divide-a-long-corridor`

**7B** (3E/5M/4H): `count-elements-with-strictly-smaller-and-greater-elements` `rearrange-array-elements-by-sign` `find-all-lonely-numbers-in-the-array` `maximum-good-people-based-on-statements` `keep-multiplying-found-values-by-two` `all-divisions-with-the-highest-score-of-a-binary-array` `find-substring-with-given-hash-value` `groups-of-strings` `minimum-sum-of-four-digit-number-after-splitting-digits` `partition-array-according-to-given-pivot` `minimum-cost-to-set-cooking-time` `minimum-difference-in-sums-after-removal-of-elements`

**7C** (4E/5M/3H): `sort-even-and-odd-indices-independently` `smallest-value-of-the-rearranged-number` `minimum-time-to-remove-all-cars-containing-illegal-goods` `count-operations-to-obtain-zero` `minimum-operations-to-make-the-array-alternating` `removing-minimum-number-of-magic-beans` `maximum-and-sum-of-array` `count-equal-and-divisible-pairs-in-an-array` `find-three-consecutive-integers-that-sum-to-a-given-number` `maximum-split-of-positive-even-integers` `count-good-triplets-in-an-array` `count-integers-with-even-digit-sum`

**7D** (3E/6M/3H): `merge-nodes-in-between-zeros` `construct-string-with-repeat-limit` `count-array-pairs-divisible-by-k` `counting-words-with-a-given-prefix` `minimum-number-of-steps-to-make-two-strings-anagram-ii` `minimum-time-to-complete-trips` `minimum-time-to-finish-the-race` `most-frequent-number-following-key-in-an-array` `sort-the-jumbled-numbers` `all-ancestors-of-a-node-in-a-directed-acyclic-graph` `minimum-number-of-moves-to-make-palindrome` `cells-in-a-range-on-an-excel-sheet`

**7E** (3E/6M/3H): `append-k-integers-with-minimal-sum` `create-binary-tree-from-descriptions` `replace-non-coprime-numbers-in-array` `find-all-k-distant-indices-in-an-array` `count-artifacts-that-can-be-extracted` `maximize-the-topmost-element-after-k-moves` `minimum-weighted-subgraph-with-the-required-paths` `divide-array-into-equal-pairs` `maximize-number-of-subsequences-in-a-string` `minimum-operations-to-halve-array-sum` `minimum-white-tiles-after-covering-with-carpets` `count-hills-and-valleys-in-an-array`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 8 — 60 slugs (scrape batches 9, 11)

**8A** (3E/6M/3H): `count-collisions-on-a-road` `maximum-points-in-an-archery-competition` `longest-substring-of-one-repeating-character` `find-the-difference-of-two-arrays` `minimum-deletions-to-make-array-beautiful` `find-palindrome-with-fixed-length` `maximum-value-of-k-coins-from-piles` `minimum-bit-flips-to-convert-number` `find-triangular-sum-of-an-array` `number-of-ways-to-select-buildings` `sum-of-scores-of-built-strings` `minimum-number-of-operations-to-convert-time`

**8B** (2E/8M/2H): `find-players-with-zero-or-one-losses` `maximum-candies-allocated-to-k-children` `largest-number-after-digit-swaps-by-parity` `minimize-result-by-adding-parentheses-to-expression` `maximum-product-after-k-increments` `maximum-total-beauty-of-the-gardens` `append-characters-to-string-to-make-subsequence` `remove-nodes-from-linked-list` `count-subarrays-with-median-k` `circular-sentence` `divide-players-into-teams-of-equal-skill` `minimum-score-of-a-path-between-two-cities`

**8C** (3E/4M/5H): `divide-nodes-into-the-maximum-number-of-groups` `maximum-value-of-a-string-in-an-array` `maximum-star-sum-of-a-graph` `frog-jump-ii` `minimum-total-cost-to-make-arrays-unequal` `delete-greatest-value-in-each-row` `longest-square-streak-in-an-array` `maximum-number-of-points-from-grid-queries` `count-pairs-of-similar-strings` `smallest-value-after-replacing-with-sum-of-prime-factors` `add-edges-to-make-degrees-of-all-nodes-even` `cycle-length-queries-in-a-tree`

**8D** (3E/7M/2H): `maximum-enemy-forts-that-can-be-captured` `reward-top-k-students` `minimize-the-maximum-of-two-arrays` `count-anagrams` `shortest-distance-to-target-string-in-a-circular-array` `take-k-of-each-character-from-left-and-right` `maximum-tastiness-of-candy-basket` `number-of-great-partitions` `count-the-digits-that-divide-a-number` `distinct-prime-factors-of-product-of-array` `partition-string-into-substrings-with-values-at-most-k` `closest-prime-numbers-in-range`

**8E** (3E/6M/3H): `categorize-box-according-to-criteria` `find-consecutive-integers-from-a-data-stream` `find-xor-beauty-of-array` `maximize-the-minimum-powered-city` `maximum-count-of-positive-integer-and-negative-integer` `maximal-score-after-applying-k-operations` `make-number-of-distinct-characters-equal` `time-to-cross-a-bridge` `difference-between-element-sum-and-digit-sum-of-an-array` `increment-submatrices-by-one` `count-the-number-of-good-subarrays` `difference-between-maximum-and-minimum-price-sum`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 9 — 60 slugs (scrape batches 11)

**9A** (3E/5M/4H): `minimum-common-value` `minimum-operations-to-make-array-equal-ii` `maximum-subsequence-score` `check-if-point-is-reachable` `alternating-digit-sum` `sort-the-students-by-their-kth-score` `apply-bitwise-operations-to-make-strings-equal` `minimum-cost-to-split-an-array` `count-distinct-numbers-on-board` `count-collisions-of-monkeys-on-a-polygon` `put-marbles-in-bags` `count-increasing-quadruplets`

**9B** (3E/7M/2H): `separate-the-digits-in-an-array` `maximum-number-of-integers-to-choose-from-a-range-i` `maximize-win-from-two-segments` `disconnect-path-in-a-binary-matrix-by-at-most-one-flip` `take-gifts-from-the-richest-pile` `count-vowel-strings-in-ranges` `house-robber-iv` `rearranging-fruits` `find-the-array-concatenation-value` `count-the-number-of-fair-pairs` `substring-xor-queries` `subsequence-with-the-minimum-score`

**9C** (3E/6M/3H): `maximum-difference-by-remapping-a-digit` `minimum-score-by-changing-two-elements` `minimum-impossible-or` `handling-sum-queries-after-update` `merge-two-2d-arrays-by-summing-values` `minimum-operations-to-reduce-an-integer-to-0` `count-the-number-of-square-free-subsets` `find-the-string-with-lcp` `left-and-right-sum-differences` `find-the-divisibility-array-of-a-string` `find-the-maximum-number-of-marked-indices` `minimum-time-to-visit-a-cell-in-a-grid`

**9D** (3E/5M/4H): `split-with-minimum-sum` `count-total-number-of-colored-cells` `count-ways-to-group-overlapping-ranges` `count-number-of-possible-root-nodes` `pass-the-pillow` `kth-largest-sum-in-a-binary-tree` `split-the-array-to-make-coprime-products` `number-of-ways-to-earn-points` `count-the-number-of-vowel-strings-in-range` `rearrange-array-to-maximize-prefix-score` `count-the-number-of-beautiful-subarrays` `minimum-time-to-complete-all-tasks`

**9E** (3E/8M/1H): `distribute-money-to-maximum-children` `maximize-greatness-of-an-array` `find-score-of-an-array-after-marking-all-elements` `minimum-time-to-repair-cars` `number-of-even-and-odd-bits` `check-knight-tour-configuration` `the-number-of-beautiful-subsets` `smallest-missing-non-negative-integer-after-operations` `k-items-with-the-maximum-sum` `prime-subtraction-operation` `minimum-operations-to-make-all-array-elements-equal` `collect-coins-in-a-tree`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 10 — 60 slugs (scrape batches 11, 12)

**10A** (3E/6M/3H): `form-smallest-number-from-two-digit-arrays` `find-the-substring-with-maximum-cost` `make-k-subarray-sums-equal` `shortest-cycle-in-a-graph` `find-the-longest-balanced-substring-of-a-binary-string` `convert-an-array-into-a-2d-array-with-conditions` `mice-and-cheese` `minimum-reverse-operations` `prime-in-diagonal` `sum-of-distances` `minimize-the-maximum-difference-of-pairs` `minimum-number-of-visited-cells-in-a-grid`

**10B** (6E/5M/1H): `find-the-width-of-columns-of-a-grid` `find-the-score-of-all-prefixes-of-an-array` `cousins-in-binary-tree-ii` `row-with-maximum-ones` `find-the-maximum-divisibility-score` `minimum-additions-to-make-valid-string` `minimize-the-total-price-of-the-trips` `calculate-delayed-arrival-time` `sum-multiples` `sliding-subarray-beauty` `minimum-number-of-operations-to-make-all-array-elements-equal-to-1` `maximum-sum-with-exactly-k-elements`

**10C** (3E/7M/2H): `find-the-prefix-common-array-of-two-arrays` `maximum-number-of-fish-in-a-grid` `make-array-empty` `determine-the-winner-of-a-bowling-game` `first-completely-painted-row-or-column` `minimum-cost-of-a-path-with-special-roads` `lexicographically-smallest-beautiful-string` `find-the-distinct-difference-array` `number-of-adjacent-elements-with-the-same-color` `make-costs-of-paths-equal-in-a-binary-tree` `number-of-senior-citizens` `sum-in-a-matrix`

**10D** (4E/6M/2H): `maximum-or` `power-of-heroes` `find-the-losers-of-the-circular-game` `neighboring-bitwise-xor` `maximum-number-of-moves-in-a-grid` `count-the-number-of-complete-components` `minimum-string-length-after-removing-substrings` `lexicographically-smallest-palindrome` `find-the-punishment-number-of-an-integer` `modify-graph-edge-weights` `buy-two-chocolates` `extra-characters-in-a-string`

**10E** (4E/5M/3H): `maximum-strength-of-a-group` `greatest-common-divisor-traversal` `remove-trailing-zeros-from-a-string` `difference-of-number-of-distinct-values-on-diagonals` `minimum-cost-to-make-all-characters-equal` `maximum-strictly-increasing-cells-in-a-matrix` `minimize-string-length` `semi-ordered-permutation` `sum-of-matrix-after-queries` `count-of-integers` `check-if-the-number-is-fascinating` `find-the-longest-semi-repetitive-substring`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 11 — 60 slugs (scrape batches 12)

**11A** (3E/6M/3H): `movement-of-robots` `find-a-good-subset-of-the-matrix` `neither-minimum-nor-maximum` `lexicographically-smallest-string-after-substring-operation` `collecting-chocolates` `maximum-sum-queries` `total-distance-traveled` `find-the-value-of-the-partition` `special-permutations` `painting-the-walls` `find-maximum-number-of-string-pairs` `construct-the-longest-new-string`

**11B** (3E/7M/2H): `decremental-string-concatenation` `count-zero-request-servers` `number-of-beautiful-pairs` `minimum-operations-to-make-the-integer-zero` `ways-to-split-array-into-good-subarrays` `robot-collisions` `longest-even-odd-subarray-with-threshold` `prime-pairs-with-target-sum` `continuous-subarrays` `sum-of-imbalance-numbers-of-all-subarrays` `longest-alternating-subarray` `relocate-marbles`

**11C** (3E/8M/1H): `partition-string-into-minimum-beautiful-substrings` `number-of-black-blocks` `find-the-maximum-achievable-number` `maximum-number-of-jumps-to-reach-the-last-index` `longest-non-decreasing-subarray-from-two-arrays` `apply-operations-to-make-all-array-elements-equal-to-zero` `sum-of-squares-of-special-elements` `maximum-beauty-of-an-array-after-applying-operation` `minimum-index-of-a-valid-split` `length-of-the-longest-valid-substring` `check-if-array-is-good` `sort-vowels-in-a-string`

**11D** (3E/6M/3H): `visit-array-positions-to-maximize-score` `ways-to-express-an-integer-as-sum-of-powers` `split-strings-by-separator` `largest-element-in-an-array-after-merge-operations` `maximum-number-of-groups-with-increasing-length` `count-paths-that-can-form-a-palindrome-in-a-tree` `number-of-employees-who-met-the-target` `count-complete-subarrays-in-an-array` `shortest-string-that-contains-three-strings` `count-stepping-numbers-in-range` `account-balance-after-rounded-purchase` `insert-greatest-common-divisors-in-linked-list`

**11E** (3E/6M/3H): `minimum-seconds-to-equalize-a-circular-array` `minimum-time-to-make-array-sum-at-most-x` `faulty-keyboard` `check-if-it-is-possible-to-split-array` `find-the-safest-path-in-a-grid` `maximum-elegance-of-a-k-length-subsequence` `max-pair-sum-in-an-array` `double-a-number-represented-as-a-linked-list` `minimum-absolute-difference-between-elements-with-constraint` `apply-operations-to-maximize-score` `count-pairs-whose-sum-is-less-than-target` `make-string-a-subsequence-using-cyclic-increments`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 12 — 60 slugs (scrape batches 12)

**12A** (3E/6M/3H): `sorting-three-groups` `number-of-beautiful-integers-in-the-range` `check-if-a-string-is-an-acronym-of-words` `determine-the-minimum-sum-of-a-k-avoiding-array` `maximize-the-profit-as-the-salesman` `find-the-longest-equal-subarray` `furthest-point-from-origin` `find-the-minimum-possible-sum-of-a-beautiful-array` `minimum-operations-to-form-subsequence-with-target-sum` `maximize-value-of-function-in-a-ball-passing-game` `check-if-strings-can-be-made-equal-with-operations-i` `check-if-strings-can-be-made-equal-with-operations-ii`

**12B** (3E/6M/3H): `maximum-sum-of-almost-unique-subarray` `count-k-subsequences-of-a-string-with-maximum-beauty` `count-symmetric-integers` `minimum-operations-to-make-a-special-number` `count-of-interesting-subarrays` `minimum-edge-weight-equilibrium-queries-in-a-tree` `points-that-intersect-with-cars` `determine-if-a-cell-is-reachable-at-a-given-time` `minimum-moves-to-spread-stones-over-grid` `string-transformation` `minimum-right-shifts-to-sort-the-array` `minimum-array-length-after-pair-removals`

**12C** (3E/6M/3H): `count-pairs-of-points-with-distance-k` `minimum-edge-reversals-so-every-node-is-reachable` `sum-of-values-at-indices-with-k-set-bits` `happy-students` `maximum-number-of-alloys` `maximum-element-sum-of-a-complete-subset-of-indices` `maximum-odd-binary-number` `beautiful-towers-i` `beautiful-towers-ii` `count-valid-paths-in-a-tree` `minimum-operations-to-collect-elements` `minimum-number-of-operations-to-make-array-empty`

**12D** (4E/5M/3H): `split-array-into-maximum-number-of-subarrays` `maximum-number-of-k-divisible-components` `maximum-value-of-an-ordered-triplet-i` `maximum-value-of-an-ordered-triplet-ii` `minimum-size-subarray-in-infinite-array` `count-visited-nodes-in-a-directed-graph` `divisible-and-non-divisible-sums-difference` `minimum-processing-time` `apply-operations-to-make-two-strings-equal` `apply-operations-on-array-to-maximize-sum-of-squares` `last-visited-integers` `longest-unequal-adjacent-groups-subsequence-i`

**12E** (3E/7M/2H): `longest-unequal-adjacent-groups-subsequence-ii` `count-of-sub-multisets-with-bounded-sum` `find-indices-with-index-and-value-difference-i` `shortest-and-lexicographically-smallest-beautiful-string` `find-indices-with-index-and-value-difference-ii` `construct-product-matrix` `minimum-sum-of-mountain-triplets-i` `minimum-sum-of-mountain-triplets-ii` `minimum-number-of-groups-to-create-a-valid-assignment` `minimum-changes-to-make-k-semi-palindromes` `subarrays-distinct-element-sum-of-squares-i` `minimum-number-of-changes-to-make-binary-string-beautiful`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 13 — 60 slugs (scrape batches 12)

**13A** (3E/6M/3H): `length-of-the-longest-subsequence-that-sums-to-target` `subarrays-distinct-element-sum-of-squares-ii` `find-the-k-or-of-an-array` `minimum-equal-sum-of-two-arrays-after-replacing-zeros` `minimum-increment-operations-to-make-array-beautiful` `maximum-points-after-collecting-coins-from-all-nodes` `find-champion-i` `find-champion-ii` `maximum-score-after-applying-operations-on-a-tree` `maximum-balanced-subsequence-sum` `distribute-candies-among-children-i` `distribute-candies-among-children-ii`

**13B** (3E/6M/3H): `number-of-strings-which-can-be-rearranged-to-contain-substring` `maximum-spending-after-buying-items` `maximum-strong-pair-xor-i` `high-access-employees` `minimum-operations-to-maximize-last-elements-in-arrays` `maximum-strong-pair-xor-ii` `make-three-strings-equal` `separate-black-and-white-balls` `maximum-xor-product` `find-building-where-alice-and-bob-can-meet` `find-words-containing-character` `maximize-area-of-square-hole-in-grid`

**13C** (3E/5M/4H): `minimum-number-of-coins-for-fruits` `find-maximum-non-decreasing-array-length` `matrix-similarity-after-cyclic-shifts` `count-beautiful-substrings-i` `make-lexicographically-smallest-array-by-swapping-elements` `count-beautiful-substrings-ii` `find-the-peaks` `minimum-number-of-coins-to-be-added` `count-complete-substrings` `count-the-number-of-infection-sequences` `find-common-elements-between-two-arrays` `remove-adjacent-almost-equal-characters`

**13D** (3E/6M/3H): `length-of-longest-subarray-with-at-most-k-frequency` `number-of-possible-sets-of-closing-branches` `count-tested-devices-after-test-operations` `double-modular-exponentiation` `count-subarrays-where-max-element-appears-at-least-k-times` `count-the-number-of-good-partitions` `find-missing-and-repeated-values` `divide-array-into-arrays-with-max-difference` `minimum-cost-to-make-array-equalindromic` `apply-operations-to-maximize-frequency-score` `count-the-number-of-incremovable-subarrays-i` `find-polygon-with-the-largest-perimeter`

**13E** (3E/5M/4H): `count-the-number-of-incremovable-subarrays-ii` `find-number-of-coins-to-place-in-tree-nodes` `minimum-number-game` `maximum-square-area-by-removing-fences-from-a-field` `minimum-cost-to-convert-string-i` `minimum-cost-to-convert-string-ii` `check-if-bitwise-or-has-trailing-zeros` `find-longest-special-substring-that-occurs-thrice-i` `find-longest-special-substring-that-occurs-thrice-ii` `palindrome-rearrangement-queries` `smallest-missing-integer-greater-than-sequential-prefix-sum` `minimum-number-of-operations-to-make-array-xor-equal-to-k`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 14 — 60 slugs (scrape batches 12, 14)

**14A** (4E/3M/5H): `minimum-number-of-operations-to-make-x-and-y-equal` `count-the-number-of-powerful-integers` `maximum-area-of-longest-diagonal-rectangle` `find-the-key-of-the-numbers` `hash-divided-string` `find-the-count-of-good-integers` `minimum-amount-of-damage-dealt-to-bob` `check-if-two-chessboard-squares-have-the-same-color` `k-th-nearest-obstacle-queries` `select-cells-in-grid-with-maximum-score` `maximum-xor-score-subarray-queries` `convert-date-to-binary`

**14B** (2E/6M/4H): `maximize-score-of-numbers-in-ranges` `reach-end-of-array-with-max-score` `maximum-number-of-moves-to-kill-all-pawns` `find-indices-of-stable-mountains` `find-a-safe-walk-through-a-grid` `find-the-maximum-sequence-value-of-array` `length-of-the-longest-increasing-path` `the-two-sneaky-numbers-of-digitville` `maximum-multiplication-score` `minimum-number-of-valid-strings-to-form-target-i` `minimum-number-of-valid-strings-to-form-target-ii` `report-spam-message`

**14C** (2E/7M/3H): `minimum-number-of-seconds-to-make-mountain-height-zero` `count-substrings-that-can-be-rearranged-to-contain-a-string-i` `count-substrings-that-can-be-rearranged-to-contain-a-string-ii` `minimum-element-after-replacement-with-digit-sum` `maximize-the-total-height-of-unique-towers` `find-the-lexicographically-smallest-valid-sequence` `find-the-occurrence-of-first-almost-equal-substring` `find-the-k-th-character-in-string-game-i` `count-of-substrings-containing-every-vowel-and-k-consonants-i` `count-of-substrings-containing-every-vowel-and-k-consonants-ii` `find-the-k-th-character-in-string-game-ii` `maximum-possible-number-by-binary-concatenation`

**14D** (2E/5M/5H): `remove-methods-from-project` `construct-2d-grid-matching-graph-layout` `sorted-gcd-pair-queries` `construct-the-minimum-bitwise-array-i` `construct-the-minimum-bitwise-array-ii` `find-maximum-removals-from-source-string` `find-the-number-of-possible-ways-for-an-event` `find-x-sum-of-all-k-long-subarrays-i` `k-th-largest-perfect-subtree-size-in-binary-tree` `count-the-number-of-winning-sequences` `find-x-sum-of-all-k-long-subarrays-ii` `find-the-sequence-of-strings-appeared-on-the-screen`

**14E** (2E/6M/4H): `count-substrings-with-k-frequency-characters-i` `minimum-division-operations-to-make-array-non-decreasing` `check-if-dfs-strings-are-palindromes` `find-the-original-typed-string-i` `find-subtree-sizes-after-changes` `maximum-points-tourist-can-earn` `find-the-original-typed-string-ii` `find-the-maximum-factor-score-of-array` `total-characters-in-string-after-transformations-i` `find-the-number-of-subsequences-with-equal-gcd` `total-characters-in-string-after-transformations-ii` `check-balanced-string`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 15 — 60 slugs (scrape batches 14)

**15A** (3E/4M/5H): `find-minimum-time-to-reach-last-room-i` `find-minimum-time-to-reach-last-room-ii` `count-number-of-balanced-permutations` `smallest-divisible-digit-product-i` `maximum-frequency-of-an-element-after-performing-operations-i` `maximum-frequency-of-an-element-after-performing-operations-ii` `smallest-divisible-digit-product-ii` `adjacent-increasing-subarrays-detection-i` `adjacent-increasing-subarrays-detection-ii` `sum-of-good-subsequences` `count-k-reducible-numbers-less-than-n` `make-array-elements-equal-to-zero`

**15B** (3E/6M/3H): `zero-array-transformation-i` `zero-array-transformation-ii` `minimize-the-maximum-adjacent-element-difference` `stone-removal-game` `shift-distance-between-two-strings` `zero-array-transformation-iii` `find-the-maximum-number-of-fruits-collected` `minimum-positive-sum-subarray` `rearrange-k-substrings-to-form-target-string` `minimum-array-sum` `maximize-sum-of-weights-after-edge-removals` `smallest-number-with-all-set-bits`

**15C** (3E/6M/3H): `identify-the-largest-outlier-in-an-array` `maximize-the-number-of-target-nodes-after-connecting-trees-i` `maximize-the-number-of-target-nodes-after-connecting-trees-ii` `minimum-operations-to-make-array-values-equal-to-k` `minimum-time-to-break-locks-i` `digit-operations-to-make-two-integers-equal` `count-connected-components-in-lcm-graph` `transformed-array` `maximum-area-rectangle-with-point-constraints-i` `maximum-subarray-sum-with-length-divisible-by-k` `maximum-area-rectangle-with-point-constraints-ii` `button-with-longest-push-time`

**15D** (3E/5M/4H): `maximize-amount-after-two-days-of-conversions` `count-beautiful-splits-in-an-array` `minimum-operations-to-make-character-frequencies-equal` `count-subarrays-of-length-three-with-a-condition` `count-paths-with-the-given-xor-value` `check-if-grid-can-be-cut-into-sections` `subsequences-with-a-unique-middle-mode-i` `minimum-number-of-operations-to-make-elements-in-array-distinct` `maximum-number-of-distinct-elements-after-operations` `smallest-substring-with-identical-characters-i` `smallest-substring-with-identical-characters-ii` `minimum-operations-to-make-columns-strictly-increasing`

**15E** (3E/6M/3H): `find-the-lexicographically-largest-string-from-the-box-i` `count-special-subsequences` `count-the-number-of-arrays-with-k-matching-adjacent-elements` `substring-matching-pattern` `longest-subsequence-with-decreasing-adjacent-difference` `maximize-subarray-sum-after-removing-all-occurrences-of-one-element` `maximum-subarray-with-equal-products` `find-mirror-score-of-a-string` `maximum-coins-from-k-consecutive-bags` `maximum-score-of-non-overlapping-intervals` `zigzag-grid-traversal-with-skip` `maximum-amount-of-money-robot-can-earn`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 16 — 60 slugs (scrape batches 14)

**16A** (3E/5M/4H): `minimize-the-maximum-edge-weight-of-graph` `count-non-decreasing-subarrays-after-k-operations` `maximum-difference-between-adjacent-elements-in-a-circular-array` `minimum-cost-to-make-arrays-identical` `longest-special-path` `manhattan-distances-of-all-arrangements-of-pieces` `sum-of-variable-length-subarrays` `maximum-and-minimum-sums-of-at-most-size-k-subsequences` `paint-house-iv` `maximum-and-minimum-sums-of-at-most-size-k-subarrays` `count-partitions-with-even-sum-difference` `count-mentions-per-user`

**16B** (2E/6M/4H): `maximum-frequency-after-subarray-operation` `frequencies-of-shortest-supersequences` `find-valid-pair-of-adjacent-digits-in-string` `reschedule-meetings-for-maximum-free-time-i` `reschedule-meetings-for-maximum-free-time-ii` `minimum-cost-good-caption` `maximum-difference-between-even-and-odd-frequency-i` `maximum-manhattan-distance-after-k-changes` `minimum-increments-for-target-multiples-in-an-array` `maximum-difference-between-even-and-odd-frequency-ii` `sort-matrix-by-diagonals` `assign-elements-to-groups-with-constraints`

**16C** (3E/4M/5H): `count-substrings-divisible-by-last-digit` `maximize-the-minimum-game-score` `sum-of-good-numbers` `separate-squares-i` `separate-squares-ii` `shortest-matching-substring` `find-special-substring-of-length-k` `eat-pizzas` `select-k-disjoint-special-substrings` `length-of-longest-v-shaped-diagonal-segment` `check-if-digits-are-equal-in-string-after-operations-i` `maximum-sum-with-at-most-k-elements`

**16D** (3E/5M/4H): `check-if-digits-are-equal-in-string-after-operations-ii` `maximize-the-distance-between-points-on-a-square` `transform-array-by-parity` `find-the-number-of-copy-arrays` `find-minimum-cost-to-remove-array-elements` `permutations-iv` `find-the-largest-almost-missing-integer` `longest-palindromic-subsequence-after-at-most-k-operations` `sum-of-k-subarrays-with-length-at-least-m` `lexicographically-smallest-generated-string` `fruits-into-baskets-ii` `choose-k-elements-with-maximum-sum`

**16E** (3E/5M/4H): `fruits-into-baskets-iii` `maximize-subarrays-after-removing-one-conflicting-pair` `unique-3-digit-even-numbers` `longest-common-prefix-of-k-strings-after-removal` `longest-special-path-ii` `maximum-unique-subarray-sum-after-deletion` `closest-equal-element-queries` `zero-array-transformation-iv` `count-beautiful-numbers` `maximum-containers-on-a-ship` `properties-graph` `find-the-minimum-amount-of-time-to-brew-potions`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 17 — 60 slugs (scrape batches 14, 16)

**17A** (3E/2M/7H): `minimum-operations-to-make-array-elements-zero` `reverse-degree-of-a-string` `maximize-active-section-with-trade-i` `minimum-cost-to-divide-array-into-subarrays` `maximize-active-section-with-trade-ii` `minimum-cost-to-reach-every-position` `longest-palindrome-after-substring-concatenation-i` `longest-palindrome-after-substring-concatenation-ii` `minimum-operations-to-make-elements-within-k-subarrays-equal` `minimum-pair-removal-to-sort-array-i` `maximum-product-of-subsequences-with-an-alternating-sum-equal-to-k` `minimum-pair-removal-to-sort-array-ii`

**17B** (3E/6M/3H): `minimum-distance-between-three-equal-elements-ii` `maximum-path-score-in-a-grid` `maximize-cyclic-partition-score` `maximize-expression-of-three-elements` `minimum-string-length-after-balanced-removals` `count-distinct-integers-after-removing-zeros` `count-stable-subarrays` `minimum-number-of-flips-to-reverse-binary-string` `total-waviness-of-numbers-in-range-i` `lexicographically-smallest-negated-permutation-that-sums-to-target` `total-waviness-of-numbers-in-range-ii` `concatenate-non-zero-digits-and-multiply-by-sum-i`

**17C** (1E/8M/3H): `find-maximum-balanced-xor-subarray-length` `concatenate-non-zero-digits-and-multiply-by-sum-ii` `number-of-effective-subsequences` `count-elements-with-at-least-k-greater-values` `maximum-substrings-with-distinct-start` `minimum-absolute-distance-between-mirror-pairs` `minimum-operations-to-equalize-subarrays` `complete-prime-number` `minimum-operations-to-make-binary-palindrome` `maximize-points-after-choosing-k-tasks` `minimum-inversion-count-in-subarrays-of-fixed-length` `sort-integers-by-binary-reflection`

**17D** (2E/7M/3H): `largest-prime-from-consecutive-prime-sum` `total-score-of-dungeon-runs` `maximum-subgraph-score-in-a-tree` `absolute-difference-between-maximum-and-minimum-k-elements` `reverse-words-with-same-vowel-count` `minimum-moves-to-balance-circular-array` `minimum-deletions-to-make-alternating-substring` `minimum-number-of-operations-to-have-distinct-elements` `maximum-sum-of-three-numbers-divisible-by-three` `maximum-score-after-binary-swaps` `last-remaining-integer-after-alternating-deletion-operations` `mirror-distance-of-an-integer`

**17E** (2E/6M/4H): `minimum-deletion-cost-to-make-all-characters-equal` `minimum-swaps-to-avoid-forbidden-values` `total-sum-of-interaction-cost-in-tree-groups` `maximum-score-of-a-split` `minimum-cost-to-acquire-required-items` `smallest-all-ones-multiple` `number-of-balanced-integers-in-a-range` `reverse-string-prefix` `minimum-subarray-length-with-distinct-sum-at-least-k` `find-maximum-value-in-a-constrained-sequence` `count-routes-to-climb-a-rectangular-grid` `largest-even-number`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 18 — 60 slugs (scrape batches 16)

**18A** (2E/7M/3H): `word-squares-ii` `minimum-cost-to-make-two-binary-strings-equal` `minimum-cost-to-merge-sorted-lists` `count-residue-prefixes` `number-of-centered-subarrays` `count-caesar-cipher-pairs` `maximum-bitwise-and-after-increment-operations` `best-reachable-tower` `minimum-operations-to-reach-target-array` `number-of-alternating-xor-partitions` `minimum-edge-toggles-on-a-tree` `vowel-consonant-score`

**18B** (2E/7M/3H): `maximum-capacity-within-budget` `lexicographically-smallest-string-after-deleting-duplicate-characters` `minimum-prefix-removal-to-make-array-strictly-increasing` `rotate-non-negative-elements` `pythagorean-distance-nodes-in-a-tree` `find-nth-smallest-integer-with-k-one-bits` `reverse-letters-then-special-characters-in-a-string` `minimum-k-to-reduce-array-within-limit` `longest-strictly-increasing-subsequence-with-non-zero-bitwise-and` `minimum-partition-score` `count-monobit-integers` `final-element-after-subarray-deletions`

**18C** (3E/6M/3H): `longest-alternating-subarray-after-removing-at-most-one-element` `count-dominant-indices` `merge-adjacent-equal-elements` `count-subarrays-with-cost-less-than-or-equal-to-k` `maximum-score-using-exactly-k-pairs` `weighted-word-mapping` `number-of-prefix-connected-groups` `house-robber-v` `palindromic-path-queries-in-a-tree` `toggle-light-bulbs` `first-element-with-unique-frequency` `longest-almost-palindromic-substring`

**18D** (2E/7M/3H): `maximum-subarray-xor-with-bounded-range` `find-the-score-difference-in-a-game` `check-digitorial-permutation` `maximum-bitwise-xor-after-rearrangement` `count-sequences-to-k` `smallest-pair-with-different-frequencies` `merge-close-characters` `minimum-operations-to-make-array-parity-alternating` `sum-of-k-digit-numbers-in-a-range` `trim-trailing-vowels` `minimum-cost-to-split-into-ones` `minimum-bitwise-or-from-grid`

**18E** (3E/6M/3H): `count-subarrays-with-k-distinct-integers` `minimum-capacity-box` `find-the-smallest-balanced-index` `minimum-operations-to-sort-a-string` `minimum-cost-to-partition-a-binary-string` `first-unique-even-element` `sum-of-gcd-of-formed-pairs` `minimum-cost-to-equalize-arrays-using-swaps` `count-fancy-numbers-in-a-range` `count-commas-in-range` `count-commas-in-range-ii` `longest-arithmetic-sequence-after-changing-at-most-one-element`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 19 — 60 slugs (scrape batches 16)

**19A** (3E/4M/5H): `maximum-points-activated-with-one-addition` `construct-uniform-parity-array-i` `construct-uniform-parity-array-ii` `minimum-removals-to-achieve-target-xor` `count-good-subarrays` `minimum-absolute-difference-between-two-values` `direction-assignments-with-exactly-k-visible-people` `minimum-xor-path-in-a-grid` `count-non-decreasing-arrays-with-given-digit-sums` `first-matching-character-from-both-ends` `sum-of-sortable-integers` `incremental-even-weighted-cycle-queries`

**19B** (2E/7M/3H): `mirror-frequency-distance` `integers-with-multiple-sum-of-two-cubes` `minimum-increase-to-maximize-special-indices` `minimum-operations-to-achieve-at-least-k-peaks` `traffic-signal-color` `count-digit-appearances` `minimum-operations-to-transform-array-into-alternating-prime` `maximum-value-of-concatenated-binary-segments` `find-the-degree-of-each-vertex` `angles-of-a-triangle` `longest-balanced-substring-after-one-swap` `good-subsequence-queries`

**19C** (3E/5M/4H): `smallest-stable-index-i` `smallest-stable-index-ii` `multi-source-flood-fill` `count-good-integers-on-a-grid-path` `valid-digit-number` `compare-sums-of-bitonic-parts` `count-connected-subgraphs-with-even-node-sum` `k-th-smallest-remaining-even-integer-in-subarray-queries` `valid-elements-in-an-array` `sort-vowels-by-frequency` `minimum-operations-to-make-array-non-decreasing` `maximum-sum-of-alternating-subsequence-with-distance-at-least-k`

**19D** (3E/6M/3H): `count-indices-with-opposite-parity` `sum-of-primes-between-number-and-its-reverse` `minimum-cost-to-move-between-indices` `maximize-fixed-points-after-deletions` `score-validator` `minimum-flips-to-make-binary-string-coherent` `minimum-generations-to-target-point` `minimum-threshold-path-with-limited-heavy-edges` `concatenate-array-with-reverse` `count-valid-word-occurrences` `minimize-array-sum-using-divisible-replacements` `minimum-cost-to-buy-apples-ii`

**19E** (3E/6M/3H): `check-adjacent-digit-differences` `count-k-th-roots-in-a-range` `largest-local-values-in-a-matrix-ii` `smallest-unique-subarray` `minimum-swaps-to-move-zeros-to-end` `minimum-operations-to-make-array-modulo-alternating-i` `maximum-path-intersection-sum-in-a-grid` `count-non-adjacent-subsets-in-a-rooted-tree` `limit-occurrences-in-sorted-array` `password-strength` `minimum-operations-to-sort-a-permutation` `number-of-pairs-after-increment`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

### Wave 20 — 40 slugs (scrape batches 2, 4, 6, 7, 9, 11, 14, 16) — **design-shaped packs, budget extra care**

**20A** (3E/5M/4H): `digit-frequency-score` `maximum-number-of-items-from-sale-i` `maximum-number-of-items-from-sale-ii` `lexicographically-maximum-mex-array` `exactly-one-consecutive-set-bits-pair` `minimum-energy-to-maintain-brightness` `maximum-total-value-of-covered-indices` `maximum-score-with-co-prime-element` `sum-of-compatible-numbers-in-range-i` `valid-binary-strings-with-cost-limit` `maximum-sum-of-m-non-overlapping-subarrays-i` `maximum-sum-of-m-non-overlapping-subarrays-ii`

**20B** (0E/9M/1H): `mini-parser` `exam-room` `maximum-frequency-stack` `complete-binary-tree-inserter` `find-elements-in-a-contaminated-binary-tree` `tweet-counts-per-frequency` `design-a-stack-with-increment-operation` `design-browser-history` `subrectangle-queries` `throne-inheritance`

**20C** (0E/7M/3H): `fancy-sequence` `design-front-middle-back-queue` `operations-on-tree` `stock-price-fluctuation` `simple-bank-system` `walking-robot-simulation-ii` `sequentially-ordinal-rank-tracker` `design-bitset` `encrypt-and-decrypt-strings` `design-memory-allocator`

**20D** (0E/7M/1H): `design-graph-with-shortest-path-calculator` `frequency-tracker` `design-task-manager` `design-spreadsheet` `implement-router` `design-auction-system` `design-ride-sharing-system` `design-event-manager`

After the wave: sweep → `--check` all of the wave's slugs → delete quarantined →
freeze (`--bundle` + `cargo test` + `npm run build`) → fold missing/failed into the next wave.

## Final wave — defer-candidate confirmation (37 slugs) — RESOLVED

Two Sonnet subagents split the 37 heuristically-flagged slugs in half and, per slug,
either confirmed the defer (§5-keyed reason) or authored it where the heuristic was
wrong. Batch B's agent dropped mid-run on the same transient "Connection closed" API
error seen throughout this project (after landing 3 files); resumed in place per the
established pattern, finished cleanly. **Result: 3 authored, 34 confirmed defers, 0
forced/incorrect packs.**

### Authored (heuristic false positives — 3)

- `shuffle-the-array`: judge `exact`. Deterministic interleave of two array halves
  (`[x1..xn,y1..yn] → [x1,y1,...]`) — matched the heuristic on the word "shuffle" but
  has no RNG.
- `shuffle-string`: judge `exact`. Deterministic index-permutation restore (`s[i]`
  moves to `indices[i]`) — same false-positive keyword match, no RNG.
- `iterator-for-combination`: judge `design`, `no_anchor_ok: true`. Constructor args
  (`characters: str`, `combinationLength: int`) are plain JSON scalars, not an
  injected external object — it's internal traversal over its own constructor args,
  so unlike `binary-search-tree-iterator` (see below) it needs no `io_types`/node
  support at all and rides the `design` judge cleanly.

### Confirmed defers (34), reasons keyed to CONTENT_PIPELINE.md §5

**Next-pointer / N-ary / quad / multilevel trees**: `populating-next-right-pointers-in-each-node`,
`populating-next-right-pointers-in-each-node-ii` — extra `next` sibling-link field not
representable in the `tree` io_type wire form.

**Graph Node**: `clone-graph` — adjacency-list `Node` graph, not list/tree.

**Random-pointer lists**: `copy-list-with-random-pointer` — second cross-link (`random`)
pointing anywhere in the list, unencodable in `[v0,...]`.

**Cyclic linked lists**: `linked-list-cycle`, `linked-list-cycle-ii` — cyclic `pos`
structure can't be expressed in the acyclic wire form.

**Node-reference args**: `lowest-common-ancestor-of-a-binary-search-tree`,
`lowest-common-ancestor-of-a-binary-tree` — `p`/`q` are references into the *same*
tree instance as `root`, not independently constructible values.

**Interactive / non-deterministic**: `first-bad-version` (injected `isBadVersion`
callback), `guess-number-higher-or-lower` (injected `guess` callback),
`insert-delete-getrandom-o1`, `insert-delete-getrandom-o1-duplicates-allowed`,
`linked-list-random-node`, `shuffle-an-array`, `random-pick-index`,
`generate-random-point-in-a-circle`, `random-point-in-non-overlapping-rectangles`,
`random-flip-matrix`, `random-pick-with-weight`, `random-pick-with-blacklist` — all
RNG-backed ops with no single correct output; confirmed via the `design` judge's
actual cross-check code (`run_design`/`runDesign` in both harnesses) that it does
structural exact-match per op with no per-op validator hook, so randomness can't
cross-check python vs js vs oracle.

**Codec round-trip**: `serialize-and-deserialize-binary-tree`,
`serialize-and-deserialize-bst` — encoding is the author's free choice, only
`deserialize(serialize(x)) == x` is checkable; no judge type supports round-trip
identity today.

**Iterator / injected objects**: `peeking-iterator` (wraps an injected `Iterator`
object), `flatten-nested-list-iterator` (injected `NestedInteger` object graph),
`find-positive-integer-solution-for-a-given-equation` (injected `CustomFunction`),
`guess-the-word` (injected `Master.guess()`), `find-in-mountain-array` (injected
`MountainArray.get()/.length()`).

**Not in the §5 table — genuine concurrency, no threading-capable harness**:
`print-in-order`, `print-foobar-alternately`, `print-zero-even-odd`, `building-h2o`,
`fizz-buzz-multithreaded`, `the-dining-philosophers` — the runner executes solutions
single-threaded with no synchronization primitives exposed to Python/JS; each of
these needs real concurrent threads/processes racing or coordinating via
mutex/semaphore/barrier, which the harness doesn't provide. Worth adding as an
explicit §5 category (concurrency/threading) if this project revisits Tier-B scope.

**§4b explicit Tier-B exclusion**: `binary-search-tree-iterator` — constructor takes
`root: TreeNode`, but confirmed in both `harness.py`/`harness.js` that `io_types`
deserialization only applies outside `design` mode; `run_design`/`runDesign` gets raw
JSON with no TreeNode conversion, so a real tree object can't reach the constructor.
PACK_AUTHORING_GUIDE.md §4b already lists "BST iterator" by name as out of scope.

## Progress ledger (tick as waves freeze)

| wave | slugs | dispatched | frozen | redo carried |
|---|---|---|---|---|
| 0 | 8 | yes | 0 | 8 (all quarantined, deleted) |
| 1 | 68 (60 + 8 Wave-0 redo folded in) | yes | 42 | 13 (1D dropped mid-wave on API error before writing any files) |
| 2 | 73 (60 + 13 Wave-1 redo folded in) | yes | 73 | 0 |
| 3 | 60 | yes | 6 | 54 (all 5 subagents hit the session API rate limit almost immediately; only 7 files landed, 1 quarantined+deleted) |
| 4 | 114 (60 + 54 Wave-3 redo folded in) | yes | 114 | 0 |
| 5 | 60 | yes | 60 | 0 |
| 6 | 60 | yes | 60 | 0 |
| 7 | 60 | yes | 60 | 0 |
| 8 | 60 | yes | 60 | 0 |
| 9 | 60 | yes | 60 | 0 |
| 10 | 60 | yes | 60 | 0 |
| 11 | 60 | yes | 60 | 0 |
| 12 | 60 | yes | 9 | 51 (all 5 subagents hit the session API rate limit almost immediately; only 9 files landed across 12C/12E, 0 quarantined) |
| 13 | 111 (60 + 51 Wave-12 redo folded in) | yes | 82 | 29 (13B fully dropped on API error before writing any files; 13A/13E each dropped 3-4 slugs mid-run on the same error; 0 quarantined) |
| 14 | 89 (60 + 29 Wave-13 redo folded in) | yes | 56 | 33 (14B dropped 15/18 mid-run on the same transient API error; 14D fully dropped, 0/18, before writing any files; 0 quarantined) |
| 15 | 93 (60 + 33 Wave-14 redo folded in) | yes | 45 | 48 (15A dropped 0/19 before writing any files; 15C dropped 0/19 before writing any files; 15D dropped 4/18 mid-run; 15E dropped 6/18 mid-run — all on the same session-limit API error) |
| 16 | 108 (60 + 48 Wave-15 redo folded in) | yes | **108** | 0 (first pass landed 35/108 — 16B/16C/16D/16E all dropped mid-run on the same transient API error, 2 quarantined for real bugs; rather than deferring the other 73 to Wave 17, per explicit user instruction they were retried in place — smaller sub-splits (down to 7 slugs) on repeated drops — until every slug either passed `--check` or a legitimate DEFER was recorded; see Wave 16 notes) |
| 17 | 60 (0 Wave-16 redo folded in) | yes | **60** | 0 (all 5 chunks 17A-17E landed cleanly on the first pass, 0 quarantined, 0 defers) |
| 18 | 60 (0 Wave-17 redo folded in) | yes | **60** | 0 (all 5 chunks 18A-18E landed cleanly on the first pass, 0 quarantined, 0 defers) |
| 19 | 60 (0 Wave-18 redo folded in) | yes | 47 | 13 (19B dropped 0/12 before writing any files on a transient API-connection-closed error; 19E's `maximum-path-intersection-sum-in-a-grid` is a genuine algorithmic failure — not a deferred shape, the agent could not find an O(m·n·polylog) algorithm and left it unauthored rather than ship something wrong/too slow — see Wave 19 notes) |
| 20 | 53 (40 + 13 Wave-19 redo folded in) | yes | **52** | 1 (`maximum-path-intersection-sum-in-a-grid` deferred a second time — genuine hard-algorithm case, no efficient-enough correct approach found; see Wave 20 notes) |
| defer | 37 | yes | **3** | 0 (34 confirmed as genuine defers, not redo — see "Final wave" section above for full reasons) |
| 48-branch | 42 | yes | **42** | 0 (the closing-the-48 sweep — see the section below) |

### Closing-the-48 branch (resolves the confirmed defers)

The `48` branch closed every confirmed defer except the 6 concurrency problems by
extending the harness itself (`.docs/48.md` has the inventory, `.docs/48_PLAN.md`
the design). New capabilities, each proven by a pilot pack before its wave ran:

- **Wire types** (`PACK_AUTHORING_GUIDE.md` §4c): `cyclic_list`, `random_list`
  (freshness-checked), `graph` (freshness-checked), `n_ary_tree`, `quad_tree`,
  `next_tree` (serialized by following the next pointers), `multilevel_list`,
  plus the param-referencing forms `node_ref` / `clone_of` / `tail_of` /
  `node_index_of` and `ctx_only`.
- **`design_io`**: node-typed constructor/method boundaries in design mode —
  un-deferred `binary-search-tree-iterator` (the §4b Tier-B exclusion above).
- **New judges**: `round_trip` (codecs: `decode(encode(x))` canonicalized) and
  `property` (randomized outputs validated by a pack-shipped op-replay validator;
  build cross-check runs every implementation through the validator instead of
  byte comparison).
- **Injected shims**: `iterator`, `nested_integer`, `custom_function`,
  `master_guess`, `mountain_array`, `is_bad_version`, `guess_oracle`, `rand7`
  (deterministic LCG) — with call-budget enforcement and JS stub currying.
- **`maximum-path-intersection-sum-in-a-grid`** (the Wave-19/20 hard-algorithm
  defer): resolved with the exact column-interval DP — state after each column is
  both players' exit rows; each column's shared cells are one interval overlap.
  O(n·m⁴) with small pack sizes (≤12 rows, ≤144 cells); anchors against both
  statement examples and a brute-force path-pair oracle.

Still deferred (6, permanent unless the sandbox grows real threading):
`print-in-order` `print-foobar-alternately` `print-zero-even-odd` `building-h2o`
`fizz-buzz-multithreaded` `the-dining-philosophers` — correctness for these is
absence of races/deadlocks across interleavings, JS has no shared-memory threads
(so the cross-language differential cannot exist), and a run-based judge cannot
prove the property. Recorded as a deliberate product decision, not a gap.

### Wave 1 notes

Redo slugs to fold into Wave 2's smallest chunks (chunk 1D never wrote any files before
its subagent dropped on a transient API error — all 13 are unauthored, not quarantined):
`maximum-score-from-grid-operations` `maximum-number-of-operations-to-move-ones-to-the-end`
`minimum-operations-to-make-array-equal-to-target` `count-the-number-of-substrings-with-dominant-ones`
`check-if-the-rectangle-corner-is-reachable` `minimum-number-of-flips-to-make-binary-grid-palindromic-i`
`minimum-number-of-flips-to-make-binary-grid-palindromic-ii` `time-taken-to-mark-all-nodes`
`shortest-distance-after-road-addition-queries-i` `shortest-distance-after-road-addition-queries-ii`
`alternating-groups-iii` `snake-in-matrix` `sort-array-by-parity`

Permanent defers hit during Wave 1 (not redo — genuinely unsupported shapes, add to the
defer-candidate list's tally, no pack expected): `intersection-of-two-linked-lists`
(shared-reference intersection, not constructible in the wire format), `delete-node-in-a-linked-list`
(node-reference arg), `construct-quad-tree` (quad-tree Node), `n-ary-tree-level-order-traversal`
(n-ary Node), `flatten-a-multilevel-doubly-linked-list` (multilevel Node), `implement-rand10-using-rand7`
(RNG-injected/interactive), `encode-and-decode-tinyurl` (codec round-trip), `logical-or-of-two-binary-grids-represented-as-quad-trees`
(quad-tree Node), `maximum-depth-of-n-ary-tree` (n-ary Node), `n-ary-tree-preorder-traversal` (n-ary Node),
`n-ary-tree-postorder-traversal` (n-ary Node), `all-nodes-distance-k-in-binary-tree` (node-reference arg),
`find-a-corresponding-node-of-a-binary-tree-in-a-clone-of-that-tree` (node-reference arg).

Operational: an emergency mid-wave repair was needed — `tools/test-packs.json` had a single
corrupted byte (a `\r`→form-feed bit flip in JSON whitespace, ~4.7MB in) that crashed every
`build_packs.py` invocation, including plain `--check`. Confirmed the only anomalous byte in the
file, patched it in place (user-approved), then re-verified with a full `--bundle` + `cargo test`
+ `npm run build` pass before continuing the wave. Also found and cleared a stale `tools/packs/.lock`
left behind by a subagent that must have run a real build instead of `--check` (against the brief) —
no active process was holding it. Two subagents (1B, 1C) also ran read-only `git status`/`git diff`
to investigate the index.json changes despite the "never run git" rule; no destructive commands were
run and nothing was altered, but future briefs should state the rule covers read-only git commands too.

### Wave 2 notes

All 5 chunks (2A–2E, 73 slugs total including the 13 Wave-1 redo slugs folded in — 3 into 2A,
3 into 2B, 2 into 2C, 2 into 2D, 3 into 2E) landed cleanly: every slug reported OK, zero DEFERs,
zero quarantines, zero redo carried into Wave 3. `--check --only` across all 73 slugs passed
before freezing; `--bundle` + `cargo test` + `npm run build` all green. Bundle now holds 1,916
packs (1,801 prior + 8 Wave-0 + 42 Wave-1 + 73 Wave-2 minus resolved dupes/reworks along the way).

Notable subagent findings (informational, not redo):
- `best-position-for-a-service-centre`: fixed a Weiszfeld-iteration convergence bug (a
  "skip near-zero distance" guard oscillated forever on coincident points) before shipping.
- `parallel-courses-ii`: an oracle timeout from `bin(x).count('1')` was fixed by switching to
  `int.bit_count()` in both solution and oracle.
- `count-submatrices-with-all-ones`: a wrong incremental-sum recurrence was caught and fixed
  pre-freeze.
- `magnetic-force-between-two-balls`: an oracle that brute-forced every gap value timed out on
  a huge-span boundary case; replaced with an independently-shaped binary-search oracle.
- One subagent (2B) internally split its 15 slugs into its own sub-batches after two of its
  five middle slugs dropped on a transient "Connection closed" API error mid-response; it
  re-authored those directly and all 15 still landed OK — noted here since it's a variance in
  how a subagent recovered from the same class of transient error described in Wave 1.
- `create-components-with-same-value` and `maximize-palindrome-length-from-subsequences`
  (pre-existing frozen packs, not part of this wave) showed source-changed-but-frozen hash
  mismatches during `--bundle`; left frozen per the tool's default (no `--allow-refreeze`
  passed) — flagging in case a future wave needs to intentionally re-verify them.

Redo slugs to fold into Wave 3: none.

### Wave 3 notes

All 5 subagents (3A–3E) were terminated almost immediately by the session's API rate
limit ("You've hit your session limit · resets 10:50pm Africa/Johannesburg") — this hit
the whole session, not one subagent, so essentially no authoring work happened. Trusted
the disk per the orchestration procedure rather than the (failure) reports:

- Only 7 of the 60 target files existed on disk after the sweep: `count-unhappy-friends`,
  `make-sum-divisible-by-p`, `matrix-diagonal-sum`, `minimum-cost-to-connect-two-groups-of-points`,
  `number-of-ways-to-split-a-string`, `rearrange-spaces-between-words` (all 6 passed
  `--check` and are now frozen), plus `maximum-length-of-subarray-with-positive-product`
  which was quarantined (`brute force disagrees on [[0, 1, -2, -3, -4]]: 3 vs 2`) and
  deleted.
- Froze the 6 valid packs: `--bundle` (bundle now holds 1,922 packs), `cargo test`
  (all green), `npm run build` (green). The two pre-existing "source changed but frozen"
  warnings (`create-components-with-same-value`, `maximize-palindrome-length-from-subsequences`)
  are the same ones noted in Wave 2 — left frozen, no `--allow-refreeze` passed.
- Redo list for the next wave (54 slugs — all of Wave 3 except the 6 frozen above,
  including the quarantined one since its failure looks like an authoring bug, not a
  genuine defer):
  `minimum-number-of-vertices-to-reach-all-nodes` `minimum-numbers-of-function-calls-to-make-target-array`
  `detect-cycles-in-2d-grid` `most-visited-sector-in-a-circular-track` `maximum-number-of-coins-you-can-get`
  `find-latest-group-of-size-m` `stone-game-v` `detect-pattern-of-length-m-repeated-k-or-more-times`
  `maximum-length-of-subarray-with-positive-product` `minimum-number-of-days-to-disconnect-island`
  `number-of-ways-to-reorder-array-to-get-same-bst` `shortest-subarray-to-be-removed-to-make-array-sorted`
  `count-all-possible-routes` `replace-all-s-to-avoid-consecutive-repeating-characters`
  `number-of-ways-where-square-of-number-is-equal-to-product-of-two-numbers` `minimum-time-to-make-rope-colorful`
  `remove-max-number-of-edges-to-keep-graph-fully-traversable` `special-positions-in-a-binary-matrix`
  `check-if-string-is-transformable-with-substring-sort-operations` `sum-of-all-odd-length-subarrays`
  `maximum-sum-obtained-of-any-permutation` `strange-printer-ii` `split-a-string-into-the-max-number-of-unique-substrings`
  `maximum-non-negative-product-in-a-matrix` `crawler-log-folder` `maximum-profit-of-operating-a-centennial-wheel`
  `maximum-number-of-achievable-transfer-requests` `design-parking-system`
  `alert-using-same-key-card-three-or-more-times-in-a-one-hour-period` `find-valid-matrix-given-row-and-column-sums`
  `find-servers-that-handled-most-number-of-requests` `special-array-with-x-elements-greater-than-or-equal-x`
  `even-odd-tree` `maximum-number-of-visible-points` `minimum-one-bit-operations-to-make-integers-zero`
  `maximum-nesting-depth-of-the-parentheses` `maximal-network-rank` `split-two-strings-to-make-palindrome`
  `count-subtrees-with-max-distance-between-cities` `mean-of-array-after-removing-some-elements`
  `coordinate-with-maximum-network-quality` `number-of-sets-of-k-non-overlapping-line-segments`
  `largest-substring-between-two-equal-characters` `lexicographically-smallest-string-after-applying-operations`
  `best-team-with-no-conflicts` `graph-connectivity-with-threshold` `slowest-key` `arithmetic-subarrays`
  `path-with-minimum-effort` `rank-transform-of-a-matrix` `sort-array-by-increasing-frequency`
  `widest-vertical-area-between-two-points-containing-no-points` `count-substrings-that-differ-by-one-character`
  `number-of-ways-to-form-a-target-string-given-a-dictionary`
- Operational note for next wave: given the reset time quoted by the rate limit
  ("resets 10:50pm Africa/Johannesburg"), do not dispatch a new wave in this session
  until that window has passed — restarting immediately will likely hit the same wall
  before any subagent writes a file.

### Wave 4 notes

All 5 chunks (4A–4E) landed cleanly on the first dispatch — zero DEFERs, zero
quarantines, zero redo carried forward. All 54 Wave-3 redo slugs were folded in
(~11 per chunk, since that session had barely progressed past the rate limit), so
this wave authored 114 packs instead of the usual ~60: 4A/4B/4C/4D at 23 slugs each,
4E at 22. Every chunk was independently `--check`-verified against disk (not just
trusted from subagent reports) before freezing. `--bundle` verified/rebuilt all 114,
skipped 1,920 already-frozen, quarantined 0 (bundle now holds **2036** packs, up from
1,922). `cargo test` and `npm run build` both green. The 2 pre-existing "source
changed but frozen" warnings (`create-components-with-same-value`,
`maximize-palindrome-length-from-subsequences`) are unchanged from Wave 2/3 — left
frozen, no `--allow-refreeze` passed.

Notable subagent findings (informational, not redo):
- `maximum-length-of-subarray-with-positive-product` (the slug quarantined and
  redo-carried from Wave 3) was re-authored correctly this time: the subagent hand-
  traced the failing case `[0, 1, -2, -3, -4]` — the zero contributes a length-0
  segment, and within `[1, -2, -3, -4]` the correct answer is 3 (`[1,-2,-3]`, 2
  negatives → positive product), not 2. Both solution and oracle were checked against
  this case before shipping.
- `design-parking-system` (design/class-method shape) was authorable in the current
  schema — not deferred.
- `find-valid-matrix-given-row-and-column-sums` needed an `any_valid` judge (the
  statement's example matrix isn't the only valid answer for given row/col sums);
  the subagent manually verified the validator accepts the reference output and an
  alternate valid matrix, and rejects sum/negative violations.
- Several chunks' subagents internally parallelized their own slug lists via forked
  sub-agents (visible as chunk-internal "group A/B/C/D" or "batch A–E" labels in
  their own tool use); some of those forks' status messages surfaced to the
  orchestrating session directly, which briefly looked like a second concurrent
  orchestration session running the same wave before being traced back to this
  wave's own subagents — no actual duplicate/external session was involved.
- Multiple chunks (4A, 4B, 4E) reported sub-batches dropping mid-run on transient
  "Connection closed" API errors, consistent with prior waves; each recovered by
  sweeping disk and redispatching only the missing slugs before reporting.

Redo slugs to fold into Wave 5: none.

### Wave 5 notes

All 5 chunks (5A–5E, 60 slugs) landed cleanly — zero DEFERs, zero quarantines, zero
redo carried forward. Two subagents (5A, 5D) dropped mid-run on the same transient
"Connection closed" API error seen in prior waves (5A after 1/12 slugs, 5D after
7/12); both were resumed in place via a follow-up message rather than redispatched
fresh, and both finished their remaining slugs cleanly on resume. `--check --only`
across all 60 slugs passed before freezing (0 quarantined). `--bundle` verified/
rebuilt all 60, skipped 2,032 already-frozen (bundle now holds **2096** packs, up
from 2036). `cargo test` and `npm run build` both green.

Two packs used `any_valid` judges (`find-unique-binary-string`,
`find-array-given-subset-sums` in 5A) plus one in 5D (`find-missing-observations`,
`no_anchor_ok: true`) — all three had validator behavior manually verified (accepts
reference + alternative valid outputs, rejects malformed ones) before shipping.
`two-out-of-three` (5D) and `find-original-array-from-doubled-array` (5C) used
`unordered` judges (multiset-valued results).

Notable subagent findings (informational, not redo):
- `first-day-where-you-have-been-in-all-the-rooms` (5B): initial oracle used a
  "first-visit" simulation rule that disagreed with the intended parity-based rule;
  caught and fixed before freeze, verified against all official examples + 300
  random cases.
- `gcd-sort-of-an-array` (5B): DSU-over-prime-factors solution cross-checked against
  an independently-shaped pairwise-gcd DSU oracle, 300 random cases, 0 mismatches.
- `second-minimum-time-to-reach-destination` (5E): a naive "bipartite ⇒ +2" shortcut
  oracle was tried first and found insufficient under fuzzing; replaced with an
  independent parity-doubled-state BFS oracle.
- `number-of-valid-words-in-a-sentence` (5E): the statement's own example prose
  ("Examples of valid words include...") confused the anchor parser into treating it
  as a 4th unanchored example block; shipped with `no_anchor_ok: true` and verified
  via 8 authored `edge_inputs` instead.
- `partition-array-into-two-arrays-to-minimize-sum-difference` (5D): true brute
  force is infeasible at the n=15 boundary case included in `edge_inputs`;
  correctness established via offline differential testing at small n plus a timing
  check at the boundary (no `oracle_python` shipped for this one).
- Unrelated to this wave's slugs: two *additional* pre-existing frozen packs showed
  source-changed-but-frozen hash mismatches during `--bundle` —
  `largest-substring-between-two-equal-characters` and
  `lexicographically-smallest-string-after-applying-operations` (both Wave 3/4
  slugs). Their on-disk files were modified today (timestamps land inside this
  wave's dispatch window) though no Wave 5 subagent's report mentions touching
  them. Investigated post-freeze: diffed both against their frozen entries in
  `tools/test-packs.json` — `solutions` (Python+JS), `judge`, `pattern`, and
  `stress` were byte-identical; the only change was a `charset` field added to
  each string constraint (`abcdefghijklmnopqrstuvwxyz` / `0123456789`), which
  matches the real problem's actual constraint and isn't consumed anywhere in
  `build_packs.py` (inert metadata, no functional effect). User approved
  refreezing: ran `--allow-refreeze --only <the 2 slugs>`, then `--bundle`,
  `cargo test`, `npm run build` — all green, manifest hashes now current.
- The 2 long-standing mismatches from Wave 2/3 (`create-components-with-same-value`,
  frozen 2026-06-23, predating the wave system) were also investigated and resolved
  this session. Diff against `tools/test-packs.json`: solutions/judge/pattern/stress
  byte-identical; the frozen `tests` array for each contained an exact **duplicate**
  pair (e.g. `create-components-with-same-value` shipped 9 cases where indices 0-1
  were verbatim-repeated as 2-3; `maximize-palindrome-length-from-subsequences`
  shipped 9 where 0-2 repeated as 3-5). Current on-disk `edge_inputs` had those
  duplicates already removed (7 and 6 unique cases respectively) — a dedup cleanup,
  not a correctness change. Both `RE-VERIFIED` cleanly; user approved refreezing:
  `--allow-refreeze --only <the 2 slugs>` → `--bundle` → `cargo test` → `npm run
  build`, all green. **All 4 known source-changed-but-frozen mismatches are now
  resolved** — `--bundle` reports `locked (changed): 0` going into Wave 6.

Redo slugs to fold into Wave 6: none.

### Wave 6 notes

All 5 chunks (6A–6E, 60 slugs) landed cleanly — zero DEFERs, zero quarantines, zero
redo carried forward. `--check --only` across all 60 slugs passed before freezing (0
quarantined). `--bundle` verified/rebuilt all 60, skipped 2,096 already-frozen (bundle
now holds **2156** packs, up from 2096). `cargo test` (87+ tests across all suites) and
`npm run build` both green.

Notable subagent findings (informational, not redo):
- `recover-the-original-array` (6E): `any_valid` judge with a manually-verified
  `validator_python`/`validator_javascript` — accepts the reference output plus a
  differently-ordered/differently-k'd valid alternative, rejects corrupted/wrong-length
  output.
- `maximum-employees-to-be-invited-to-a-meeting` (6E): hardest pack this wave —
  functional-graph 2-cycle + in-tree-depth solution, checked against a genuinely
  independent brute-force oracle over small n.
- `valid-arrangement-of-pairs` and `find-subsequence-of-length-k-with-the-largest-sum`
  (6C): both `any_valid` judges, spot-checked that a differently-ordered/differently-
  chosen-but-equally-valid alternative output is accepted and a broken/wrong-sum output
  is rejected.
- `maximum-number-of-tasks-you-can-assign` (6B): the deque-greedy had a real bug caught
  by cross-checking against a brute-force oracle — window-reveal condition must be
  `workers[ptr] + strength >= tasks[i]`, not `workers[ptr] >= tasks[i]`; fixed before
  shipping.
- `abbreviating-the-product-of-a-range` (6D): a real bug caught by the statement-example
  anchor (`1,4` → `"24e0"`) — initial implementation omitted the `e0` suffix when
  trailing-zero count was 0; fixed to always append `e{c}` in both languages + oracle.
  Avoids materializing the full (up to ~40,000-digit) product: trailing-zero count via
  factor-of-2/5 counts, last digits via a running product mod `10^11` (JS uses `BigInt`
  + manual modpow), leading digits via running `log10` sum; oracle uses exact bignum,
  kept to small `edge_inputs` ranges to stay under Python's int-to-str digit limit.
- `sum-of-k-mirror-numbers` (6B): verified sums stay well under 2^53 for all valid
  (k, n) so plain ints/doubles are safe; oracle independently generates palindromes in
  base-k as a genuine differential check (not just re-deriving the same logic).
- One subagent (6A) ran a read-only `git status` mid-task despite the hard rule against
  running git; no state-changing git command was run and no tracked files were touched,
  but the brief should be tightened to say "no git commands at all, not even read-only."

Redo slugs to fold into Wave 7: none.

### Wave 7 notes

All 5 chunks (7A–7E, 60 slugs) landed cleanly — zero DEFERs, zero quarantines, zero
redo carried forward. This wave's brief added the explicit "not even read-only git
commands" wording (per the Wave 6 finding); no subagent ran git this time. Two chunks
(7A, and part of 7A's internal batches) dropped mid-run on the same transient
"Connection closed" API error seen in prior waves and were recovered by redispatching
just the missing slugs — all 12 still landed OK. `--check --only` across all 60 slugs
was independently re-verified by the orchestrating session (not just trusted from
subagent reports) before freezing — 0 quarantined. `--bundle` verified/rebuilt all 60,
skipped 2,156 already-frozen (bundle now holds **2216** packs, up from 2156). `cargo
test` (all suites) and `npm run build` both green.

Notable subagent findings (informational, not redo):
- `maximum-and-sum-of-array` (7C): initial base-3 bitmask DP updated the answer from
  partially-filled slot states instead of only fully-placed ones; caught and fixed
  before the check passed.
- `maximum-split-of-positive-even-integers` (7C): `any_valid` judge with
  `no_anchor_ok: true` — the greedy reference's canonical split differs from the
  statement's own example split for the same max count; validator manually verified
  to accept the reference output, an alternate valid split, and a reordered valid
  split, and to reject wrong-size/duplicate/odd/sum-mismatch outputs.
- `find-all-lonely-numbers-in-the-array` (7B): `unordered` judge (multiset-valued
  result, order not significant).
- `groups-of-strings` (7B): an initial "replace" bridge over-merged unrelated
  bitmasks (~1.7% mismatch over 3,000 randomized trials against a pairwise oracle);
  fixed by namespacing replace-edges separately from add/delete edges.
- `find-substring-with-given-hash-value` (7B): needed `BigInt` in JS (power/modulo up
  to 1e9 overflows float64 precision) and exact replication of the official
  leftmost-match tie-break.
- `minimum-time-to-finish-the-race` (7D): first oracle draft used
  `functools.lru_cache` recursion and hit Python's recursion-depth limit at the
  `numLaps=1000` boundary case; replaced with a bottom-up iterative memo table.
- `append-k-integers-with-minimal-sum` (7E): naive linear-walk oracle timed out for
  k up to 1e8; replaced with an independently-shaped binary-search + prefix-sum
  oracle.
- `maximize-the-topmost-element-after-k-moves` (7E): hardest pack this wave — an
  initial "well-known-looking" closed-form formula was found wrong in the `k == n-1`
  and general branches via an exhaustive pile/removed-stack BFS simulator built
  offline and checked against 200k+ random small cases; the shipped formula is the
  one that survived exhaustive validation.
- `count-hills-and-valleys-in-an-array` (7E): first oracle attempt double-counted
  multi-index equal-value runs, disagreeing with the statement's own worked example;
  fixed to only evaluate the first index of each equal-value run.
- `create-binary-tree-from-descriptions` (7E): uses `io_types` with `returns: "tree"`
  — in scope per the guide, not a deferred shape (input is a plain edge-description
  array, not an injected `Node`).

Redo slugs to fold into Wave 8: none.

### Wave 8 notes

All 5 chunks (8A–8E, 60 slugs) landed — zero DEFERs, zero quarantines, zero redo
carried forward. Two chunks (8B, 8C) dropped mid-run on the same transient
"Connection closed" API error seen in prior waves; both were resumed in place via a
follow-up message (per the Wave 5 pattern) rather than redispatched fresh, and both
finished their remaining slugs cleanly on resume. `--check --only` across all 60
slugs was independently re-verified by the orchestrating session (not just trusted
from subagent reports) before freezing — 0 quarantined. `--bundle` verified/rebuilt
all 60, skipped 2,216 already-frozen (bundle now holds **2276** packs, up from
2216). `cargo test` (all suites) and `npm run build` both green.

Notable subagent findings (informational, not redo):
- `minimize-the-maximum-of-two-arrays` (8D): the divisor↔array exclusion roles are
  the reverse of a naive reading (arr1 excludes divisor1, not divisor2); caught and
  fixed by hand-tracing the statement's own examples before shipping.
- `number-of-ways-to-select-buildings` (8A): initial solution swapped the
  zeros/ones "after" counters for the `'0'`/`'1'` middle-selection branches (wrong
  on inputs like `"11100"`); fixed to use `ones_after` for a `'0'` middle and
  `zeros_after` for a `'1'` middle, mirrored in JS.
- `find-palindrome-with-fixed-length` (8A): the oracle's brute-force enumeration
  guard short-circuited to `-1` for large `intLength` (skipped length 15);
  replaced with an independent digit-by-digit arithmetic-construction oracle
  correct at every length.
- `minimum-score-of-a-path-between-two-cities` (8B): caught a constraint mismatch
  before shipping — road distance is bounded `1..10^4`, not `10^5`.
- `time-to-cross-a-bridge` (8E): the dev scrape's own body text and its
  example_tests input disagree on Example 2's `time` values (`pick=5` in prose vs
  `pick=9` in the input array); shipped `no_anchor_ok: true`, with both statement
  examples hand-verified against an independent oracle and placed in
  `edge_inputs` instead of relying on anchor auto-parsing.
- `minimize-result-by-adding-parentheses-to-expression` (8B): `any_valid` judge,
  manually verified in both Python and JS — accepts the canonical minimal split,
  accepts a genuinely tied alternative split found by exhaustive search, rejects a
  malformed-shape output and a structurally-valid-but-non-minimal output.
- `maximum-points-in-an-archery-competition` and `find-the-difference-of-two-arrays`
  (8A): both `any_valid` judges, manually spot-checked accepting the reference plus
  a genuine alternative and rejecting sum/set-mismatched or suboptimal outputs.
- `find-consecutive-integers-from-a-data-stream` (8E): design-shaped
  (`ops`/`argLists` input), `no_anchor_ok: true` — in scope per the guide, not
  deferred.
- `remove-nodes-from-linked-list` (8B): `io_types: linked_list` in/out — plain
  singly-linked list, not a deferred §5 shape (no cycle/random pointer).

Redo slugs to fold into Wave 9: none.

### Wave 9 notes

All 5 chunks (9A–9E, 60 slugs) landed — zero DEFERs, zero quarantines, zero redo
carried forward. Chunk 9A dropped mid-run on the same transient "Connection closed"
API error seen in prior waves; it was resumed in place via a follow-up message (per
the Wave 5/8 pattern) rather than redispatched fresh, and finished its remaining
slugs cleanly on resume. `--check --only` across all 60 slugs was independently
re-verified by the orchestrating session (not just trusted from subagent reports)
before freezing — 0 quarantined. `--bundle` verified/rebuilt all 60, skipped 2,276
already-frozen (bundle now holds **2336** packs, up from 2276). `cargo test` (all
suites, 87+ tests) and `npm run build` both green.

Notable subagent findings (informational, not redo):
- `minimum-operations-to-reduce-an-integer-to-0` (9C): first oracle draft used a
  recursive formulation that looped infinitely at `n=1`; replaced with a bounded
  BFS shortest-path search before shipping.
- `check-if-point-is-reachable` (9A): shipped without an `oracle_python` — a safe
  brute-force simulator over the infinite move space isn't tractable; anchor +
  edge_inputs + cross-language check cover it instead (oracles are optional per
  the guide).
- `check-knight-tour-configuration` (9E): uses hand-constructed genuine knight's-
  tour grids (5x5/6x6/7x7, generated offline via backtracking) for real valid/
  invalid boundary cases, including a "valid moves but wrong starting cell" trap.
- `collect-coins-in-a-tree` and `the-number-of-beautiful-subsets` (9E): each ship
  an oracle algorithmically distinct from the optimal solution (multi-pair BFS
  Steiner-tree marking vs. leaf-peeling; full 2^n subset enumeration vs. the
  mod-k chain DP).
- `disconnect-path-in-a-binary-matrix-by-at-most-one-flip` and
  `rearranging-fruits` (9B): both use a true independent-algorithm oracle (BFS
  flip-and-check; Dijkstra over the swap graph) restricted to small inputs so
  they stay fast.
- One subagent (9C) accidentally wrote a throwaway `scratch_dump.json` to the
  repo root while inspecting the scrape; caught and deleted it before finishing —
  confirmed gone during the orchestrating session's sweep.

Redo slugs to fold into Wave 10: none.

### Wave 10 notes

All 5 chunks (10A–10E, 60 slugs) landed — zero DEFERs, zero quarantines, zero redo
carried forward. Chunk 10D dropped mid-run on the same transient "Connection closed"
API error seen in prior waves; it was resumed in place via a follow-up message (per
the Wave 5/8/9 pattern) rather than redispatched fresh, and finished its remaining
slugs cleanly on resume. `--check --only` across all 60 slugs was independently
re-verified by the orchestrating session (not just trusted from subagent reports)
before freezing — 0 quarantined. `--bundle` verified/rebuilt all 60, skipped 2,336
already-frozen (bundle now holds **2396** packs, up from 2336). `cargo test` (all
suites) and `npm run build` both green.

Notable subagent findings (informational, not redo):
- `modify-graph-edge-weights` (10D): `any_valid` judge (multiple edge-weight
  assignments can hit the target distance) with `no_anchor_ok: true` — the
  Dijkstra-based "raise one free edge at a time" reference differs numerically
  from LeetCode's own example weights where multiple valid assignments exist;
  validator adversarially tested (accepts reordered/`u,v`-swapped edges, rejects
  corrupted fixed/free weights, missing edges, out-of-range weights, false
  infeasibility, and fabricated assignments on a genuinely infeasible instance).
- `convert-an-array-into-a-2d-array-with-conditions` (10A): `any_valid` judge —
  validator manually verified against the reference output, two alternative valid
  regroupings, and rejected duplicate-in-row/too-many-rows/missing-element/
  extra-element outputs.
- `minimum-reverse-operations` (10A) and `difference-of-number-of-distinct-values-on-diagonals`,
  `count-of-integers` (10E): all `no_anchor_ok: true` — statement examples weren't
  cleanly parseable by the anchor parser; verified by hand against `edge_inputs`
  instead. `count-of-integers` keeps its digit-DP mod-reduced throughout so no
  BigInt is needed in JS.
- `greatest-common-divisor-traversal` (10E): union-find over an SPF-sieve of
  shared-prime "hub" nodes rather than pairwise GCD checks, to stay subquadratic.
- `minimum-cost-of-a-path-with-special-roads` (10C) and
  `maximum-number-of-fish-in-a-grid` (10C): each ship an oracle algorithmically
  distinct from the optimal solution (Floyd–Warshall vs. Dijkstra; flood-fill vs.
  union-find).
- `cousins-in-binary-tree-ii` (10B): `io_types: {params:["tree"], returns:"tree"}`
  — a supported tree→tree shape per the guide, not a deferred §5 shape.
- One subagent (10B), while authoring its own assigned 4 slugs, also edited the
  other 8 files in its 12-slug chunk to fix constraint-fidelity bugs it spotted
  (overlapping in time with the two other sub-workers editing those same files);
  a scope overstep, though confined to files within the same chunk. The
  orchestrating session independently re-ran `--check` on all 12 afterward and
  confirmed no corruption — all well-formed, 0 quarantined.
- One nested sub-worker (10E) ran read-only `git status`/`git diff` mid-task
  despite the hard "never run git, not even read-only" rule (restated after the
  Wave 6 finding); no state-changing git command was run, no tracked files were
  touched, and it self-reported and stopped. The rule is already explicit in the
  brief — repeated here as a reminder it still needs enforcing, not a new finding.

Redo slugs to fold into Wave 11: none.

### Wave 11 notes

All 5 chunks (11A–11E, 60 slugs) landed — zero DEFERs, zero quarantines, zero redo
carried forward. `--check --only` across all 60 slugs was independently re-verified
by the orchestrating session (not just trusted from subagent reports) before
freezing — 0 quarantined, 0 locked. `--bundle` verified/rebuilt all 60, skipped
2,396 already-frozen (bundle now holds **2456** packs, up from 2396). `cargo test`
(all suites) and `npm run build` both green.

Notable subagent findings (informational, not redo):
- `find-a-good-subset-of-the-matrix` and `neither-minimum-nor-maximum` (11A):
  legitimate multi-valid-answer problems where a deterministic reference pick can
  differ from LeetCode's own example answer — both set `"no_anchor_ok": true` with
  8 `edge_inputs` each plus a validated `any_valid` validator (manually confirmed to
  accept alternate valid answers and reject wrong ones).
- `maximum-sum-queries` (11A) uses `exact` (not `unordered`) since its output is one
  answer per query in query order.
- `double-a-number-represented-as-a-linked-list` (11E) and
  `insert-greatest-common-divisors-in-linked-list` (11D) use the supported
  `io_types: linked_list` node adapter, not a deferred cyclic/random-pointer shape.
- `cousins`-style false positive avoided: `number-of-employees-who-met-the-target`
  (11D) was confirmed to be the array-based problem, not the SQL-employees family.
- Bugs caught and fixed by subagents before their final green `--check` (not
  shipped wrong, per the "defer rather than ship wrong" rule):
  - `largest-element-in-an-array-after-merge-operations` (11D): a naive
    leftmost-greedy oracle disagreed with the anchor (merge order isn't confluent);
    replaced with an exhaustive recursive search over all valid merge orders.
  - `minimum-operations-to-make-the-integer-zero` (11B): a recursive split-simulation
    oracle free-ran without a termination bound past a popcount/`k` overshoot;
    fixed with an early infeasibility cutoff.
  - `minimum-absolute-difference-between-elements-with-constraint` (11E): the
    sliding window wasn't lagged by `x`, so `x=0` trivially compared an index
    against itself; fixed by lagging the window by `max(x,1)`.
  - `make-string-a-subsequence-using-cyclic-increments` (11E): a constraint-schema
    error (string-kind constraint with a non-numeric `value`) caught before
    freezing.
- One subagent (11B) ran a read-only `git status` mid-task despite the hard "never
  run git" rule; no state-changing git command was run, no tracked files were
  touched, and it self-reported. Repeated here as a reminder the rule still needs
  enforcing (same pattern seen in Wave 10's 10E finding), not a new/separate risk.

Redo slugs to fold into Wave 12: none.

### Wave 12 notes

All 5 subagents (12A–12E) hit the session API rate limit almost immediately after
dispatch (same pattern as Wave 3) — only 9 files landed on disk before they stopped,
none quarantined. Per ORCHESTRATION.md, trusted the disk over the agent reports:
swept `tools/packs/` for all 60 target slugs, found 9 present (4 from 12C, 5 from
12E), `--check --only` all 9.

One fix applied before freezing: `minimum-number-of-groups-to-create-a-valid-assignment`
(12E) had no parseable statement-example anchor (`--check` reported "no anchor").
The pack was otherwise complete — original prose, python+js solutions, oracle, 7
`edge_inputs` covering edge/boundary/trap cases. Added `"no_anchor_ok": true` (per
the authoring guide's rule for unparseable anchors) rather than discarding the pack;
re-`--check` passed clean.

Froze all 9: `--bundle` verified/rebuilt 9, skipped 2,456 already-frozen (bundle now
holds **2465** packs, up from 2456). `cargo test` (all suites) and `npm run build`
both green.

Redo slugs to fold into Wave 13's smallest chunks (51 total — all of 12A, 12B, 12D;
8 of 12C; 7 of 12E):

**12A (12, none landed):** `sorting-three-groups` `number-of-beautiful-integers-in-the-range`
`check-if-a-string-is-an-acronym-of-words` `determine-the-minimum-sum-of-a-k-avoiding-array`
`maximize-the-profit-as-the-salesman` `find-the-longest-equal-subarray` `furthest-point-from-origin`
`find-the-minimum-possible-sum-of-a-beautiful-array` `minimum-operations-to-form-subsequence-with-target-sum`
`maximize-value-of-function-in-a-ball-passing-game` `check-if-strings-can-be-made-equal-with-operations-i`
`check-if-strings-can-be-made-equal-with-operations-ii`

**12B (12, none landed):** `maximum-sum-of-almost-unique-subarray` `count-k-subsequences-of-a-string-with-maximum-beauty`
`count-symmetric-integers` `minimum-operations-to-make-a-special-number` `count-of-interesting-subarrays`
`minimum-edge-weight-equilibrium-queries-in-a-tree` `points-that-intersect-with-cars`
`determine-if-a-cell-is-reachable-at-a-given-time` `minimum-moves-to-spread-stones-over-grid`
`string-transformation` `minimum-right-shifts-to-sort-the-array` `minimum-array-length-after-pair-removals`

**12C (8 of 12; `beautiful-towers-ii` `count-valid-paths-in-a-tree` `minimum-operations-to-collect-elements`
`minimum-number-of-operations-to-make-array-empty` landed OK):** `count-pairs-of-points-with-distance-k`
`minimum-edge-reversals-so-every-node-is-reachable` `sum-of-values-at-indices-with-k-set-bits`
`happy-students` `maximum-number-of-alloys` `maximum-element-sum-of-a-complete-subset-of-indices`
`maximum-odd-binary-number` `beautiful-towers-i`

**12D (12, none landed):** `split-array-into-maximum-number-of-subarrays` `maximum-number-of-k-divisible-components`
`maximum-value-of-an-ordered-triplet-i` `maximum-value-of-an-ordered-triplet-ii`
`minimum-size-subarray-in-infinite-array` `count-visited-nodes-in-a-directed-graph`
`divisible-and-non-divisible-sums-difference` `minimum-processing-time`
`apply-operations-to-make-two-strings-equal` `apply-operations-on-array-to-maximize-sum-of-squares`
`last-visited-integers` `longest-unequal-adjacent-groups-subsequence-i`

**12E (7 of 12; `construct-product-matrix` `minimum-sum-of-mountain-triplets-i`
`minimum-sum-of-mountain-triplets-ii` `minimum-number-of-groups-to-create-a-valid-assignment`
`subarrays-distinct-element-sum-of-squares-i` landed OK):**
`longest-unequal-adjacent-groups-subsequence-ii` `count-of-sub-multisets-with-bounded-sum`
`find-indices-with-index-and-value-difference-i` `shortest-and-lexicographically-smallest-beautiful-string`
`find-indices-with-index-and-value-difference-ii` `minimum-changes-to-make-k-semi-palindromes`
`minimum-number-of-changes-to-make-binary-string-beautiful`

### Wave 13 notes

Wave 13 dispatched 111 slugs across 5 subagents (13A–13E): the wave's own 60-slug plan (12 per
chunk) plus all 51 Wave-12 redo slugs folded in (13A +11, 13B–13E +10 each), since all five
Wave-13 chunks started equal-sized. Two subagents (13B, 13D) self-reported failure on a transient
"Connection closed" API error mid-response; a third (13A) later confirmed the same failure mode
via its stale last-progress message ("Wave E2 complete... waiting on wave E1..."). Per
ORCHESTRATION.md, trusted the disk over every agent report — swept `tools/packs/` for all 111
target slugs.

82 of 111 landed on disk, 0 quarantined. `--check --only` across all 82 passed clean on the first
pass — no fixes needed. Breakdown by chunk:
- **13A**: 19/23 landed (dropped its last 4 slugs mid-run on the API error).
- **13B**: 0/22 landed (dropped before writing any file — its self-report of "failed" was accurate).
- **13C**: 22/22 landed (clean run; author flagged and fixed one internal correctness bug —
  `find-maximum-non-decreasing-array-length`'s first attempt used an unverified O(n log n)
  monotonic-deque optimization that differential testing caught as buggy; replaced with a verified
  O(n²) DP + independent brute-force oracle, complexity documented honestly).
- **13D**: 22/22 landed despite self-reporting failure on the same API error — its last message
  ("All four groups are done — 22/22 reported OK") turned out to be accurate, confirmed via
  `--check` rather than trusted blindly.
- **13E**: 19/22 landed (dropped its last 3 slugs mid-run on the same API error).

Froze all 82: `--bundle` verified/rebuilt 82, skipped 2,465 already-frozen (bundle now holds
**2547** packs, up from 2465). `cargo test` (all suites) and `npm run build` both green.

Redo slugs to fold into Wave 14 (29 total — 4 from 13A, all 22 of 13B, 3 from 13E):

**13A (4 of 23 missing):** `find-the-minimum-possible-sum-of-a-beautiful-array`
`minimum-operations-to-form-subsequence-with-target-sum`
`maximize-value-of-function-in-a-ball-passing-game`
`check-if-strings-can-be-made-equal-with-operations-i`

**13B (22 of 22 missing — entire chunk, including 10 slugs that were already Wave-12 redo carries
now failing a second time on the same transient error):**
`number-of-strings-which-can-be-rearranged-to-contain-substring` `maximum-spending-after-buying-items`
`maximum-strong-pair-xor-i` `high-access-employees` `minimum-operations-to-maximize-last-elements-in-arrays`
`maximum-strong-pair-xor-ii` `make-three-strings-equal` `separate-black-and-white-balls`
`maximum-xor-product` `find-building-where-alice-and-bob-can-meet` `find-words-containing-character`
`maximize-area-of-square-hole-in-grid` `check-if-strings-can-be-made-equal-with-operations-ii`
`maximum-sum-of-almost-unique-subarray` `count-k-subsequences-of-a-string-with-maximum-beauty`
`count-symmetric-integers` `minimum-operations-to-make-a-special-number` `count-of-interesting-subarrays`
`minimum-edge-weight-equilibrium-queries-in-a-tree` `points-that-intersect-with-cars`
`determine-if-a-cell-is-reachable-at-a-given-time` `minimum-moves-to-spread-stones-over-grid`

**13E (3 of 22 missing):** `minimum-cost-to-convert-string-i` `minimum-changes-to-make-k-semi-palindromes`
`minimum-number-of-changes-to-make-binary-string-beautiful`

### Wave 14 notes

Wave 14 dispatched 89 slugs across 5 subagents (14A–14E): the wave's own 60-slug plan (12 per
chunk) plus all 29 Wave-13 redo slugs folded in evenly (14A–14D +6 each, 14E +5), since all five
Wave-14 chunks started equal-sized. Two subagents (14B, 14D) were terminated mid-response by the
same transient "Connection closed" API error seen in prior waves. Per ORCHESTRATION.md, trusted
the disk over every agent report — swept `tools/packs/` for all 89 target slugs.

56 of 89 landed on disk, 0 quarantined. `--check --only` across all 56 passed clean on the first
pass — no fixes needed. Breakdown by chunk:
- **14A**: 18/18 landed (clean run).
- **14B**: 3/18 landed (dropped after its 3rd slug on the API error).
- **14C**: 18/18 landed (clean run).
- **14D**: 0/18 landed (dropped before writing any file).
- **14E**: 17/17 landed (clean run; one internal bug caught and fixed during authoring —
  `minimum-changes-to-make-k-semi-palindromes`'s `semipal_cost` helper returned `None` for
  length-1 substrings, crashing the DP; fixed by returning `float('inf')` so the DP naturally
  avoids length-1 pieces).

Froze all 56: `--bundle` verified/rebuilt 56, skipped 2,547 already-frozen (bundle now holds
**2603** packs, up from 2547). `cargo test` (all suites) and `npm run build` both green.

Redo slugs to fold into Wave 15 (33 total — 15 from 14B, all 18 of 14D):

**14B (15 of 18 missing):** `maximum-number-of-moves-to-kill-all-pawns` `find-a-safe-walk-through-a-grid`
`find-the-maximum-sequence-value-of-array` `length-of-the-longest-increasing-path`
`the-two-sneaky-numbers-of-digitville` `maximum-multiplication-score`
`minimum-number-of-valid-strings-to-form-target-i` `minimum-number-of-valid-strings-to-form-target-ii`
`report-spam-message` `maximum-strong-pair-xor-i` `high-access-employees`
`minimum-operations-to-maximize-last-elements-in-arrays` `maximum-strong-pair-xor-ii`
`make-three-strings-equal` `separate-black-and-white-balls`

**14D (18 of 18 missing — entire chunk):** `remove-methods-from-project`
`construct-2d-grid-matching-graph-layout` `sorted-gcd-pair-queries`
`construct-the-minimum-bitwise-array-i` `construct-the-minimum-bitwise-array-ii`
`find-maximum-removals-from-source-string` `find-the-number-of-possible-ways-for-an-event`
`find-x-sum-of-all-k-long-subarrays-i` `k-th-largest-perfect-subtree-size-in-binary-tree`
`count-the-number-of-winning-sequences` `find-x-sum-of-all-k-long-subarrays-ii`
`find-the-sequence-of-strings-appeared-on-the-screen`
`count-k-subsequences-of-a-string-with-maximum-beauty` `count-symmetric-integers`
`minimum-operations-to-make-a-special-number` `count-of-interesting-subarrays`
`minimum-edge-weight-equilibrium-queries-in-a-tree` `points-that-intersect-with-cars`

### Wave 15 notes

Wave 15 dispatched 93 slugs across 5 subagents (15A–15E): the wave's own 60-slug plan (12 per
chunk) plus all 33 Wave-14 redo slugs folded in (15A +7, 15B +7, 15C +7, 15D +6, 15E +6). All
five subagents hit the same "session limit" API cutoff seen in prior waves (reset 1:50am
Africa/Johannesburg) at various points in their run; 15B alone finished its full report before
any agent was cut off. Per ORCHESTRATION.md, trusted the disk over every agent report — swept
`tools/packs/` for all 93 target slugs.

45 of 93 landed on disk, 0 quarantined. `--check --only` across all 45 passed clean on the first
pass — no fixes needed. Breakdown by chunk:
- **15A**: 0/19 landed (dropped before writing any file).
- **15B**: 19/19 landed (clean run; two bugs caught and fixed during authoring —
  `find-the-maximum-number-of-fruits-collected`'s oracle DP didn't require the second and third
  children to actually terminate at the bottom-right corner, silently accepting invalid end
  states; and `maximum-strong-pair-xor-ii`'s trie "window empty" check read a `root.count` field
  that was never incremented, always reading as empty).
- **15C**: 0/19 landed (dropped before writing any file).
- **15D**: 14/18 landed (dropped 4 mid-run on the session-limit error).
- **15E**: 12/18 landed (dropped 6 mid-run on the session-limit error).

Froze all 45: `--bundle` verified/rebuilt 45, skipped 2,603 already-frozen (bundle now holds
**2648** packs, up from 2603). `cargo test` (all suites) and `npm run build` both green.

Redo slugs to fold into Wave 16 (48 total — all 19 of 15A, all 19 of 15C, 4 of 15D, 6 of 15E):

**15A (19 of 19 missing — entire chunk):** `find-minimum-time-to-reach-last-room-i`
`find-minimum-time-to-reach-last-room-ii` `count-number-of-balanced-permutations`
`smallest-divisible-digit-product-i` `maximum-frequency-of-an-element-after-performing-operations-i`
`maximum-frequency-of-an-element-after-performing-operations-ii` `smallest-divisible-digit-product-ii`
`adjacent-increasing-subarrays-detection-i` `adjacent-increasing-subarrays-detection-ii`
`sum-of-good-subsequences` `count-k-reducible-numbers-less-than-n` `make-array-elements-equal-to-zero`
`maximum-number-of-moves-to-kill-all-pawns` `find-a-safe-walk-through-a-grid`
`find-the-maximum-sequence-value-of-array` `length-of-the-longest-increasing-path`
`the-two-sneaky-numbers-of-digitville` `maximum-multiplication-score`
`minimum-number-of-valid-strings-to-form-target-i`

**15C (19 of 19 missing — entire chunk):** `identify-the-largest-outlier-in-an-array`
`maximize-the-number-of-target-nodes-after-connecting-trees-i`
`maximize-the-number-of-target-nodes-after-connecting-trees-ii`
`minimum-operations-to-make-array-values-equal-to-k` `minimum-time-to-break-locks-i`
`digit-operations-to-make-two-integers-equal` `count-connected-components-in-lcm-graph`
`transformed-array` `maximum-area-rectangle-with-point-constraints-i`
`maximum-subarray-sum-with-length-divisible-by-k` `maximum-area-rectangle-with-point-constraints-ii`
`button-with-longest-push-time` `separate-black-and-white-balls` `remove-methods-from-project`
`construct-2d-grid-matching-graph-layout` `sorted-gcd-pair-queries`
`construct-the-minimum-bitwise-array-i` `construct-the-minimum-bitwise-array-ii`
`find-maximum-removals-from-source-string`

**15D (4 of 18 missing):** `minimum-operations-to-make-character-frequencies-equal`
`smallest-substring-with-identical-characters-ii` `find-the-number-of-possible-ways-for-an-event`
`count-the-number-of-winning-sequences`

**15E (6 of 18 missing):** `count-k-subsequences-of-a-string-with-maximum-beauty`
`count-symmetric-integers` `minimum-operations-to-make-a-special-number`
`count-of-interesting-subarrays` `minimum-edge-weight-equilibrium-queries-in-a-tree`
`points-that-intersect-with-cars`

### Wave 16 notes

Wave 16 dispatched 108 slugs across 5 subagents (16A-16E): the wave's own 60-slug plan (12 per
chunk) plus all 48 Wave-15 redo slugs folded in (16A +10, 16B +10, 16C +10, 16D +9, 16E +9). All
four of 16B/16C/16D/16E hit the same transient "Connection closed mid-response" API error seen in
prior waves, at various points in their run; 16A alone finished cleanly (and, after an internal
mishap where it mistakenly tried to nest-fork a sub-group instead of authoring directly, caught
its own mistake and authored that group's files itself). Per ORCHESTRATION.md, trusted the disk
over every agent report — swept `tools/packs/` for all 108 target slugs.

37 of 108 landed on disk. `--check --only` across those 37 found 2 real bugs and quarantined them
(both deleted):
- `find-the-minimum-amount-of-time-to-brew-potions`: brute force disagreed with the authored
  solution on `[[1,5,2,4],[5,1,4,2]]` (110 vs 88).
- `maximize-the-minimum-game-score`: the optimal solution disagreed with the pack's own stated
  example `[[2,4],3]` (got 2, statement says 4).

Net 35 packs frozen. Breakdown by chunk:
- **16A**: 22/22 landed (full chunk; one real bug caught and fixed by the authoring agent itself
  before reporting — `longest-special-path` initially explored its tree as undirected rather than
  rooted at node 0, producing false paths through a parent to a sibling).
- **16B**: 0/22 landed (dropped before writing any file).
- **16C**: 2/22 landed, 1 quarantined post-check (net 1 kept) — dropped mid-run while mid-fix on a
  pack.
- **16D**: 0/21 landed (dropped before writing any file).
- **16E**: 13/21 landed, 1 quarantined post-check (net 12 kept) — dropped mid-run on the same
  error.

Froze the 35 clean packs (interim freeze): `--bundle` verified/rebuilt 35, skipped 2,648
already-frozen (bundle held **2683** packs at this point). `cargo test` (all suites) and
`npm run build` both green.

#### Wave 16 completion — retry-in-place instead of deferring to Wave 17

Per explicit user instruction, the remaining 73 slugs were **not** deferred to a "Wave 17" —
they were retried in place, in the same session, until 100% of Wave 16 landed. Policy for this
retry loop: on any drop (API error or otherwise), immediately re-sweep disk for exactly that
chunk's slugs and dispatch a fresh subagent for whatever was still missing; if a chunk dropped
2+ times at the same size, split it into smaller halves before retrying again.

The 73 remaining slugs were re-split into 5 fresh groups (redo-1..redo-5, ~14-15 slugs each) and
dispatched. Outcomes:
- **redo-1** (15 slugs): completed fully on the first attempt, 0 quarantined.
- **redo-2** (15 slugs, the old 16B tail + 16C head): dropped once (0/15 written), retried at
  full size, completed fully second attempt, 0 quarantined.
- **redo-3** (15 slugs, old 16C tail + 16D head): completed fully on the first attempt, 0
  quarantined — including a fix for the `maximize-the-minimum-game-score` bug quarantined in the
  first pass (root cause: the binary-search feasibility check wrongly assumed the walk must end
  at the last index).
- **redo-4** (14 slugs, old 16D middle): dropped twice at full size (0/14 written both times), so
  split into two 7-slug halves (4a, 4b); 4a completed on its first attempt, 4b likewise — both 0
  quarantined.
- **redo-5** (14 slugs, old 16D tail + all of 16E's missing set): completed fully on the first
  attempt, 0 quarantined — including a fix for the `find-the-minimum-amount-of-time-to-brew-potions`
  bug quarantined in the first pass (root cause: a wrong prefix-max recurrence; re-derived
  `S_j = S_{j-1} + max_i(P_{j-1}[i] - P_j[i-1])`, confirmed 110 is correct, not 88).

After all 5 redo groups (and their retries) reported back, an independent sweep + `--check` of
all 108 original Wave-16 target slugs (not just the 73 redone) confirmed **108/108 pass, 0
quarantined**. Froze the final 73 newly-verified packs: `--bundle` verified/rebuilt 73, skipped
2,683 already-frozen — bundle now holds **2756** packs (up from 2648 pre-Wave-16). `cargo test`
(all suites) and `npm run build` both green.

**Wave 16 final result: 108/108 slugs authored and frozen, 0 defers, 0 redo carried into Wave
17.**

### Wave 17 notes

Wave 16 carried 0 redo slugs forward, so Wave 17 dispatched exactly its own 60-slug plan (17A-17E,
12 slugs each) with nothing folded in. All 5 subagents completed cleanly on the first pass — no
API drops, no defers, no quarantines reported.

An independent sweep confirmed all 60 files present on disk, and `python tools/build_packs.py
--check --only <60 slugs>` verified **60/60 pass, 0 quarantined** before freezing.

Also found and removed 3 stray untracked scratch files at the repo root (`.txt`,
`scratch_problems.txt`, `scratch_all_bodies.txt`) left over from subagent research — they
contained verbatim scraped LeetCode problem text and were not gitignored, which is a legal risk
per CLAUDE.md's "never persist LeetCode text, not even in a private file" rule. Deleted before
freezing; none were referenced by any pack.

Froze: `--bundle` verified/rebuilt 60, skipped 2,756 already-frozen — bundle now holds **2816**
packs (up from 2756 pre-Wave-17). `cargo test` (all suites, including sandbox hardening) and
`npm run build` both green.

**Wave 17 final result: 60/60 slugs authored and frozen, 0 defers, 0 redo carried into Wave 18.**

### Wave 18 notes

Wave 17 carried 0 redo slugs forward, so Wave 18 dispatched exactly its own 60-slug plan (18A-18E,
12 slugs each) with nothing folded in. All 5 subagents completed cleanly on the first pass — no
API drops, no defers, no quarantines reported.

An independent sweep confirmed all 60 files present on disk, and `python tools/build_packs.py
--check --only <60 slugs>` verified **60/60 pass, 0 quarantined** before freezing.

Froze: `--bundle` verified/rebuilt 60, skipped 2,816 already-frozen — bundle now holds **2876**
packs (up from 2816 pre-Wave-18). `cargo test` (all suites, including sandbox hardening) and
`npm run build` both green.

**Wave 18 final result: 60/60 slugs authored and frozen, 0 defers, 0 redo carried into Wave 19.**

### Wave 19 notes

Wave 18 carried 0 redo slugs forward, so Wave 19 dispatched exactly its own 60-slug plan
(19A-19E, 12 slugs each) with nothing folded in.

- 19A: 12/12 OK, clean first pass.
- 19B: dropped 0/12 — the subagent hit a "Connection closed mid-response" API error before
  writing any files. All 12 slugs fold into the Wave 20 redo list.
- 19C: 12/12 OK, clean first pass.
- 19D: 12/12 OK, clean first pass (agent ran two read-only `git status` calls while
  investigating pre-existing repo state — no mutating git command, flagged for transparency
  by the agent itself; no action needed).
- 19E: 11/12 OK. `maximum-path-intersection-sum-in-a-grid` was left unauthored — the agent
  determined it is not a deferred/Tier-B shape (plain grid, no node/graph/interactive
  elements) but could not find an algorithm both correct and efficient enough for the
  m·n ≤ 5·10^5 constraint (ruled out a Cherry-Pickup-style synchronized DP via brute-force
  counterexample; the correct full-state DP is O(m·n²), too slow). Folds into the Wave 20
  redo list as a genuine hard-algorithm retry, not a defer.

An independent sweep confirmed 47/60 files present on disk (13 missing: the 12 from 19B plus
`maximum-path-intersection-sum-in-a-grid` from 19E). `python tools/build_packs.py --check
--only <47 slugs>` verified **47/47 pass, 0 quarantined** before freezing.

Froze: `--bundle` verified/rebuilt 47, skipped 2,876 already-frozen — bundle now holds
**2923** packs (up from 2876 pre-Wave-19). `cargo test` (all suites, including sandbox
hardening) and `npm run build` both green.

**Wave 19 final result: 47/60 slugs authored and frozen, 0 defers, 13 redo carried into Wave 20.**

Redo slugs to fold into Wave 20's smallest chunks: `mirror-frequency-distance`,
`integers-with-multiple-sum-of-two-cubes`, `minimum-increase-to-maximize-special-indices`,
`minimum-operations-to-achieve-at-least-k-peaks`, `traffic-signal-color`,
`count-digit-appearances`, `minimum-operations-to-transform-array-into-alternating-prime`,
`maximum-value-of-concatenated-binary-segments`, `find-the-degree-of-each-vertex`,
`angles-of-a-triangle`, `longest-balanced-substring-after-one-swap`,
`good-subsequence-queries`, `maximum-path-intersection-sum-in-a-grid`.

### Wave 20 notes — final authoring wave

The 13 Wave-19 redo slugs are all plain algorithmic problems (no design/class shape), so
they were folded entirely into 20A rather than split across the design chunks 20B–20D —
20A ran 25 slugs (12 original + 13 redo), 20B/20C/20D ran their planned 10/10/8 design-shaped
slugs unchanged.

- 20A: 24/25 OK, clean first pass. `maximum-path-intersection-sum-in-a-grid` was deferred
  again — the agent confirmed it is not a Tier-B/§5 shape (plain grid, no node/graph/
  interactive/random elements) but could not find an algorithm that is both correct and
  efficient enough for m·n ≤ 5·10^5: the two paths' shared cells aren't guaranteed to be
  visited at the same step count by each path, so the natural synchronized two-robot DP
  doesn't apply, and the only fully general DP found (keyed on column + both paths' row
  intervals) is Θ(m·n·m), too slow. This is the second consecutive wave a competent agent
  has failed to crack this slug — it should stay a permanent defer rather than get folded
  into a future wave again.
- 20B: 10/10 OK, clean first pass (all design-shaped). One correctness bug caught and fixed
  by the agent itself before reporting: `find-elements-in-a-contaminated-binary-tree`'s
  first draft used naive fixed `2i+1`/`2i+2` offsets on the trimmed level-order array, wrong
  whenever an internal `null` causes slot-skipping — rewritten as a proper queue-driven BFS
  reconstruction matching the harness's own tree deserializer, with a trap test added for
  the mismatch.
- 20C: 10/10 OK, clean first pass (all design-shaped). Notable per-pack care: `design-front-
  middle-back-queue`'s `popMiddle` tie-break (front half's last element when the two deque
  halves are equal size) was verified by hand-tracing the statement example against an
  initially-backwards draft before finalizing; `sequentially-ordinal-rank-tracker`'s two-heap
  "growing-k" technique was likewise hand-traced against the example before trusting it.
- 20D: 8/8 OK, clean first pass (all design-shaped, all judge type `design` with
  `no_anchor_ok: true` since design op-sequences can't anchor to statement examples).

An independent sweep confirmed 52/53 files present on disk (only the one confirmed defer
missing). `python tools/build_packs.py --check --only <52 slugs>` verified **52/52 pass, 0
quarantined** before freezing.

Froze: `--bundle` verified/rebuilt 52, skipped 2,923 already-frozen — bundle now holds
**2975** packs (up from 2923 pre-Wave-20). `cargo test` (all suites, including sandbox
hardening) and `npm run build` both green.

**Wave 20 final result: 52/53 slugs authored and frozen (40 planned + 12 of 13 Wave-19 redo
carries; the 13th, `maximum-path-intersection-sum-in-a-grid`, is now a confirmed permanent
defer), 0 quarantined, 0 redo carried forward.**

**This was the last authoring wave.** All planned waves (1–20) are now dispatched and frozen.
Remaining work per this file: the final defer-candidate confirmation wave (37 slugs, see
above) plus the one permanent defer from this wave. No further authoring waves are planned.

### Defer-confirmation wave notes — project complete

Ran as 2 Sonnet subagents (batch A: 19 slugs, batch B: 18 slugs) per the "Final wave"
section above. Batch B dropped mid-run on the same transient "Connection closed" API
error seen throughout this project; the disk sweep found 3 files had already landed
(`shuffle-the-array`, `shuffle-string`, `iterator-for-combination`, all `--check`-clean),
so it was resumed in place to finish the remaining 15 rather than redispatched fresh —
consistent with the resume-in-place pattern used in Waves 5/8. No slug was force-shipped;
every defer reason is keyed to a §5 category or an explicitly reasoned exception
(concurrency/threading — not in the §5 table but confirmed genuinely unexpressible).

Froze: `--bundle` verified/rebuilt 3, skipped 2,975 already-frozen — bundle now holds
**2978** packs. `cargo test` (all suites, including sandbox hardening) and `npm run build`
both green.

**Final tally: 2,978 packs frozen and verified (Python + JavaScript). 35 permanent defers**
(34 from this wave + `maximum-path-intersection-sum-in-a-grid` from Wave 20) remain
basic-mode-only, each with a recorded, category-keyed reason above — additive future
upgrades per CONTENT_PIPELINE.md §5, not planning gaps. **Pack authoring for the current
catalog (`.docs/my_questions.json`, 3,026 questions) is complete; no further waves are
planned.**

