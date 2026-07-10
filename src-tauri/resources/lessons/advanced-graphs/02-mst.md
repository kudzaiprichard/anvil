---
id: 02-mst
unit: advanced-graphs
subpattern: "Minimum spanning tree (Prim)"
trigger_signals:
  - "You need to connect *every* node with minimum total edge cost — not the shortest path between two specific nodes."
  - "The phrase \"connect all points/nodes as cheaply as possible\" or \"minimum cost network\" — every node must end up reachable."
  - "The graph is often implicit (e.g., every pair of points is a potential edge, weighted by distance) rather than given as an edge list."
worked_example: min-cost-to-connect-all-points
diagram: 02-mst.diagram.json
quiz: 02-mst.quiz.json
practice:
  - network-delay-time
  - cheapest-flights-within-k-stops
recap:
  - 01-dijkstra
follow_up:
  - "What if a few connections were mandatory (must be in the final network) — how would you seed the tree before running Prim?"
  - "What if the graph were sparse instead of complete — would Kruskal's (sort edges, union-find) beat Prim's here?"
---

## The one idea

Prim's algorithm grows a single tree one edge at a time, always adding the
*cheapest* edge that reaches from a node already in the tree to a node that
isn't yet — the same min-heap-pop engine as Dijkstra, but the heap key is a
single edge's weight, not a cumulative path distance.

```python
import heapq

def min_cost_connect_points(points: list[list[int]]) -> int:
    n = len(points)

    def dist(i: int, j: int) -> int:
        (x1, y1), (x2, y2) = points[i], points[j]
        return abs(x1 - x2) + abs(y1 - y2)

    visited: set[int] = set()
    heap = [(0, 0)]                 # (edge weight into the tree, node)
    total = 0
    while len(visited) < n:
        w, u = heapq.heappop(heap)
        if u in visited:
            continue
        visited.add(u)
        total += w
        for v in range(n):
            if v not in visited:
                heapq.heappush(heap, (dist(u, v), v))
    return total
```

## Why it beats the obvious approach

Trying every spanning tree is exponential — there are `n^(n-2)` of them on
`n` nodes (Cayley's formula). Prim instead grows the tree greedily: at every
step, the cheapest edge crossing the "cut" between the tree and everything
outside it is always safe to add (the cut property guarantees no smaller
spanning tree could skip it). That's provably optimal, not just a good
heuristic.

Notice the skeleton is identical to Dijkstra's — pop the heap, skip if
already visited, otherwise finalize and push new frontier entries — but the
*meaning* of the heap key changed. Dijkstra's key is `distance-from-source +
edge_weight` (a running total); Prim's key is just `edge_weight` (one hop
from whatever's already in the tree). That's the whole difference between
"cheapest path to reach this node" and "cheapest way to attach this node to
what we've already built."

## Reading the trigger

Say it out loud: **"connect every node, minimize total edge cost, no
particular start or end matters."** Whenever a problem gives you (or lets
you compute) pairwise weights and asks for the cheapest way to make
everything reachable — not the cheapest way from A to B — that's a minimum
spanning tree, and Prim's min-heap frontier builds one in O(E log V).
