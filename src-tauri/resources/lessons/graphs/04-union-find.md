---
id: 04-union-find
unit: graphs
subpattern: "Union-Find / connected components"
trigger_signals:
  - "You're repeatedly told \"these two items are connected\" and need to answer \"are these two in the same group\" or \"how many groups are there\" — without re-walking the whole graph each time."
  - "The graph arrives as an adjacency matrix or an edge list rather than a grid, and connectivity — not shortest path, not ordering — is the whole question."
  - "You need to merge groups incrementally as edges stream in, or count connected components once all edges are known."
worked_example: number-of-provinces
diagram: 04-union-find.diagram.json
quiz: 04-union-find.quiz.json
practice:
  - number-of-islands
  - max-area-of-island
recap:
  - 01-grid-dfs
follow_up:
  - "What if edges arrived one at a time and you had to answer \"are these connected yet?\" after each one — how does maintaining Union-Find incrementally beat re-running DFS from scratch every time?"
  - "What if you also tracked each group's size — where would that update inside `union`, and what would it cost?"
---

## The one idea

Union-Find (disjoint set union) keeps a `parent` array where every node
points toward a representative "root" for its group. `find` walks up to that
root — with **path compression** to flatten future lookups — and `union`
merges two groups by pointing one root at the other. Two nodes are connected
exactly when `find(a) == find(b)`.

```python
def find_circle_num(is_connected: list[list[int]]) -> int:
    n = len(is_connected)
    parent = list(range(n))

    def find(x: int) -> int:
        if parent[x] != x:
            parent[x] = find(parent[x])   # path compression
        return parent[x]

    def union(a: int, b: int) -> None:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb

    for i in range(n):
        for j in range(i + 1, n):
            if is_connected[i][j] == 1:
                union(i, j)

    return len({find(i) for i in range(n)})
```

## Why it beats the obvious approach

A DFS/BFS flood fill also counts components, but it needs the full
adjacency structure walked fresh every time you ask "how many groups now?"
Union-Find instead maintains groups **incrementally**: each `union` is
amortized close to O(1) — with path compression it's O(α(n)), the inverse
Ackermann function, which is effectively constant for any input size you'll
ever see. Processing all `n²` matrix entries here still costs O(n²) just to
read the input, but the real payoff shows up whenever connectivity queries
and merges are interleaved with other work — Union-Find answers "same
group?" in near-constant time, where DFS would have to re-walk the graph
from scratch on every query.

## Reading the trigger

Say it out loud: **"merge things into groups, then ask how many groups /
are these two together."** Whenever the question is fundamentally about
connectivity — not shortest path, not ordering — and edges arrive as pairs
to merge, reach for `parent` + `find` + `union` before reaching for another
flood fill.
