---
id: 01-fixed-window
unit: sliding-window
subpattern: "Fixed-size sliding window"
trigger_signals:
  - "You're scanning once left-to-right and only need the best result measured against the best-so-far (a running min or max)."
  - "The window never needs to shrink — one rule governs it at every step: the right edge always advances, the left edge only ever jumps forward."
  - "Brute force pairs every (i, j) with i < j — O(n²) — but the answer only ever depends on a single running extreme."
worked_example: best-time-to-buy-and-sell-stock
diagram: 01-fixed-window.diagram.json
quiz: 01-fixed-window.quiz.json
practice:
  - valid-anagram
  - permutation-in-string
follow_up:
  - "What if you could buy and sell multiple times (unlimited transactions) — does a single running minimum still capture the answer?"
  - "What if the window also had to shrink sometimes, not just jump forward — how would you decide when?"
---

## The one idea

A sliding window collapses "compare every pair" problems into a single
left-to-right scan by keeping only the state you need as you go, instead of
remembering every element you've passed. In its simplest, **fixed** form, one
rule governs the window at every step: the right edge advances every
iteration, and the left edge only ever jumps forward — never backward — when
it finds something strictly better. There's no condition to check before
shrinking; the rule never changes.

For Best Time to Buy and Sell Stock, the window is `[cheapest day seen so
far, today]`. Walk the prices once; at each day, ask "if I sold today, having
bought at the cheapest point so far, what's my profit?" — then check whether
today itself beats the current cheapest, and if so slide the window's left
edge up to it.

## Why it beats the obvious approach

The brute-force answer tries every `(buy, sell)` pair with `buy < sell` — two
nested loops, O(n²). Almost all of that work is wasted: the best sell day
only ever needs to be paired with the single cheapest buy day before it, not
every earlier day individually.

```python
def max_profit(prices: list[int]) -> int:
    lo = prices[0]                    # window's left edge: cheapest price so far
    best = 0
    for price in prices[1:]:          # window's right edge advances every step
        best = max(best, price - lo)  # profit if we sold today
        lo = min(lo, price)           # left edge jumps forward if today is cheaper
    return best
```

One running variable (`lo`) replaces the entire inner loop. Because the left
edge only moves forward and the right edge always advances, each element is
visited exactly once — O(n) time, O(1) space.

## Reading the trigger

Say the problem out loud: **"one pass, and I only need my best result
measured against the best value I've seen so far."** That's the fixed-window
fingerprint — no need to shrink the window based on a violated condition
(that's next lesson's job), just slide forward and update a running extreme.
Whenever nested loops over `(i, j)` pairs collapse to "the answer only
depends on one running min/max," reach for this before writing two loops.
