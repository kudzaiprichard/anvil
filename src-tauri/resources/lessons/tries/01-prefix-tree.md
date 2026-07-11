---
id: 01-prefix-tree
unit: tries
subpattern: "Prefix tree: insert / search / startsWith"
trigger_signals:
  - "You need repeated `startsWith`/prefix queries against a set of strings — a hash set can't answer those."
  - "Many words in your dictionary share common prefixes and rescanning the whole list per query is wasteful."
  - "The API you're asked for is literally `insert` / `search` / `startsWith`, or a dictionary of words probed one character at a time."
worked_example: implement-trie-prefix-tree
diagram: 01-prefix-tree.diagram.json
quiz: 01-prefix-tree.quiz.json
practice:
  - design-add-and-search-words-data-structure
  - group-anagrams
recap: []
follow_up:
  - "What if a query character could match *any* letter — a wildcard `.` — how do you search a trie when the path isn't fixed?"
  - "What if you also needed to delete a word, cleaning up any nodes that become unused?"
---

## The one idea

A trie (prefix tree) stores strings **letter by letter, layer by layer**,
instead of whole-string by whole-string. Each node is a tiny dict from
"next character" to "child node," plus a flag saying "a word ends here."
Words that share a prefix — `"an"` and `"ant"` — literally share the same
path through the tree, so the shared part is stored, and looked up, exactly
once.

```python
class TrieNode:
    def __init__(self) -> None:
        self.children: dict[str, "TrieNode"] = {}
        self.is_word: bool = False

class Trie:
    def __init__(self) -> None:
        self.root = TrieNode()

    def insert(self, word: str) -> None:
        node = self.root
        for ch in word:
            node = node.children.setdefault(ch, TrieNode())
        node.is_word = True

    def _walk(self, s: str) -> "TrieNode | None":
        node = self.root
        for ch in s:
            node = node.children.get(ch)
            if node is None:
                return None
        return node

    def search(self, word: str) -> bool:
        node = self._walk(word)
        return node is not None and node.is_word

    def starts_with(self, prefix: str) -> bool:
        return self._walk(prefix) is not None
```

## Why it beats the obvious approach

The obvious structure for "is this word in my dictionary?" is a `set` of
whole strings — exact membership in O(1). But a set has no notion of
*prefix*: to answer `startsWith("an")` it would have to scan every one of
the N stored words and compare up to L characters each, O(N·L) per query.
A trie answers the same question by walking L characters from the root —
O(L), completely independent of how many words are stored. `insert` and
`search` cost O(L) too. That's the trade: a little more memory for shared
prefix nodes, in exchange for every operation scaling with *query length*
instead of *dictionary size*.

## Reading the trigger

Ask yourself: am I checking exact membership once, or am I going to probe
prefixes of a growing/shared word set again and again — autocomplete,
"does any word start with…," a word-game dictionary? The moment "prefix"
enters the question, or the interface literally asks for `insert` /
`search` / `startsWith`, reach for a trie before reaching for a `set`.
