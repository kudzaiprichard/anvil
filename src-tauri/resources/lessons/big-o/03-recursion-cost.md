---
id: 03-recursion-cost
unit: big-o
subpattern: "Recursion cost: call stacks & recursion trees"
trigger_signals:
  - "A function calls itself more than once per call (fib(n-1) + fib(n-2)) — the work may fan out exponentially."
  - "You see the same arguments computed again and again in the recursion — a sign memoization can collapse the tree."
  - "You're asked for space complexity of a recursive solution that allocates nothing — the call stack itself is the answer."
worked_example: fibonacci-number
diagram: 03-recursion-cost.diagram.json
quiz: 03-recursion-cost.quiz.json
practice:
  - n-th-tribonacci-number
  - reverse-string
recap:
  - 01-complexity-classes
  - 02-amortized-space
follow_up:
  - "Can you get Fibonacci below O(n) time, for example with fast matrix exponentiation, and what would the cost be?"
  - "If the input were n = 10 million, would your recursive memoized version even run in Python, and why or why not?"
---

## The one idea

To cost a recursive function, **draw its recursion tree**. Every call is a
node; every recursive call it makes is a child. Then ask two questions:

1. **How many nodes are there?** Roughly `branching_factor ^ depth` if the tree
   is full, and that count times the work per node is your time.
2. **How deep does the tree go?** The *depth* is how many calls are alive at
   once on the call stack, and every live call holds a frame, so depth is
   your **space**, even when the function creates no lists at all.

The Fibonacci Number problem is the textbook case: `fib(n)` calls `fib(n-1)`
and `fib(n-2)`, so the tree branches by two and is about `n` levels deep.

## Why it beats the obvious approach

The obvious recursion, `fib(n) = fib(n-1) + fib(n-2)`, builds a tree with
roughly `2^n` nodes, so it is **O(2^n)** time (the tight bound is about
`1.6^n`, but "exponential, around 2^n" is what interviewers want). The waste
is obvious in the tree: `fib(3)` is recomputed from scratch under many
different parents. The tree holds only `n + 1` *distinct* subproblems, yet it
solves them exponentially many times.

Memoization caches each answer the first time it is computed. The second
visit to `fib(k)` becomes a lookup, so the tree collapses to one real
computation per distinct argument. Going bottom-up avoids recursion entirely:

```python
def fib(n: int) -> int:
    if n < 2:
        return n
    prev, cur = 0, 1                  # fib(0), fib(1)
    for _ in range(2, n + 1):         # n - 1 iterations
        prev, cur = cur, prev + cur   # slide the two-value window forward
    return cur
```

**Time:** one loop of about `n` steps, each O(1) arithmetic, so **O(n)**
(counting big-integer growth as constant, which is the usual convention).
**Space:** two variables, so **O(1)**. The top-down memoized version is also
O(n) time, but it needs an O(n) cache *and* an O(n) call stack of depth `n`,
so its space is **O(n)**. Same answer, but the bottom-up loop wins on memory.

Two more shapes worth knowing, informally (the "master theorem" intuition is
just *work per level times number of levels*):

- **Halving, one branch** (binary search): each call does O(1) work and
  recurses on half the input. `log n` levels, O(1) each, so **O(log n)**.
- **Halving, two branches** (merge sort): each level does O(n) total work
  across all its nodes, and there are `log n` levels, so **O(n log n)**.

## Reading the trigger

Count the recursive calls in the body. **One** call that shrinks `n` by one
(factorial, reversing a string recursively) gives a *line*: O(n) time and O(n)
stack. **Two or more** calls that shrink `n` by a constant give a *bushy
tree*: exponential, unless subproblems repeat, in which case memoize or go
bottom-up. Calls that **halve** the input give a logarithmic depth. Do not
confuse these with loops: a loop adds its cost per iteration, but recursion
multiplies by its fan-out. And remember space is separate: a one-branch
recursion can be fast and still burn O(n) stack, which also risks a
`RecursionError` in Python near depth 1000.

## Saying it in the interview

"The direct recursion calls itself twice per level, so the recursion tree has
about 2^n nodes and runs in exponential time. The tree keeps recomputing the
same few values, since there are only n + 1 distinct arguments. I'll build
upward from fib(0) and fib(1), keeping just the last two values, which gives
O(n) time and O(1) space. If I memoized top-down instead, time would also be
O(n), but the stack depth and the cache would cost O(n) space." Pitfalls
interviewers watch for: saying a memoized recursion is O(1) space because
"there are no arrays" (the stack counts), and treating the recursion as O(n)
because the loop version is. Edge cases to name: `n = 0` and `n = 1`.
