---
id: 03-merge-dummy
unit: linked-list
subpattern: "Merge with a dummy head"
trigger_signals:
  - "You're merging two (or more) already-sorted linked lists into one sorted list."
  - "The result's first node isn't known ahead of time — it could come from either input list."
  - "You want to avoid special-casing \"is this the first node I'm attaching?\" inside the loop."
worked_example: merge-two-sorted-lists
diagram: 03-merge-dummy.diagram.json
quiz: 03-merge-dummy.quiz.json
practice:
  - remove-nth-node-from-end-of-list
  - reorder-list
recap:
  - 02-fast-slow
follow_up:
  - "What if you had k sorted lists instead of two — how would you extend the same dummy-head merge, and what does picking the smallest of k heads cost each step?"
  - "What if the two lists weren't sorted — does a dummy head still help, or does the trick depend entirely on sortedness?"
---

## The one idea

A dummy (sentinel) node placed *before* the real head turns "attach the
first node" into just another iteration of the same loop — no special
case, because `dummy.next` is always where the answer begins.

```python
def merge_two_lists(l1: Optional[ListNode], l2: Optional[ListNode]) -> Optional[ListNode]:
    dummy = ListNode()
    tail = dummy
    while l1 and l2:
        if l1.val <= l2.val:
            tail.next, l1 = l1, l1.next
        else:
            tail.next, l2 = l2, l2.next
        tail = tail.next
    tail.next = l1 or l2          # attach whatever's left — already sorted
    return dummy.next              # skip the sentinel, return the real head
```

## Why it beats the obvious approach

The naive fix branches on every iteration: `if head is None: head = node
else: tail.next = node`, handling the very first attachment differently
from every later one. That's repeated logic for a one-time event. A dummy
head removes the branch entirely — `tail` always has somewhere valid to
point, from the first comparison onward, because the sentinel absorbs that
first write.

When one list runs out, the remainder of the other is already sorted, so
it can be spliced on directly with `tail.next = l1 or l2` — no need to
walk it node by node.

Two things carry the whole trick:

1. `dummy` itself is thrown away at the end — only `dummy.next` (the real
   first node) is returned.
2. `tail` always trails one step behind the last attached node, so
   `tail.next = ...` is always a valid write, even before anything real has
   been attached.

## Reading the trigger

Say it out loud: **"walk two sorted sequences in lock-step, always taking
the smaller head, and stitch the result together."** Whenever a problem
builds a new linked list by repeatedly picking from one of several
sources — merging sorted lists, splicing after removing a node,
partitioning by value — reach for a dummy head first; it saves you the
"first node is special" branch every single time.
