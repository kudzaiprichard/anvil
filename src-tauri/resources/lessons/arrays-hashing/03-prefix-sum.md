---
id: 03-prefix-sum
unit: arrays-hashing
subpattern: "Prefix sums for O(1) range queries"
trigger_signals:
  - "You'll be asked the same kind of \"combine everything except / before / up to index i\" question many times over the same array."
  - "The naive per-query answer requires rescanning a range, giving O(n) per query and O(n^2) (or worse) overall."
  - "The array doesn't change between queries — a precompute-once, answer-many-times shape."
worked_example: product-of-array-except-self
diagram: 03-prefix-sum.diagram.json
quiz: 03-prefix-sum.quiz.json
practice:
  - range-sum-query-immutable
  - two-sum
recap:
  - 01-hashmap-lookup
follow_up:
  - "What if the array *could* be updated between queries — would a plain prefix array still give O(1) updates?"
  - "What if the combining operation were multiplication with zeros allowed — how does that break the pure prefix-product trick?"
---

## The one idea

If a per-index answer only depends on "everything before `i`" combined with
"everything after `i`," you don't need to rescan the array for each index.
Precompute a running total from the left once, a running total from the
right once, and then every index's answer is just those two precomputed
numbers combined — O(1) per index after an O(n) setup.

## Why it beats the obvious approach

The brute-force answer to *"for each index, combine every other element"* is,
for each `i`, to loop over the whole array skipping `i` — O(n) work per
index, O(n²) overall. Nearly all of that work is repeated: the product of
elements `0..i-1` for index `i` is almost the same computation as for index
`i+1`. A prefix array remembers that running result instead of recomputing it:

```python
def product_except_self(nums: list[int]) -> list[int]:
    n = len(nums)
    prefix = [1] * n          # prefix[i] = product of nums[0..i-1]
    for i in range(1, n):
        prefix[i] = prefix[i - 1] * nums[i - 1]

    suffix = 1                 # running product of nums[i+1..n-1]
    result = prefix
    for i in range(n - 1, -1, -1):
        result[i] *= suffix    # combine "everything before" and "everything after"
        suffix *= nums[i]
    return result
```

Two passes, each O(n), no division needed (which matters — division breaks
when a zero is in the array). The pattern generalizes past products: prefix
*sums* answer "sum of any range `[l, r]`" in O(1) after one O(n) precompute,
which is exactly what powers repeated range-sum queries. The shared idea is
always the same — trade one O(n) precompute for turning every future query
into O(1), instead of paying O(n) per query.

## Reading the trigger

Ask: *"will I be asked this same kind of combine-a-range question more than
once against an array that isn't changing?"* If yes, and the naive per-query
cost is a rescan, that's the signal to precompute prefix (and often suffix)
running totals up front. The tell in the wording is usually "except self,"
"range sum," or "for every index" — anything that smells like O(n) work
repeated for every one of n positions is a candidate to become O(n) total
instead of O(n²).
