---
id: 03-bounded-hops
unit: advanced-graphs
subpattern: "Bounded-hops shortest path (Bellman-Ford)"
trigger_signals:
  - "You need the cheapest path but it's capped at a maximum number of edges/stops — not just \"the cheapest path, however long.\""
  - "A cheaper-looking route might need more hops than the budget allows, so \"finalize on first pop\" (Dijkstra's trick) no longer applies."
  - "You're told to relax edges a bounded number of rounds, not run until nothing changes."
worked_example: cheapest-flights-within-k-stops
diagram: 03-bounded-hops.diagram.json
quiz: 03-bounded-hops.quiz.json
practice:
  - network-delay-time
  - swim-in-rising-water
recap:
  - 01-dijkstra
follow_up:
  - "What if there were no hop limit at all — could you just run to convergence, and how many rounds would that take in the worst case?"
  - "What if a negative-weight cycle existed within the hop budget — how would you detect that rather than silently returning a wrong answer?"
---

## The one idea

When a path is capped at `k` stops, Dijkstra breaks: once it finalizes a
node's distance, it never revisits that node even though the cheapest route
*within budget* might need to pass through it differently. Bellman-Ford
instead relaxes **every edge**, in rounds, exactly `k + 1` times — round `i`
guarantees every path using at most `i` edges has been considered.

```python
def find_cheapest_price(
    n: int, flights: list[list[int]], src: int, dst: int, k: int
) -> int:
    dist = [float("inf")] * n
    dist[src] = 0
    for _ in range(k + 1):                  # at most k+1 edges = k stops
        new_dist = dist[:]                  # snapshot: relax off LAST round only
        for u, v, w in flights:
            if dist[u] != float("inf") and dist[u] + w < new_dist[v]:
                new_dist[v] = dist[u] + w
        dist = new_dist
    return dist[dst] if dist[dst] != float("inf") else -1
```

## Why it beats the obvious approach

Trying every path up to `k` edges by DFS is exponential in `k`. Dijkstra is
fast but wrong here, because its greedy "finalize on first pop" argument
assumes cheaper always wins outright — it has no notion of a hop budget.
Bellman-Ford trades that speed for a different guarantee: after round `i`,
`dist[v]` is correct for the cheapest path to `v` using **at most `i`**
edges, no more, no less. Running exactly `k + 1` rounds (one per allowed
edge, since `k` stops = `k + 1` edges) gives exactly the bounded answer the
problem asks for.

The snapshot (`new_dist = dist[:]`) is the detail that makes it correct:
relaxing in place would let one round silently chain two edges together
through a node updated earlier in that same round, smuggling in an extra
hop the budget didn't allow. Each round touches every edge once, so `k + 1`
rounds cost O(k · E) — worse than Dijkstra's O((V + E) log V) in general,
but Dijkstra can't answer this question at all.

## Reading the trigger

Say it out loud: **"cheapest path, but capped at a number of stops/edges."**
The instant a shortest-path problem adds a hop or edge-count ceiling, drop
the min-heap and relax every edge for exactly `budget + 1` rounds, snapshotting
between rounds so each round advances the path by at most one hop.
