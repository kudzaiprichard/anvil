---
id: 01-reversal
unit: linked-list
subpattern: "In-place linked-list reversal"
trigger_signals:
  - "The problem asks you to reverse a linked list, or a piece of one, without building a new list."
  - "You need to flip the direction every `next` pointer faces while visiting each node exactly once."
  - "The phrase \"in place\" or \"O(1) extra space\" appears next to a linked-list problem."
worked_example: reverse-linked-list
diagram: 01-reversal.diagram.json
quiz: 01-reversal.quiz.json
practice:
  - reorder-list
  - palindrome-linked-list
recap: []
follow_up:
  - "What if you only needed to reverse a sub-range `[left, right]` of the list, not the whole thing — where do the three pointers start and stop?"
  - "What if you had to reverse the list recursively instead of iteratively — how does the call stack take over the job of the `prev` pointer?"
---

## The one idea

Reversing a singly linked list in place means walking it once and, at each
node, flipping its `next` pointer to point *backward* instead of forward —
using three tracking pointers (`prev`, `curr`, `nxt`) so you never lose the
rest of the list the moment you overwrite a pointer.

```python
def reverse_list(head: Optional[ListNode]) -> Optional[ListNode]:
    prev, curr = None, head
    while curr:
        nxt = curr.next        # save the rest of the list before we lose it
        curr.next = prev       # flip this node's pointer backward
        prev, curr = curr, nxt # slide both pointers one step forward
    return prev                 # prev is the new head
```

## Why it beats the obvious approach

The naive fix is to build a **new** reversed list: walk the original, and for
each node, prepend a freshly allocated node to a result list. That's O(n)
time, but it's also O(n) *extra* space, and the problem usually forbids new
nodes entirely — it wants the existing nodes rewired.

The in-place version does the same O(n) single pass but reuses every node: at
each step, `curr.next` is overwritten to point at `prev` instead of forward.
The only reason this doesn't strand the rest of the list is that `nxt` was
saved *before* the overwrite — miss that ordering and the list falls apart
after the first node. Three pointers, one pass, zero new allocations:

1. `nxt` remembers where to go next before the pointer is destroyed.
2. `curr.next = prev` performs the actual flip.
3. `prev, curr = curr, nxt` advances the whole apparatus one node at a time.

## Reading the trigger

Say the shape out loud: **"rewire every `next` pointer on a singly linked
list to face the other way, without allocating anything new."** Whenever a
problem wants an existing structure walked in reverse — or a *segment* of it
reversed, as in reordering or palindrome checks that fold the list back on
itself — this three-pointer sweep is the tool, not a recursive rebuild or a
list copied into an array and read backward.
