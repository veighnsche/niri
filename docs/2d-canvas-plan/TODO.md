# Global TODO List — 2D Canvas Refactor

> **Check this file first** before starting work.
> This is the single source of truth for what needs to be done.

**Last updated**: TEAM_060

---

# 🚨 WORKSPACE → CANVAS2D TERMINOLOGY MIGRATION

> **CRITICAL**: Workspaces are **COMPLETELY REMOVED** from Canvas2D.  
> Rows are NOT workspaces. See `README.md` for full explanation.

## Migration Status Overview

| Area | Status | Notes |
|------|--------|-------|
| **Internal Layout Code** | ✅ Complete | TEAM_060 - WorkspaceId→RowId, method renames |
| **Config (niri-config)** | ✅ Complete | TEAM_055 |
| **Test Operations (Op::)** | ✅ Complete | TEAM_014 |
| **Test Function Names** | ⏳ Pending | Still use "workspace" |
| **IPC Commands** | ⏳ Pending | Will be redesigned |
| **User Documentation** | ⏳ Pending | After code migration |

---

## 📋 TERMINOLOGY MIGRATION CHECKLIST

### Legend
- ✅ = Complete
- 🔄 = In Progress  
- ⏳ = Pending
- ❌ = Blocked

---

### 1. Type/Struct Renames

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `WorkspaceId` | `RowId` | `src/layout/row_types.rs` | ✅ Done |
| `WorkspaceAddWindowTarget` | `RowAddWindowTarget` | `src/layout/row_types.rs` | ✅ Done |
| `Workspace` (config) | `RowConfig` | `niri-config/src/` | ✅ Done |
| `WorkspaceName` | `RowName` | `niri-config/src/` | ✅ Done |

### 2. Method Renames (src/layout/mod.rs)

| Old Name | New Name | Status |
|----------|----------|--------|
| `move_to_workspace()` | `move_to_row()` | ✅ Done |
| `move_to_workspace_up()` | `move_to_row_up()` | ✅ Done |
| `move_to_workspace_down()` | `move_to_row_down()` | ✅ Done |
| `move_column_to_workspace()` | `move_column_to_row()` | ⏳ Pending |
| `focus_workspace()` | `focus_row()` | ⏳ Pending |
| `focus_workspace_up()` | `focus_row_up()` | ⏳ Pending |
| `focus_workspace_down()` | `focus_row_down()` | ⏳ Pending |
| `active_workspace()` | `active_row()` | ✅ Done |
| `active_workspace_idx()` | `active_row_idx()` | ✅ Done |
| `find_workspace_by_name()` | `find_row_by_name()` | ✅ Done |
| `ensure_named_workspace()` | `ensure_named_row()` | ✅ Done |

### 3. Method Renames (src/layout/monitor/)

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `active_workspace_idx()` | `active_row_idx()` | `mod.rs` | ✅ Done |
| `workspaces()` | `rows()` | `mod.rs` | ✅ Done |
| `workspaces_mut()` | `rows_mut()` | `mod.rs` | ✅ Done |

### 4. Method Renames (src/layout/canvas/)

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `workspaces()` | `rows()` | `operations.rs` | ✅ Done |
| `workspaces_mut()` | `rows_mut()` | `operations.rs` | ✅ Done |

### 5. Field Renames

| Old Name | New Name | File | Status |
|----------|----------|------|--------|
| `last_active_workspace_id` | `last_active_row_id` | `src/layout/mod.rs` | ✅ Done |
| `workspace_id_counter` | `row_id_counter` | `src/layout/mod.rs` | ✅ Done |
| `workspace_id` | `row_id` | Various | ⏳ Pending |

### 6. Test Operation Renames (src/layout/tests.rs)

| Old Name | New Name | Status |
|----------|----------|--------|
| `Op::MoveWindowToWorkspace*` | `Op::MoveWindowToRow*` | ✅ Done |
| `Op::MoveColumnToWorkspace*` | `Op::MoveColumnToRow*` | ✅ Done |
| `Op::FocusWorkspace*` | `Op::FocusRow*` | ✅ Done |
| `Op::MoveWorkspace*` | `Op::MoveRow*` | ✅ Done |
| `Op::SetWorkspaceName` | `Op::SetRowName` | ✅ Done |

### 7. Test Function Renames (src/layout/tests.rs)

> **Note**: These test user-facing behavior, so "workspace" in names may be intentional.
> Decide: Keep for user-facing semantics OR rename to "row"?

