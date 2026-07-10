---
id: 01-top-k
unit: heap
subpattern: "Top-K with a heap"
trigger_signals:
  - "You need the k largest (or k smallest) elements out of a stream or array, not the full order — sorting everything is overkill."
  - "New elements keep arriving (a stream, repeated `add` calls) and you must answer \"what's the current k-th largest?\" after each one."
  - "You only care about a fixed-size \"top k\" at any moment, so you want O(log k) per update instead of O(n) or O(n log n) per query."
worked_example: kth-largest-element-in-a-stream
diagram: 01-top-k.diagram.json
quiz: 01-top-k.quiz.json
practice:
  - top-k-frequent-elements
  - kth-largest-element-in-an-array
recap: []
follow_up:
  - "What if you needed the k smallest instead of the k largest — does the heap orientation flip?"
  - "What if elements could also be removed from the stream — can you still keep O(log k) per operation?"
---

## The one idea

A heap keeps exactly the k best elements seen so far, updating in O(log k)
per insertion, without ever sorting anything. In Python, `heapq` is always a
**min-heap**, so if you keep a min-heap capped at size k, the smallest of
your "top k" always sits at the root — ready to be evicted the instant a
bigger element shows up.

```python
import heapq

class KthLargest:
    def __init__(self, k: int, nums: list[int]):
        self.k = k
        self.heap = nums[:]
        heapq.heapify(self.heap)
        while len(self.heap) > k:
            heapq.heappop(self.heap)

    def add(self, val: int) -> int:
        heapq.heappush(self.heap, val)
        if len(self.heap) > self.k:
            heapq.heappop(self.heap)
        return self.heap[0]   # k-th largest = smallest of the current top k
```

## Why it beats the obvious approach

The brute-force answer is to keep the whole stream and re-sort on every
query — O(n log n) per `add`, growing as the stream grows. But you never
need the *full* order, only "am I in the current top k, and if so, who is
my weakest member?" A size-k min-heap answers both in O(log k): pushing a
value that doesn't beat the current worst member does nothing useful once
you pop it back off, and pushing a value that does beat it evicts the
weakest member automatically. The root of that min-heap, by construction,
*is* the k-th largest value seen so far — no scan required.

## Reading the trigger

Say it out loud: **"running top-k / k-th largest, values keep arriving."**
Whenever the question is about a *fixed-size best-k set* under insertions —
not "sort everything, then look" — reach for a size-k heap before a full
sort or a repeated max-scan. Min-heap when you want the k largest; max-heap
(negate the values in Python) when you want the k smallest.
