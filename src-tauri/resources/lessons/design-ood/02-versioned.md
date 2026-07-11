---
id: 02-versioned
unit: design-ood
subpattern: "Versioned / time-keyed store"
trigger_signals:
  - "You need to keep every value ever set for a key over time, and later ask \"what was this key's value at or before timestamp T?\" — not just its current value."
  - "Timestamps for a given key arrive in non-decreasing order, so each key's own history is already sorted — you can binary-search it instead of scanning it."
  - "A plain dict overwrites the old value on every set; here every version has to stay queryable."
worked_example: time-based-key-value-store
diagram: 02-versioned.diagram.json
quiz: 02-versioned.quiz.json
practice:
  - lru-cache
  - implement-trie-prefix-tree
recap:
  - 01-lru
follow_up:
  - "What if sets for a key could arrive out of timestamp order — would binary search over that key's list still be valid?"
  - "What if you needed the value at or *after* a timestamp instead of at-or-before — how does the bisect call change?"
---

## The one idea

Pair a dict with a second structure again — but this time the second
structure isn't a linked list, it's a **sorted list per key**. Map each key to
a growing list of `(timestamp, value)` pairs. Because every `set` for a given
key arrives with a timestamp at least as large as the one before it,
appending never breaks the sort — each key's history is sorted *for free*,
which is exactly what a binary search needs.

```python
from bisect import bisect_right

class TimeMap:
    def __init__(self) -> None:
        self.store: dict[str, list[tuple[int, str]]] = {}

    def set(self, key: str, value: str, timestamp: int) -> None:
        self.store.setdefault(key, []).append((timestamp, value))

    def get(self, key: str, timestamp: int) -> str:
        entries = self.store.get(key, [])
        # rightmost entry whose timestamp <= the query timestamp
        i = bisect_right(entries, (timestamp, chr(0x10FFFF))) - 1
        return entries[i][1] if i >= 0 else ""
```

The tuple `(timestamp, chr(0x10FFFF))` is a trick, not magic: it's a probe
that's guaranteed to sort just *after* any real `(timestamp, value)` pair with
that same timestamp, so `bisect_right` lands exactly one past the last entry
whose timestamp is `<= timestamp` — subtract one to get it.

## Why it beats brute force

Scanning a key's whole history backward until you find the first timestamp
`<=` the query is O(m) per `get`, for a key with m versions. But the list is
already sorted — sets only append, and timestamps only grow — so a linear
scan is throwing away information you already have. Binary search exploits
that sortedness directly: O(log m) per `get`, O(1) amortized per `set`.

This is the same composition move as the LRU cache (recap `01-lru`): there,
pairing a dict with a doubly linked list bought O(1) *recency* order; here,
pairing a dict with a sorted list buys O(log m) *time-travel* lookup. Same
shape, different second structure, chosen for the query the problem actually
asks.

## Reading the trigger

Say it out loud: **"multiple values over time for the same key; give me the
value as of a timestamp."** Whenever "as of time T" shows up over a history
that only grows forward, don't scan it — store each key's versions in a list
that stays sorted by construction, and reach for `bisect` to answer the query
in O(log m).
