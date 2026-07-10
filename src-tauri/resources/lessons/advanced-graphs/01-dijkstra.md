---
id: 01-dijkstra
unit: advanced-graphs
subpattern: "Dijkstra shortest path"
trigger_signals:
  - "You need the shortest path (or minimum cost/time) from one source to every other node in a weighted graph."
  - "Edge weights are non-negative — there's no discount for revisiting or backtracking through a cheaper detour later."
  - "A plain BFS won't work because edges don't all cost the same; \"fewest hops\" and \"cheapest total\" are different answers."
worked_example: network-delay-time
diagram: 01-dijkstra.diagram.json
quiz: 01-dijkstra.quiz.json
practice:
  - swim-in-rising-water
  - number-of-islands
recap: []
follow_up:
  - "What if some edge weights could be negative — does \"once popped, distance is final\" still hold?"
  - "What if you needed the actual shortest *path*, not just its length — what would you track alongside `dist` to reconstruct it?"
---

## The one idea

Dijkstra explores nodes in order of the cheapest distance found *so far*,
using a min-heap so the next node popped is always the closest unfinished
one. Because every edge weight is non-negative, the first time a node is
popped, its distance is guaranteed final — nothing relaxed later could ever
produce a cheaper route to it.

```python
import heapq
from collections import defaultdict

def network_delay_time(times: list[list[int]], n: int, k: int) -> int:
    graph = defaultdict(list)
    for u, v, w in times:
        graph[u].append((v, w))

    dist: dict[int, int] = {}
    heap = [(0, k)]                     # (distance so far, node)
    while heap:
        d, node = heapq.heappop(heap)
        if node in dist:                # a cheaper route already finalized it
            continue
        dist[node] = d
        for nei, w in graph[node]:
            if nei not in dist:
                heapq.heappush(heap, (d + w, nei))

    return max(dist.values()) if len(dist) == n else -1
```

## Why it beats the obvious approach

A plain BFS assumes every edge costs 1, so its queue order reflects hop
count, not true distance — it can report a "closer" node that's actually
more expensive. Trying every path is exponential. The heap fixes this by
always expanding the frontier's *globally cheapest* unfinished node next: by
the time a node is popped, every other route to it would have to be at
least as expensive (the heap would have popped that cheaper route first).
That single guarantee — pop order is distance order — is what lets you
finalize a node's distance the moment you see it and never revisit it.

A node can sit in the heap more than once (pushed once per relaxing edge);
the `if node in dist: continue` check is what skips those stale, more
expensive duplicates cheaply instead of overwriting a final answer. With a
binary heap this whole scan is O((V + E) log V) time and O(V + E) space.

## Reading the trigger

Say it out loud: **"weighted graph, non-negative weights, shortest path (or
time, or cost) from one source."** Whenever a problem hands you directed,
weighted edges and asks "how long until every node hears the signal" or
"what's the cheapest way to reach node X," reach for a min-heap frontier
before reaching for BFS — BFS only works when every edge secretly costs the
same.
