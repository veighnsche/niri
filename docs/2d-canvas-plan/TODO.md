# Global TODO List — 2D Canvas Refactor

> **Check this file first** before starting work.
> This is the single source of truth for what needs to be done.

**Last updated**: TEAM_053

---

# 📊 CURRENT STATUS

| Metric | Value |
|--------|-------|
| **Build** | ✅ Compiles |
| **Tests** | 213 passed, 55 failed (79.5%) |
| **Golden Tests** | ✅ 88/88 pass (0 failing) |
| **TODOs in codebase** | 84 total |

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

**Why Complex**: 
- Requires implementing name collision detection across all rows in a canvas
- Need to decide on conflict resolution strategy (reject, auto-rename, etc.)
- Must integrate with existing `set_row_name` API without breaking callers
- Test failure indicates this affects `move_window_to_workspace_with_different_active_output`

**Requirements**:
1. Scan all existing row names in canvas when setting new name
2. Implement conflict resolution policy
3. Maintain API compatibility
4. Add tests for name collision scenarios

---

## 🟡 MEDIUM PRIORITY (Functional Enhancements)

### src/layout/mod.rs - Line 798
**TODO**: `TEAM_024: Implement canvas cleanup logic`

**Why Complex**:
- Need to understand canvas lifecycle and when rows should be removed
- Current code shows partial logic for removing empty rows
- Must ensure cleanup doesn't break active window management
- Need to define "empty canvas" conditions and edge cases

**Requirements**:
1. Define canvas cleanup triggers (output removal, row emptiness, etc.)
2. Implement safe row removal without breaking active windows
3. Handle edge cases (all rows empty, single row remaining, etc.)

### src/layout/mod.rs - Line 1052
**TODO**: `TEAM_023: Implement window height setting on canvas/row`

**Why Complex**:
- Requires designing Canvas2D API for window height manipulation
- Original Workspace API had height setting methods that need Canvas2D equivalent
- Must coordinate between Row and Column layout systems
- Height setting affects column width calculations and view offsets

**Requirements**:
1. Design Canvas2D window height API
2. Implement height propagation from Row to Column
3. Update layout calculations for height changes
4. Maintain compatibility with existing resize logic

### src/layout/mod.rs - Line 1069
**TODO**: `TEAM_023: Implement proper workspace ID to row mapping`

**Why Complex**:
- Legacy workspace ID system doesn't map cleanly to Canvas2D row indices
- External systems (IPC, protocols) still expect workspace IDs
- Need to maintain backward compatibility while using row-based internals
- Current default to row 0 is a temporary workaround

**Requirements**:
1. Design stable ID mapping strategy (row index → workspace ID)
2. Handle row creation/deletion without breaking ID stability
3. Update external system interfaces to use new mapping
4. Consider migration path for existing workspace ID users

### src/layout/row/operations/move_col.rs - Line 52
**TODO**: `TEAM_006: Animate column movement (port from ScrollingSpace)`

**Why Complex**:
- Requires extracting animation logic from original ScrollingSpace
- Animation system needs to work with Row-based layout instead of workspace
- Must coordinate with existing view offset animations
- Need to handle animation interruption and queuing

**Requirements**:
1. Extract column movement animation from ScrollingSpace
2. Adapt animation to Row context and Canvas2D coordinates
3. Integrate with existing animation framework
4. Handle animation state management and interruptions

### src/layout/row/mod.rs - Line 2002
**TODO**: `Implement proper conversion using working area`

**Why Complex**:
- `floating_logical_to_size_frac` needs proper coordinate system conversion
- Must account for working area constraints (panels, docks, etc.)
- Conversion affects floating window positioning and sizing
- Need to understand SizeFrac coordinate system requirements

**Requirements**:
1. Implement working area aware coordinate conversion
2. Handle edge cases (windows larger than working area)
3. Ensure conversion is reversible and accurate
4. Add tests for various working area configurations

### src/layout/monitor/render.rs - Line 45
**TODO**: `TEAM_022: Implement proper insert hint rendering with canvas`

**Why Complex**:
- Insert hint rendering needs Canvas2D integration
- Must calculate hint positions in 2D canvas coordinates
- Rendering system needs to handle canvas viewport and zoom
- Original workspace-based hint rendering won't work directly

**Requirements**:
1. Design Canvas2D insert hint positioning algorithm
2. Implement hint rendering with canvas coordinate system
3. Handle viewport culling and zoom transformations
4. Maintain visual consistency with original hints

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
- ✅ **Completed**: 3 (documentation cleanups)
- 🔴 **High Priority**: 1 (causing test failures)
- 🟡 **Medium Priority**: 6 (functional enhancements)

**Next Team Focus**: Should prioritize the high-priority duplicate name checking issue as it's causing test failures.

**Implementation Order Recommendation**:
1. Fix duplicate name checking (test failure)
2. Implement workspace ID to row mapping (external compatibility)
3. Design Canvas2D window height API
4. Port column movement animations
5. Implement canvas cleanup logic
6. Fix coordinate conversion in floating_logical_to_size_frac
7. Implement insert hint rendering

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
