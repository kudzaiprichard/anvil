---
id: 02-two-sequence
unit: dp-2d
subpattern: "Two-sequence DP (LCS)"
trigger_signals:
  - "You're comparing two sequences (strings or arrays) and the answer depends on aligning or matching their elements pairwise."
  - "The question asks for a longest/shortest/count relationship *between* two sequences, not a search within a single one."
  - "A recursive solution would branch on \"do these two current elements match or not,\" re-deriving the same (prefix-of-A, prefix-of-B) subproblem over and over."
worked_example: longest-common-subsequence
diagram: 02-two-sequence.diagram.json
quiz: 02-two-sequence.quiz.json
practice:
  - unique-paths
  - coin-change-ii
recap:
  - 01-grid-paths
follow_up:
  - "What if you needed the actual subsequence, not just its length — how would you recover it by walking the table backwards?"
  - "What if insertions and deletions each cost something (edit distance) instead of only matching — how does the recurrence change?"
---

## The one idea

Two-sequence DP builds a 2-D table `dp[i][j]` that answers a question about
"the first `i` elements of A and the first `j` elements of B" — the same
grid shape as grid-path counting, but now the two axes are the two input
sequences instead of two spatial dimensions. Each cell's recurrence branches
on whether `A[i-1]` and `B[j-1]` match.

## Why it beats the obvious approach

Comparing every pair of subsequences directly is exponential — there are 2^n
subsequences of A alone. Instead, notice the longest common subsequence of
prefixes `A[:i]` and `B[:j]` has only two cases: the last elements match, so
they belong in the LCS and you extend the LCS of the prefixes before them; or
they don't match, so the answer is the better of dropping the last element of
A or of B.

```python
def longest_common_subsequence(a: str, b: str) -> int:
    m, n = len(a), len(b)
    dp = [[0] * (n + 1) for _ in range(m + 1)]   # dp[i][j]: LCS of a[:i], b[:j]
    for i in range(1, m + 1):
        for j in range(1, n + 1):
            if a[i - 1] == b[j - 1]:
                dp[i][j] = dp[i - 1][j - 1] + 1
            else:
                dp[i][j] = max(dp[i - 1][j], dp[i][j - 1])
    return dp[m][n]
```

The table carries an extra row and column of zeros for the "empty prefix"
base case, same as grid-path counting's seeded first row/column. Filling row
by row keeps every dependency (`dp[i-1][j-1]`, `dp[i-1][j]`, `dp[i][j-1]`)
already computed. That's O(m·n) time and space, versus exponential
subsequence enumeration.

## Reading the trigger

Ask: "am I comparing two sequences, and does the answer come from walking
both of them together, deciding match / no-match at each step?" That's the
tell for two-sequence DP. Recall the grid-path lesson: same 2-D table, same
"fill in an order that respects dependencies" discipline — except now the two
axes are "how far into A" and "how far into B" rather than row and column in
physical space.
