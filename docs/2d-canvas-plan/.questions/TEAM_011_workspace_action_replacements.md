# Workspace Action Replacements — Questionnaire

> **Purpose**: Define what replaces each workspace action in 2D canvas mode.
> **Context**: Workspaces are being removed. One infinite 2D canvas per monitor.
> **Key insight**: Rows replace workspaces. `Mod+Up/Down` navigates between rows.

---

## Pre-Answered Based on README.md

From the 2D Canvas vision:

```
ROW 0   | Column A    | Column B  |        Column C                 |
        |   App A     | TileC     |     "Important App"             |
─────────────────────────────────────────────────────────────────────
ROW 1   |   App B     | TileE     |      (spans 2 rows)             |
```

- **Rows** replace workspaces (horizontal strips stacked vertically)
- **Camera bookmarks** replace workspace switching (`Mod+1/2/3`)
- **Geometric navigation** crosses rows at edges

---

## Part 1: Focus Navigation (Workspace → Row)

### 1.1 FocusWorkspaceDown / FocusWorkspaceUp

**Current**: Switch to workspace below/above.

**Proposed replacement**: Focus row below/above (move camera Y).

| Old Action | New Action | Notes |
|------------|------------|-------|
| `FocusWorkspaceDown` | `FocusRowDown` | Camera pans to row below |
| `FocusWorkspaceUp` | `FocusRowUp` | Camera pans to row above |

**Question**: Is this correct?
- [X] Yes, rows replace workspaces for vertical navigation
- [ ] No, different behavior: _______________

### 1.2 FocusWindowOrWorkspaceDown / FocusWindowOrWorkspaceUp

**Current**: Focus next window in column, OR switch workspace if at edge.

**Proposed replacement**: Focus next window in column, OR focus row if at edge.

| Old Action | New Action | Notes |
|------------|------------|-------|
| `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` | At bottom of column → focus row below |
| `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` | At top of column → focus row above |

**Question**: Is this the intended behavior from README?
- [X] Yes — "Navigate up/down (geometric, crosses rows at edges)"
- [ ] No, different behavior: _______________

### 1.3 FocusWorkspace(N) — Jump to specific workspace

**Current**: Jump to workspace N (1-9).

**Proposed replacement**: Jump to camera bookmark N.

| Old Action | New Action | Notes |
|------------|------------|-------|
| `FocusWorkspace(1)` | `JumpToBookmark(1)` | Phase 5: Camera bookmarks |
| `FocusWorkspace(2)` | `JumpToBookmark(2)` | |

**Question**: Camera bookmarks store position + zoom. Correct?
- [ ] Yes, position (x, y) + zoom level
- [ ] Just position (x, y), no zoom
- [X] Other: No replacement... No Jump to Bookmark feature...

### 1.4 FocusWorkspacePrevious

**Current**: Jump back to previously focused workspace.

**Proposed replacement**: Jump back to previous camera position?

**Question**: Should there be a "previous position" memory?
- [X] Yes, like browser back button
- [ ] No, use bookmarks instead
- [ ] Other: _______________

---

## Part 2: Move Window/Column (Workspace → Row)

### 2.1 MoveWindowToWorkspaceDown / MoveWindowToWorkspaceUp

**Current**: Move active window to workspace below/above.

**Proposed replacement**: Move active window to row below/above.

| Old Action | New Action | Notes |
|------------|------------|-------|
| `MoveWindowToWorkspaceDown` | `MoveWindowToRowDown` | Window moves to row below |
| `MoveWindowToWorkspaceUp` | `MoveWindowToRowUp` | Window moves to row above |

**Question**: When moving to a row, where in that row?
- [ ] Same column index (if exists)
- [ ] Rightmost/leftmost position
- [X] Next to geometrically nearest window
- [ ] Other: _______________

### 2.2 MoveWindowDownOrToWorkspaceDown / MoveWindowUpOrToWorkspaceUp

