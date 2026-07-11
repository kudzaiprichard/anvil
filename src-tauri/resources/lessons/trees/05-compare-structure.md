---
id: 05-compare-structure
unit: trees
subpattern: "Comparing & matching trees"
trigger_signals:
  - "The question asks whether two trees (or a tree and a candidate subtree) are structurally identical, not just about one tree's own shape."
  - "You can phrase the check recursively: 'these two nodes match if their values match AND their left subtrees match AND their right subtrees match.'"
  - "A mismatch anywhere — a missing node, an extra node, a different value — should short-circuit the whole comparison to false."
worked_example: same-tree
diagram: 05-compare-structure.diagram.json
quiz: 05-compare-structure.quiz.json
practice:
  - subtree-of-another-tree
  - balanced-binary-tree
recap:
  - 01-dfs-traversal
follow_up:
  - "'Subtree of another tree' calls this same-tree check at every node of the bigger tree — how would you bound that so it isn't O(n * m) in the worst case?"
  - "What if the trees could contain duplicate values — does that change how you'd try to shortcut the subtree search?"
---

## The one idea

Comparing two trees is DFS on *two* trees at once: recurse on both structures
in lockstep, and a pair of nodes only "matches" if their values agree **and**
their left children match **and** their right children match, all the way
down.

```python
def is_same_tree(p: TreeNode | None, q: TreeNode | None) -> bool:
    if p is None and q is None:
        return True
    if p is None or q is None:          # one is None, the other isn't
        return False
    if p.val != q.val:
        return False
    return is_same_tree(p.left, q.left) and is_same_tree(p.right, q.right)
```

Three failure cases end the recursion immediately: one side ran out of nodes
while the other didn't (shape mismatch), or both sides have a node but the
values differ. Anything that survives all three checks recurses into both
pairs of children.

## Why it beats the obvious approach

You might think to flatten each tree into a list (say, a pre-order sequence
of values) and compare the two lists — but a pre-order sequence alone
doesn't uniquely determine a tree's shape unless you also encode where the
`None`s are, so you'd need to bake null markers into the flattening anyway.
Once you do that, you've reconstructed exactly the same node-by-node
comparison, just with an extra list-building pass in front of it. Comparing
the trees directly, node pair by node pair, skips that detour entirely.

The other property worth naming: `and` **short-circuits**. The moment any
node pair fails to match, the whole call returns `False` without visiting
the rest of either tree — you don't pay for comparing subtrees you already
know can't help. In the worst case (the trees really are identical, so
nothing short-circuits early) every node pair is visited once: O(min(n, m))
time, O(h) space for the recursion stack.

## Reading the trigger

Say it out loud: **"are these two trees the same shape with the same
values,"** or **"does this smaller tree appear, unchanged, somewhere inside
the bigger one."** Both reduce to the same recursive matcher — same-tree
first tries the match at the two roots directly; subtree-matching additionally
walks the bigger tree, trying the same-tree check rooted at every one of its
nodes until one matches (or none do).
