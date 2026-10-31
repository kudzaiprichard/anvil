---
id: 06-bfs-shortest-path
unit: graphs
subpattern: "BFS shortest path in an unweighted graph"
trigger_signals:
  - "The question asks for the fewest moves, steps or transformations between a start and a target, and every move costs the same."
  - "The graph is implicit: you are never handed edges, you generate neighbours from a state (a grid cell, a lock combination, a gene string)."
  - "You need the length of the shortest route, not every route — so the first arrival at the target is the answer."
worked_example: shortest-path-in-binary-matrix
diagram: 06-bfs-shortest-path.diagram.json
quiz: 06-bfs-shortest-path.quiz.json
practice:
  - open-the-lock
  - minimum-genetic-mutation
recap:
  - 01-grid-dfs
  - 02-grid-bfs
follow_up:
  - "What if every step had a different cost — which algorithm replaces the plain queue, and what does it add?"
  - "If both the start and the target are known, how could searching from both ends at once shrink the explored area?"
---

## The one idea

Breadth-first search visits nodes in **rings of equal distance**: everything one
step from the start, then everything two steps away, then three. Because a
queue is first-in-first-out, a node is never taken off the queue before every
closer node has been. So when every edge costs exactly 1, **the first time you
reach a node is guaranteed to be by a shortest route** — you never need to
compare alternatives.

In the binary-matrix problem you walk from the top-left to the bottom-right
through open cells, moving in any of 8 directions, and count cells on the path.
Each ring of the BFS is one more cell on the path, so the ring number at which
the goal appears *is* the answer.

## Why it beats the obvious approach

The obvious approach is a DFS that tries every route and keeps the shortest
one. With 8 directions and a "don't revisit on this route" rule, the number of
distinct routes is exponential in the number of cells. Even with a global
visited set, a DFS commits to the first route it finds, which may be long and
winding, and it would have to re-open cells to look for a better one. DFS does
not give shortest paths; it gives *a* path.

BFS never revisits anything, and the first arrival is already optimal:

```python
from collections import deque

def shortest_path_binary_matrix(grid: list[list[int]]) -> int:
    n = len(grid)
    if grid[0][0] == 1 or grid[n - 1][n - 1] == 1:
        return -1                                   # start or goal is blocked
    seen = {(0, 0)}                                 # mark when we ENQUEUE
    queue = deque([(0, 0)])
    dist = 1                                        # cells on the path so far
    while queue:
        for _ in range(len(queue)):                 # one full ring per pass
            r, c = queue.popleft()
            if (r, c) == (n - 1, n - 1):
                return dist
            for dr in (-1, 0, 1):
                for dc in (-1, 0, 1):
                    nr, nc = r + dr, c + dc
                    if 0 <= nr < n and 0 <= nc < n and grid[nr][nc] == 0 and (nr, nc) not in seen:
                        seen.add((nr, nc))          # claim it now, not when popped
                        queue.append((nr, nc))
        dist += 1                                   # the whole ring is done
    return -1                                       # queue drained: unreachable
```

**Time:** each cell is enqueued at most once and dequeued once, and each
dequeue looks at 8 neighbours, so O(V + E) which for an n x n grid is
O(n² · 8) = O(n²). **Space:** the `seen` set and the queue can each hold
O(n²) cells, so O(n²). The recursion depth is not an issue because BFS is
iterative — a real advantage over DFS on big grids.

Two details carry the correctness. First, **mark visited when you enqueue**.
If you wait until you pop, the same cell can be pushed by several neighbours
in the same ring and the queue balloons with duplicates, sometimes badly enough
to time out. Second, counting `dist` per ring (the `for _ in range(len(queue))`
loop) is how you turn "visit order" into "distance".

## Reading the trigger

The words are *minimum number of moves / steps / changes* with uniform cost.
Then ask whether the graph is given or implicit. In the lock problem a state is
a 4-digit combination and its neighbours are the 8 combinations reachable by
turning one wheel one notch; in the gene-mutation problem a state is a string
and its neighbours are the valid strings one letter away. No adjacency list
exists — you write a `neighbours(state)` function and the same BFS runs
unchanged. Forbidden states are simply pre-loaded into `seen`.

Contrast with look-alikes. If edges have different weights, BFS's ring
argument breaks and you need Dijkstra (see the advanced-graphs unit). If you
only need *whether* something is reachable or to explore every cell of a
region, a DFS flood fill is enough and uses less memory. If you must list
*every* path, that is backtracking, not BFS.

## Saying it in the interview

"A DFS would find a path but not necessarily the shortest, and enumerating all
routes is exponential. Since every move costs one, I'll use BFS: the queue
processes nodes in order of distance, so the first time I reach the goal is
optimal. I start from the top-left with distance 1, expand ring by ring, and
mark cells visited when I push them so none is queued twice. Time is O(V + E),
which is O(n²) for the grid since each cell has at most 8 neighbours; space is
O(n²) for the visited set and queue." Then name your edge cases: a blocked
start or goal, a 1 x 1 grid whose answer is 1, and the goal being unreachable
so the queue empties and you return -1. The two bugs interviewers watch for
are marking visited on pop (duplicates and slowdowns) and forgetting to count
the starting cell, which makes every answer off by one.
