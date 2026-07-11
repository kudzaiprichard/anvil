---
id: 03-bfs-level
unit: trees
subpattern: "BFS level-order traversal"
trigger_signals:
  - "The question is about the tree grouped or answered PER LEVEL/depth — 'return each row,' 'what do you see from the side,' 'the last node in each row.'"
  - "You need the fewest hops between nodes in an unweighted tree shape — BFS discovers nodes in order of distance from the root; DFS doesn't."
  - "The natural container is a FIFO queue, not the call stack — you want nodes processed in discovery order, not in the order a subtree happens to finish."
worked_example: binary-tree-level-order-traversal
diagram: 03-bfs-level.diagram.json
quiz: 03-bfs-level.quiz.json
practice:
  - binary-tree-right-side-view
  - invert-binary-tree
recap:
  - 01-dfs-traversal
follow_up:
  - "What if you only needed the *last* node of each level (the view from the right) — could you skip building each full level's list?"
  - "What if the tree were extremely wide, so one level could hold most of its nodes — how does that change the algorithm's worst-case space?"
---

## The one idea

DFS answers "what's true about this subtree," recursing depth-first through
the call stack. BFS answers a different question — "what does the tree look
like level by level" — by processing nodes in a **queue**, in the order they
were discovered, one whole depth at a time.

```python
from collections import deque

def level_order(root: TreeNode | None) -> list[list[int]]:
    if root is None:
        return []
    result: list[list[int]] = []
    queue = deque([root])
    while queue:
        level_size = len(queue)          # snapshot: how many nodes are on this level
        level = []
        for _ in range(level_size):
            node = queue.popleft()
            level.append(node.val)
            if node.left:
                queue.append(node.left)
            if node.right:
                queue.append(node.right)
        result.append(level)
    return result
```

## Why it beats the obvious approach

The trick that makes this work is `level_size = len(queue)`, taken **before**
the inner loop starts. That freezes exactly how many nodes belong to the
current level, so the inner `for` loop processes precisely that many nodes
and stops — even though it's enqueuing next level's children into the same
queue as it goes. Without that snapshot, there'd be no way to tell where one
level ends and the next begins.

You could try to fake level-grouping with DFS — recurse with a depth
counter and append into `result[depth]` — but that means allocating buckets
for depths you haven't reached yet and writing out of order relative to the
recursion. BFS's queue gives you the levels *in order*, for free, because a
queue is FIFO: everything discovered at depth `d` is popped before anything
discovered at depth `d + 1`.

Every node is enqueued and dequeued exactly once — O(n) time. The queue can
hold at most one full level, which in the worst case (a wide, bottom-heavy
tree) is close to `n/2` nodes — O(n) space.

## Reading the trigger

Say it out loud: **"group the tree by depth"** or **"fewest hops from the
root."** Whenever a problem is naturally about *rows* of a tree rather than
subtrees, or about shortest distance in an unweighted shape, reach for a
queue and the `level_size` snapshot before reaching for recursion.
