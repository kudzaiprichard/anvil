---
id: 01-xor-tricks
unit: math-bit
subpattern: "XOR pairing tricks"
trigger_signals:
  - "Every value in the array appears an even number of times except exactly one — find the odd one out."
  - "You want O(1) extra space where a hash-map frequency count would cost O(n)."
  - "The problem talks about 'cancel pairs' or 'find the single/missing value' with no sorting required."
worked_example: single-number
diagram: 01-xor-tricks.diagram.json
quiz: 01-xor-tricks.quiz.json
practice:
  - missing-number
  - number-of-1-bits
recap: []
follow_up:
  - "What if two values were unpaired instead of one — could a single XOR pass still isolate them, or do you need an extra split step?"
  - "What if you needed the *missing* number from a range instead of the *duplicate* one — does the same cancellation idea still apply?"
---

## The one idea

XOR has two properties that make it a demolition tool for "find the one that
doesn't pair up": `x ^ x == 0` (a value cancels itself) and `x ^ 0 == x`
(anything XORed with nothing is unchanged). Because XOR is also commutative
and associative, the order you combine values in doesn't matter — every
duplicate pair collapses to zero no matter where it sits in the array, and
whatever is left over is the answer.

```python
def single_number(nums: list[int]) -> int:
    result = 0
    for x in nums:
        result ^= x            # duplicates cancel: a ^ a == 0
    return result
```

## Why it beats the obvious approach

The brute-force fix is a hash map: count every value, then scan the counts
for the one that isn't 2. That works, but it spends O(n) extra memory to
answer a question that doesn't actually need bookkeeping — you never need to
know *how many times* you've seen a value, only whether it eventually
cancels. A single accumulator XORed against every element does the same job
in one pass with **O(1) extra space**: no map, no sort, no second loop over
counts.

The same cancellation idea shows up whenever a problem's structure guarantees
that "noise" appears in pairs (or multiples that XOR away) and exactly one
value breaks the pattern. Spotting that structure — not the one line of code
— is the actual skill.

## Reading the trigger

Say the shape out loud: **"every element repeats except one, find it, and do
it in O(1) space."** That combination — pairing/cancelling structure plus a
tight space budget — is the signal to reach for XOR before you reach for a
dict. If the problem instead needed *counts* (how many times did each value
appear?), a hash map is still the right tool; XOR only wins when the answer
is "what's left after everything else cancels."