| Current Name | Proposed Name | Status |
|--------------|---------------|--------|
| `move_to_workspace_by_idx_*` | `move_to_row_by_idx_*` | ⏳ Pending |
| `move_workspace_to_output` | `move_row_to_output` | ⏳ Pending |
| `removing_all_outputs_preserves_empty_named_workspaces` | `..._named_rows` | ⏳ Pending |
| `removing_output_must_keep_empty_focus_on_primary` | Keep? | ⏳ Decide |

### 8. IPC Commands (src/niri.rs, niri-ipc/)

| Old Command | New Command | Status |
|-------------|-------------|--------|
| `focus-workspace` | `focus-row` OR `camera-goto` | ⏳ Pending |
| `move-window-to-workspace` | `move-window-to-row` | ⏳ Pending |
| `move-column-to-workspace` | `move-column-to-row` | ⏳ Pending |

### 9. Config Options (niri-config/)

| Old Option | New Option | Status |
|------------|------------|--------|
| `workspace { }` | `row { }` | ✅ Done |
| `open-on-workspace` | `open-on-row` | ✅ Done |
| `workspace-switch` animation | `row-switch` animation | ✅ Done |
| `empty-workspace-above-first` | `empty-row-above-first` | ✅ Done |

### 10. NEW: Camera Bookmark System

> **These are NEW features**, not renames

| Feature | Description | Status |
|---------|-------------|--------|
| `CameraBookmark` struct | Stores `(x, y, zoom)` | ⏳ Pending |
| `camera-bookmark-save` IPC | Save current position | ⏳ Pending |
| `camera-bookmark-goto` IPC | Jump to bookmark | ⏳ Pending |
| `Mod+1/2/3` bindings | Jump to bookmark | ⏳ Pending |
| `Mod+Shift+1/2/3` bindings | Save bookmark | ⏳ Pending |

---

## 🎯 MIGRATION PRIORITY ORDER

1. **Phase A**: Internal type renames (`WorkspaceId` → `RowId`)
2. **Phase B**: Internal method renames (all `*workspace*` → `*row*`)
3. **Phase C**: Test function renames (if decided)
4. **Phase D**: IPC command redesign
5. **Phase E**: Camera bookmark implementation
6. **Phase F**: User documentation update

---

## ⚠️ MIGRATION RULES

1. **Never use "workspace" in new code** - use "row" or "canvas"
2. **Rows are NOT workspaces** - they're horizontal layout strips
3. **Camera bookmarks replace workspace switching** - different concept entirely
4. **One Canvas2D per output** - no discrete containers
5. **Update imports** when renaming files/types

---

# 📊 CURRENT STATUS

| Metric | Value |
|--------|-------|
| **Build** | ✅ Compiles |
| **Tests** | 256 passed, 16 failed (94.1%) |
| **Golden Tests** | ❌ Snapshot regressions detected |
| **TODOs in codebase** | 84 total |

---

# 🚨 TEST FAILURES (TEAM_059)

> **Status**: IN PROGRESS  
> **Date**: Nov 28, 2025  

## Recently Fixed (TEAM_059)

1. ✅ `move_window_to_workspace_maximize_and_fullscreen` - Fixed maximize state preservation
2. ✅ `move_to_workspace_by_idx_does_not_leave_empty_workspaces` - Fixed row cleanup/renumbering

## Remaining Failures

### Pattern 1: Floating Window State Issues
**Tests**: `restore_to_floating_persists_across_fullscreen_maximize`, `unmaximize_during_fullscreen_does_not_float`

**Issue**: Floating windows incorrectly appearing in tiled space after fullscreen/maximize operations.

### Pattern 2: Output/Row Management Issues  
**Tests**: `move_workspace_to_output`, `removing_all_outputs_preserves_empty_named_workspaces`, `removing_output_must_keep_empty_focus_on_primary`

**Issue**: Row creation, deletion, and index tracking not working correctly when outputs change.

### Pattern 3: Golden Snapshot Regressions
**Tests**: Various floating/animation tests

**Issue**: Floating window size preservation and animation capture issues.

---

# 🔄 CONTINUOUS TEST ITERATION (TEAM_043 → TEAM_044)

> **Goal**: Run all tests iteratively until 100% pass rate
> **Status**: IN PROGRESS

## Fixes Applied (TEAM_043)

