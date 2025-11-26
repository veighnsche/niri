# TEAM_004: Keyboard Shortcuts for 2D Canvas

## Context
The 2D canvas introduces rows, row-spanning windows, and camera zoom. This fundamentally changes navigation and window management, requiring new keyboard shortcuts.

## Questions for USER

### 1. Navigation in 2D Space

**Current (1D):**
- `Mod+Left/Right` — Focus column left/right
- `Mod+Up/Down` — Focus window up/down within column

**Proposed for 2D:**
- `Mod+Left/Right` — Focus column left/right (same row)
- `Mod+Up/Down` — Focus window up/down OR move to adjacent row?

**Question:** When pressing `Mod+Up` at the top of a column:
- (A) Stay in current row, no action
- (B) Move to the row above (if exists)
- (C) Configurable behavior

I will pick B, but why not like... if there are multiple tiles in a column that you are at the bottom tile of 2 tiles, then the row above has another column that is 400 px offset to the left of the current row for example... then you are at the bottom. then press up then you stay at the same column in the same row... then press up again.. then you shift to the row above and find the nearest column closest to the 0,0 point.


---

### 2. Row Navigation

**New shortcuts needed:**
- Focus row above/below (independent of column focus)
- Move window/column to row above/below
- Create new row above/below

**Question:** What modifier combination for row operations?
- (A) `Mod+Shift+Up/Down` — Row focus
- (B) `Mod+Ctrl+Up/Down` — Row focus
- (C) Dedicated keys like `Mod+PageUp/PageDown`
- (D) Other?

Well... for a real 2D effect I assume that - looking at my previous answer... we don't treat the rows as different workspaces like the current implementation where we want to switch across multiple workspaces.. now that they are rows.. when we go up or down we are not switching workspaces.. we are moving to the row above or below. 

---

### 3. Row Spanning

**New shortcuts needed:**
- Increase/decrease row span of focused window
- Set specific row span (1, 2, 3, etc.)

**Question:** What shortcuts for row spanning?
- (A) `Mod+Ctrl+Plus/Minus` — Increase/decrease span
- (B) `Mod+Shift+1/2/3` — Set specific span
- (C) Both
- (D) Other?

I think that you are the best at picking this.

---

### 4. Camera/Zoom

**New shortcuts needed:**
- Zoom in/out
- Reset zoom to 100%
- Zoom to fit focused window
- Zoom to fit all visible

**Question:** What shortcuts for zoom?
- (A) `Mod+Plus/Minus` — Zoom in/out (conflicts with current preset width cycling?)
- (B) `Mod+Scroll` — Mouse wheel zoom
- (C) `Mod+Ctrl+Plus/Minus` — Zoom
- (D) Other?

Definitely mod+scroll then please make the best choices for me... look online and find out which games do that too

---

### 5. Workspace vs Canvas

**Current:** Multiple workspaces per output, switch with `Mod+1/2/3...`

**Question:** With 2D canvas, do we:
- (A) Keep workspaces as separate canvases (current behavior, just with 2D)
- (B) Replace workspaces entirely with one infinite canvas
- (C) Hybrid: one canvas per output, but can have multiple "views" saved
- (D) Other?

This affects whether `Mod+1/2/3` switches workspaces or does something else.

B... I have explained multiple times that we're going to get rid of all the workspaces.

---

### 6. Existing Shortcuts to Preserve

**Must keep working:**
- `Mod+Left/Right` — Column focus (within row)
- `Mod+Shift+Left/Right` — Move column
- `Mod+R` — Cycle preset widths
- `Mod+F` — Maximize column
- `Mod+Shift+F` — Fullscreen
- `Mod+C` — Center column
- `Mod+[/]` — Consume/expel window

**Question:** Any of these should change behavior in 2D mode?

I don't think so

---

### 7. Configuration

**Question:** Should 2D canvas be:
- (A) Always enabled (breaking change)
- (B) Opt-in via config flag
- (C) Feature flag during development, then default later

A, BREAK IT. I think that we should always enable it.

---

## Summary

The 2D canvas needs shortcuts for:
1. **Row navigation** — Move focus between rows
2. **Row operations** — Move windows/columns between rows
3. **Row spanning** — Control how many rows a window spans
4. **Camera/zoom** — Control view zoom level
5. **Canvas vs workspace** — How to handle multiple "spaces"

Please answer these questions so we can plan the keyboard shortcut scheme before implementing Phase 1+.

---

## Status: AWAITING USER INPUT
