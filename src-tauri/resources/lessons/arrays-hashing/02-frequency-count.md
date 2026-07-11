---
id: 02-frequency-count
unit: arrays-hashing
subpattern: "Frequency counting with a hash map"
trigger_signals:
  - "You need to group or compare items by \"what they're made of\" rather than by order — anagrams, multisets, histograms."
  - "The question mentions counts of characters, digits, or elements repeating."
  - "Sorting each item to build a comparison key would work but feels wasteful for many items."
worked_example: group-anagrams
diagram: 02-frequency-count.diagram.json
quiz: 02-frequency-count.quiz.json
practice:
  - top-k-frequent-elements
  - valid-anagram
recap:
  - 01-hashmap-lookup
follow_up:
  - "What if you only needed to check two strings are anagrams, not group a whole list — does the counting key change?"
  - "What if the alphabet were huge (Unicode) instead of 26 lowercase letters — would a sorted-string key ever beat a count key?"
---

## The one idea

Two strings are anagrams exactly when they have the **same letter counts**.
That single fact turns "group these words by anagram" into a hash-map problem:
build a key that captures *only* the letter frequencies, throw away order, and
let equal keys fall into the same bucket. Every word with an identical
frequency profile lands together automatically — one pass, no pairwise
comparisons.

## Why it beats the obvious approach

The naive approach compares every pair of strings to check if one is a
rearrangement of the other — O(n²) comparisons, and each comparison itself
costs O(k) for strings of length k. Instead, count each string's letters into
a fixed-size array (26 slots for lowercase English), turn that array into a
hashable key, and group by key in a dict:

```python
from collections import defaultdict

def group_anagrams(strs: list[str]) -> list[list[str]]:
    groups: dict[tuple[int, ...], list[str]] = defaultdict(list)
    for s in strs:
        counts = [0] * 26
        for ch in s:
            counts[ord(ch) - ord("a")] += 1
        groups[tuple(counts)].append(s)   # counts is the key, not the string
    return list(groups.values())
```

The key insight: the map key is no longer a value you're *looking for* (like
Two Sum's complement) — it's a **derived signature** that's identical for
every member of a group. Building that signature costs O(k) per string of
length k, so the whole pass is O(n·k) instead of the O(n²·k) of pairwise
comparison. A sorted-string key (`"".join(sorted(s))`) works too and is
simpler to write, but costs O(k log k) per string instead of O(k) — the count
array is the asymptotically tighter choice.

## Reading the trigger

Ask yourself: *"do two items belong together because of what they contain,
regardless of arrangement?"* Anagram grouping, "same multiset of characters,"
histogram-equality checks — anything where order shouldn't matter but
composition should — is answered by counting into a dict (or a fixed array)
and using that count as the grouping key. If you catch yourself reaching for
`sorted()` on every item just to compare them, ask whether a frequency count
gets you there in linear time instead.
