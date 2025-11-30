# niri Refactor — TODO

> **Last Updated**: Nov 30, 2025 (TEAM_106 bug documentation)
> **Status**: 🔴 CRITICAL BUGS — Fix Priority 0 bugs before continuing refactor
> **Masterplan**: See [`phases/README.md`](phases/README.md) for detailed phase breakdown

---

## 🚨 START HERE

**New to the codebase?** Go to [`phases/README.md`](phases/README.md) first.

---

## 📊 Current Status Summary

### ✅ COMPLETED (Verified by TEAM_066)

| Phase | Description | Team |
|-------|-------------|------|
| Phase 0 | Dead code cleanup | TEAM_062 |
| Phase 1 | elements/ module creation | TEAM_062 |
| Phase 2 | FloatingSpace → canvas/floating/ | TEAM_063 |
| Phase 3 | tile.rs → tile/ module | TEAM_063 |
| Phase 5.1-5.8 | Layout implementation extraction | TEAM_063 |
| Phase 5.9 | interactive_move.rs extraction | TEAM_064 |
| Phase 5.10 | render.rs extraction | TEAM_064 |
| Phase 6 | canvas/operations/ split | TEAM_065 |
| Deferred 1 | row/mod.rs partial split | TEAM_064 |

### 📁 Current Module Architecture

```
src/layout/  (79 files)
├── mod.rs (1860 LOC)           # Core Layout struct
├── types.rs (66 LOC)           # Shared types
├── row_types.rs (66 LOC)       # RowId, etc.
├── snapshot.rs                  # Golden testing
├── tests.rs + tests/           # Tests
├── deprecated/                  # scrolling.rs (reference only)
│
├── animated_value/ (2 files)    # Animation values
├── canvas/ (10 files)           # Canvas2D struct
│   ├── floating/ (4 files)     # FloatingSpace
│   └── operations/ (5 files)   # Window/tile/row ops
├── column/ (12 files)           # Column struct
├── elements/ (7 files)          # Render elements
├── layout_impl/ (12 files)      # Layout methods (3831 LOC)
├── monitor/ (7 files)           # Monitor struct
├── row/ (16 files)              # Row struct
└── tile/ (3 files)              # Tile struct
```

### 🧪 Test Status

- **All tests pass**: 270 tests ✅
- **Layout tests**: 187 tests ✅
- **Golden tests**: 88 tests ✅
- **Warnings**: 48 (unused imports - minor cleanup)

---

## 🐛 CRITICAL BUGS (Priority 0 — Fix Before Refactor Continues)

> These bugs break core UX and must be fixed before proceeding.

### BUG-001: Floating windows not rendering on top of tiles ✅ FIXED
**Severity**: High  
**Expected**: Floating windows should always render above tiled windows (their own layer)  
**Actual**: Floating windows render behind tiles when clicking on tiles  
**Original behavior**: Main branch keeps floaters on top even when tiles get focus  
**Fix**: TEAM_106 - Fixed render order (floating first) and added hit testing for floating windows

### BUG-002: Mod+drag causes all tiles to animate off-screen ✅ FIXED
**Severity**: Critical  
**Fix**: In `view_offset_gesture_end()`, detect DnD gestures and delegate to `dnd_scroll_gesture_end()` to preserve view position.  
**See**: `.bugs/BUG_mod_drag_tiles.md`, `.teams/TEAM_110_mod_drag_tiles_fix.md`

### BUG-002a: Insert hint bar not showing during drag ⚠️ NEW
**Severity**: Medium (UX regression)  
**Expected**: Blue bar shows where window will be inserted  
**Actual**: No insert hint appears  
**See**: `.bugs/BUG_mod_drag_subbug.md`

### BUG-002b: Cannot drag window from right to left ⚠️ NEW
**Severity**: High (Core functionality)  
**Expected**: Can swap window positions in both directions  
**Actual**: Can only drag left→right, not right→left  
**See**: `.bugs/BUG_mod_drag_subbug.md`  

### BUG-003: Cannot drag floating windows by title bar ✅ FIXED
**Severity**: High  
**Expected**: Floating windows can be dragged by clicking and dragging their title bar (CSD)  
**Actual**: Title bar drag does not move floating windows  
**Root Cause**: `move_grab.rs` checked `is_floating` using `workspaces()` which doesn't include floating space - always returned `false` for floating windows, causing horizontal drags to fail  
**Fix**: TEAM_109 - Added `Layout::is_window_floating()` method and updated move_grab.rs. See `.bugs/BUG_floating_drag.md`

