---
id: 03-kadane
unit: greedy
subpattern: "Kadane's algorithm (best subarray ending here)"
trigger_signals:
  - "You need the best contiguous subarray (largest sum, longest run, best score) and the array contains negative values."
  - "At each index there are only two choices: extend the best run that ended at the previous index, or start fresh from here."
  - "Brute force tries every start and end in O(n^2) (or O(n^3)), but the answer for a run ending at i depends only on the run ending at i - 1."
worked_example: maximum-subarray
diagram: 03-kadane.diagram.json
quiz: 03-kadane.quiz.json
practice:
  - maximum-subarray-sum-with-one-deletion
  - longest-turbulent-subarray
recap:
  - 02-exchange-argument
follow_up:
  - "How would you also return the start and end indices of the best subarray, and where does `start` get reset?"
  - "Why can't you use the same trick for the maximum *product* subarray as-is, and what extra value do you carry (think about a negative times a negative)?"
---

## The one idea

Ask a smaller question at each position: **what is the best subarray sum that
must end exactly at index `i`?** Call it `ending`. A run ending at `i` is
either just `nums[i]` by itself, or the best run ending at `i - 1` with
`nums[i]` appended. So

`ending = max(x, ending_prev + x)`

If the previous run has a negative total, it only hurts; the greedy choice is
to drop it and start over at `x`. After computing `ending` for every `i`, the
answer is the largest `ending` you ever saw, because the best subarray has to
end *somewhere*. That is the whole algorithm: one decision per element, made
locally, with a global `best` tracked on the side.

## Why it beats the obvious approach

The brute force tries every start `i` and every end `j`, summing each range:
O(n^3), or O(n^2) with a running sum per start. Almost all of it is repeated
work, because the sum of `nums[i..j]` is just the sum of `nums[i..j-1]` plus
one element, which you've effectively already computed. Kadane's keeps only the
single number that matters for extending: the best run ending right here.

```python
def max_sub_array(nums: list[int]) -> int:
    best = ending = nums[0]            # not 0: an all-negative array must return its largest element
    for x in nums[1:]:
        ending = max(x, ending + x)    # extend the run, or restart because the prefix is negative
        best = max(best, ending)       # the best run ends somewhere; remember the largest
    return best
```

One pass, two `max` calls per element: **O(n) time**. Only `best` and `ending`
are stored, so **O(1) extra space**. The subtle point is the initialisation.
If you start `best` at `0` you silently allow the empty subarray, and an array
such as all negative numbers wrongly returns `0` instead of its largest
(least negative) element. Seed both variables with `nums[0]` and loop from the
second element.

## Reading the trigger

The signal is **"best contiguous segment"** plus a local choice at every step:
carry on or reset. Compare the look-alikes. A fixed or variable sliding
window works when the values are non-negative and growing the window can only
help or hurt predictably; with negatives, shrinking from the left does not
monotonically improve anything, which is why Kadane's resets instead.
A prefix-sum plus hash map answers "subarray summing to exactly a target", a
different question. Kadane's is also the simplest bridge from greedy into
dynamic programming: `ending` is a one-state DP, and you will meet the same
shape again in the dp-1d unit, for example keeping a running max *and* min to
handle products. Variants build on the same skeleton: one deletion allowed
means carrying two states (no deletion yet, one used), and a longest "zigzag"
run means the carried state is a run length reset by a comparison instead of
a sum reset by a negative.

## Saying it in the interview

"Brute force tries every pair of endpoints, O(n^2) with running sums. I'll
instead track the best sum of a subarray that ends at the current index. It's
either the element alone or the previous best plus the element, so I take the
max of those two; if the previous run is negative, it's discarded. A second
variable keeps the best value seen across all indices. That's one pass with O(1)
space, O(n) time. I'd test an all-negative array, a single element, a
mix where the best run is in the middle, and an all-positive array where the
answer is the total." Pitfalls interviewers watch for: initialising `best`
to `0` (wrong for all-negative input), forgetting that the subarray must be
non-empty, and resetting `ending` to `0` instead of to `x`, which has the same
all-negative bug in disguise.
