---
id: 01-grid-dfs
unit: graphs
subpattern: "Grid DFS: flood fill & islands"
trigger_signals:
  - "You're given a 2D grid of cells (land/water, 0/1, colors) and need to find or count connected regions."
  - "\"Connected\" means sharing an up/down/left/right edge — a grid is really a graph where each cell is a node with up to 4 neighbors."
  - "Each connected component should be visited exactly once — you need a way to mark a cell \"already counted\" so you don't loop forever or double count."
worked_example: number-of-islands
diagram: 01-grid-dfs.diagram.json
quiz: 01-grid-dfs.quiz.json
practice:
  - max-area-of-island
  - surrounded-regions
recap: []
follow_up:
  - "What if you needed the *size* of each island, not just the count — where would you accumulate that inside the DFS?"
  - "What if the grid were huge and recursion could blow the call stack — how would you rewrite the DFS with an explicit stack?"
---

## The one idea

A grid is a graph in disguise: every cell is a node, and its up/down/left/right
neighbors are its edges. DFS from an unvisited land cell "floods" outward
through every cell reachable by land, and every cell it touches belongs to the
same island — so counting islands is just counting how many times you *start*
a fresh flood fill.

```python
def num_islands(grid: list[list[str]]) -> int:
    rows, cols = len(grid), len(grid[0])

    def dfs(r: int, c: int) -> None:
        if r < 0 or r >= rows or c < 0 or c >= cols or grid[r][c] != "1":
            return
        grid[r][c] = "0"          # mark visited: sink this land cell
        dfs(r + 1, c)
        dfs(r - 1, c)
        dfs(r, c + 1)
        dfs(r, c - 1)

    islands = 0
    for r in range(rows):
        for c in range(cols):
            if grid[r][c] == "1":
                islands += 1
                dfs(r, c)
    return islands
```

## Why it beats the obvious approach

Without a visited marker, you'd have to ask "have I counted this cell's
island already?" for every land cell you see, which means re-walking whole
regions over and over — easily O(n²) on an adversarial grid. Sinking a cell
to `"0"` the moment DFS visits it is what makes each cell get touched
*exactly once*: the outer double loop only ever starts a new flood fill on a
cell no earlier flood fill has already sunk.

That single guard — `r < 0 or r >= rows or c < 0 or c >= cols or grid[r][c]
!= "1"` — does two jobs at once: it keeps DFS inside the grid's bounds, and
it stops the recursion the instant it steps onto water or an already-sunk
cell. Every cell is pushed onto the call stack at most once, so the whole
scan is O(rows × cols) time and, in the worst case (one giant island), O(rows
× cols) space for the recursion stack.

## Reading the trigger

Say it out loud: **"2D grid, connected region, count or measure it."**
Whenever a problem hands you a matrix of cells and asks about connected
blobs — how many, how big, does one touch the edge — reach for DFS (or BFS)
flood fill from each unvisited cell before reaching for anything fancier.
The recursion *is* the graph traversal; the grid coordinates are just how the
node's identity and its edges happen to be encoded.