1. ✅ **Fixed refresh not calling Row::refresh()** - Windows weren't receiving configure events
2. ✅ **Fixed width parameter ignored in Monitor::add_window()** - Was hardcoded to 1.0
3. ✅ **Added floating space refresh** - Floating windows now get refreshed
4. ✅ **Fixed set_column_width for floating** - Now routes to FloatingSpace
5. ✅ **Fixed floating set_window_width/height** - Uses size() instead of expected_size()

## Fixes Applied (TEAM_044)

6. ✅ **Fixed Layout::update_window missing floating check** - Floating windows now get on_commit called
7. ✅ **Fixed Row::update_window missing serial parameter** - on_commit now called for tiled windows
8. ✅ **Fixed floating window toggle position** - Now sets floating position based on render position like original Workspace
9. ✅ **Fixed floating focus state management** - Added focus_tiling(), focus_floating(), switch_focus_floating_tiling() to Canvas2D

## Known Issues (TEAM_044 → TEAM_045)

### ✅ Floating Animation Regression (Resolved by TEAM_045)
- **Test**: `golden_u4_toggle_floating_back_to_tiled`
- **Previous Issue**: Missing animations when toggling window from floating back to tiled (animations list empty in snapshot).
- **Fix (TEAM_045)**: Start a tile move animation when re-inserting a window from floating back to tiled in `Canvas2D::toggle_floating_window_by_id`, so `Row::snapshot()` records tile edge animations that match the golden baseline.
- **Status**: **Resolved – all golden tests now pass (88/88)**.

## Remaining Test Categories

- **Floating tests**: ~22 failing (size preservation issues - complex expected_size() interactions)
  - ✅ **RESOLVED (TEAM_054)**: `unfocus_preserves_current_size` - Root cause was Canvas2D's `find_wl_surface()` not searching floating space, so `window.on_commit()` was never called for floating windows.
- **Animation tests**: ~10 failing (move animations)
- **Fullscreen tests**: ~5 failing (view offset preservation)
- **Window opening tests**: ~10 failing (workspace targeting)
- **Interactive move tests**: ~8 failing

---

# 🎯 PHASE 1: Config Migration (CURRENT PRIORITY)

> **Goal**: Replace all `workspace` terminology with `row`
> **Decision**: Remove immediately, no deprecation period

## Config Changes Needed

### niri-config/src/ (TEAM_055 - COMPLETE ✅)

- [x] **workspace.rs** → rename to `row.rs`
  - [x] Rename `Workspace` struct to `RowConfig`
  - [x] Rename `WorkspaceName` to `RowName`
  - [x] Update all references

- [x] **lib.rs**
  - [x] Change `workspaces: Vec<Workspace>` to `rows: Vec<RowConfig>`
  - [x] Update `pub use` statements

- [x] **window_rule.rs** (or wherever window rules are)
  - [x] Rename `open-on-workspace` to `open-on-row`

- [x] **animations.rs**
  - [x] Rename `workspace_switch` to `row_switch` (or remove if not needed)

- [x] **layout.rs**
  - [x] Rename `empty_workspace_above_first` to `empty_row_above_first`

### src/layout/ (TEAM_055 - COMPLETE ✅)

- [x] **workspace_types.rs** → rename to `row_types.rs`
  - [x] Rename `WorkspaceId` to `RowId`
  - [x] Rename `WorkspaceAddWindowTarget` to `RowAddWindowTarget`
  - [x] Update all imports across codebase

- [x] **mod.rs**
  - [x] Rename `find_workspace_by_name` to `find_row_by_name`
  - [x] Rename `ensure_named_workspace` to `ensure_named_row`
  - [x] Rename `last_active_workspace_id` to `last_active_row_id`
  - [x] Rename `workspace_id_counter` to `row_id_counter`

### src/handlers/ (TEAM_055 - COMPLETE ✅)

- [x] **xdg_shell.rs**
  - [x] Update `workspace_name` variable to `row_name`
  - [x] Update `InitialConfigureState::Configured` fields

- [x] **compositor.rs**
  - [x] Update `workspace_id` to `row_id`

### Tests (TEAM_055 - COMPLETE ✅)

- [x] **src/tests/window_opening.rs**
  - [x] Update test configs to use `row` syntax
  - [x] Rename test functions if needed

---

# ✅ RESOLVED: Animation System Bug

> **Status**: FIXED by TEAM_056
> **Result**: All 12 animation tests passing

## Root Causes Found

