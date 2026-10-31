---
id: 04-hidden-costs
unit: big-o
subpattern: "Hidden costs: what built-ins really cost"
trigger_signals:
  - "A loop body calls a one-line built-in (`in`, `index`, `remove`, slicing, `+=` on a string) — the line may hide its own O(n) loop."
  - "You're about to test membership against a list or string inside a loop — a set or dict would make it O(1)."
  - "Your solution looks like one pass but the interviewer keeps asking \"what does that line cost?\""
worked_example: jewels-and-stones
diagram: 04-hidden-costs.diagram.json
quiz: 04-hidden-costs.quiz.json
practice:
  - intersection-of-two-arrays
  - reverse-words-in-a-string
recap:
  - 01-complexity-classes
  - 02-amortized-space
  - 03-recursion-cost
follow_up:
  - "If the jewels string is tiny and fixed (say, at most 5 characters), is the list scan still worth replacing with a set?"
  - "How would you count how many stones of each jewel type there are, instead of just the total?"
---

## The one idea

A loop is only as cheap as the **most expensive line inside it**, and many
single-line built-ins hide a loop of their own. Counting only the visible
`for` gives you the wrong Big-O. The habit to build: for every call in the
body, know its cost, then multiply by how often the body runs.

Here is the table interviewers expect (Python first, JavaScript in brackets):

| Operation | Cost |
|---|---|
| `x in list`, `list.index(x)`, `list.remove(x)` [`includes`, `indexOf`] | O(n) |
| `list.insert(0, x)`, `list.pop(0)` [`unshift`, `shift`] | O(n), everything shifts |
| `list.append(x)`, `list.pop()` | O(1) amortized |
| `x in set`, `x in dict`, `dict[k]` [`Set.has`, `Map.get`] | O(1) average |
| `a[i:j]`, `s[:]`, `list(a)` [`slice`] | O(k) copy, k = slice length |
| `s += ch` in a loop | up to O(n^2) total, since each add may copy the string |
| `''.join(parts)` | O(total length) |
| `sorted(a)`, `a.sort()` [`sort`] | O(n log n) |
| `len(x)`, `deque.popleft()`, `deque.append(x)` | O(1) |

## Why it beats the obvious approach

Jewels and Stones counts how many of your stones are also jewels. The natural
code keeps the jewels as a string (or list) and tests each stone with `in`.
That `in` scans up to `j` characters for each of `s` stones, so it is
**O(j * s)** time, even though it looks like a single loop. The repeated
work: the same jewels get rescanned from the start for every stone.

Build a set of jewels once, and each membership test becomes one hash lookup:

```python
def num_jewels_in_stones(jewels: str, stones: str) -> int:
    jewel_set = set(jewels)                     # O(j): hash each jewel once
    count = 0
    for ch in stones:                           # s iterations
        if ch in jewel_set:                     # O(1) average lookup
            count += 1
    return count
```

**Time:** building the set is O(j); the loop is `s` iterations of O(1)
average work, so **O(j + s)**. **Space:** the set holds at most `j` distinct
characters, so **O(j)** extra (it is O(1) if the alphabet is fixed, but say
"O(j), bounded by the alphabet size"). You traded a little memory for a
product-to-sum drop in time. Note that `sum(ch in jewel_set for ch in stones)`
is the same cost; one-liners do not change Big-O.

## Reading the trigger

Scan your own code for these shapes: an `in`, `index`, or `remove` on a list
**inside a loop** (O(n) times O(n)); slicing inside a loop, as in
`helper(a[1:])` recursion, which copies every level and turns O(n) into O(n^2);
`s = s + ch` repeated n times; and `pop(0)` or `insert(0, ...)` used as a
queue, where `collections.deque` gives O(1) at both ends. The fix is almost
always one of: use a set or dict, use indices instead of slices, collect
pieces in a list and `''.join` once, or use a deque. The look-alike: a single
`in` on a list *outside* any loop is just O(n), which is fine.

## Saying it in the interview

"The brute force checks each stone against the jewel string with `in`, which
scans the jewels every time, so it is O(j times s). The insight is that I
only ask a membership question, so I'll hash the jewels into a set once.
Then one pass over the stones does O(1) average lookups: O(j + s) time and
O(j) space for the set." Be ready to defend the "average": hash lookups
degrade only on pathological collisions. Pitfalls interviewers watch for:
claiming a loop is O(n) while ignoring an O(n) `list.remove` or slice inside
it, and building a string with `+=` and calling it linear. Edge cases to
name: empty jewels, empty stones, and case sensitivity (`a` and `A` differ).
