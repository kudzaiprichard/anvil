---
id: 01-opposite-ends
unit: two-pointers
subpattern: "Opposite-ends converging pointers"
trigger_signals:
  - "You're checking a whole-sequence symmetric property (is this a palindrome, is it mirrored) and don't want to build a reversed copy."
  - "The condition only depends on the pair currently at the extremes, so shrinking inward from both ends can never skip a pair that matters."
  - "Brute force costs O(n) extra space (reverse-and-compare) or worse; you want a single pass with O(1) extra space."
worked_example: valid-palindrome
diagram: 01-opposite-ends.diagram.json
quiz: 01-opposite-ends.quiz.json
practice:
  - container-with-most-water
  - two-sum
recap: []
follow_up:
  - "What if you had to return the *first* mismatching pair instead of a yes/no answer?"
  - "What if the sequence were a linked list instead of an array — could you still start pointers at both ends?"
---

## The one idea

Two pointers start at opposite ends of the sequence — `lo` at the front, `hi`
at the back — and walk toward each other, checking one condition per step.
The moment they disagree, you're done: return early. The moment they meet or
cross, every pair has been checked and the property holds. Neither pointer
ever needs to look backward, because the pair it's about to check is the
*only* pair left that could still break the property.

```python
def is_palindrome(s: str) -> bool:
    lo, hi = 0, len(s) - 1
    while lo < hi:
        while lo < hi and not s[lo].isalnum():
            lo += 1                      # skip punctuation/spaces
        while lo < hi and not s[hi].isalnum():
            hi -= 1
        if s[lo].lower() != s[hi].lower():
            return False                 # mismatch: stop immediately
        lo += 1
        hi -= 1
    return True
```

## Why it beats the obvious approach

The naive answer to "is this a palindrome?" is to build a cleaned, reversed
copy of the sequence and compare it to the original — correct, but it pays
O(n) extra space for a copy you throw away immediately. The opposite-ends
walk needs only two integers, `lo` and `hi`. Each step retires exactly one
pair of positions from consideration, so the whole scan is O(n) time and
**O(1) extra space** — you never materialize a reversed string at all.

This is the same shape you'll meet whenever the question is "does this
symmetric condition hold across the whole thing?" — checking a mirrored
array, verifying a sequence reads the same from both directions, or (later)
deciding which of two boundary elements to move based on a comparison rather
than pure symmetry.

## Reading the trigger

Say the problem out loud: **"check whether the sequence is the same read
from both ends."** Whenever the condition you're testing only cares about the
element at each extreme — not the middle, not order beyond that pair — reach
for two pointers converging from the outside in, before you reach for a
reversed copy or a second data structure.
