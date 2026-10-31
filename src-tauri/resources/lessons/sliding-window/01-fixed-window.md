---
id: 01-fixed-window
unit: sliding-window
subpattern: "Fixed-size sliding window"
trigger_signals:
  - "The problem names an exact window length — \"every subarray of length k\", \"any k consecutive days\", \"a substring of size k\"."
  - "You need an aggregate over each of those windows (sum, average, count, character frequencies) and then the best one or how many qualify."
  - "Brute force re-scans all k elements for every window — O(n·k) — even though neighbouring windows share k − 1 elements."
worked_example: maximum-average-subarray-i
diagram: 01-fixed-window.diagram.json
quiz: 01-fixed-window.quiz.json
practice:
  - permutation-in-string
  - find-all-anagrams-in-a-string
  - maximum-number-of-vowels-in-a-substring-of-given-length
follow_up:
  - "What if you needed the maximum *element* of every k-window instead of the sum — can a single running number still capture it when the current max slides out?"
  - "What if the window had no fixed length and instead had to satisfy a condition (like \"sum at least target\") — what decides when the left edge moves?"
---

## The one idea

A fixed-size sliding window keeps exactly `k` elements in view and moves
one step at a time. Each step changes the window by just two elements: one
enters on the right and one leaves on the left. So instead of recomputing
the window's aggregate from scratch, you **update** it: add the newcomer,
subtract the one that left. The left edge always trails the right edge by
exactly `k − 1` positions — they move in lockstep, and the window's size
never changes once it's full.

For Maximum Average Subarray I, the aggregate is a running sum. Because
every window has the same length `k`, the window with the largest sum is
also the one with the largest average — so track the best *sum* while
sliding and divide by `k` once at the end.

## Why it beats the obvious approach

The brute-force answer sums every length-`k` window independently: about
`n` windows, `k` additions each — O(n·k). Almost all of that work is
repeated: windows starting at `i` and `i + 1` share `k − 1` elements, so
summing them again from scratch throws away a result you already had.

```python
def find_max_average(nums: list[int], k: int) -> float:
    window = sum(nums[:k])            # build the first full window once
    best = window
    for right in range(k, len(nums)):
        window += nums[right]         # element entering on the right
        window -= nums[right - k]     # element leaving on the left
        best = max(best, window)
    return best / k                   # same length everywhere: max sum = max average
```

Every element is added to the window exactly once and subtracted at most
once, so the scan is O(n) time with O(1) extra space — independent of `k`.

## Reading the trigger

Say the problem out loud: **"every window of exactly length k, and I need
something computed over each one."** That's the fixed-window fingerprint.
The same skeleton works for any aggregate you can update in O(1) per step:
a sum, a count of vowels, or a 26-slot letter-frequency array (compare it
against a target frequency array to spot anagrams of a pattern). Two common
slips to watch for in an interview: forgetting to subtract the element that
left (the "window" silently grows), and an off-by-one when deciding the
window is full — it holds `right - left + 1` elements, not `right - left`.
If the window's length isn't given and instead depends on a condition,
you're in variable-window territory — that's the next lesson.
