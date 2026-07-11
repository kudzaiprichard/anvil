---
id: 03-partition-scan
unit: two-pointers
subpattern: "Fix one, two-pointer the rest (3Sum)"
trigger_signals:
  - "You need k-sum for k >= 3 — fix the first k-2 indices and reduce the remaining two to a sorted-array two-pointer scan."
  - "The problem wants every unique combination, not just one hit — sortedness plus pointer-skipping lets you dedupe without a set."
  - "Brute force is three (or more) nested loops, O(n^3); fixing one index and two-pointering the rest drops it to O(n^2)."
worked_example: 3sum
diagram: 03-partition-scan.diagram.json
quiz: 03-partition-scan.quiz.json
practice:
  - container-with-most-water
  - two-sum-ii-input-array-is-sorted
recap:
  - 02-sorted-pair-sum
follow_up:
  - "What if the target were k values instead of 3 — could you recurse this same fix-one-and-reduce idea down to a base case of 2?"
  - "What if you only needed to know whether *any* triple exists, not enumerate all of them — does the scan still need the duplicate-skipping logic?"
---

## The one idea

3Sum asks for every triple that sums to zero. A triple loop is the obvious
answer and it's O(n^3). Instead, **fix one index** as the first number of the
triple, and hand the other two off to last lesson's sorted-array two-pointer
scan, now hunting for a pair that sums to `-nums[i]`. Sorting first also
makes duplicate triples easy to skip: equal values sit next to each other,
so you just refuse to re-fix (or re-select) a value you already used.

```python
def three_sum(nums: list[int]) -> list[list[int]]:
    nums.sort()
    res, n = [], len(nums)
    for i in range(n - 2):
        if i > 0 and nums[i] == nums[i - 1]:
            continue                       # skip a repeated "fixed" value
        lo, hi, target = i + 1, n - 1, -nums[i]
        while lo < hi:
            total = nums[lo] + nums[hi]
            if total == target:
                res.append([nums[i], nums[lo], nums[hi]])
                lo += 1
                hi -= 1
                while lo < hi and nums[lo] == nums[lo - 1]:
                    lo += 1                # skip a repeated lo value
                while lo < hi and nums[hi] == nums[hi + 1]:
                    hi -= 1                # skip a repeated hi value
            elif total < target:
                lo += 1
            else:
                hi -= 1
    return res
```

## Why it beats the obvious approach

Nested triple loops re-derive the same "find a pair summing to X" question
from scratch for every choice of the third value, at O(n^3). Fixing one
index turns that into n separate runs of an O(n) two-pointer scan — exactly
the sub-pattern from the previous lesson — for an overall O(n^2). The sort
that makes duplicates adjacent costs O(n log n), which the O(n^2) scan
dominates. This is the general move behind every k-sum problem: **reduce
k to k-1 by fixing an index, until you bottom out at the 2-pointer case**.

## Reading the trigger

Say it out loud: **"sorted, find every combination of three (or more)
numbers hitting a target."** Whenever a sum problem has more than two free
slots, don't write another nested loop — fix all but the last two indices
and finish with the sorted-array two-pointer scan you already know.