### BUG-004: Mod+R/Mod+F affects tiled windows when floating is active ✅ FIXED

### BUG-005: Floating window close animation missing ✅ FIXED
**Severity**: Medium  
**Expected**: Floating windows should have close animation like tiled windows  
**Actual**: Floating windows disappear instantly when closed  
**Root Cause**: `Layout::store_unmap_snapshot` didn't check floating space - snapshot never stored  
**Fix**: TEAM_107 - Added `store_unmap_snapshot_if_empty`/`clear_unmap_snapshot` to FloatingSpace, updated Layout methods to check floating first. See `.bugs/BUG_floating_close_animation.md`

### BUG-006: No window selected after floating window closes ✅ FIXED
**Severity**: High  
**Expected**: When floating window closes, focus should fall back to a tiled window  
**Actual**: No window is selected/focused after closing floating window  
**Root Cause**: `Layout::remove_window()` bypassed `Canvas2D::remove_window()`, directly calling `floating.remove_tile()` without `update_focus_after_removing()`  
**Fix**: TEAM_108 - Made `update_focus_after_removing` public, added calls in `Layout::remove_window()` for both MonitorSet branches. See `.bugs/BUG_floating_close_no_focus.md`  

---

## 🎯 REMAINING WORK

> ⚠️ **BUGS FIRST**: Fix Priority 0 bugs before continuing refactor.
> Clean architecture enables sustainable feature development.

---

### Part A: niri.rs Modular Refactor 🔴 BLOCKED (pending bug fixes)

Split `src/niri.rs` (6604 LOC) into focused modules (<500 LOC each).

| Phase | File | Description | Status |
|-------|------|-------------|--------|
| [**A1**](phases/phase-A1-niri-types.md) | niri/types.rs | Extract pure data types | 🔄 CURRENT |
| [A2](phases/phase-A2-niri-output.md) | niri/output.rs | Output management | ⏳ Pending |
| [A3](phases/phase-A3-niri-hit-test.md) | niri/hit_test.rs | Hit testing queries | ⏳ Pending |
| [A4](phases/phase-A4-niri-lock.md) | niri/lock.rs | Session lock | ⏳ Pending |
| [A5](phases/phase-A5-niri-render.md) | niri/render.rs | Rendering + frame callbacks | ⏳ Pending |
| [A6](phases/phase-A6-niri-capture.md) | screenshot/screencopy/screencast | Screen capture | ⏳ Pending |
| [A7](phases/phase-A7-niri-input.md) | niri/pointer.rs + rules.rs | Input & rules | ⏳ Pending |
| [A8](phases/phase-A8-niri-init.md) | niri/init.rs | Constructor extraction | ⏳ Pending |

**Estimated Time**: ~8 hours total

---

### Priority 2: Minor Cleanup Tasks (During Refactor)

- [ ] Remove 48 unused import warnings
- [ ] Clean up internal `seen_workspace_*` variable names → `seen_row_*`
- [ ] Delete `deprecated/scrolling.rs` after confirming no references

**Total**: ~1 hour

---

### Priority 3: Granular Redraw Optimization

**Location**: `src/input/actions.rs` (80 occurrences of `// FIXME: granular`)

**Problem**: Many actions call `queue_redraw_all()` which redraws ALL outputs, even when only ONE output was affected. This is wasteful on multi-monitor setups.

**Why it matters**:
1. **Performance**: Redrawing all monitors when only one changed wastes GPU cycles
2. **Power consumption**: Unnecessary redraws drain laptop batteries faster
3. **Latency**: Full redraws take longer than targeted ones

**Solution**: Replace `queue_redraw_all()` with `queue_redraw(&output)` where the affected output can be determined.

**Affected Actions** (62 unique actions, 80 total FIXME comments):