**Current**: Move window down in column, OR to workspace if at edge.

**Proposed replacement**: Move window down in column, OR to row if at edge.

| Old Action | New Action | Notes |
|------------|------------|-------|
| `MoveWindowDownOrToWorkspaceDown` | `MoveWindowDownOrToRowDown` | |
| `MoveWindowUpOrToWorkspaceUp` | `MoveWindowUpOrToRowUp` | |

**Question**: This matches README "Mod+Shift+Up/Down: Move window/column to row above/below"?
- [X] Yes
- [ ] No: _______________

### 2.3 MoveColumnToWorkspaceDown / MoveColumnToWorkspaceUp

**Current**: Move entire column to workspace below/above.

**Proposed replacement**: Move entire column to row below/above.

| Old Action | New Action | Notes |
|------------|------------|-------|
| `MoveColumnToWorkspaceDown` | `MoveColumnToRowDown` | |
| `MoveColumnToWorkspaceUp` | `MoveColumnToRowUp` | |

**Question**: Is this correct?
- [X] Yes
- [ ] No: _______________

### 2.4 MoveWindowToWorkspace(N) / MoveColumnToWorkspace(N)

**Current**: Move to specific workspace N.

**Proposed replacement**: Move to camera bookmark position? Or remove entirely?

**Question**: Should users be able to move windows to bookmark positions?
- [ ] Yes, move window to bookmark N's row
- [X] No, remove this action
- [ ] Other: _______________

---

## Part 3: Workspace Reordering → Row Reordering?

### 3.1 MoveWorkspaceDown / MoveWorkspaceUp

**Current**: Reorder workspaces (swap positions).

**Proposed replacement**: Reorder rows?

**Question**: Should rows be reorderable?
- [X] Yes, `MoveRowDown` / `MoveRowUp`
- [ ] No, rows have fixed indices (0, 1, 2, ...)
- [ ] Other: _______________

### 3.2 MoveWorkspaceToIndex(N)

**Current**: Move workspace to specific index.

**Proposed replacement**: Move row to specific index?

**Question**: Same as above — should rows be reorderable?
- [X] Yes
- [ ] No
- [ ] Other: _______________

---

## Part 4: Cross-Monitor Workspace Movement

### 4.1 MoveWorkspaceToMonitor(output)

**Current**: Move entire workspace to another monitor.

**Proposed replacement**: Move entire row to another monitor?

**Question**: In 2D canvas mode, each monitor has its own canvas. Should rows be movable between monitors?
- [X] Yes, `MoveRowToMonitor` (look if there is a window that spans multiple rows... then if the windows 0,0 point in on that row... then make that enlarged tile into a 1 row tile and then move... if it is not in that row... then make it smaller and put it on the row where the apps 0,0 point is at.)
- [ ] No, remove this action
- [ ] Other: _______________

### 4.2 MoveWorkspaceToMonitorLeft/Right/Up/Down

**Current**: Move workspace to adjacent monitor.

**Question**: Same as above.
- [X] Yes, replace with `MoveRowToMonitor*`
- [ ] No, remove
- [ ] Other: _______________

---

## Part 5: Workspace Naming

### 5.1 SetWorkspaceName / UnsetWorkspaceName

**Current**: Name a workspace for reference.

**Proposed replacement**: Name a row? Or name a camera bookmark?

**Question**: Should rows or bookmarks be nameable?
- [X] Rows can be named
- [ ] Bookmarks can be named
- [ ] Both
- [ ] Neither — remove naming
- [ ] Other: _______________

---

## Part 6: Gestures

### 6.1 Workspace Switch Gesture (touchpad swipe up/down)

**Current**: Three-finger swipe up/down switches workspaces.

**Proposed replacement**: Three-finger swipe up/down pans camera vertically (between rows).

**Question**: Is this correct?
- [X] Yes, vertical pan gesture
- [ ] No: _______________

