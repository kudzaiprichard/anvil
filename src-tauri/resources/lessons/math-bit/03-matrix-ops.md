---
id: 03-matrix-ops
unit: math-bit
subpattern: "In-place matrix transforms"
trigger_signals:
  - "You must transform a 2D matrix — rotate, transpose, or spiral-traverse it — without allocating a second grid."
  - "The transform decomposes into a couple of simple index operations: transpose then reverse rows, or a boundary that shrinks inward on each pass."
  - "The board may be square (rotate) or rectangular (spiral), and correctness hinges on exact row/column bounds as you walk it."
worked_example: rotate-image
diagram: 03-matrix-ops.diagram.json
quiz: 03-matrix-ops.quiz.json
practice:
  - spiral-matrix
  - single-number
recap: []
follow_up:
  - "What if the matrix weren't square — how would in-place transpose need to change, and is a true in-place rotation even possible without extra memory?"
  - "What if you had to rotate by an arbitrary multiple of 90 degrees — could you just repeat the transpose-then-reverse building block?"
---

## The one idea

A 90-degree rotation is just two cheap, composable passes: **transpose**
(swap `matrix[i][j]` with `matrix[j][i]` for every `i < j`, flipping across
the main diagonal) followed by **reversing every row**. Neither pass needs a
second grid — each one only swaps values that are already sitting in the
matrix, so the whole rotation happens in O(1) extra space.

```python
def rotate(matrix: list[list[int]]) -> None:
    n = len(matrix)
    for i in range(n):                      # transpose in place
        for j in range(i + 1, n):
            matrix[i][j], matrix[j][i] = matrix[j][i], matrix[i][j]
    for row in matrix:                      # reverse each row
        row.reverse()
```

## Why it beats the obvious approach

The naive fix allocates a brand-new `n x n` grid and computes
`new_matrix[j][n-1-i] = matrix[i][j]` for every cell — correct, but it costs
O(n^2) extra space just to hold the rotated copy. Both approaches still touch
every cell once, so time stays O(n^2) either way; the win from
transpose-then-reverse is purely in **memory** — down to O(1) extra space,
because every operation is a swap or an in-place reversal on the original
matrix.

This "decompose into named, reusable index operations" habit generalizes:
spiral traversal is the same idea applied to a shrinking boundary — four
pointers (`top`, `bottom`, `left`, `right`) walk one edge each and then step
inward, instead of trying to compute a spiral index formula directly.

## Reading the trigger

Say it out loud: **"transform a grid in place, no extra grid allowed."** That
phrase — a 2D matrix plus an O(1)-extra-space constraint — is the signal to
break the operation into small index transforms (transpose + reverse for
rotation, boundary-shrinking four-pointer walk for spiral) rather than
building a second grid and copying values across.
