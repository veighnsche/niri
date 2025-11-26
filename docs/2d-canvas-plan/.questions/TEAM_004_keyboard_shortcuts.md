# TEAM_004: Keyboard Shortcuts for 2D Canvas

## Status: RESOLVED 

---

## Final Decisions

### 1. Navigation — Geometric/Nearest Neighbor

**Decision**: `Mod+Up/Down` uses **geometric navigation**:
- At bottom tile → press Up → move up within column
- At top tile → press Up → move to row above, find **nearest column to origin (0,0)**
- Same logic for Down

This creates natural 2D navigation without separate "row switching" shortcuts.

### 2. Row Navigation — Unified with Window Navigation

**Decision**: No separate row shortcuts needed. `Mod+Up/Down` handles it:
- Rows are NOT separate workspaces
- Moving up/down at column edges crosses row boundaries
- `Mod+Shift+Up/Down` — Move window/column to row above/below (creates row if needed)

### 3. Row Spanning

**Decision**: 
- `Mod+Ctrl+Plus/Minus` — Increase/decrease row span
- `Mod+Ctrl+1/2/3` — Set specific row span (1, 2, 3 rows)

### 4. Camera/Zoom

**Decision** (based on game conventions like Factorio, Cities Skylines, RTS games):
- `Mod+Scroll` — Zoom in/out (primary)
- `Mod+0` — Reset zoom to 100%
- `Mod+=` — Zoom to fit focused window
- `Mod+Shift+=` — Zoom to fit all visible windows

### 5. Workspaces — REMOVED

**Decision**: **Replace workspaces entirely with one infinite canvas per output.**
- `Mod+1/2/3...` — **Repurposed**: Jump to saved camera positions (bookmarks)
- `Mod+Shift+1/2/3...` — Save current camera position as bookmark
- No more workspace switching — everything is on one canvas

### 6. Existing Shortcuts — Preserved

All existing shortcuts keep working:
- `Mod+Left/Right` — Focus column left/right (within current row)
- `Mod+Shift+Left/Right` — Move column left/right
- `Mod+R` — Cycle preset widths
- `Mod+F` — Maximize column
- `Mod+Shift+F` — Fullscreen
- `Mod+C` — Center column
- `Mod+[/]` — Consume/expel window

### 7. Configuration — Breaking Change

**Decision**: Always enabled. No opt-in flag. **Break it.**

---

## New Shortcuts Summary

| Shortcut | Action |
|----------|--------|
| `Mod+Up/Down` | Navigate up/down (geometric, crosses rows) |
| `Mod+Shift+Up/Down` | Move window/column to row above/below |
| `Mod+Ctrl+Plus/Minus` | Increase/decrease row span |
| `Mod+Ctrl+1/2/3` | Set row span to 1/2/3 |
| `Mod+Scroll` | Zoom in/out |
| `Mod+0` | Reset zoom to 100% |
| `Mod+=` | Zoom to fit focused window |
| `Mod+Shift+=` | Zoom to fit all visible |
| `Mod+1/2/3...` | Jump to camera bookmark |
| `Mod+Shift+1/2/3...` | Save camera bookmark |

---

## Implementation Notes

1. **Geometric navigation** requires calculating "nearest column to origin" when crossing rows
2. **Camera bookmarks** replace workspace concept — store (x, y, zoom) tuples
3. **Row creation** is implicit — moving window up from top row creates new row
4. **Row deletion** is automatic — empty rows are removed

---

*Resolved by TEAM_004 based on USER feedback.*