1. **Missing column render offset** in `Row::tiles_with_render_positions()` - Column's move animation offset wasn't included in position calculation
2. **Asymmetric resize handling** in `Row::update_window()` - Only animated columns to the right, not columns to the left

## Fixes Applied

### Bug 1: `src/layout/row/layout.rs`
Added `col.render_offset()` to tile position calculation:
```rust
let col_render_off = col.render_offset();
let tile_pos = Point::from((
    view_off_x + col_x + col_render_off.x + tile_offset.x + tile.render_offset().x,
    y_offset + col_render_off.y + tile_offset.y + tile.render_offset().y,
));
```

### Bug 2: `src/layout/row/mod.rs`
Added symmetric animation for left-side column resize:
```rust
} else {
    // Resizing a column to the left of active
    for col in &mut self.columns[..=col_idx] {
        col.animate_move_from_with_config(-offset, ...);
    }
}
```

## Test Results
- Animation tests: 12/12 passing ✅
- Golden tests: 86/88 passing (remaining 2 unrelated to animation)

---

# 📋 REMAINING TODOs FROM CODEBASE

## Analysis by TEAM_057

**Status**: Easy TODOs completed, complex items documented below  
**Date**: Nov 28, 2025

---

## 🔴 HIGH PRIORITY (Causing Test Failures)

### src/layout/mod.rs - Line 4752
**TODO**: `TEAM_018: implement proper duplicate name checking for canvas rows`

**Status**: ✅ **FIXED by TEAM_057**

**Root Cause Analysis**:
The test failure was caused by TWO separate issues:
1. **Duplicate row names**: Names weren't checked for duplicates across rows
2. **Duplicate row IDs**: Row IDs were colliding across canvases (different monitors)

**Fixes Implemented**:
1. **canvas/navigation.rs**: Added duplicate name checking in `set_row_name()` - if another row has the same name, clear it first (move the name to the new row)
2. **canvas/operations.rs**: Changed row ID stride from +1 to +1000 in `ensure_row()` to prevent ID collisions between canvases

**Test Result**: `move_window_to_workspace_with_different_active_output` now passes

---

## 🟡 MEDIUM PRIORITY (Functional Enhancements)

### src/layout/mod.rs - Line 798
**TODO**: `TEAM_024: Implement canvas cleanup logic`

**Status**: ✅ **FIXED by TEAM_057**

**Issue**: When `empty_row_above_first` is enabled and there are exactly 2 empty rows, one needs to be removed.

**Fix**: Implemented logic to find and remove the non-origin row (row != 0) when both rows are empty. The origin row (row 0) is always preserved.

**Tests**: All `ewaf` (empty_row_above_first) tests pass.

### src/layout/mod.rs - Line 1052
**TODO**: `TEAM_023: Implement window height setting on canvas/row`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Added `set_window_height()` method to Canvas2D that finds the row containing the window and delegates to Row's existing `set_window_height()` method.

### src/layout/mod.rs - Line 1069
**TODO**: `TEAM_023: Implement proper workspace ID to row mapping`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Added `find_row_by_id()` method to Canvas2D that searches all rows for matching workspace ID. Used in `AddWindowTarget::Workspace` handling.

