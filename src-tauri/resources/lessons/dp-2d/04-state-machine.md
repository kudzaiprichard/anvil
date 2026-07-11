---
id: 04-state-machine
unit: dp-2d
subpattern: "State-machine DP (buy/sell with cooldown)"
trigger_signals:
  - "You're making a sequence of decisions over time (buy/sell/hold, on/off, locked/unlocked) where today's best move depends on which \"state\" you're currently in, not just the raw index."
  - "A constraint couples consecutive decisions (a cooldown, a transaction limit, a lockout period), so a greedy local choice can be wrong."
  - "The problem can be redrawn as a small, named set of states with transitions between them, and you want the best value achievable in each state at each step."
worked_example: best-time-to-buy-and-sell-stock-with-cooldown
diagram: 04-state-machine.diagram.json
quiz: 04-state-machine.quiz.json
practice:
  - unique-paths
  - target-sum
recap:
  - 01-grid-paths
follow_up:
  - "What if there were a fixed transaction fee on every sale instead of a cooldown — how would the state transitions change?"
  - "What if you were limited to at most k transactions — how would you add a third axis to the table for \"transactions used so far\"?"
---

## The one idea

State-machine DP tracks, at every index, a **table of best values, one per
named state** you could be in — here, `held` (currently holding a share) and
`sold` / `rest` (currently not holding, in various cooldown phases). Each
column of the table is one day; each row is one state; the recurrence is the
state machine's transition diagram written as formulas.

## Why it beats the obvious approach

Trying every subset of buy/sell days is exponential, and a naive greedy
("sell whenever tomorrow's price is lower") breaks the moment a cooldown
constraint couples today's sell to what you're allowed to do tomorrow.
Instead, define a small fixed set of states and, for each day, compute the
best profit achievable while ending the day in each one:

```python
def max_profit(prices: list[int]) -> int:
    if not prices:
        return 0
    held, sold, rest = float("-inf"), 0, 0   # day 0: can't have sold yet
    for price in prices:
        prev_sold = sold
        sold = held + price                 # sell today: must have been holding
        held = max(held, rest - price)      # keep holding, or buy today (from rest)
        rest = max(rest, prev_sold)         # keep resting, or cooldown just ended
    return max(sold, rest)
```

`held`, `sold`, and `rest` are three rows of a conceptual 2-D table indexed by
(day, state); each cell only reads the *previous* day's cells, so one linear
pass fills the whole table left to right — O(n) time, O(1) space once you
keep only the previous column. The cooldown rule (can't buy the day right
after a sell) is enforced structurally: `held` can only be entered from
`rest`, never straight from `sold`, so a just-sold state has to pass through
`rest` first.

## Reading the trigger

Ask: "am I making a sequence of decisions where 'what can I do next' depends
on a small, nameable status (holding vs. not, on vs. off, locked-out vs.
free), and today's status limits tomorrow's options?" That's state-machine
DP. It's still the same 2-D table discipline from the grid-path lesson — one
axis is "how far along the sequence," the other is now "which named state"
instead of "column position" — but because each day only looks at the
*immediately preceding* day, you can roll the table down to a handful of
variables.
