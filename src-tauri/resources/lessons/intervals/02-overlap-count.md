---
id: 02-overlap-count
unit: intervals
subpattern: "Counting overlaps / non-overlapping"
trigger_signals:
  - "You need the maximum number of intervals that can coexist without overlapping — or, equivalently, the minimum number you'd have to remove so none overlap."
  - "The question wants a *count*, not a merged range — \"how many intervals/meetings overlap\" or \"how many must be discarded,\" not \"what does the covered range look like.\""
  - "Sorting by **end** (not start) exposes a greedy choice: always keep whichever interval frees up the earliest room for what comes after it."
worked_example: non-overlapping-intervals
diagram: 02-overlap-count.diagram.json
quiz: 02-overlap-count.quiz.json
practice:
  - meeting-rooms-iii
  - jump-game
recap:
  - 01-merge-insert
follow_up:
  - "What if you needed the actual intervals kept, not just the count removed?"
  - "meeting-rooms-iii adds a live room count that changes as meetings start and end — how would you track that alongside the same greedy scan?"
---

## The one idea

The last lesson sorted by **start** to fuse overlapping ranges together. This
one sorts by **end** to do the opposite — pick the largest possible set of
intervals that *don't* overlap at all. Walk the end-sorted list once, keeping
`prev_end`, the end time of the last interval you decided to keep. An
interval is compatible if its start is at or after `prev_end`; otherwise it
overlaps a kept interval and must be dropped.

```python
def erase_overlap_intervals(intervals: list[list[int]]) -> int:
    intervals.sort(key=lambda iv: iv[1])   # preprocessing: sort by end
    prev_end = float("-inf")
    removed = 0
    for start, end in intervals:
        if start >= prev_end:              # compatible: keep it
            prev_end = end
        else:                              # overlaps the last kept interval
            removed += 1
    return removed
```

## Why it beats the obvious approach

Trying every subset of intervals to find the largest non-overlapping one is
O(2ⁿ). Sorting by end makes a greedy, one-pass choice provably optimal: among
all intervals you could keep next, the one that ends *earliest* leaves the
most room for everything still to come — keeping any interval with a later
end can only ever block more future options, never fewer. That's the same
exchange argument the greedy unit used for jump game's "farthest reach
dominates" — here the dominant quantity is the earliest `prev_end`, and any
interval failing to beat it is strictly worse to keep. One O(n log n) sort
plus an O(n) scan replaces the exponential search.

Sorting by start, as in the merge lesson, would not work here — a long
interval that starts early but ends late can block far more future intervals
than a short one that starts slightly later. End time, not start time, is
what a *count/selection* problem needs to sort by.

## Reading the trigger

Say the shape out loud: **"maximum intervals that fit without overlapping"**
or **"minimum removed so none overlap."** That's the tell to sort by end and
greedily keep whatever is compatible with the last kept interval's end — the
opposite preprocessing key from the merge lesson's sort-by-start, because
counting/selecting cares about what an interval blocks *next*, not what it
covers now.
