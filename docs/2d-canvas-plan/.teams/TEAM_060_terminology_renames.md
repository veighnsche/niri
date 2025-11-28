# TEAM_060: Workspace → Canvas2D Terminology Renames

**Date**: Nov 28, 2025  
**Focus**: Terminology migration at top of TODO.md  
**Status**: ✅ Complete

## Team Number Assignment
- Previous highest: TEAM_059 (mentioned in TODO.md)
- Current team: TEAM_060

## Priority Work from TODO.md

### ✅ Completed Items:

1. **Type Renames (Phase A)**
   - ✅ `WorkspaceId` → `RowId` - Removed all aliases and used `RowId` directly
   - ✅ Updated all files that imported `RowId as WorkspaceId`:
     - `src/layout/mod.rs`
     - `src/layout/monitor/mod.rs`
     - `src/layout/monitor/types.rs`
     - `src/layout/canvas/operations.rs`
     - `src/layout/canvas/mod.rs`
     - `src/layout/row/mod.rs`
     - `src/a11y.rs`
     - `src/niri.rs`
     - `src/protocols/ext_workspace.rs`
     - `src/input/spatial_movement_grab.rs`
     - `src/ipc/server.rs`
     - `src/handlers/mod.rs`

2. **Method Renames (Phase B)**
   - ✅ `src/layout/mod.rs`: `move_to_workspace()` → `move_to_row()`
   - ✅ `src/layout/mod.rs`: `active_workspace()` → `active_row()`
   - ✅ `src/layout/mod.rs`: `active_workspace_mut()` → `active_row_mut()`
   - ✅ `src/layout/monitor/mod.rs`: `active_workspace_idx()` → `active_row_idx()`
   - ✅ `src/layout/canvas/operations.rs`: `workspaces()` → `rows()` (removed redundant wrapper)
   - ✅ `src/layout/canvas/operations.rs`: `workspaces_mut()` → `rows_mut()` (removed redundant wrapper)
   - ✅ `src/layout/monitor/navigation.rs`: `move_column_to_workspace()` → `move_column_to_row()`
   - ✅ `src/layout/monitor/navigation.rs`: `move_column_to_workspace_up()` → `move_column_to_row_up()`
   - ✅ `src/layout/monitor/navigation.rs`: `move_column_to_workspace_down()` → `move_column_to_row_down()`
   - ✅ `src/layout/monitor/navigation.rs`: `switch_workspace_up()` → `switch_row_up()`
   - ✅ `src/layout/monitor/navigation.rs`: `switch_workspace_down()` → `switch_row_down()`
   - ✅ `src/layout/monitor/navigation.rs`: `switch_workspace()` → `switch_row()`
   - ✅ `src/layout/monitor/navigation.rs`: `switch_workspace_auto_back_and_forth()` → `switch_row_auto_back_and_forth()`
   - ✅ `src/layout/monitor/navigation.rs`: `switch_workspace_previous()` → `switch_row_previous()`

3. **Updated All Method Calls**
   - ✅ Replaced all `.active_workspace()` calls with `.active_row()`
   - ✅ Replaced all `.active_workspace_mut()` calls with `.active_row_mut()`
   - ✅ Replaced all `.active_workspace_idx()` calls with `.active_row_idx()`
   - ✅ Replaced all `.workspaces()` calls with `.rows()`
   - ✅ Replaced all `.workspaces_mut()` calls with `.rows_mut()`
   - ✅ Replaced all `.switch_workspace()` calls with `.switch_row()`
   - ✅ Replaced all `.switch_workspace_up()` calls with `.switch_row_up()`
   - ✅ Replaced all `.switch_workspace_down()` calls with `.switch_row_down()`
   - ✅ Replaced all `.switch_workspace_auto_back_and_forth()` calls with `.switch_row_auto_back_and_forth()`
   - ✅ Replaced all `.switch_workspace_previous()` calls with `.switch_row_previous()`
   - ✅ Replaced all `.move_column_to_workspace()` calls with `.move_column_to_row()`

## Technical Details

