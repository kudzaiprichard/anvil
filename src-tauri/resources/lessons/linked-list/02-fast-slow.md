---
id: 02-fast-slow
unit: linked-list
subpattern: "Fast & slow pointers"
trigger_signals:
  - "You need the middle node of a singly linked list in one pass, without knowing its length ahead of time."
  - "The problem talks about a node reached at \"half the speed\" of another, or about detecting a cycle."
  - "You want O(1) extra space instead of counting the length first or copying nodes into an array."
worked_example: middle-of-the-linked-list
diagram: 02-fast-slow.diagram.json
quiz: 02-fast-slow.quiz.json
practice:
  - palindrome-linked-list
  - valid-palindrome
recap:
  - 01-reversal
follow_up:
  - "What if the list might contain a cycle — how does the same two-speed idea detect that instead of finding a midpoint?"
  - "What if you needed the *first* middle of an even-length list instead of the second — which pointer starts one step behind?"
---

## The one idea

Walk two pointers from the head at different speeds — `slow` moves one node
at a time, `fast` moves two — and by the time `fast` runs out of list,
`slow` is sitting exactly on the midpoint. One pass, no length count, no
extra storage.

```python
def middle_node(head: Optional[ListNode]) -> Optional[ListNode]:
    slow = fast = head
    while fast and fast.next:
        slow = slow.next          # one step
        fast = fast.next.next     # two steps
    return slow                    # slow has covered exactly half the distance
```

## Why it beats the obvious approach

The naive fix is two passes: walk the list once to count its length `n`,
then walk it again to node `n // 2`. That works, but it requires knowing
`n` up front, which is awkward if the list is effectively a stream. Copying
every value into an array and indexing the middle avoids the two-pass
requirement but costs O(n) *extra* space you don't need.

Fast/slow gets both wins in a single pass: because `fast` always covers
twice the ground `slow` does, the moment `fast` reaches the end, `slow` has
covered exactly half. No counting, no array.

1. The loop guards on `fast and fast.next` — both must exist, or
   `fast.next.next` would fail on a missing node.
2. `slow`'s total distance traveled is always half of `fast`'s when the
   loop stops, regardless of whether the list has odd or even length.

## Reading the trigger

Say the shape out loud: **"traverse once, land on the halfway point,
without counting first."** Whenever a problem needs a midpoint, a node
"k nodes from the end," or wants a cycle detected without external
markers, the two-speed pointer sweep is the tool to reach for — before a
length counter or an array copy.
