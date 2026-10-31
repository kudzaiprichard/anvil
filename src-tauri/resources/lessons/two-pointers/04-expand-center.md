---
id: 04-expand-center
unit: two-pointers
subpattern: "Expand around center (palindromes)"
trigger_signals:
  - "You must find the longest palindromic substring, or count or list every palindromic substring, of a single string."
  - "A palindrome is fully determined by its middle: if the ends of a candidate match, the inside only has to be a palindrome too."
  - "Checking every substring costs O(n^3) and a DP table costs O(n^2) memory, but n is only a few thousand, so an O(n^2) time, O(1) space answer is the target."
worked_example: longest-palindromic-substring
diagram: 04-expand-center.diagram.json
quiz: 04-expand-center.quiz.json
practice:
  - palindromic-substrings
recap:
  - 01-opposite-ends
follow_up:
  - "Manacher's algorithm finds the longest palindrome in O(n). What idea does it reuse from earlier centers, and why wouldn't you be expected to code it cold?"
  - "How would you count palindromic substrings instead of finding the longest, and what is the maximum the count can reach for a string of n identical letters?"
---

## The one idea

Every palindrome mirrors around a center. Instead of asking "is this
substring a palindrome?" for each of the O(n^2) substrings, flip the
question: **for each possible center, how far can I grow outward before the
two ends disagree?** Two pointers start on the center and move apart, one
left and one right, while the characters they point at are equal. The moment
they differ (or one runs off the string) the palindrome around that center is
as large as it will ever get.

There are `2n - 1` centers, not `n`. An odd-length palindrome is centered on a
character (`l = r = c`). An even-length one is centered *between* two
characters (`l = c`, `r = c + 1`). Skipping the even centers is the classic
bug: a string like a doubled letter has no odd center at all. For Longest
Palindromic Substring, try both centers at every index, remember the longest
span seen, and slice it out at the end.

## Why it beats the obvious approach

The brute force enumerates all O(n^2) substrings and checks each with an O(n)
reversal comparison: **O(n^3)**. The check redoes work: if `s[i..j]` is a
palindrome, then `s[i+1..j-1]` is one too, but the naive check never reuses
that. A DP table `is_pal[i][j]` reuses it and gets O(n^2) time, but it stores
n^2 booleans: O(n^2) space. Expanding from the center keeps the reuse (growing
outward means the inside is already known to match) and needs no table.

```python
def longest_palindrome(s: str) -> str:
    if not s:
        return ""
    start, best_len = 0, 1
    for c in range(len(s)):
        for l, r in ((c, c), (c, c + 1)):          # odd center, even center
            while l >= 0 and r < len(s) and s[l] == s[r]:
                l -= 1                              # grow outward
                r += 1
            length = r - l - 1                      # loop overshot by one on each side
            if length > best_len:
                best_len, start = length, l + 1     # record the span, slice at the end
    return s[start:start + best_len]
```

There are `2n - 1` centers and each expansion walks at most `n / 2` steps
outward, so the worst case (a string of one repeated letter) is **O(n^2)
time**. Beyond a few integers there is nothing to store, so extra space is
**O(1)** (the returned slice is output, not working memory). Compare O(n^3)
brute force and O(n^2)-space DP. Manacher's algorithm gets the time down to
O(n) by reusing mirrored results between neighbouring centers; it exists and
it is good to know the name, but interviewers do not expect you to produce it.

## Reading the trigger

The words that fire this pattern are **"palindromic substring"** (contiguous,
not subsequence) in a single string, asking for the longest one or a count of
them. Contrast it with lesson 01: opposite-ends pointers answer *"is this
whole string a palindrome?"* by walking inward from both ends in O(n), while
expand-around-center answers *"which substrings are palindromes?"* by walking
outward from a middle. A palindromic **subsequence** (characters need not be
adjacent) is not this pattern at all: it is a 2D DP over a range. If the
problem asks for the minimum cuts to split a string into palindromes, you need
palindrome checks as a building block inside a DP, and expanding from centers
is one cheap way to fill that table.

## Saying it in the interview

"Checking every substring is O(n^3), and a DP table fixes the time but costs
O(n^2) memory. A palindrome is defined by its center, so for each of the `2n - 1`
centers I expand two pointers outward while the characters match, once for an
odd center and once for an even one. I track the best start and length and
slice at the end. Time is O(n^2) because each expansion is at most linear and
there are linear many centers; space is O(1). There is an O(n) algorithm,
Manacher's, but I'd only reach for it if the constraints demanded it. I'd test
a single character, two equal characters, two different characters, an
all-same string and a palindrome that fills the whole input." Pitfalls
interviewers watch for: forgetting the even centers, and an off-by-one on the
slice: after the loop the pointers have stepped *past* the palindrome, so the
length is `r - l - 1` and the start is `l + 1`.
