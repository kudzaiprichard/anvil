---
id: 03-stack-design
unit: stack
subpattern: "Augmented-stack design (O(1) min)"
trigger_signals:
  - "You need push/pop plus an extra query — min, max, running total — that must also answer in O(1), not by rescanning."
  - "The extra value you need at any moment depends only on what's currently on the stack, and disappears the instant something pops."
  - "Recomputing the aggregate after every pop would mean rescanning whatever's left — O(n) per pop instead of O(1)."
worked_example: min-stack
diagram: 03-stack-design.diagram.json
quiz: 03-stack-design.quiz.json
practice:
  - evaluate-reverse-polish-notation
  - generate-parentheses
recap:
  - 01-lifo-parsing
follow_up:
  - "What if you needed O(1) max *and* min at the same time — does one shadow stack still cover both?"
  - "What if pushes and pops could happen from either end (a deque) — does the same trick still give O(1)?"
---

## The one idea

When a stack needs an extra O(1) query — "what's the minimum of everything
currently on me?" — don't recompute it, *carry* it. Keep a second, parallel
stack where each slot stores the running minimum of everything pushed up to
that point. Push onto both stacks together; pop from both together. The top
of the shadow stack is always exactly the minimum of what's left, because
popping the data stack and popping its matching minimum happen in lockstep.

```python
class MinStack:
    def __init__(self) -> None:
        self.stack: list[int] = []
        self.min_stack: list[int] = []  # min_stack[i] = min of stack[:i+1]

    def push(self, val: int) -> None:
        self.stack.append(val)
        prev_min = self.min_stack[-1] if self.min_stack else val
        self.min_stack.append(min(val, prev_min))

    def pop(self) -> None:
        self.stack.pop()
        self.min_stack.pop()

    def top(self) -> int:
        return self.stack[-1]

    def get_min(self) -> int:
        return self.min_stack[-1]
```

## Why it beats the obvious approach

Without the shadow stack, `get_min` has only two bad options: scan the whole
stack every call (O(n) per query), or re-scan the remaining elements after
every pop to find the new minimum (also O(n)). Pairing each pushed value with
a snapshot of "the minimum including me" means `get_min` is just `min_stack[-1]`
— an O(1) peek — and `pop` stays O(1) too, because popping the top of both
stacks together *automatically* restores the correct earlier minimum. Nothing
gets recomputed; the old answer was sitting right underneath the whole time.

## Reading the trigger

Say it out loud: "I need a stack with push and pop, plus a query — min, max,
sum — over everything currently on the stack, and it all has to be O(1)."
Whenever the extra fact you need only depends on what's *still on the stack*
(not the full push history, since popped values stop counting), pair every
value with a snapshot of that fact at push time: one stack for data, one
shadow stack for state, popped together, always in sync.
