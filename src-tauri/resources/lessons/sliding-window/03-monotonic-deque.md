---
id: 03-monotonic-deque
unit: sliding-window
subpattern: "Monotonic deque (window max / min)"
trigger_signals:
  - "A window of fixed length k slides across the array and you need the maximum (or minimum) element of every position — not a sum."
  - "A running sum can't help because when the current maximum leaves the window you need the *next best* element, and you don't know who that is."
  - "A DP or greedy transition asks for the best value among the last k states (\"best over the previous k positions\"), so every step is itself a sliding-window max/min."
worked_example: sliding-window-maximum
diagram: 03-monotonic-deque.diagram.json
quiz: 03-monotonic-deque.quiz.json
practice:
  - longest-continuous-subarray-with-absolute-diff-less-than-or-equal-to-limit
  - jump-game-vi
recap:
  - 01-fixed-window
  - 02-dynamic-window
follow_up:
  - "Your deque stores indices, not values. Could you store values only? What would you lose, and what exactly do you need the index for?"
  - "How would you track the window maximum *and* minimum together to keep the spread under a limit, and how does that make a variable window possible?"
---

## The one idea

A running sum survives a slide because subtraction undoes addition. A
maximum has no such inverse: when the current champion leaves the window you
must know who the runner-up was. The trick is to *never throw away a
runner-up that could still matter* and to throw away everyone who never can.

Keep a **deque of indices** whose values are strictly decreasing from front
to back. The front is always the window maximum. When a new element `x`
arrives, any element at the back that is `<= x` is older **and** no larger,
so it will leave the window earlier than `x` and can never be the maximum
while `x` is present — pop it. Then push `x`'s index. Separately, if the
front index has slipped out of the window (`front <= i - k`), pop it from the
front. For Sliding Window Maximum, read the front after every step once the
first `k` elements are in.

## Why it beats the obvious approach

The brute force scans all `k` elements of every window with `max()` — O(n·k).
A max-heap does better, O(n log k), but stale entries (indices already out of
the window) have to be discarded lazily and each push or pop costs a log
factor. The deque reaches O(n) because it exploits an ordering the heap
ignores: age. Older and smaller is useless.

```python
from collections import deque

def max_sliding_window(nums: list[int], k: int) -> list[int]:
    dq = deque()                          # indices; nums[dq[0]] > nums[dq[1]] > ...
    result = []
    for i, x in enumerate(nums):
        while dq and nums[dq[-1]] <= x:   # back is older and not bigger: dead weight
            dq.pop()
        dq.append(i)
        if dq[0] <= i - k:                # front slid out of the window
            dq.popleft()
        if i >= k - 1:                    # first full window reached
            result.append(nums[dq[0]])    # front is the window max
    return result
```

Each index is appended exactly once and removed at most once — from the back
by a bigger newcomer, or from the front by expiry — so the total pops across
the whole run is at most `n`, and the inner `while` is amortised O(1) per
element: **O(n) time**. The deque never holds more than `k` indices (anything
older has expired), so extra space is **O(k)**, plus the output list.

## Reading the trigger

The fingerprint is **"best of the last k" asked repeatedly**. Compare the
look-alikes. A fixed-window *sum or count* is plain sliding window (lesson
01). A *variable* window that grows and shrinks on a condition is lesson 02,
but when that condition is "max minus min must stay within a limit" you need
the window's max and min on demand, so you run two deques inside the
variable window. The monotonic **stack** (stack unit, `02-monotonic`) is the
same ordering idea with only one open end: it answers "next greater element to
my right" and never expires old items by position. The deque adds the second
end precisely so that elements can expire from the front. Finally, DP
recurrences like "reach cell i from any of the last k cells" hide a window
max inside the transition; swapping a heap or a scan for a deque turns O(n·k)
or O(n log n) into O(n).

## Saying it in the interview

"The naive way is to take the max of each window, which is O(n·k). A heap gets
O(n log k) but has to skip stale entries. The insight is that if a newer
element is at least as large as an older one, the older one can never be the
answer again, so I can discard it. I keep a deque of indices with decreasing
values: before pushing the new index I pop smaller-or-equal values off the
back, and I pop the front when its index is no longer inside the window. The
front is the max, so each step is O(1) amortised. Every index is pushed and
popped at most once, giving O(n) time and O(k) space. I'd test a window size
of 1, `k` equal to the array length, an all-equal array and a strictly
decreasing array." Pitfalls interviewers watch for: storing values instead of
indices (you can't tell when the front expired), popping with `<` instead of
`<=` (equal values pile up, harmless for correctness but it loses the
strictly-decreasing invariant), and checking expiry with the wrong boundary —
the window is `i - k + 1 .. i`, so the front expires when it is `<= i - k`.
