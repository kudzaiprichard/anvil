---
id: 01-lru
unit: design-ood
subpattern: "Hash map + doubly linked list (LRU cache)"
trigger_signals:
  - "You need O(1) get/put on a fixed-capacity cache, and eviction must remove the *least recently used* entry — not the oldest inserted, not random."
  - "A plain dict gives O(1) lookup but no O(1) way to know usage order; a plain list gives order but O(n) lookup and O(n) removal from the middle."
  - "Every get or put has to instantly promote that key to 'most recently used' without rebuilding or rescanning anything."
worked_example: lru-cache
diagram: 01-lru.diagram.json
quiz: 01-lru.quiz.json
practice:
  - min-stack
  - time-based-key-value-store
recap: []
follow_up:
  - "What if capacity could grow or shrink at runtime — do the dict and the linked list still stay in sync for free?"
  - "What if you needed LFU (least *frequently* used) instead of LRU — does the same doubly-linked-list trick still apply, or do you need a different second structure?"
---

## The one idea

No single built-in structure gives you both O(1) lookup-by-key *and* O(1)
reordering-by-recency. So don't pick one — compose two. A `dict` maps each key
straight to its node (O(1) lookup); a doubly linked list keeps those nodes in
recency order and lets you unlink or relink any node in O(1), because you
never have to shift anything or scan for a position.

```python
class Node:
    def __init__(self, key: int, val: int) -> None:
        self.key, self.val = key, val
        self.prev: "Node | None" = None
        self.next: "Node | None" = None

class LRUCache:
    def __init__(self, capacity: int) -> None:
        self.cap = capacity
        self.map: dict[int, Node] = {}
        self.head, self.tail = Node(0, 0), Node(0, 0)  # sentinels
        self.head.next, self.tail.prev = self.tail, self.head

    def _remove(self, n: Node) -> None:
        n.prev.next, n.next.prev = n.next, n.prev

    def _push_front(self, n: Node) -> None:  # front = most recently used
        n.next, n.prev = self.head.next, self.head
        self.head.next.prev = n
        self.head.next = n

    def get(self, key: int) -> int:
        if key not in self.map:
            return -1
        n = self.map[key]
        self._remove(n)
        self._push_front(n)          # touched -> promote to MRU
        return n.val

    def put(self, key: int, val: int) -> None:
        if key in self.map:
            self._remove(self.map[key])
        n = Node(key, val)
        self.map[key] = n
        self._push_front(n)
        if len(self.map) > self.cap:
            lru = self.tail.prev      # tail side = least recently used
            self._remove(lru)
            del self.map[lru.key]
```

## Why it beats the obvious approach

A dict alone can tell you *what* a key maps to, but not *when* it was last
touched, short of tagging every entry with a timestamp and scanning all of
them on eviction — O(n) per put. A plain list can track order, but finding a
given key in it, and removing it from the middle to re-insert at the front,
is also O(n). The doubly linked list fixes exactly that: every node knows its
own neighbors, so once the dict hands you the node in O(1), unlinking and
relinking it costs O(1) too — no shifting, no scanning. Sentinel head/tail
nodes remove the edge cases (empty list, removing the true head or tail) from
every call site.

## Reading the trigger

Say it out loud: **"fixed-capacity cache, O(1) get/put, evict the least
recently used on overflow."** Whenever a problem needs fast lookup *and* fast
reordering by recency at the same time, reach for this composite: a dict for
the O(1) address book, a doubly linked list for the O(1) recency order,
kept in lockstep on every access.
