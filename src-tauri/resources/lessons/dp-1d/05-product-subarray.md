---
id: 05-product-subarray
unit: dp-1d
subpattern: "Tracking running min & max (max product)"
trigger_signals:
  - "You need the best contiguous subarray product (or similarly sign-flippable running value), not a subsequence."
  - "A single negative number can flip your running best into your running worst and vice versa, so tracking only a running max isn't enough."
  - "The recurrence needs two rolling states per position — a running max AND a running min — rather than just one."
worked_example: maximum-product-subarray
diagram: 05-product-subarray.diagram.json
quiz: 05-product-subarray.quiz.json
practice:
  - house-robber
  - longest-increasing-subsequence
recap:
  - 01-fibonacci
follow_up:
  - "What if the array could contain zeros — how does a zero reset both cur_max and cur_min?"
  - "What if you needed the actual subarray (start and end indices), not just the product value — what extra state would you track?"
---

## The one idea

Maximum product subarray looks like a Kadane's-style running-max problem
(max subarray *sum*), but multiplication has a trap sum doesn't: a negative
number flips the sign of whatever it touches. The smallest (most negative)
running product at position `i-1` can become the *largest* at `i` if
`nums[i]` is negative. So you track **two** rolling states — `cur_max` and
`cur_min` — and at each step the new max might come from either.

## Why it beats the obvious approach

Brute force checks every contiguous subarray's product — O(n²), or worse if
each product is recomputed from scratch. The DP insight is that the best
product ending exactly at index `i` is either `nums[i]` alone, or `nums[i]`
times the best product ending at `i-1` — except "best" is ambiguous with
negative numbers, so we must carry the worst (most negative) product forward
too, since multiplying it by a negative `nums[i]` can produce the new best:

```python
def max_product(nums: list[int]) -> int:
    cur_max = cur_min = best = nums[0]
    for x in nums[1:]:
        candidates = (x, cur_max * x, cur_min * x)
        cur_max, cur_min = max(candidates), min(candidates)
        best = max(best, cur_max)
    return best
```

O(n) time, O(1) space — one pass, two rolling variables instead of the
running-max-only single variable that would suffice for a *sum*. Note that
this is a Fibonacci-shaped lookback (only `i-1` matters, exactly like House
Robber's rolling variables) — the new idea here isn't the lookback window,
it's that a single scalar of "best so far" isn't enough state; sign flips
force you to carry the extremes on both ends.

## Reading the trigger

Whenever a "best contiguous subarray" problem involves **multiplication**
(or any operation where a value can flip magnitude/sign), a single running
max is not enough — ask "could an extreme *minimum* become the new maximum
if the next element is negative?" If yes, carry both a running max and a
running min forward together, updating both from the same three candidates
at each step.
