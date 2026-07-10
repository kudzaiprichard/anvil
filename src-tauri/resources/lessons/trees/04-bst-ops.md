---
id: 04-bst-ops
unit: trees
subpattern: "BST ordering & validation"
trigger_signals:
  - "The problem tests a BST invariant — 'is this a valid BST,' 'find the kth smallest/largest' — anything depending on a node's order relative to ALL its ancestors, not just its parent."
  - "A naive local check (compare a node only to its immediate children) would accept some trees that are locally sorted at every parent-child pair but globally out of order."
  - "An in-order traversal of a BST visits nodes in ascending sorted order — 'kth smallest' or 'k values in order' is a strong signal to walk it in-order."
worked_example: validate-binary-search-tree
diagram: 04-bst-ops.diagram.json
quiz: 04-bst-ops.quiz.json
practice:
  - kth-smallest-element-in-a-bst
  - subtree-of-another-tree
recap:
  - 01-dfs-traversal
follow_up:
  - "What if you needed the kth LARGEST value instead of smallest — would you change the traversal order, or just count differently?"
  - "What if you had to validate the BST with O(1) auxiliary space beyond the input — how would an in-order walk without recursion or a stack (Morris traversal) help?"
---

## The one idea

A binary search tree's ordering property isn't local — it's that **every**
node in a subtree is bounded by *all* of its ancestors, not just its
immediate parent. A node's left subtree must be less than the node, and its
right subtree greater, transitively, all the way down. Validating that means
carrying a shrinking `(low, high)` range through the DFS recursion, not just
comparing a node to its direct children:

```python
def is_valid_bst(
    root: TreeNode | None,
    low: float = float("-inf"),
    high: float = float("inf"),
) -> bool:
    if root is None:
        return True
    if not (low < root.val < high):
        return False
    return is_valid_bst(root.left, low, root.val) and is_valid_bst(
        root.right, root.val, high
    )
```

## Why it beats the obvious approach

The tempting shortcut is a *local* check: `left.val < node.val < right.val`
at every node. It fails on trees that are locally sorted everywhere but
globally broken. Take `[5, 4, 6, null, null, 3, 7]`: root 5, left child 4,
right child 6; node 6's own children are 3 and 7. Every parent-child pair
passes locally — `4 < 5 < 6` and `3 < 6 < 7` — yet node **3** sits in the
root's *right* subtree, where every value must exceed 5. A local check never
catches this, because it never compares 3 against 5, only against its
immediate parent 6.

The range-propagation version does catch it: by the time recursion reaches
node 3, its inherited range is `(5, 6)` — `5` from the root's right-subtree
bound, narrowed to `6` by node 6's own left-subtree bound — so `low < 3 < high`
fails immediately. Each node is visited once, so this is still O(n) time,
O(h) space, the same bound as any other DFS on a tree — the only change is
*what* gets threaded through the recursion.

A related tool for BST questions is that **in-order traversal of a BST
visits nodes in ascending order**, for free — no sorting required. That's
exactly why "kth smallest" reduces to "walk in-order and stop after k
values": the BST's shape has already done the sorting work.

## Reading the trigger

Say it out loud: **"does this respect BST order everywhere,"** or **"the
kth value in sorted order."** Both are really the same signal — a BST's
ordering is a global, ancestor-wide property. Validate it with a shrinking
range passed down the recursion; read sorted order out of it with a plain
in-order walk.
