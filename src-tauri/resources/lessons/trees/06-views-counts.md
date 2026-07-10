---
id: 06-views-counts
unit: trees
subpattern: "Level views & path counting"
trigger_signals:
  - "The question is about what's VISIBLE per level or depth (leftmost, rightmost, top view) — a per-level selection, not every node on the level."
  - "The question is about a property along the ROOT-TO-NODE PATH (a running max, a running sum so far) rather than a subtree aggregate — information flows top-down through the recursion, not bottom-up."
  - "You can answer it with either BFS (keep the last/first node seen per level) or a DFS ordered so the 'interesting' child is visited first."
worked_example: binary-tree-right-side-view
diagram: 06-views-counts.diagram.json
quiz: 06-views-counts.quiz.json
practice:
  - count-good-nodes-in-binary-tree
  - maximum-depth-of-binary-tree
recap:
  - 03-bfs-level
follow_up:
  - "What about the LEFT side view instead of the right — what's the one thing you'd flip in either the BFS or DFS version?"
  - "What if you needed to count good nodes AND report their actual values, not just the count — how would you collect them during the same top-down pass?"
---

## The one idea

The right-side view asks for one node per level — the rightmost one — not
the whole level. Reusing this unit's BFS level-order shape, you don't need
to build each level's full list; you only need to remember the *last* node
popped during each level's inner loop:

```python
from collections import deque

def right_side_view(root: TreeNode | None) -> list[int]:
    if root is None:
        return []
    result = []
    queue = deque([root])
    while queue:
        level_size = len(queue)
        for i in range(level_size):
            node = queue.popleft()
            if i == level_size - 1:      # last node processed this level
                result.append(node.val)
            if node.left:
                queue.append(node.left)
            if node.right:
                queue.append(node.right)
    return result
```

## Why it beats the obvious approach

You could build every level's full list (Lesson 3's approach) and then take
`level[-1]` from each — correct, but wasteful, since you throw away every
value except the last one anyway. Tracking `i == level_size - 1` inline
keeps the same O(n) time and O(n) queue space without the extra list
allocation per level.

There's also a DFS alternative worth knowing, because it shows the same
answer from a different angle: visit **right before left**, and record a
node's value only the first time you reach its depth.

```python
def right_side_view_dfs(root: TreeNode | None) -> list[int]:
    result: list[int] = []
    def dfs(node: TreeNode | None, depth: int) -> None:
        if node is None:
            return
        if depth == len(result):         # first node seen at this depth
            result.append(node.val)
        dfs(node.right, depth + 1)
        dfs(node.left, depth + 1)
    dfs(root, 0)
    return result
```

Because right is visited before left, the first node DFS ever reaches at a
given depth is guaranteed to be the rightmost one at that depth.

This lesson's other shape — counting **good nodes** — looks similar but
flows the opposite direction: instead of a bottom-up subtree aggregate
(Lesson 2), it's a **top-down path aggregate**. A node is "good" if its
value is at least the maximum value seen so far on the path from the root,
so you carry that running maximum *down* into the recursion as an argument,
rather than combining values coming back *up*:

```python
def good_nodes(root: TreeNode, max_so_far: int = float("-inf")) -> int:
    if root is None:
        return 0
    count = 1 if root.val >= max_so_far else 0
    new_max = max(max_so_far, root.val)
    return count + good_nodes(root.left, new_max) + good_nodes(root.right, new_max)
```

Both are still a single DFS/BFS pass — O(n) time — but the direction the
state travels through the recursion (down the path vs. up from subtrees) is
the detail that decides which one you need.

## Reading the trigger

Say it out loud: **"what's visible from one side"** (a per-level pick — BFS
keep-last, or right-first DFS keep-first) or **"a property of the path from
the root to here"** (a top-down running value threaded as a recursion
argument). Neither is a subtree aggregate like Lesson 2 — one narrows a
level to a single node, the other narrows a path to a running value.
