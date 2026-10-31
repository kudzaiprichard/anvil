---
id: 04-reverse-k-group
unit: linked-list
subpattern: "Reversing in k-groups"
trigger_signals:
  - "The problem asks you to reverse a linked list in consecutive blocks of a given size k, leaving a short final block alone (or treating it by a rule)."
  - "Pointers must be re-stitched on both sides of every reversed block: the node before the block and the node after it."
  - "The instruction says to do it in place with O(1) extra space, and changing only node values (rather than relinking nodes) is not allowed."
worked_example: reverse-nodes-in-k-group
diagram: 04-reverse-k-group.diagram.json
quiz: 04-reverse-k-group.quiz.json
practice:
  - swap-nodes-in-pairs
  - reverse-nodes-in-even-length-groups
recap:
  - 01-reversal
  - 03-merge-dummy
follow_up:
  - "If the final group of fewer than k nodes should *also* be reversed, which single check in the loop do you change?"
  - "Rewrite this recursively: what does each call return, how deep does the stack get, and why is that worse than the iterative O(1) space?"
---

## The one idea

Reversing a whole list (lesson 01) is a loop with `prev`, `cur`, `nxt`.
Reversing in groups of `k` is the same loop run on **one block at a time**,
plus the bookkeeping to reconnect the block to its neighbours. Keep three
landmarks per block: `group_prev` (the node just *before* the block, the tail
of what is already finished), `kth` (the last node of the block) and
`group_next` (the node just *after* it).

For each block: first walk `k` steps from `group_prev` to prove the block
really has `k` nodes. If you fall off the end, the leftover is shorter than
`k` and stays exactly as it is. Otherwise reverse just those `k` nodes in
place, then wire `group_prev` to the new front (`kth`) and let the old front,
which is now the block's tail, become the next `group_prev`. A dummy head in
front of the list (lesson 03) gives the very first block a `group_prev` too, so
no special case is needed.

## Why it beats the obvious approach

The tempting approach is to copy values into an array, reverse chunks of the
array, and write them back or build a new list: it works but costs O(n) extra
space and, when the statement forbids changing values, it isn't allowed at all.
A brute force that re-walks from the head to find each block's predecessor is
O(n^2 / k) in time. Keeping `group_prev` means each node is visited a constant
number of times.

```python
class ListNode:
    def __init__(self, val=0, next=None):
        self.val = val
        self.next = next

def reverse_k_group(head, k):
    dummy = ListNode(0, head)
    group_prev = dummy
    while True:
        kth = group_prev
        for _ in range(k):                      # check that k nodes remain
            kth = kth.next
            if not kth:
                return dummy.next               # leftover < k: leave it untouched
        group_next = kth.next
        prev, cur = group_next, group_prev.next # prev starts at group_next: tail auto-links
        while cur is not group_next:            # classic reversal, bounded to this block
            nxt = cur.next
            cur.next = prev
            prev, cur = cur, nxt
        tail = group_prev.next                  # old first node is now the block's tail
        group_prev.next = kth                   # attach the new front of the block
        group_prev = tail                       # the tail is the predecessor of the next block
```

Starting `prev` at `group_next` rather than `None` is the neat part: when the
reversal finishes, the block's old first node already points at whatever came
after the block, so only the front link needs fixing. Each node is touched once
by the length check and once by the reversal, so the total is **O(n) time**
(roughly `2n` pointer steps). Only a handful of pointers are held, so **O(1)
extra space**. The recursive version (reverse the first `k`, recurse on the
rest) is also O(n) time but uses **O(n / k) stack frames**, one per block.

## Reading the trigger

"Reverse every group of k" or "swap adjacent pairs" (the `k = 2` case) is the
fingerprint, and so is any variant where the block size follows a rule, such
as growing by one each time and reversing only blocks whose length is even.
In those variants the block length is computed first (and may be cut short by
the end of the list), but the machinery of `group_prev`, a bounded reversal
and a reconnect is identical. Distinguish it from the look-alikes: plain
reversal (lesson 01) has no neighbours to reconnect, "reverse positions
`left..right`" is a single block, and fast/slow pointers (lesson 02) are about
finding a position, not reshaping a segment. The word that tells you to
**verify before reversing** is "if fewer than k nodes remain, leave them as
they are": reversing first and undoing later is a bug magnet.

## Saying it in the interview

"A plain reversal handles one block; I'll apply it repeatedly. I put a dummy
node before the head so the first block has a predecessor. For each block I
first walk `k` nodes from `group_prev` to confirm they exist; if not, I stop and
leave the rest unchanged. Otherwise I reverse those `k` nodes with the standard
`prev/cur/next` loop, starting `prev` at the node after the block so the tail
reconnects itself, then point `group_prev` at the block's new front and advance
`group_prev` to its old first node. Each node is visited a constant number of
times, so time is O(n), and I use O(1) space; a recursive version would use
O(n / k) stack. I'd test `k = 1` (no change), `k` equal to the length, `k`
larger than the length, a length that isn't a multiple of `k`, and a single
node." Pitfalls interviewers watch for: reversing before confirming `k` nodes
exist, losing the old first node so the next block can't be linked, and
returning `head` instead of `dummy.next`.
