---
id: 02-grid-bfs
unit: graphs
subpattern: "Multi-source BFS on a grid"
trigger_signals:
  - "The question asks for the minimum number of steps/minutes/hops for something to spread or reach across a grid — \"shortest\" on an unweighted grid is BFS's signature."
  - "There are multiple starting points at once (several rotten oranges, several fires) that all spread simultaneously, not one single source."
  - "You need to process the grid in rounds (\"after one minute...\") rather than fully exploring one region before moving to the next."
worked_example: rotting-oranges
diagram: 02-grid-bfs.diagram.json
quiz: 02-grid-bfs.quiz.json
practice:
  - number-of-islands
  - pacific-atlantic-water-flow
recap:
  - 01-grid-dfs
follow_up:
  - "What if a fresh orange were unreachable from any rotten one — how does the BFS tell you that, and what should the function return?"
  - "What if oranges could also rot diagonally — which part of the neighbor-generation code changes?"
---

## The one idea

The previous lesson flooded outward from **one** starting cell with DFS.
Multi-source BFS starts from **every** rotten cell at once, pushing them all
into a queue before the first step — so processing the queue level by level
naturally counts "minutes," because everything popped in the same wave
rotted at the same time.

```python
from collections import deque

def oranges_rotting(grid: list[list[int]]) -> int:
    rows, cols = len(grid), len(grid[0])
    queue = deque()
    fresh = 0
    for r in range(rows):
        for c in range(cols):
            if grid[r][c] == 2:
                queue.append((r, c))       # every rotten orange is a source
            elif grid[r][c] == 1:
                fresh += 1

    minutes = 0
    while queue and fresh:
        minutes += 1
        for _ in range(len(queue)):        # snapshot: this minute's wave
            r, c = queue.popleft()
            for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nr, nc = r + dr, c + dc
                if 0 <= nr < rows and 0 <= nc < cols and grid[nr][nc] == 1:
                    grid[nr][nc] = 2
                    fresh -= 1
                    queue.append((nr, nc))
    return minutes if fresh == 0 else -1
```

## Why it beats the obvious approach

A single-source flood fill measures distance *from one point*. Run it from
one rotten orange at a time and you'd have to combine those distances
correctly across every source to know when the *last* fresh orange rots —
distance-from-nearest-source isn't something a one-source DFS gives you at
all. Seeding the queue with every rotten cell up front sidesteps that
entirely: it's the same `level_size = len(queue)` snapshot from level-order
tree traversal, transplanted onto a grid. Each wave popped is exactly the set
of oranges that rot in the same minute, so `minutes` counts itself correctly
for free. Every cell enters and leaves the queue once — O(rows × cols) time.

## Reading the trigger

Say it out loud: **"several starting points spread outward together, and I
need the round where the last one is reached."** Multiple simultaneous
sources plus a shortest-spread-time question on an unweighted grid means:
seed the queue with every source before you start popping, and let the
wave structure count the rounds.
