---
id: 04-scheduling
unit: heap
subpattern: "Greedy scheduling with a heap"
trigger_signals:
  - "You must greedily pick the 'currently most-needed' item each round, and picking it changes what's most-needed next round."
  - "There's a cooldown or spacing constraint (can't repeat the same item within a window) that forces you to temporarily set an item aside."
  - "Re-scanning for the current max frequency every round would be O(k) per round; you want O(log k) via a heap instead."
worked_example: task-scheduler
diagram: 04-scheduling.diagram.json
quiz: 04-scheduling.quiz.json
practice:
  - design-twitter
  - kth-largest-element-in-an-array
recap:
  - 01-top-k
follow_up:
  - "What if two tasks are tied for most-frequent — does the choice between them ever change the total schedule length?"
  - "What if the cooldown applied per pair of task types instead of globally — would a single side queue still be enough?"
---

## The one idea

Greedy task scheduling always runs the task with the highest *remaining*
count first — the running-maximum heap from two lessons ago, applied to
counts instead of raw values. The twist is the cooldown: once a task runs,
it can't run again for `n` slots, so a just-run task can't stay in the
heap where it would simply get picked again immediately. It waits in a
side queue instead, and re-enters the heap exactly when its cooldown ends.

```python
import heapq
from collections import Counter, deque

def least_interval(tasks: list[str], n: int) -> int:
    counts = Counter(tasks)
    heap = [-c for c in counts.values()]
    heapq.heapify(heap)
    cooldown = deque()   # (ready_time, remaining_count)
    time = 0
    while heap or cooldown:
        time += 1
        if heap:
            c = heapq.heappop(heap) + 1     # run the most frequent eligible task
            if c:
                cooldown.append((time + n, c))
        if cooldown and cooldown[0][0] == time:
            heapq.heappush(heap, cooldown.popleft()[1])
    return time
```

## Why it beats the obvious approach

Recomputing "which task has the most remaining instances?" by scanning
every count on every single time slot is O(k) per slot for k distinct task
types, and it recurs for the whole schedule length — the classic setup for
a heap. Popping the max-count task is O(log k); the only new idea is that a
just-run task is temporarily disqualified, so it sits in a small FIFO
cooldown queue and re-enters the heap the instant its cooldown time is
reached. The heap still always tells you, in O(log k), the best *eligible*
choice for right now — the cooldown queue just controls who's allowed to
compete.

## Reading the trigger

Say it: **"greedily run the most-frequent or most-urgent item each round,
with a spacing or cooldown rule before it can repeat."** Whenever a
schedule or selection problem keeps re-asking "what's most needed *right
now*" under a constraint that temporarily disqualifies the last thing you
picked, reach for a max-heap of counts plus a side queue for the
disqualified items — the same running-extremum heap from the stone-smashing
lesson, with one extra queue bolted on for the cooldown.
