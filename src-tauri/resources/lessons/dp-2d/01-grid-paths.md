---
id: 01-grid-paths
unit: dp-2d
subpattern: "Grid-path counting DP"
trigger_signals:
  - "You're counting the number of distinct paths (or ways) through a grid where movement is restricted to one or two fixed directions (e.g. right / down)."
  - "The problem hands you an m x n grid and asks \"how many ways,\" not \"what is the shortest/longest path.\""
  - "Each cell's answer depends only on a small, fixed set of neighboring cells — the ones you could have arrived from."
worked_example: unique-paths
diagram: 01-grid-paths.diagram.json
quiz: 01-grid-paths.quiz.json
practice:
  - longest-common-subsequence
  - target-sum
recap: []
follow_up:
  - "What if some cells were blocked obstacles — how does the recurrence change at a blocked cell?"
  - "Could you compute the same answer in O(n) space instead of O(m*n) by rolling one row at a time?"
---

## The one idea

Build a 2-D table `dp[r][c]` where each cell answers "how many ways can I
reach here?" using only the cells that could feed into it. For grid-path
counting, a cell's move set is tiny and fixed — you can only arrive from the
cell above or the cell to the left — so `dp[r][c] = dp[r-1][c] + dp[r][c-1]`.
Filling the table row by row turns an exponential branching search into one
pass over every cell.

## Why it beats the obvious approach

The brute-force answer recursively explores "go right, then count paths from
there; go down, then count paths from there," which revisits the same
sub-grid an exponential number of times — a naive recursion without
memoization redoes the same overlapping work at every shared cell.

```python
def unique_paths(m: int, n: int) -> int:
    dp = [[1] * n for _ in range(m)]        # first row/col: exactly one way
    for r in range(1, m):
        for c in range(1, n):
            dp[r][c] = dp[r - 1][c] + dp[r][c - 1]
    return dp[m - 1][n - 1]
```

Base case: the first row and first column each have exactly one path (keep
moving in the only direction available). Every other cell adds the two ways
to reach it — from above, from the left — because those are the only two
moves the grid allows. Filling row-major means both dependencies
(`dp[r-1][c]` and `dp[r][c-1]`) are always already computed by the time you
need them. That's O(m·n) time and O(m·n) space (or O(n) if you roll one row
array), versus exponential blind recursion.

## Reading the trigger

Ask: "does this problem hand me a grid (or something grid-shaped) and ask
*how many ways*, rather than *the shortest way*?" If each cell's count is
built from a small, fixed set of neighbors — the cell above, the cell to the
left, sometimes a diagonal — you're looking at grid-path counting DP:
allocate the table, fill it in an order that respects the dependencies, and
read the answer out of the last cell.
