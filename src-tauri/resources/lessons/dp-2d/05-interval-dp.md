---
id: 05-interval-dp
unit: dp-2d
subpattern: "Interval DP (best split of a range)"
trigger_signals:
  - "You repeatedly split a contiguous range into two parts, and the cost or value of a split depends on the range being split."
  - "The best answer for a range is built from the best answers of two smaller sub-ranges that share a boundary, so dp is indexed by a left and a right end."
  - "The order of operations matters (which cut first, which balloon last), and trying all orders is factorial."
worked_example: minimum-cost-to-cut-a-stick
diagram: 05-interval-dp.diagram.json
quiz: 05-interval-dp.quiz.json
practice:
  - burst-balloons
  - minimum-score-triangulation-of-polygon
recap:
  - 01-grid-paths
  - 02-two-sequence
follow_up:
  - "Can you reconstruct which order of cuts achieves the minimum, not just its cost?"
  - "Why does a plain left-to-right sweep over i fail to fill this table, and what ordering fixes it?"
---

## The one idea

Define `dp[i][j]` as the best answer for the sub-range **between boundary `i`
and boundary `j`**. To solve a range, try every split point `k` strictly between
them: the range becomes two smaller ranges `(i, k)` and `(k, j)` whose answers
you already have, plus the cost of the split itself:

`dp[i][j] = min over k of (dp[i][k] + dp[k][j]) + cost(i, j)`

To make sure the smaller ranges are ready, **fill the table by increasing
interval length**, not by row. A range of length 2 depends only on ranges of
length 1, length 3 on lengths 1 and 2, and so on.

In the stick problem, a cut costs the current length of the piece you cut. Add
the two ends as sentinel boundaries (`0` and `n`) and sort the cuts. Then
`dp[i][j]` is the cheapest way to make every cut that lies strictly between
`pts[i]` and `pts[j]`, and the cost of the *first* cut in that piece is its
whole length `pts[j] - pts[i]`.

## Why it beats the obvious approach

Brute force tries every order of the m cuts: m! orders, and each order costs
O(m) to price. The repeated work is huge. Once you decide that cut `k` is made
first in a piece, the left part and the right part are completely independent,
and the same sub-pieces (say, "the piece from position 2 to 7") appear in many
different orders. Memoising them by their two boundaries collapses m! orders
into about m² distinct sub-ranges.

```python
def min_cost(n: int, cuts: list[int]) -> int:
    pts = [0] + sorted(cuts) + [n]                 # sentinel boundaries, in order
    m = len(pts)
    dp = [[0] * m for _ in range(m)]               # dp[i][j]: cheapest cuts strictly between
    for length in range(2, m):                     # interval length, smallest first
        for i in range(m - length):
            j = i + length
            best = min(dp[i][k] + dp[k][j] for k in range(i + 1, j))   # try each first cut k
            dp[i][j] = best + pts[j] - pts[i]      # that first cut costs the piece length
    return dp[0][m - 1]
```

**Time:** there are O(m²) pairs `(i, j)` and each tries up to m split points
for O(1) each, so O(m³) where m = number of cuts + 2. **Space:** the table
holds m² integers, so O(m²). There is no recursion in this bottom-up version,
so no stack cost; a top-down memoised version would add O(m) recursion depth.

Note what the loop order does. Adjacent boundaries (`j = i + 1`) have no cut
between them, so those cells stay at `0`, the base case. Because `k` lies
strictly inside `(i, j)`, both `dp[i][k]` and `dp[k][j]` are shorter
intervals, already computed by the time we reach length `j - i`.

## The "last operation" reframing

The deepest trick of this pattern is choosing what `k` *means*. In the stick
problem `k` is the **first** cut: it costs the full length of the piece, and
afterwards the two halves are independent. In the balloon-bursting problem the
cost of bursting a balloon depends on its neighbours at that time, which change
as others burst, so thinking about the *first* burst entangles the two sides.
Instead let `k` be the **last** balloon burst in the range: then its neighbours
are the fixed boundaries `i` and `j`, and the two sides are independent again.
For polygon triangulation, `k` is the apex of the triangle that sits on edge
`(i, j)`. The skill is picking the split so the sub-ranges stop interacting.

## Reading the trigger

"Minimum cost / maximum score of cutting, merging or removing in some order"
over a sequence, with n up to a few hundred (a cubic bound). Contrast with
two-sequence DP (`02`), where `dp[i][j]` indexes a prefix of each of two inputs
and moves one step at a time, with no inner loop over split points. And contrast
with greedy: if a locally cheapest cut were always right, the problem would be
a sort; here the order genuinely changes later costs.

## Saying it in the interview

"Trying every order of cuts is m! and repeats the same sub-pieces constantly.
Once I fix the first cut in a piece, the two halves are independent, so I define
`dp[i][j]` as the cheapest way to finish all cuts strictly inside boundaries `i`
and `j`, after adding `0` and `n` as sentinels and sorting. The recurrence tries
every cut `k` in between and adds the piece length. I fill by increasing
interval length so sub-ranges are ready. Time O(m³), space O(m²)." Edge cases:
a single cut costs exactly the stick length, and cuts given out of order must be
sorted first. Pitfalls: iterating `i` then `j` in plain nested order so a
dependency is still empty, and forgetting the sentinels so the outermost piece
has no length to charge.