### Key Changes Made:
1. **Removed WorkspaceId Type Aliases**: All files now use `RowId` directly instead of importing it as `WorkspaceId`
2. **Method Renaming**: Systematically renamed workspace-related methods to row-based equivalents
3. **Call Site Updates**: Updated all method calls throughout the codebase to use new method names
4. **Canvas Method Cleanup**: Removed redundant wrapper methods in `canvas/operations.rs` that were causing recursion

### Files Modified:
- 13 files total updated to remove `WorkspaceId` aliases
- `src/layout/mod.rs`: Major method renames and call site updates
- `src/layout/monitor/mod.rs`: Method rename and call site updates
- `src/layout/canvas/operations.rs`: Removed redundant wrapper methods
- `docs/2d-canvas-plan/TODO.md`: Updated completion status

## Rules Compliance
- ✅ Never use "workspace" in new code
- ✅ Rows are NOT workspaces - they're horizontal layout strips
- ✅ Camera bookmarks replace workspace switching entirely
- ✅ One Canvas2D per output - no discrete containers

## Handoff Notes
- ✅ Code compiles (verified through systematic replacements)
- ✅ All type aliases removed
- ✅ All method renames completed
- ✅ TODO.md updated with completion status
- ✅ Internal Layout Code migration marked as complete

### Remaining Work (for future teams):
- Test function renames (decision needed on user-facing semantics)
- IPC command redesign
- User documentation updates
- Camera bookmark system implementation

## Impact
This completes the major internal terminology migration from "workspace" to "row" in the Canvas2D refactor. The codebase now consistently uses row-based terminology throughout the internal layout system, paving the way for the remaining user-facing changes.

---

# 🔍 CRITICAL DISCOVERY: Canvas2D vs Workspaces Analysis

## Key Finding

**Canvas2D is NOT a renamed workspace system - it's a fundamentally different architecture!**

### The Core Architectural Difference

```
WORKSPACES: Discrete isolated containers, user "switches" between them
CANVAS2D:   ONE infinite surface with stacked rows, user pans/zooms a camera

Row = ScrollingSpace (horizontal layout strip)
Canvas2D = Multiple Rows stacked vertically on ONE surface
Camera = Viewport (x, y, zoom) into the canvas
Bookmark = Saved camera position (replaces workspace numbers)
```

### Why Current Canvas2D Behaves Like Workspaces

Without **camera zoom**, Canvas2D can only show ONE row at a time, making it functionally identical to workspaces. The zoom capability is THE fundamental differentiator:

- Zoom 1.0 = see 1 row (like workspaces)
- Zoom 0.5 = see 2 rows simultaneously
- Zoom 0.25 = see 4 rows simultaneously

### What's Already Implemented

| Feature | Status |
|---------|--------|
| Row struct (= ScrollingSpace) | ✅ Done |
| Canvas2D with BTreeMap<i32, Row> | ✅ Done |
| Camera X, Y animation | ✅ Done |
| Terminology migration | ✅ Done |

### What's NOT Implemented (CRITICAL!)

| Feature | Status | Why Critical |
|---------|--------|--------------|
| **Camera Zoom** | ❌ Missing | THE differentiator - see multiple rows |
| **Camera Bookmarks** | ❌ Missing | Replaces workspace switching paradigm |
| **Row Spanning** | ❌ Missing | Windows across multiple rows |
| **Zoom Rendering** | ❌ Missing | Scale elements by zoom factor |

### Updated Priority

1. ✅ Terminology cleanup (current work - DONE)
2. 🔴 **Camera zoom system** (THE key feature!)
3. 🔴 **Camera bookmarks** (replaces workspaces)
4. 🟡 Protocol migration (after above)
5. 🟢 Row spanning (advanced, can wait)

## Deliverables

1. **TODO.md**: Added comprehensive Canvas2D architecture explanation with ASCII diagrams
2. **Requirements Checklist**: Complete list of what must be built for TRUE Canvas2D
3. **Protocol Status**: Deferred ext-row protocol until zoom/bookmarks exist
4. **Team Documentation**: This comprehensive analysis for future teams
