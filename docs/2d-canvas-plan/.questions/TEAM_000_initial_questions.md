# 2D Canvas — Follow-Up Questionnaire

> Based on your answers, I've identified several tensions that need resolution before we can architect this properly.

---

## Key Insights from Round 1

What I understood:

| Topic | Your Answer |
|-------|-------------|
| Canvas | Infinite 2D plane |
| Origin (0,0) | Fixed, determines leading/trailing edges in both X and Y |
| Workspaces | Eliminated — one infinite 2D canvas per monitor |
| Columns | Keep as grouping, BUT tiles can "transcend" with different scales |
| Per-window scale | Yes, window A at 100%, window B at 50% |
| Overlap | Never (except floating layer) |
| Zoom | Replaces overview, fully interactive |
| Gutter-bar | First-class citizen for window placement |

---

## Part 1: The Scale Paradox

You want:
1. Per-window scale (window A at 100%, window B at 50%)
2. No overlap (always tiled)
3. Columns as grouping mechanism

**The Problem**: If windows have different scales, how do they tile without gaps or overlaps?

### Example Scenario

```
┌─────────────────┐  ┌────────┐
│                 │  │        │
│   Window A      │  │ Win B  │   <- B is 50% scale
│   (100%)        │  │ (50%)  │
│                 │  │        │
│                 │  ├────────┤
│                 │  │ Win C  │   <- C is 50% scale  
│                 │  │ (50%)  │
└─────────────────┘  └────────┘
```

No this is not what I have in mind
Alright (x is 0,0 point)

----------------------------x---------------------------------------This is a row   | this      | tileC |                   |
                | is   (A)  | 100%  |                   |
                | a normal  | ------|    important      |
                | app in    | tileD |       app         |
                | column    | 100%  |                   |
------------------------------------|      that is      |-----------
This is a row           | this      |        at         |
                        | is   (B)  |       200%        |
                        | a normal  |      this is      |
                        | app in    |       at max      |
                        | column    |       size        |
--------------------------------------------------------------------

I hope that you can see this...

But we want to make use of the existing infrastructure as much as possible... we just want to make some windows to be transcend between rows... so that it can only take up max 2 rows. So the row logic already exist in niri. the column logic I want to keep intact.. I just want rows to be stacked.. and the capabilitiy to grow a window to take up 2 rows.

---

YES! This makes things **much** clearer. Let me reflect back what I understand:

## Your Mental Model: Rows + Row-Spanning Windows

```
                        x (origin 0,0)
                        |
─────────────────────────────────────────────────────────────────────
ROW 0   | Column A    | Column B  |        Column C                 |
        | (normal)    | TileC 1×  |                                 |
        |             |-----------|     "Important App"             |
        |   App A     | TileD 1×  |       spans 2 rows              |
        |             |           |      (2× row height)            |
─────────────────────────────────────────────────────────────────────
ROW 1   |             | Column B  |                                 |
        |   App B     | (cont.)   |      (same window,              |
        |  (normal)   |           |       still here)               |
        |             |           |                                 |
─────────────────────────────────────────────────────────────────────
```

### Key Insight: "Scale" = Row Span, Not Pixel Density

What you're describing isn't per-window *pixel scale* — it's **row span**:
- Normal window: spans 1 row
- "200%" window: spans 2 rows (takes up 2× vertical space)
- The actual pixel content is the same resolution

### The Camera Behavior

When focus changes:
1. **Focus on App B (1-row window)**: Camera shows Row 1, zoomed to fit 1 row height
2. **Focus on Important App (2-row window)**: Camera zooms OUT to show both Row 0 and Row 1, so the 2-row window fits on screen

The "200%" window doesn't have more pixels — it has the **same pixels** but the camera adjusts so it fills the screen. Other windows appear smaller because the camera is further out.

### This Defeats the Paradox!

There's no tiling conflict because:
- Windows don't overlap
- A 2-row window simply occupies the space of 2 rows in that column
- Adjacent columns can have their own 1-row or 2-row windows
- The camera zoom adjusts based on what needs to be visible

---

## What This Means Architecturally

### Current Niri Structure
```
Workspace
└── ScrollingSpace (1D horizontal strip)
    └── Column[] (each column = vertical stack of tiles)
        └── Tile[] (windows stacked vertically)
```

### Your 2D Vision
```
Canvas (replaces Workspace)
└── Row[] (NEW: vertical stacking of rows)
    └── Column[] (horizontal strip, like current ScrollingSpace)
        └── Tile[] (windows in column)
        
+ Window can span multiple rows (row_span: 1 or 2)
+ Camera has (x, y, zoom) instead of just (x_offset)
```

### The Key Changes

| Component | Current | 2D Version |
|-----------|---------|------------|
| [ScrollingSpace](cci:2://file:///home/vince/Projects/niri/src/layout/scrolling_original.rs:33:0-93:1) | One row of columns | Becomes one `Row` |
| [Workspace](cci:2://file:///home/vince/Projects/niri/src/layout/workspace.rs:44:0-112:1) | Contains one [ScrollingSpace](cci:2://file:///home/vince/Projects/niri/src/layout/scrolling_original.rs:33:0-93:1) | Contains `Row[]` |
| `ViewOffset` | `f64` (X only) | `(x, y, zoom)` |
| [Column](cci:2://file:///home/vince/Projects/niri/src/layout/scrolling/column/mod.rs:27:0-98:1) | Unchanged | Unchanged (still vertical tile stack) |
| `Tile` | No row span | `row_span: u8` (1 or 2) |
| Camera | Follows active column | Follows active tile, adjusts zoom for row span |

---

## Questions to Confirm

### 1. Row Span Limits
You said "max 2 rows." Is this:
- [ ] Hard limit: windows can only span 1 or 2 rows, never 3+
- [X] Soft limit: 2 is typical, but could be extended later
- [ ] Other? ___

### 2. Row Height
Are all rows the same height (before spanning)?
- [X] Yes, uniform row height (like current working_area height)
- [ ] No, rows can have different heights
- [ ] Other? ___

### 3. Column Independence Across Rows
In your diagram, Column B appears in both Row 0 and Row 1. Are columns:
- [X] Independent per row (Row 0's columns ≠ Row 1's columns)
- [ ] Shared across rows (Column B is the same column in both rows)
- [ ] Other? ___

### 4. Navigation
When focused on App A (Row 0) and you press "down":
- [ ] Move to App B (Row 1, same column position)
- [ ] Move to the tile below within the same column (if App A has tiles below)
- [X] Other? If pressing down then it goes to the app that is the closest to the middle point of the down edge.

### 5. Spawning Windows
When you spawn a new window while focused on the "Important App" (2-row span):
- [X] New window goes in the same column, same row span
- [ ] New window goes in adjacent column, 1-row span
- [ ] Depends on origin position (your RTL/LTR logic)
- [ ] Other? ___

---

## Feasibility Assessment

This is **much more tractable** than I initially thought:

| Aspect | Difficulty | Notes |
|--------|------------|-------|
| Add `Row[]` to workspace | Medium | New container, but similar to existing patterns |
| Row-spanning tiles | Medium | Need to track span, adjust positioning |
| Camera with zoom | Medium | Already exists for overview, needs to be generalized |
| Keep columns intact | Easy | No changes to column logic |
| Origin-based leading edge | Already WIP | Your RTL refactor handles this |

**Estimated scope**: 4-6 weeks for MVP (rows + zoom), not 3+ months.