### 6.2 DnD Edge Scrolling

**Current**: Dragging window to top/bottom edge scrolls workspaces.

**Proposed replacement**: Dragging to edge scrolls camera (rows).

**Question**: Is this correct?
- [X] Yes
- [ ] No, remove DnD edge scrolling
- [ ] Other: _______________

---

## Part 7: Shortcut Discovery

You mentioned "shortkeys are discovered during daily use."

**Question**: For the initial release, which shortcuts are essential?

BASICALLY DO NOT MESS WITH THE EXISTING SHORTCUTS EXCEPT WORKSPACES!!!

### Essential (must work on day 1)
- [X] `Mod+Up/Down/Left/Right` — Focus row above/below
- [X] `Mod+Shift+Up/Down/Left/Right` — Move window to row above/below
- [ ] `Mod+1/2/3` — Camera bookmarks (Phase 5)
- [ ] Other: _______________

### Can be added later
- [ ] Row reordering
- [ ] Cross-monitor row movement
- [ ] Row naming
- [ ] Other: _______________

---

## Part 8: Row-Spanning Windows (Edge Case)

**Question**: What happens when moving a row that contains a window spanning multiple rows?

**Answer** (from USER):
> The window's 0,0 point determines which row it "belongs to". When moving that row:
> 1. The window shrinks back to 1 row span
> 2. Then the row moves
> 3. The window stays with its "home" row (where its 0,0 is)

**Example**:
```
ROW 0   | App A |  Big App (spans rows 0-1, 0,0 is in row 0)  |
─────────────────────────────────────────────────────────────────
ROW 1   | App B |                                              |
```

If user does `MoveRowToMonitor` on Row 0:
1. Big App shrinks to 1 row span (stays in row 0)
2. Row 0 (with App A and now-shrunk Big App) moves to other monitor
3. Row 1 (with App B) stays

If user does `MoveRowToMonitor` on Row 1:
1. Big App's 0,0 is in Row 0, so it stays in Row 0
2. Row 1 (with App B) moves to other monitor
3. Big App shrinks because Row 1 is gone

---

## Summary Table (Updated with USER answers)

| Old Action | New Action | Phase | Notes |
|------------|------------|-------|-------|
| `FocusWorkspaceDown` | `FocusRowDown` | 2 | |
| `FocusWorkspaceUp` | `FocusRowUp` | 2 | |
| `FocusWindowOrWorkspaceDown` | `FocusWindowOrRowDown` | 2 | Geometric nav |
| `FocusWindowOrWorkspaceUp` | `FocusWindowOrRowUp` | 2 | Geometric nav |
| `FocusWorkspace(N)` | **REMOVE** | — | No bookmark feature |
| `FocusWorkspacePrevious` | `JumpToPreviousPosition` | 5 | Like browser back |
| `MoveWindowToWorkspaceDown` | `MoveWindowToRowDown` | 2 | Geometric placement |
| `MoveWindowToWorkspaceUp` | `MoveWindowToRowUp` | 2 | Geometric placement |
| `MoveColumnToWorkspaceDown` | `MoveColumnToRowDown` | 2 | |
| `MoveColumnToWorkspaceUp` | `MoveColumnToRowUp` | 2 | |
| `MoveWindowToWorkspace(N)` | **REMOVE** | — | No bookmark feature |
| `MoveWorkspaceDown/Up` | `MoveRowDown/Up` | 2 | Row reordering ✓ |
| `MoveWorkspaceToMonitor` | `MoveRowToMonitor` | 2 | Shrinks spanning windows |
| `SetWorkspaceName` | `SetRowName` | 2 | Rows can be named |
| Workspace gesture | Row/camera pan gesture | 2 | Vertical pan |
| DnD edge scroll | Row/camera pan | 2 | ✓ |

---

*TEAM_011: Questionnaire complete — USER has answered all questions*
