---
id: 02-dynamic-window
unit: sliding-window
subpattern: "Variable-size window with a hash map"
trigger_signals:
  - "You're scanning a string or array and the window must keep satisfying a constraint continuously (e.g. no repeated characters) — grow while it holds, shrink the moment it breaks."
  - "Unlike a fixed window, both edges move: the right edge always advances, but the left edge only creeps forward when the constraint is violated."
  - "You need a fast \"is X currently inside my window?\" check — a hash map (or set) tracks window membership in O(1)."
worked_example: longest-substring-without-repeating-characters
diagram: 02-dynamic-window.diagram.json
quiz: 02-dynamic-window.quiz.json
practice:
  - longest-repeating-character-replacement
  - best-time-to-buy-and-sell-stock
recap:
  - 01-fixed-window
follow_up:
  - "What if the constraint were \"at most k distinct characters\" instead of zero repeats — does the same shrink rule generalize?"
  - "What if you needed the *count* of valid windows, not just the longest one?"
---

## The one idea

Building on the fixed window from the last lesson, a variable window adds a
second moving part: the left edge can advance too — not on a fixed rule, but
whenever the window breaks a constraint. A hash map tracks what's currently
*inside* the window so that check is O(1) instead of a rescan.

For Longest Substring Without Repeating Characters, the constraint is "no
character appears twice inside the window." Walk the string with a right
pointer, keeping a map of `char -> last index seen`. If the incoming
character's last-seen index already sits inside the window, drag the left
edge past it before extending; then record the window's length.

## Why it beats the obvious approach

Brute force checks every substring for uniqueness — O(n²) just to try every
start/end pair, and re-verifying each one costs more on top. That repeats
work: sliding the window from `[l, r]` to `[l, r+1]` doesn't need to
re-check the whole span, only whether the new character collides with
something already in the window.

```python
def length_of_longest_substring(s: str) -> int:
    last_seen: dict[str, int] = {}
    left = 0
    best = 0
    for right, ch in enumerate(s):
        if ch in last_seen and last_seen[ch] >= left:
            left = last_seen[ch] + 1     # shrink: jump past the duplicate
        last_seen[ch] = right
        best = max(best, right - left + 1)
    return best
```

`right` visits each character exactly once, and `left` also only ever moves
forward over the whole scan — so even though it looks like two loops, total
pointer movement is bounded by `n`. That's O(n) time and O(min(n, alphabet))
space for the map.

## Reading the trigger

Recall the fixed window: the right edge advances every step, and the left
edge only ever jumps on one unconditional rule. Here both edges move, but the
left edge's rule is now conditional — "shrink until the constraint holds
again." Whenever a problem says "longest/shortest substring or subarray such
that some condition holds continuously," picture growing the window with a
hash map, and dragging the left edge forward the moment the map tells you the
condition just broke.
