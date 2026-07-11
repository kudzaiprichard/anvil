---
id: 04-weighted-grid
unit: advanced-graphs
subpattern: "Dijkstra on a weighted grid"
trigger_signals:
  - "A 2D grid where moving into a cell costs something that depends on the cell itself (elevation, time, risk) — not a flat cost of 1 per step."
  - "You need the minimum possible *bottleneck* along a path (the worst single cell you're forced to cross), not a sum of step costs."
  - "Neighbors are still the usual up/down/left/right, but BFS's uniform-edge-cost assumption no longer holds."
worked_example: swim-in-rising-water
diagram: 04-weighted-grid.diagram.json
quiz: 04-weighted-grid.quiz.json
practice:
  - network-delay-time
  - min-cost-to-connect-all-points
recap:
  - 01-dijkstra
follow_up:
  - "What if you needed the actual path, not just the minimum bottleneck value — what would you store alongside each cell to reconstruct it?"
  - "What if the cost combined additively instead of by worst-cell (e.g. total energy spent) — does swapping `max` for `+` in the push still give a correct Dijkstra?"
---

## The one idea

Treat every grid cell as a graph node with up to 4 weighted edges to its
neighbors, and run the same min-heap frontier expansion as Dijkstra —
except the "distance" being minimized is the path's **bottleneck** (the
worst cell value seen along the way), not a running sum.

```python
import heapq

def swim_in_water(grid: list[list[int]]) -> int:
    n = len(grid)
    visited = [[False] * n for _ in range(n)]
    heap = [(grid[0][0], 0, 0)]        # (max elevation on path so far, r, c)
    while heap:
        t, r, c = heapq.heappop(heap)
        if r == n - 1 and c == n - 1:
            return t
        if visited[r][c]:
            continue
        visited[r][c] = True
        for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nr, nc = r + dr, c + dc
            if 0 <= nr < n and 0 <= nc < n and not visited[nr][nc]:
                heapq.heappush(heap, (max(t, grid[nr][nc]), nr, nc))
    return -1
```

## Why it beats the obvious approach

Binary search over the time threshold, paired with a BFS/DFS "can we reach
the end using only cells ≤ T" check, also works, but costs O(n^2 log n) and
needs two nested pieces of logic. The heap-frontier version finds the
answer directly, in one pass: because the heap always pops the path with
the smallest bottleneck so far, the first time the destination is popped,
that value is provably the minimum possible bottleneck across *any* path —
the exact same "finalize on first pop" argument as plain Dijkstra, just with
`max` standing in for `+` as the way a new edge combines with the running
path cost.

Plain BFS is wrong here because it counts hops, not elevation — a
short-looking path could still force you through one very high cell partway
along it, and BFS has no way to notice that.

## Reading the trigger

Say it out loud: **"weighted grid, and the path cost isn't the hop
count — it's a value baked into each cell (elevation, time, risk)."**
Reach for a min-heap of `(cost-so-far, row, col)` and pop-to-finalize, same
as any other Dijkstra. If the way costs combine is "worst cell visited"
instead of "sum of edges," swap `+` for `max` in the push — the pop-order
argument that makes it correct doesn't change.
