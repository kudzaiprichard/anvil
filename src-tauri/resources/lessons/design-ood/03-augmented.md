---
id: 03-augmented
unit: design-ood
subpattern: "Augmented data-structure design (O(1) min)"
trigger_signals:
  - "You need push/pop plus another query — min, max, or a running aggregate — over only what's *currently* on the stack, answered in O(1)."
  - "Recomputing that aggregate by rescanning after every pop would cost O(n) per pop instead of O(1)."
  - "The aggregate depends only on what's still on the stack right now, not the full push history — once a value pops off, it stops counting."
worked_example: min-stack
diagram: 03-augmented.diagram.json
quiz: 03-augmented.quiz.json
practice:
  - implement-trie-prefix-tree
  - design-add-and-search-words-data-structure
recap:
  - 01-lru
  - 02-versioned
follow_up:
  - "What if you needed O(1) max *and* min at the same time — does one shadow stack cover both, or do you need two?"
  - "What if pushes and pops could happen from either end (a deque) — does the same shadow-structure trick still give O(1)?"
---

## The one idea

Same composition move as the rest of this unit, applied to a stack: don't
recompute the extra query, *carry* it. Pair the main stack with a shadow
stack that stores, at each push, the running minimum of everything on the
main stack so far. Push onto both together; pop from both together. The
shadow stack's top is always exactly the minimum of what's left, because
nothing that's already been popped can still be the minimum.

```python
class MinStack:
    def __init__(self) -> None:
        self._data: list[int] = []
        self._mins: list[int] = []       # _mins[i] = min(_data[:i+1])

    def push(self, val: int) -> None:
        self._data.append(val)
        floor = self._mins[-1] if self._mins else val
        self._mins.append(min(val, floor))

    def pop(self) -> None:
        self._data.pop()
        self._mins.pop()

    def top(self) -> int:
        return self._data[-1]

    def get_min(self) -> int:
        return self._mins[-1]
```

## Why it beats brute force

Without the shadow stack, `get_min` has two bad options: scan the whole
stack on every call (O(n) per query), or, after every `pop`, rescan what
remains to find the new minimum (also O(n)). Snapshotting "the minimum
including me" at push time turns `get_min` into a single O(1) peek, and
`pop` stays O(1) too — popping the top of both lists together *automatically*
restores the correct earlier minimum, because it was sitting right
underneath the whole time. The price is O(n) extra space for the shadow
stack, traded for O(1) time on every operation.

You've now composed structures three ways in this unit: a dict with a
doubly linked list to make *recency* O(1) (recap `01-lru`), a dict with a
sorted list to make *time-travel* O(log m) (recap `02-versioned`), and now a
stack with a shadow stack to make a *running aggregate* O(1). The move is
always the same — pair your primary structure with a second one purpose-built
to answer the one extra question fast, and keep the two updated in lockstep.

## Reading the trigger

Say it out loud: **"push/pop, plus a query over only what's currently there,
and it all has to be O(1)."** Whenever the extra fact you need depends solely
on the current contents — not the full push history, since popped values
stop counting — carry a snapshot of that fact alongside every push in a
parallel structure, and update both together.
