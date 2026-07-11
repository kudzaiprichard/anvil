---
id: 01-lifo-parsing
unit: stack
subpattern: "LIFO matching & parsing"
trigger_signals:
  - "You're scanning a sequence of opens/closes (or nested tokens) and need to check they nest correctly."
  - "The most recent unmatched item is exactly what the next token must close — last in, first out."
  - "A brute-force check would need to look arbitrarily far back to find a match; you want that lookback to be O(1)."
worked_example: valid-parentheses
diagram: 01-lifo-parsing.diagram.json
quiz: 01-lifo-parsing.quiz.json
practice:
  - generate-parentheses
  - two-sum
recap: []
follow_up:
  - "What if you needed to report *which* bracket was unmatched, not just yes/no?"
  - "What if the tokens were words or tags instead of brackets — does the same matching idea still apply?"
---

## The one idea

A stack tracks "what's still open" in exactly the order you'd need to close it.
Push when you see an opener; the moment you hit a closer, whatever sits on top
of the stack is the *only* thing it could possibly match. That's LIFO — last
in, first out — the most recently opened, still-unmatched item is always the
first one that has to close.

In Python a plain `list` is the stack — `append` to push, `pop` to pop — since
we only ever touch one end.

```python
def is_valid(s: str) -> bool:
    pairs = {")": "(", "]": "[", "}": "{"}
    stack: list[str] = []
    for ch in s:
        if ch in pairs:
            if not stack or stack.pop() != pairs[ch]:
                return False
        else:
            stack.append(ch)
    return not stack
```

## Why it beats the obvious approach

Without a stack, matching a closer means searching backward through everything
you've seen to find its opener — and nested structures make that search
ambiguous unless you track order explicitly. The stack turns "the most recent
unmatched thing" into an O(1) peek (`stack[-1]`, then `stack.pop()`) instead of
an O(n) rescan. One pass over the string, O(n) time, O(n) worst-case space for
the stack itself.

Two failure modes both fall out of the same check: hitting a closer with an
*empty* stack (nothing to match), and finishing the scan with a *non-empty*
stack (something never got closed). The algorithm isn't "does everything
eventually match" — it's "does it match in the right order," and the stack is
what makes "right order" cheap to enforce.

## Reading the trigger

Say it out loud: "a sequence of open/close tokens — or nested calls, or a walk
that can back up — where the next thing must undo the most recently unfinished
thing." Whenever "last opened, first closed" describes the rule, reach for a
stack — not a queue, not a set — because only a stack exposes "the most recent
unmatched item" in O(1).
