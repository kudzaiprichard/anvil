---
id: 02-bit-counting
unit: math-bit
subpattern: "Bit counting & masks"
trigger_signals:
  - "You need the number of set (1) bits in a value, or a full 0..n popcount table."
  - "The current answer can be built from a smaller, already-computed answer via a mask like `i & (i-1)` or `i >> 1`."
  - "A naive per-number bit-by-bit loop over n values costs O(n log n) total, and you want O(n) via a DP recurrence."
worked_example: counting-bits
diagram: 02-bit-counting.diagram.json
quiz: 02-bit-counting.quiz.json
practice:
  - number-of-1-bits
  - reverse-bits
recap:
  - 01-xor-tricks
follow_up:
  - "What if you only needed the parity of the bit count (odd or even), not the exact number — could you shortcut with a single XOR fold like the last lesson?"
  - "What if numbers arrived one at a time in a stream, instead of a fixed range 0..n — could you still reuse a smaller answer for each new value?"
---

## The one idea

`i & (i - 1)` clears the lowest set bit of `i` — it's a cheap mask, not a
loop. That means the bit-count of `i` is always exactly one more than the
bit-count of a *smaller* number you've already computed: `dp[i] = dp[i & (i
- 1)] + 1`. Build the table from `0` upward and every entry is O(1) work
that reuses a prior answer, the same way a Fibonacci table reuses `dp[i-1]`
and `dp[i-2]`.

```python
def count_bits(n: int) -> list[int]:
    dp = [0] * (n + 1)
    for i in range(1, n + 1):
        dp[i] = dp[i & (i - 1)] + 1   # clear the lowest set bit, reuse its answer
    return dp
```

## Why it beats the obvious approach

The naive fix counts each number's bits independently — shift and mask with
`&1` until the value is zero — which costs O(log i) per number and O(n log
n) across the whole 0..n range. That's wasted work: `i` and `i & (i - 1)`
differ by exactly one bit, so if you already know the smaller number's count
you get `i`'s count for free. The mask trades a whole inner loop for a
single AND and a table lookup, dropping the total to **O(n)** time and O(n)
space for the table.

The same shape — `dp[i] = dp[smaller(i)] + constant` — is the general recipe
for any "compute this for every value in a range" bit problem: find the mask
that shrinks `i` to something you've already solved.

## Reading the trigger

Say it out loud: **"count set bits for every number from 0 to n."** Repeated,
related work across a range is the tell — instead of counting each number
from scratch, look for a bitwise mask (`i & (i-1)` to drop the lowest set
bit, or `i >> 1` to drop the lowest bit entirely) that reduces `i` to a
smaller index whose answer you've already stored.