### src/layout/row/operations/move_col.rs - Line 52
**TODO**: `TEAM_006: Animate column movement (port from ScrollingSpace)`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Ported animation logic from ScrollingSpace:
- Animate the moved column from its old position
- Animate all columns in between (they shift by the moved column's width)
- Uses `animate_move_from()` on each affected column

### src/layout/row/mod.rs - Line 2002
**TODO**: `Implement proper conversion using working area`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: Implemented proper coordinate conversion:
- Subtracts working area location from logical position
- Divides by working area size to get 0.0-1.0 fractions
- Handles edge cases with max(size, 1.0)

### src/layout/monitor/render.rs - Line 45
**TODO**: `TEAM_022: Implement proper insert hint rendering with canvas`

**Status**: ✅ **FIXED by TEAM_057**

**Fix**: 
1. Added `insert_hint_area()` method to Row (ported from ScrollingSpace)
2. Updated `update_render_elements()` in monitor/render.rs to:
   - Look up the row by workspace ID
   - Call `insert_hint_area()` to compute the hint rectangle
   - Update `insert_hint_render_loc` and `insert_hint_element`

---

## 🟢 LOW PRIORITY (Documentation)

### src/layout/row_types.rs - Various lines
**TODO**: Documentation comments about removing WorkspaceId

**Status**: ✅ **COMPLETED by TEAM_057**
- These were just documentation notes, not actionable items
- Comments cleaned up to be purely informational

---

## 📊 SUMMARY

**Total TODOs Analyzed**: 9
- ✅ **Completed**: 9 (ALL DONE!)
- 🔴 **High Priority**: 0 
- 🟡 **Medium Priority**: 0

**TEAM_057 completed ALL remaining TODOs!**

**Implementation Summary**:
1. ~~Fix duplicate name checking (test failure)~~ ✅ DONE
2. ~~Implement canvas cleanup logic~~ ✅ DONE
3. ~~Implement workspace ID to row mapping~~ ✅ DONE
4. ~~Design Canvas2D window height API~~ ✅ DONE
5. ~~Port column movement animations~~ ✅ DONE
6. ~~Fix coordinate conversion~~ ✅ DONE
7. ~~Implement insert hint rendering~~ ✅ DONE

---

*Last Updated: TEAM_057 on Nov 28, 2025*

---

# 🔮 FUTURE PHASES (After Phase 1)

## Phase 2: Row System
- Row naming (any row can be named)
- Row lifecycle (creation/deletion rules)
- Global row ID counter
- Active row tracking

## Phase 3: Row Spanning
- `row_span` field on Tile
- Cross-row occupancy tracking
- `increase-row-span` / `decrease-row-span` actions

## Phase 4: Camera System
- Camera struct with (x, y, zoom)
- Auto-zoom for row span
- Zoom gestures (Mod+Scroll)
- Render transform pipeline

## Phase 5: Camera Bookmarks
- Save/restore camera positions
- `Mod+1/2/3` for bookmarks
- Optional row name reference

## Phase 6: Navigation & Polish
- Geometric navigation (find nearest tile)
- Origin-based leading edges
- Spawn direction based on quadrant
- Documentation

---

# 📝 FOLLOW-UP QUESTIONS

## From TEAM_042 Questionnaire

1. **Row 0 naming**: Can row 0 be renamed, or is "origin" special?
   - Decision: Any row can be named ✅

2. **Negative rows**: Rows can go negative (above origin)?
   - Decision: Yes, rows are indexed ..., -2, -1, 0, 1, 2, ... ✅

3. **Window spanning**: How does a window's row assignment work when spanning?
   - Decision: Top-left corner (0,0 point) determines the row ✅

4. **Zoom behavior**: When zoomed out, how does focus work?
   - Open question: Need to define focus behavior at different zoom levels

5. **Config migration**: How to handle users with old `workspace` configs?
   - Decision: Remove immediately, no deprecation ✅

---

# 🗄️ ARCHIVED (Completed Work)

<details>
<summary>Click to expand completed work history</summary>

## Compilation Fixes (TEAM_030-040)
- All MonitorSet::NoOutputs patterns updated
- All method call parens fixed
- All workspace field accesses migrated to canvas
- All Monitor/Row methods implemented
- All type mismatches resolved
- All borrow checker issues fixed

## Core Migration (TEAM_021)
- workspace.rs (1,997 lines) DELETED
- workspace_compat.rs (302 lines) DELETED
- workspace_ops.rs DELETED
- Canvas2D is sole layout system

## Row Implementation (TEAM_036-040)
- `window_under()`, `resize_edges_under()` implemented
- `activate_window()`, `is_urgent()` implemented
- `set_fullscreen()`, `toggle_fullscreen()` implemented
- `set_maximized()`, `toggle_maximized()` implemented
- `configure_new_window()`, `update_window()` implemented
- `toggle_width()`, `toggle_window_width/height()` implemented
- `find_wl_surface()`, `find_wl_surface_mut()` implemented

## Animation System (TEAM_039)
- Move animation creation logic implemented
- Old position calculation fixed
- Delta calculation working
- Animation parameters fixed (0,1,0 → 1,0,0)
- Rendering integration confirmed

## Floating Window System (TEAM_044)
- ✅ Floating toggle position calculation fixed (based on render position)
- ✅ Floating focus state management implemented
- ✅ Golden snapshot system expanded for floating windows
- ❌ **Missing**: Floating-to-tiled animation in toggle_floating_window_by_id
- ❌ **Missing**: Animation capture for golden tests when returning from floating

</details>

---

*Check `phases/` for detailed phase documentation.*
*Check `.questions/` for architecture decisions.*
*Check `.teams/` for team handoff notes.*
