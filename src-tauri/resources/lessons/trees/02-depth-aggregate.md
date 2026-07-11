---
id: 02-depth-aggregate
unit: trees
subpattern: "Bottom-up aggregation (depth & diameter)"
trigger_signals:
  - "The answer is a single aggregate value about the whole tree (a depth, a count, a boolean, a longest-path length) — not a modified tree or a list of nodes."
  - "You can define the node's answer recursively: 'this node's value is some combination of the same value computed for its left and right subtrees.'"
  - "Recomputing a helper value from scratch at every node (calling a depth-style function again inside another walk) would revisit subtrees repeatedly — an O(n^2) trap you want to avoid."
worked_example: maximum-depth-of-binary-tree
diagram: 02-depth-aggregate.diagram.json
quiz: 02-depth-aggregate.quiz.json
practice:
  - diameter-of-binary-tree
  - balanced-binary-tree
recap:
  - 01-dfs-traversal
follow_up:
  - "Diameter needs both a node's depth and a running best-so-far in the same pass — how do you avoid calling a separate depth() from scratch at every node?"
  - "What if you needed the exact subtree that first breaks a balance check, not just a yes/no answer — where would you short-circuit the recursion?"
---

## The one idea

The previous lesson used DFS to do work *at* each node. This lesson uses the
same recursion shape to *return a value from* each node — post-order, but
now every call hands a number back up to its caller, and the parent combines
its two children's numbers into its own.

```python
def max_depth(root: TreeNode | None) -> int:
    if root is None:
        return 0
    left = max_depth(root.left)
    right = max_depth(root.right)
    return 1 + max(left, right)
```

A node's depth is "1 (for itself) plus the taller of its two children's
depths." That recursive definition is the whole algorithm — no traversal
order to choose, no extra state, just trust the children's return values.

## Why it beats the obvious approach

A level-by-level BFS scan can also find the depth by counting layers, but it
needs a queue and explicit level bookkeeping for something recursion gets
for free as a return value.

The real payoff shows up on **diameter**: the longest path *through* a node
isn't that node's own depth — it's `left_depth + right_depth`, the two arms
meeting at the node. If you computed `max_depth` fresh for every node while
also walking the tree, you'd redo the same subtree work over and over,
O(n) work at O(n) nodes — O(n²) overall. The fix is to compute depth and
update a running best diameter **in the same post-order pass**:

```python
def diameter(root: TreeNode | None) -> int:
    best = 0
    def depth(node: TreeNode | None) -> int:
        nonlocal best
        if node is None:
            return 0
        left = depth(node.left)
        right = depth(node.right)
        best = max(best, left + right)   # longest path through *this* node
        return 1 + max(left, right)      # depth this node reports to its parent
    depth(root)
    return best
```

One post-order pass, O(n) time, O(h) space — the return value and the
side-effect update happen at the same recursive step, never two separate
walks.

## Reading the trigger

Say it out loud: **"one number about the whole tree, and that number at a
node is a combination of the same number at its children."** Depth, height,
diameter, "is this subtree balanced" — all reduce to the same recipe: post-order
recursion that returns a value, with the parent combining what its children
returned. Whenever you catch yourself about to call a depth-style helper
*inside* another recursive walk, that's the signal to fold both into a
single bottom-up pass instead.