| Category | Actions |
|----------|---------|
| **Focus** | `FocusColumnLeft`, `FocusColumnRight`, `FocusColumnFirst`, `FocusColumnLast`, `FocusColumnLeftOrLast`, `FocusColumnRightOrFirst`, `FocusColumn(index)`, `FocusWindowUp`, `FocusWindowDown`, `FocusWindowTop`, `FocusWindowBottom`, `FocusWindowDownOrTop`, `FocusWindowUpOrBottom`, `FocusWindowDownOrColumnLeft`, `FocusWindowDownOrColumnRight`, `FocusWindowUpOrColumnLeft`, `FocusWindowUpOrColumnRight`, `FocusWindowInColumn(index)`, `FocusWindowOrRowDown`, `FocusWindowOrRowUp`, `FocusRowDown`, `FocusRowUp`, `FocusPreviousPosition`, `FocusFloating`, `FocusTiling`, `SwitchFocusBetweenFloatingAndTiling` |
| **Move Column** | `MoveColumnToFirst`, `MoveColumnToLast`, `MoveColumnToIndex(idx)`, `MoveColumnToRowDown(focus)`, `MoveColumnToRowUp(focus)` |
| **Move Window** | `MoveWindowToRowDown(focus)`, `MoveWindowToRowUp(focus)`, `MoveWindowToFloating`, `MoveWindowToFloatingById(id)`, `MoveWindowToTiling`, `MoveWindowToTilingById(id)` |
| **Move Row** | `MoveRowDown`, `MoveRowUp`, `MoveRowToIndex(new_idx)` |
| **Resize/Layout** | `SetColumnWidth(change)`, `SetWindowWidth(change)`, `SetWindowHeight(change)`, `SetColumnDisplay(display)`, `ToggleColumnTabbedDisplay`, `SwapWindowLeft`, `SwapWindowRight`, `ConsumeWindowIntoColumn`, `ExpelWindowFromColumn`, `ConsumeOrExpelWindowLeft`, `ConsumeOrExpelWindowRight` |
| **Fullscreen** | `FullscreenWindow`, `FullscreenWindowById(id)`, `ToggleWindowedFullscreen`, `ToggleWindowedFullscreenById(id)` |
| **Maximize** | `MaximizeWindowToEdges`, `MaximizeWindowToEdgesById(id)` |
| **Center** | `CenterColumn`, `CenterWindow`, `CenterWindowById(id)`, `CenterVisibleColumns` |
| **Floating** | `ToggleWindowFloating`, `ToggleWindowFloatingById(id)` |

**Implementation approach**:
1. Most actions operate on the active output → use `self.niri.layout.active_output()`
2. Some actions (like `*ById`) may affect a different output → need to find window's output first
3. Some actions may affect multiple outputs (cross-monitor moves) → may still need `queue_redraw_all()` or redraw both

**Estimated Time**: ~4-6 hours (requires careful analysis of each action)

---

## 🚀 Part B: FEATURES (After Refactor Complete)

> These features are **BLOCKED** until Part A is complete.

| Phase | Description | Time | Status |
|-------|-------------|------|--------|
| B1 | Camera Zoom System | ~3.5h | ⏸️ Blocked |
| B2 | Camera Bookmarks | ~3h | ⏸️ Blocked |
| B3 | IPC/Protocol Migration | ~2.5h | ⏸️ Blocked |
| B4 | Row Spanning | ~2.5h | ⏸️ Blocked |

Phase files will be created when Part A is complete.

---

## 📋 Documentation Tasks (After Features)

- [ ] Update README.md with Canvas2D architecture
- [ ] Update wiki examples for new row syntax

---

## 📊 Total Remaining Effort

| Part | Phases | Time | Status |
|------|--------|------|--------|
| **Part A** | A1-A8 (niri.rs refactor) | ~8h | 🔴 CURRENT |
| Cleanup | During Part A | ~1h | Pending |
| Granular Redraw | 80 FIXMEs in actions.rs | ~5h | Pending |
| **Part B** | B1-B4 (features) | ~11.5h | ⏸️ Blocked |
| **Total** | | **~25.5h** | |

---

## ✅ Success Criteria

### Part A Complete (Refactor) ✓
- [ ] niri.rs < 700 LOC
- [ ] Each module < 500 LOC
- [ ] No `pub(super)`
- [ ] All tests pass
- [ ] No unused imports

### Part B Complete (Features) ✓
- [ ] Zoom works
- [ ] Bookmarks work
- [ ] IPC reflects rows

---

## 🗂️ Reference

### Team Files
See `docs/2d-canvas-plan/.teams/TEAM_0XX_*.md` for detailed history.

### Key Teams
- TEAM_062: Architecture refactor plan
- TEAM_063: FloatingSpace + tile + layout_impl extraction
- TEAM_064: Interactive move + render + row state extraction
- TEAM_065: canvas/operations split
- TEAM_066: TODO audit/cleanup
- TEAM_067: niri.rs refactor masterplan
- TEAM_103: Technical debt stub cleanup

### Original Docs
- `docs/2d-canvas-plan/README.md` - Original vision
- `docs/2d-canvas-plan/phases/` - Phase documentation
