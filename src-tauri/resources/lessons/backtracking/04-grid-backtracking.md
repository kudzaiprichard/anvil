---
id: 04-grid-backtracking
unit: backtracking
subpattern: "Backtracking on a grid (mark, explore, unmark)"
trigger_signals:
  - "You walk through grid cells, each cell may be used at most once per route, and you want the best or every such route."
  - "The route can start from any cell, and a different route may need to reuse a cell that another route already used."
  - "The grid is small (a few dozen cells) and no greedy or DP shortcut exists, because the value of a cell depends on which cells you have already consumed."
worked_example: path-with-maximum-gold
diagram: 04-grid-backtracking.diagram.json
quiz: 04-grid-backtracking.quiz.json
practice:
  - unique-paths-iii
recap:
  - 01-subsets
  - 03-permutations
follow_up:
  - "What if the grid were 30 x 30 with many gold cells — would this still be feasible, and what would you ask the interviewer about the constraints?"
  - "How would you also return the actual sequence of cells on the best route, not only its total?"
---

## The one idea

Run a depth-first search from every possible starting cell, but treat each
route as a **private** decision path. Before stepping into a neighbour, you
**mark** the current cell as used. You recurse. When the recursion returns, you
**unmark** it, putting the grid back exactly as you found it. The unmark is
what makes this backtracking rather than flood fill: it lets the *next* route,
starting from anywhere, use that cell again.

In the gold-collecting problem you may begin at any cell holding gold, step to
a neighbouring cell with gold, never revisit a cell on the same route, and
want the largest total. The invariant: at any moment, the cells marked used are
exactly the cells on the current route.

## Why it beats the obvious approach

The naive way to be correct is to carry a list of visited cells down the
recursion, copying it at every step, or to compare against the whole route
each time you consider a neighbour. That costs O(L) extra per step, where L is
the route length. Marking the cell in place makes "have I used this?" an O(1)
check and the undo an O(1) assignment.

```python
def get_maximum_gold(grid: list[list[int]]) -> int:
    rows, cols = len(grid), len(grid[0])

    def dfs(r: int, c: int) -> int:
        gold = grid[r][c]
        grid[r][c] = 0                       # MARK: this cell is on the current route
        best = 0
        for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nr, nc = r + dr, c + dc
            if 0 <= nr < rows and 0 <= nc < cols and grid[nr][nc] > 0:
                best = max(best, dfs(nr, nc))    # EXPLORE
        grid[r][c] = gold                    # UNMARK: restore for other routes
        return gold + best

    return max((dfs(r, c) for r in range(rows) for c in range(cols) if grid[r][c] > 0), default=0)
```

Compare with the flood fill in `graphs/01-grid-dfs`: there, a cell stays
visited forever, because you only want to know *which* cells a region contains
and each needs visiting once. Here, leaving cells marked would wrongly forbid a
second route from passing through them, and the maximum would be missed.

**Time:** a route starts at one of the cells, takes its first step in up to 4
ways and each later step in at most 3 (you cannot go back to the cell you came
from, since it is marked). A route of length L therefore costs up to 3^L, so
the total is O(cells · 3^L), where L is bounded by the number of gold cells.
Exponential in the worst case, which is why such problems keep grids small.
**Space:** O(L) for the recursion stack, one frame per cell on the current
route, plus O(1) extra since the marks live in the input grid.

## Reading the trigger

Look for "no cell more than once on a route" combined with "from any cell" or
"all such routes", and a small grid. That combination says exhaustive search
with undo. Contrast it with:

- **Flood fill / island counting** (graphs): each cell is visited once overall,
  never unmarked.
- **BFS shortest path**: you want the fewest steps, not the richest or the
  count of all routes, and a global visited set is correct.
- **DP on a grid**: only when moves are restricted (right/down), so the state
  is just `(row, col)`. Free movement in four directions with no revisits
  depends on the whole set of used cells, which DP cannot compress.

## Saying it in the interview

"Free four-direction movement with no cell reused means the best route depends
on the set of cells already taken, so DP does not apply and I'll search
exhaustively. I try every gold cell as a start and run a DFS: I save the cell's
value, zero it to mark it used, recurse into open neighbours, take the best
child result, then put the value back before returning. Time is O(cells · 3^L)
because after the first step there are at most three onward choices, and space
is O(L) for the recursion, with no extra visited structure." Name the edge
cases: an all-zero grid returns 0, a single cell, and a snake-shaped grid where
L equals the number of cells. The classic bug is forgetting the restore line,
which silently corrupts later starts and produces wrong answers that depend on
the order you iterate; the other is returning only after the loop and
mishandling the zero marker when a cell's real value is 0.
