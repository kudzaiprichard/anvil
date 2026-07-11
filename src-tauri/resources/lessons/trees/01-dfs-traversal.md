---
id: 01-dfs-traversal
unit: trees
subpattern: "DFS pre/in/post-order"
trigger_signals:
  - "The task is to visit, transform, or accumulate over every node of a tree exactly once, with no cross-branch ordering constraint."
  - "The problem statement is naturally recursive: 'do X to this node, assuming X is already done for its children.'"
  - "You can phrase the work as happening either *before* the recursive calls (top-down) or *after* they return (bottom-up)."
worked_example: invert-binary-tree
diagram: 01-dfs-traversal.diagram.json
quiz: 01-dfs-traversal.quiz.json
practice:
  - same-tree
  - count-good-nodes-in-binary-tree
recap: []
follow_up:
  - "What if you needed the *values* in sorted order rather than just transforming the tree — where in the recursion would you record them?"
  - "What if the tree were pathologically deep (a long chain) and recursion risked a stack overflow — how would you convert this into an explicit stack?"
---

## The one idea

A tree has no natural start-to-end scan the way an array does — the only way
in is the root, and the tree's own definition ("a node plus a left subtree
plus a right subtree") is recursive. So the natural way to visit every node
is recursion that mirrors that definition: solve the problem for a node by
trusting that the same call already solves it for its children.

What changes between problems is *when* you do the node's own work relative
to the two recursive calls — before them (pre-order), between them
(in-order), or after them (post-order). For inverting a tree, either
pre-order (swap first, then recurse into the now-swapped children) or
post-order (recurse first, then swap) works, because the swap at one node
doesn't depend on what happens elsewhere:

```python
def invert_tree(root: TreeNode | None) -> TreeNode | None:
    if root is None:
        return None
    # Post-order: fix the subtrees first, then swap this node's pointers.
    left = invert_tree(root.left)
    right = invert_tree(root.right)
    root.left, root.right = right, left
    return root
```

## Why it beats the obvious approach

The alternative is an iterative walk with your own explicit stack, manually
pushing and popping nodes and tracking which children you've already
visited — it does the same O(n) work but the recursion is doing that
bookkeeping for you, for free, in the call stack. Recursion also *reads* as
the problem statement: "invert a tree" becomes "invert the two subtrees,
then swap them," which is nearly a direct transcription of the code.

Every node is visited exactly once, so the work is O(n) time. The only cost
beyond that is the call stack itself, which holds at most one frame per
level of depth — O(h) space, where h is the tree's height. That's O(log n)
for a balanced tree but can degrade to O(n) for a completely skewed one, the
same shape-dependent bound every DFS-on-trees problem shares.

## Reading the trigger

Say it out loud: **"do something to every node, and the work at a node only
needs its children's results (or feeds into them)."** That's the DFS tell.
Whenever a tree problem reduces to "handle this node, trusting the recursive
call already handled its subtrees" — inverting, searching, copying,
accumulating a path sum — reach for pre/in/post-order recursion before
reaching for anything else. The only decision left is *where* to place the
node's own work relative to the two recursive calls.
