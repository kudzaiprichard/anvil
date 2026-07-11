---
id: 02-amortized-space
unit: big-o
subpattern: "Amortized time & space complexity"
trigger_signals:
  - "You're repeatedly appending to / growing a structure and wondering whether occasional resizes blow up the total cost."
  - "You're asked not just \"how fast\" but \"how much extra memory\" an approach uses beyond the input itself."
  - "You only need the best/min/max seen so far, not the whole history — a sign O(1) extra space is within reach."
worked_example: best-time-to-buy-and-sell-stock
diagram: 02-amortized-space.diagram.json
quiz: 02-amortized-space.quiz.json
practice:
  - two-sum
  - binary-search
recap:
  - 01-complexity-classes
follow_up:
  - "Could you get away with even less if you only needed a yes/no answer for whether a profitable trade exists, not the actual amount?"
  - "What if you had to support many buy/sell transactions instead of just one — does the O(1)-space trick still hold?"
---

## The one idea

Big-O has two axes, and problems ask about both: **time** (how many
operations) and **space** (how much extra memory). Space complexity counts
only the *extra* memory an approach uses beyond the input it was given — a few
running variables is O(1) space; a second array the size of the input is
O(n) space, even if the algorithm is otherwise fast.

**Amortized** time is the time-axis cousin of that same idea: some operations
are occasionally expensive, but if you spread that cost over a long sequence
of cheap ones, the *average* cost per operation stays low. A dynamic array's
`.append()` occasionally has to copy every existing element into a bigger
backing array — an O(n) step — but that only happens O(log n) times over n
appends, so the total work across all n appends is still O(n), i.e. O(1)
*amortized* per call, not O(n) per call.

## Reading it off a worked example

Best Time to Buy and Sell Stock asks for the largest profit from one buy
followed by one later sell. The one-pass solution only ever needs two running
numbers — the lowest price seen so far, and the best profit seen so far:

```python
def max_profit(prices: list[int]) -> int:
    min_price = prices[0]
    best = 0
    for price in prices[1:]:
        best = max(best, price - min_price)  # profit if we sold today
        min_price = min(min_price, price)    # cheapest buy so far
    return best
```

One pass, O(n) time — but the space is the interesting part. `min_price` and
`best` are two scalars, regardless of whether the input has 10 prices or 10
million. That's **O(1) extra space**. Compare that to a version that first
builds a whole array of running minimums (`mins[i] = min(prices[:i+1])`) and
then scans it: same O(n) time, but now O(n) *extra* space, because you're
carrying a second array the size of the input just to get the same answer.

## Reading the trigger

Ask time and space as two separate questions, because an approach can be fast
and memory-hungry, or slow and lean. "I only need what I've seen *so far*, not
the whole history" is the cue for O(1) space — a couple of running variables
replace a whole auxiliary array. And when you see a loop that grows a
structure one element at a time, remember that an occasional expensive step
(a resize, a rebuild) doesn't necessarily mean the *sequence* is slow — check
whether the cost amortizes across all the cheap steps around it.
