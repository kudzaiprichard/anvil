---
id: 05-constraint-placement
unit: backtracking
subpattern: "Constraint placement (N-Queens, Sudoku)"
trigger_signals:
  - "You must place one item per row, cell or slot so that no two placed items conflict, and you want a valid arrangement (or all of them)."
  - "Conflicts follow clear rules (same column, same diagonal, same row/box) that you can track in O(1) with sets or arrays."
  - "A partial placement can be proven hopeless early, so cutting a branch the moment it breaks a rule saves almost all the work."
worked_example: n-queens
diagram: 05-constraint-placement.diagram.json
quiz: 05-constraint-placement.quiz.json
practice:
  - n-queens-ii
  - sudoku-solver
recap:
  - 01-subsets
  - 03-permutations
  - 04-grid-backtracking
follow_up:
  - "How would you speed up the conflict check further using bitmasks instead of sets?"
  - "For Sudoku, how would you choose which empty cell to fill next to shrink the search even more?"
---

## The one idea

Build the answer **one decision at a time**, and before you commit to a
decision, check it against the rules you already enforced. If it conflicts,
skip it — and with it the entire subtree of arrangements that would have grown
from it. If it is safe, record it, recurse to the next slot, and **undo** it on
return so the next candidate starts clean.

For N-Queens the slots are rows: exactly one queen goes in each row, so you
only choose a *column*. A placement at `(r, c)` conflicts with an earlier queen
if she shares the column `c`, the diagonal `r - c` (constant along a
down-right line), or the anti-diagonal `r + c` (constant along a down-left
line). Keep three sets and each conflict test is O(1).

## Why it beats the obvious approach

Brute force places n queens anywhere on the n x n board and validates
afterwards: choosing n squares of n² is astronomically large, on the
order of C(n², n) boards. Even restricting to one queen per row and trying every column
combination gives n^n boards, each validated in O(n²) or so. The waste is that
a board with two queens attacking in the first two rows is dead, yet brute
force still builds all of its completions.

```python
def solve_n_queens(n: int) -> list[list[str]]:
    cols, diag, anti = set(), set(), set()   # columns, r-c, r+c already attacked
    placed = [0] * n                         # placed[r] = column of the queen in row r
    out: list[list[str]] = []

    def place(r: int) -> None:
        if r == n:                           # every row has a queen: record the board
            out.append(["." * c + "Q" + "." * (n - c - 1) for c in placed])
            return
        for c in range(n):
            if c in cols or (r - c) in diag or (r + c) in anti:
                continue                     # PRUNE: conflicts, skip the whole subtree
            cols.add(c); diag.add(r - c); anti.add(r + c)      # place
            placed[r] = c
            place(r + 1)                                       # explore
            cols.remove(c); diag.remove(r - c); anti.remove(r + c)   # undo
    place(0)
    return out
```

**Time:** with one queen per row, row 0 has n options, row 1 at most n - 1
safe columns (the column of the first queen is gone, and diagonals remove more),
and so on, so the tree has at most n! leaves: O(n!). That bound is loose.
Pruning cuts branches long before they reach full depth, so the real number of
nodes explored is far smaller and n = 8 or 9 runs instantly; it is still
exponential, though. Each placement also costs O(1) thanks to the sets, plus
O(n²) to write out each solution board. **Space:** the recursion goes n rows
deep and the three sets hold at most n entries each, so O(n) beyond the output.

The same skeleton solves Sudoku. The slot is an empty cell, the choices are the
digits 1 to 9, and the three conflict sets become one set per row, per column
and per 3 x 3 box. The cell index `box = (r // 3) * 3 + c // 3` finds the right
box set. You place a digit, recurse to the next empty cell, and if nothing
fits further on you return and undo, which is exactly the loop above.

## Reading the trigger

"Place items so that none attack / repeat / conflict" with "return a valid
configuration" or "count the configurations" and small n or a fixed 9 x 9
board. The tell is that a *partial* answer can already be wrong. Contrast it
with subsets or permutations (`01`, `03`), where every partial choice is valid
and nothing is pruned except duplicates. Contrast it too with DP: DP needs
overlapping subproblems with a compact state, whereas "which exact cells hold
queens" is the whole state here, so there is nothing to reuse.

## Saying it in the interview

"Brute force over all placements is exponential and mostly checks boards that
were already doomed. I'll place one queen per row, so rows never conflict and
I only choose a column. Columns, `r - c` diagonals and `r + c` anti-diagonals
go in three sets, giving O(1) conflict checks, and I prune the moment a
candidate is unsafe. After recursing I remove it again. Worst-case time is
O(n!) with heavy pruning in practice, and space is O(n) for the recursion
depth and the sets." Edge cases: n = 1 gives one board, n = 2 and 3 have no
solutions, so you return an empty list. Pitfalls: forgetting to undo one of the
three sets, which leaves phantom attacks, and appending the shared `placed`
list instead of building a fresh board, so every stored solution changes later.
