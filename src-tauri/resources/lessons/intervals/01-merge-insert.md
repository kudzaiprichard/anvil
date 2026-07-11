---
id: 01-merge-insert
unit: intervals
subpattern: "Merge & insert intervals"
trigger_signals:
  - "You have a list of intervals — possibly unsorted, possibly overlapping — and need to collapse every pair that overlaps into one, i.e. \"merge all overlapping intervals.\""
  - "You're inserting one new interval into an already sorted, non-overlapping list and must fix up whatever it now touches."
  - "The problem talks about ranges and asks for the minimal set of intervals that still covers the same territory."
worked_example: merge-intervals
diagram: 01-merge-insert.diagram.json
quiz: 01-merge-insert.quiz.json
practice:
  - insert-interval
  - meeting-rooms-iii
recap: []
follow_up:
  - "What if intervals arrived one at a time, online, rather than all at once — could you keep the list merged incrementally instead of re-sorting every time?"
  - "What if you needed the total length covered by the merged intervals, not the merged list itself?"
---

## The one idea

Sort the intervals **by start**, then walk them once, keeping a running
"current merged interval." For each next interval: if its start falls at or
before the current merged interval's end, they overlap — stretch the current
interval's end to cover both. If it starts strictly after, the current merged
interval is finished — emit it and start a new one.

```python
def merge(intervals: list[list[int]]) -> list[list[int]]:
    intervals.sort(key=lambda iv: iv[0])   # preprocessing: sort by start
    merged: list[list[int]] = []
    for start, end in intervals:
        if merged and start <= merged[-1][1]:   # overlaps the last merged one
            merged[-1][1] = max(merged[-1][1], end)
        else:
            merged.append([start, end])          # starts a new group
    return merged
```

## Why it beats the obvious approach

Checking every pair of intervals for overlap is O(n²), and it's wasted work:
once intervals are sorted by start, any interval that could still overlap the
one you're building must come immediately next in that order — nothing
further back needs re-checking, and nothing further ahead can overlap
something you've already closed off. Sorting turns "does this overlap
*any* other interval" into "does this overlap the *one* interval I'm
currently extending," which is a single O(n) sweep after the O(n log n) sort.

Insert Interval is the same idea run in reverse: the list is *already*
sorted and non-overlapping, and you're merging in exactly one new interval.
You don't need to re-sort — walk the list once, copy every interval that
ends before the new one starts untouched, absorb every interval that
overlaps the new one (growing it the same way as above), then copy the rest.
It's the identical merge rule, just applied to a single insertion instead of
a full sort.

## Reading the trigger

Say the shape out loud: **"ranges that might overlap, collapse them into the
smallest equivalent set."** That's the tell for sort-and-sweep: sort by
start, then track one running interval and only ever compare it against the
next one in order — never against the whole list. If the list is already
sorted and you're only adding one new range, skip the sort and splice
directly; the merge rule at each boundary is unchanged.
