---
id: 01-complexity-classes
unit: big-o
subpattern: "Counting operations: O(1) to O(n²)"
trigger_signals:
  - "You're comparing two approaches to the same problem and need to know which one will actually finish in time."
  - "You see a loop nested inside another loop over the same collection — that's the tell for quadratic work."
  - "A problem's constraints hint at the complexity you need (n up to 10^5 rules out O(n^2))."
worked_example: two-sum
diagram: 01-complexity-classes.diagram.json
quiz: 01-complexity-classes.quiz.json
practice:
  - contains-duplicate
  - valid-anagram
recap: []
follow_up:
  - "Given an approach that's O(n log n) because it sorts first, could you drop the sort and reach O(n) with a hash map instead?"
  - "If the input were 10x larger, would an O(n^2) approach still finish in a reasonable time?"
---

## The one idea

Big-O answers one question: **as the input grows, how does the work grow?**
Not wall-clock seconds — the *shape* of the growth curve. You find that shape
by counting the operations whose number depends on `n`, keeping only the
fastest-growing term, and throwing away constants.

```python
def has_pair_brute(nums: list[int], target: int) -> bool:
    for i in range(len(nums)):             # n iterations
        for j in range(i + 1, len(nums)):  # up to n more, each time
            if nums[i] + nums[j] == target:
                return True
    return False
```

The outer loop runs `n` times; for each of those, the inner loop runs up to
`n` more times. That's `n * n` comparisons in the worst case — **O(n²)**,
quadratic. Compare it to Two Sum's one-pass hash-map version: a single loop of
`n` steps, each doing an O(1) dict lookup, gives **O(n)** — linear. Same
question, same answer, but the growth curves diverge fast: at `n = 10,000` the
quadratic version does on the order of 100,000,000 comparisons; the linear one
does about 10,000.

## The ladder of growth classes

You'll meet the same handful of shapes again and again, in increasing order of
how fast they grow:

- **O(1)** — constant: a dict/array lookup by key or index, `x + y`.
- **O(log n)** — logarithmic: halving the search space each step (binary search).
- **O(n)** — linear: one pass over the input.
- **O(n log n)** — linearithmic: sort the input, or divide-and-conquer with a
  linear merge (`sorted(nums)`, mergesort).
- **O(n²)** — quadratic: a loop nested inside a loop over roughly the same
  input, like the brute-force pair search above.

Nesting is the tell in the code itself: a loop inside a loop over the same `n`
elements multiplies the counts, `n * n`. A single loop that does O(1) work per
element stays linear, even when that "O(1) work" is a dict operation rather
than plain arithmetic.

## Reading the trigger

Before writing code, count out loud: how many times does the innermost line
run, as a function of `n`? "One loop, O(1) work each" → O(n). "A loop inside a
loop over the same collection" → O(n²), and that's your cue to ask whether a
hash map, sorting, or a smarter pass can collapse it. Big-O isn't graded after
the fact — it's a question you ask *before* you commit to an approach.
