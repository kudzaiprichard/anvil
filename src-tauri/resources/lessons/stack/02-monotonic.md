---
id: 02-monotonic
unit: stack
subpattern: "Monotonic stack (next greater / warmer)"
trigger_signals:
  - "For each element you need the next one that's strictly greater (warmer, taller, bigger) — a \"next greater element\" style question."
  - "The obvious fix is a nested loop that rescans forward from every index looking for the first qualifying value — O(n^2)."
  - "An element can be resolved out of order: the moment a bigger value shows up, everything smaller still waiting gets answered at once."
worked_example: daily-temperatures
diagram: 02-monotonic.diagram.json
quiz: 02-monotonic.quiz.json
practice:
  - two-sum
  - valid-parentheses
recap:
  - 01-lifo-parsing
follow_up:
  - "What if you needed the next *smaller* element instead — does the stack's ordering invariant just flip?"
  - "What if the array were circular, so the next greater element might wrap around to the front?"
---

## The one idea

A monotonic stack keeps its contents in strictly increasing (or decreasing)
order at all times. Push an index; the instant a new value would *break* that
order, pop everyone it breaks the order for — each one you pop has just found
its answer, because the new value is exactly the "next greater" thing they
were waiting on.

For daily temperatures, the stack holds indices of days whose warmer day
hasn't shown up yet, kept in decreasing-temperature order top to bottom is not
required — what matters is: while the new day is warmer than the day on top,
that top day is done.

```python
def daily_temperatures(temps: list[int]) -> list[int]:
    answer = [0] * len(temps)
    stack: list[int] = []  # indices still waiting for a warmer day
    for i, t in enumerate(temps):
        while stack and temps[stack[-1]] < t:
            j = stack.pop()
            answer[j] = i - j
        stack.append(i)
    return answer
```

## Why it beats the obvious approach

The brute-force answer scans forward from every index until it finds a
warmer day — O(n^2) on a strictly decreasing run, since each day rescans
almost the whole rest of the array. The monotonic stack never rescans: each
index is pushed exactly once and popped at most once, so the total work
across the *whole* array is O(n), even though any single day might trigger
several pops.

This is still the same LIFO idea from lifo-parsing, just aimed at a
different question. There, popping meant "these two just *matched*." Here,
popping means "this one just got *resolved* by whatever's arriving now" — the
stack isn't tracking pairs, it's tracking "who's still open and unanswered,"
which is exactly what makes a single forward pass enough.

## Reading the trigger

Say it out loud: "for each element, find the next one — scanning left to
right — that satisfies some comparison, and a naive rescan from every index
would be O(n^2)." Whenever the answer for an earlier element only becomes
knowable once a later, more-extreme element shows up, that's the monotonic
stack signal — keep the "still waiting" indices on a stack and resolve them
in a burst the moment they're beaten.
