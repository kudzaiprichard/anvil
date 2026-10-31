---
id: 05-log-multivar
unit: big-o
subpattern: "Logarithms & multi-variable costs (log n, m·n, V+E)"
trigger_signals:
  - "Each step throws away half of what remains (or doubles what you've covered) — the cost is a logarithm, not a linear."
  - "The input has two independent sizes (rows and columns, vertices and edges, n items of length k) — one letter n is not enough."
  - "A constraint like n up to 10^5 or n up to 20 is hinting at the complexity the problem intends."
worked_example: sqrtx
diagram: 05-log-multivar.diagram.json
quiz: 05-log-multivar.quiz.json
practice:
  - count-negative-numbers-in-a-sorted-matrix
  - power-of-two
recap:
  - 01-complexity-classes
  - 02-amortized-space
  - 03-recursion-cost
  - 04-hidden-costs
follow_up:
  - "How would you compute the square root to a given decimal precision, and how does the number of iterations depend on that precision?"
  - "Could Newton's method beat plain binary search here, and roughly how fast does it converge?"
---

## The one idea

**O(log n) means the work shrinks (or grows) by a constant factor each step.**
Halve a search range and after `k` steps you have `n / 2^k` left, which hits 1
when `k = log2(n)`. Doubling is the same idea run backwards. That is why log n
is tiny in practice: about 30 steps for `n = 10^9`, about 20 for a million.

The second half of the lesson is a discipline: **never collapse two different
sizes into one `n`**. A grid with `m` rows and `n` columns costs O(m * n), not
O(n^2). A graph costs O(V + E). Name every independent size and keep each in
the formula.

## Why it beats the obvious approach

Integer Square Root asks for the largest `r` with `r * r <= x`. The obvious
approach tries `r = 1, 2, 3, ...` until the square exceeds `x`: that is about
`sqrt(x)` steps, so **O(sqrt x)**, around 46,000 steps for `x = 2 * 10^9`.
Better, but it ignores that the answer lives in a *sorted* space: if `r * r`
is too big, every larger `r` is too big as well. So halve the range:

```python
def my_sqrt(x: int) -> int:
    lo, hi = 0, x                     # the answer is somewhere in [0, x]
    while lo < hi:
        mid = (lo + hi + 1) // 2      # round up so the loop always shrinks
        if mid * mid <= x:
            lo = mid                  # mid works; the answer is mid or larger
        else:
            hi = mid - 1              # mid is too big; discard it and above
    return lo                         # largest r with r * r <= x
```

**Time:** the range `[lo, hi]` halves each iteration, so about `log2(x)`
iterations of O(1) work: **O(log x)**, at most ~31 steps for a 32-bit `x`.
**Space:** three integers, **O(1)**. (Python's big ints make `mid * mid`
safe; in languages with fixed-width integers, compare `mid <= x // mid`
to avoid overflow.)

Now the multi-variable side. Searching a sorted matrix for negatives by
checking every cell is O(m * n). Walking a staircase from the top-right
corner, moving left or down, takes at most `m + n` steps: **O(m + n)**. Each
step discards a whole row or column, and adding the two sizes (not
multiplying) is the whole win. Other shapes you should recognise: BFS or DFS
on a graph is O(V + E); a heap of size `k` over `n` items is O(n log k);
sorting `n` strings of length `k` is O(n * k * log n) comparisons-by-length.

## Reading the trigger

Two clues. First, **the wording of the shrinkage**: "sorted", "halve",
"each digit", "powers of two" point to log n. Counting the digits or bits of
a number is also logarithmic in its value. Second, **the constraints tell you
the target**. As a rule of thumb for about 10^8 simple operations per second:
`n <= 20` allows exponential (O(2^n)); `n <= 500` allows O(n^3); `n <= 5,000`
allows O(n^2); `n <= 10^5` needs about O(n log n) or O(n); `n <= 10^9` or
larger demands O(log n) or O(1). If a problem gives two limits, such as `m, n`
up to 200 each, the target O(m * n) is 40,000 operations and fine. Do not
mistake O(log n) for O(n log n): the first has no loop over the data.

## Saying it in the interview

"Counting up from 1 is O(sqrt x). But squares are monotonic, so I can binary
search the answer between 0 and x, discarding half the range each time. That
is O(log x) time and O(1) space." Then connect to the constraints: "With x up
to about 2 billion, that's roughly 31 iterations." Pitfalls interviewers
watch for: an infinite loop from rounding `mid` down when you set `lo = mid`,
and integer overflow on `mid * mid`. For multi-variable problems, state the
variables out loud: "this is O(m + n), where m is rows and n is columns, not
O(n)". Edge cases to name: `x = 0`, `x = 1`, and perfect squares.
