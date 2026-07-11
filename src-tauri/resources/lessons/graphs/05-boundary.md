---
id: 05-boundary
unit: graphs
subpattern: "Boundary & reverse traversal"
trigger_signals:
  - "The question asks which cells can reach MULTIPLE targets/edges of the grid (e.g. both the top-left ocean and the bottom-right ocean) — checking that per cell by running a fresh traversal from every cell would be quadratic."
  - "The natural forward rule (\"water flows from a cell to a lower-or-equal neighbor\") is cheap to check but expensive to run FROM every cell — reversing it (\"walk uphill from the boundary\") is the cheap direction."
  - "You need the intersection (or union) of two or more independent reachability sets, each computed by its own traversal."
worked_example: pacific-atlantic-water-flow
diagram: 05-boundary.diagram.json
quiz: 05-boundary.quiz.json
practice:
  - number-of-islands
  - surrounded-regions
recap:
  - 01-grid-dfs
follow_up:
  - "What if there were three or more targets instead of two — how would the intersection step generalize?"
  - "What if you only needed to know whether at least one cell qualifies, not the full list — could you stop either traversal early?"
---

## The one idea

Running a DFS/BFS from every single cell to check "can this reach the
boundary?" is O((R × C)²) — quadratic in the grid size. The trick is to run
the traversal **backward**, once, starting *from* the boundary and walking
to cells that could have flowed into it. Do that once per target boundary
(Pacific edge, Atlantic edge), and a cell that both traversals reach is a
cell that can reach both oceans.

```python
def pacific_atlantic(heights: list[list[int]]) -> list[list[int]]:
    rows, cols = len(heights), len(heights[0])

    def reachable(starts: list[tuple[int, int]]) -> set[tuple[int, int]]:
        seen = set(starts)
        stack = list(starts)
        while stack:
            r, c = stack.pop()
            for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nr, nc = r + dr, c + dc
                if (
                    0 <= nr < rows and 0 <= nc < cols
                    and (nr, nc) not in seen
                    and heights[nr][nc] >= heights[r][c]   # walking uphill, reversed
                ):
                    seen.add((nr, nc))
                    stack.append((nr, nc))
        return seen

    pacific_starts = [(r, 0) for r in range(rows)] + [(0, c) for c in range(cols)]
    atlantic_starts = [(r, cols - 1) for r in range(rows)] + [(rows - 1, c) for c in range(cols)]
    pacific = reachable(pacific_starts)
    atlantic = reachable(atlantic_starts)
    return [list(cell) for cell in pacific & atlantic]
```

## Why it beats the obvious approach

Forward water-flow ("can cell X reach the Pacific?") means starting a fresh
traversal at every one of R × C cells — O((R × C)²). Reversing the edge
direction is legal here because the flow rule is symmetric under reversal:
"X can flow to Y" (`heights[Y] <= heights[X]`) is the same relation as "Y is
reachable by walking uphill from X" (`heights[X] >= heights[Y]`, checked
from Y's side). Two boundary-seeded traversals — one multi-source flood per
ocean — cover the whole grid in O(R × C) each, and the answer is just their
**set intersection**.

## Reading the trigger

Say it out loud: **"which cells can reach every one of several targets, and
running it forward from each cell would be too slow."** Whenever the forward
rule is cheap to check but expensive to run *from* everywhere, flip it:
start multi-source from the target(s) instead, and combine the resulting
reachable sets with a union or intersection.
