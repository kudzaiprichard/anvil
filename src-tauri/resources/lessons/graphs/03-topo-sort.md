---
id: 03-topo-sort
unit: graphs
subpattern: "Topological sort (course schedule)"
trigger_signals:
  - "You're given a set of tasks/courses and \"must come before\" prerequisite pairs — a directed dependency graph, not an undirected one."
  - "The question asks whether a valid ordering exists, or asks you to produce one — a cycle in the dependencies means no ordering is possible."
  - "The shape is \"N items, M dependency edges\" — you need to process an item only after all of its prerequisites are done."
worked_example: course-schedule
diagram: 03-topo-sort.diagram.json
quiz: 03-topo-sort.quiz.json
practice:
  - number-of-islands
  - course-schedule-ii
recap:
  - 01-grid-dfs
follow_up:
  - "What if you needed the actual ordering, not just a yes/no — where would you record nodes as they finish, and in what order do you read that record back?"
  - "What if the graph had 100,000 nodes and a deep recursive DFS risked a stack overflow — how would you rewrite this with Kahn's BFS (in-degree) algorithm instead?"
---

## The one idea

Prerequisites are directed edges: "course b needs course a" means an edge
a → b. A valid schedule exists exactly when that directed graph has **no
cycle** — DFS can detect one by tracking, for the current recursion path
only, which nodes are still "in progress."

```python
def can_finish(num_courses: int, prerequisites: list[list[int]]) -> bool:
    graph: dict[int, list[int]] = {i: [] for i in range(num_courses)}
    for course, prereq in prerequisites:
        graph[prereq].append(course)

    UNVISITED, IN_PROGRESS, DONE = 0, 1, 2
    state = [UNVISITED] * num_courses

    def has_cycle(node: int) -> bool:
        if state[node] == IN_PROGRESS:
            return True                 # back edge onto the current path — a cycle
        if state[node] == DONE:
            return False                # already cleared, no cycle through here
        state[node] = IN_PROGRESS
        for nxt in graph[node]:
            if has_cycle(nxt):
                return True
        state[node] = DONE
        return False

    return not any(has_cycle(node) for node in range(num_courses))
```

## Why it beats the obvious approach

A plain visited *set* can't tell a cycle from a harmless diamond shape — two
different paths reaching the same node later is completely fine, but a path
looping back into a node still sitting on its own call stack is not. The
three-state trick (`UNVISITED` / `IN_PROGRESS` / `DONE`) tells them apart:
a node is `IN_PROGRESS` only while it's an ancestor of the current call, and
flips to `DONE` once every one of its descendants has cleared. A back edge
into an `IN_PROGRESS` node is then unambiguous — it's a cycle, full stop.
Each node is opened and closed exactly once, and each edge is followed
exactly once, so the whole check is O(V + E) time and O(V) space.

## Reading the trigger

Say it out loud: **"directed dependency graph, does a valid order exist."**
Prerequisites, build steps, task scheduling — any "X must happen before Y"
phrasing over a fixed set of items is a topological-sort candidate, and the
cycle check is exactly what tells you whether that order even exists before
you try to produce it.
