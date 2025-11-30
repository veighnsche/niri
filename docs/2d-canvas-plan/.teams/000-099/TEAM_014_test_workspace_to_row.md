# TEAM_014: Test Workspace → Row + Overview Removal (Parts 2E & 3)

## Status: COMPLETE ✅

## Team Assignment
- **Team Number**: 014
- **Task**: Phase 1.5.3 Part 2E — Replace workspace tests with row tests
- **Previous Team**: TEAM_013 (Monitor modular refactor complete)

## Starting Point
- Golden tests pass (91 tests)
- 284 tests pass total
- Parts 2A-2D complete

## Work Plan

Per `phase-1.5.3-part2e-callsites.md`:
1. Rename Op enum variants (Workspace → Row)
2. Update Op handlers (match arms)
3. Rename test functions
4. Update test data arrays
5. Verify all tests pass

## Progress

### Step 1: Rename Op Enum Variants ✅
- [x] Focus operations: FocusWindowOrWorkspaceDown/Up → FocusWindowOrRowDown/Up
- [x] Move window operations: MoveWindowDownOrToWorkspaceDown/Up → MoveWindowDownOrToRowDown/Up
- [x] Focus row operations: FocusWorkspace* → FocusRow*
- [x] Move to row operations: MoveWindowToWorkspace* → MoveWindowToRow*
- [x] Move column to row: MoveColumnToWorkspace* → MoveColumnToRow*
- [x] Row reorder: MoveWorkspace* → MoveRow*
- [x] Row naming: SetWorkspaceName/UnsetWorkspaceName → SetRowName/UnsetRowName
- [x] Gesture operations: WorkspaceSwitchGesture* → RowPanGesture*
- [x] MoveWorkspaceToOutput → MoveRowToOutput

### Step 2: Update Op Handlers ✅
- [x] All match arms updated to use new variant names
- [x] Layout method calls preserved (already renamed by TEAM_012)

### Step 3: Rename Test Functions ✅
- [x] golden_w3_focus_window_or_workspace_down → golden_w3_focus_window_or_row_down
- [x] Renamed corresponding snapshot file

### Step 4: Update Test Data Arrays ✅
- [x] Updated every_op arrays in operations_dont_panic
- [x] Updated every_op arrays in operations_from_starting_state_dont_panic
- [x] Updated setup_ops array

### Step 5: Update Individual Test Usages ✅
- [x] Updated ~50+ Op usages across tests.rs
- [x] Updated fullscreen.rs (2 usages)
- [x] Updated golden.rs (1 usage)

## Changes Made

### Files Modified
- `src/layout/tests.rs` - Op enum, handlers, test arrays, individual tests
- `src/layout/tests/fullscreen.rs` - 2 Op usages
- `src/layout/tests/golden.rs` - 1 test function renamed, 1 Op usage
- `src/layout/tests/snapshots/niri__layout__tests__golden__golden_w3_focus_window_or_row_down.snap` - Renamed from workspace version

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test --lib` - 284 tests)
- [x] Golden tests pass (`./scripts/verify-golden.sh` - 91 tests)
- [x] Team file complete

## Summary

Completed Phase 1.5.3 Part 2E: Renamed all workspace-related Op enum variants to use Row terminology. This is a cosmetic change that aligns the test infrastructure with the renamed layout methods from Parts 2A-2D.

### Op Variants Renamed (~25 total)
| Old Name | New Name |
|----------|----------|
| FocusWindowOrWorkspaceDown/Up | FocusWindowOrRowDown/Up |
| MoveWindowDownOrToWorkspaceDown/Up | MoveWindowDownOrToRowDown/Up |
| FocusWorkspaceDown/Up | FocusRowDown/Up |
| FocusWorkspace(idx) | FocusRow(idx) |
| FocusWorkspaceAutoBackAndForth | FocusRowAutoBackAndForth |
| FocusWorkspacePrevious | FocusPreviousPosition |
| MoveWindowToWorkspaceDown/Up | MoveWindowToRowDown/Up |
| MoveWindowToWorkspace | MoveWindowToRow |
| MoveColumnToWorkspaceDown/Up | MoveColumnToRowDown/Up |
| MoveColumnToWorkspace | MoveColumnToRow |
| MoveWorkspaceDown/Up | MoveRowDown/Up |
| MoveWorkspaceToIndex | MoveRowToIndex |
| MoveWorkspaceToMonitor | MoveRowToMonitor |
| MoveWorkspaceToOutput | MoveRowToOutput |
| SetWorkspaceName | SetRowName |
| UnsetWorkspaceName | UnsetRowName |
| WorkspaceSwitchGestureBegin/Update/End | RowPanGestureBegin/Update/End |

## Part 3: Overview Removal (DISABLED)

After completing Part 2E, TEAM_014 proceeded to disable overview mode:

### What Was Done
1. **Removed overview fields** from `Layout` struct (`overview_open`, `overview_progress`)
2. **Removed overview types** (`OverviewProgress`, `OverviewGesture`)
3. **Made overview methods return constants**:
   - `is_overview_open()` → always `false`
   - `overview_zoom()` → always `1.0`
4. **Disabled overview gestures** (4-finger swipe now no-op)
5. **Disabled hot corner** overview trigger
6. **Disabled overview actions** (`ToggleOverview`, `OpenOverview`, `CloseOverview`)
7. **Added `DEPRECATED(overview)` tags** to all affected code for easy cleanup

### Files Modified for Overview Removal
- `src/layout/mod.rs` — Removed fields, types, methods
- `src/layout/monitor/mod.rs` — Removed `overview_open`, `overview_progress` fields
- `src/layout/monitor/types.rs` — Removed `OverviewProgress` type
- `src/layout/monitor/workspace_compat.rs` — Stubbed `overview_zoom()`
- `src/layout/monitor/render.rs` — Disabled workspace shadows
- `src/layout/monitor/hit_test.rs` — Removed overview checks
- `src/layout/monitor/gestures.rs` — Disabled DnD scroll gesture
- `src/layout/monitor/navigation.rs` — Removed overview animation sync
- `src/layout/workspace.rs` — Disabled workspace shadow config
- `src/layout/tests.rs` — Made overview ops no-ops
- `src/input/mod.rs` — Disabled gestures, hot corner, actions
- `src/input/move_grab.rs` — Removed overview handling
- `src/input/touch_overview_grab.rs` — Still exists but unused

### Cleanup Guide Created
See `docs/2d-canvas-plan/phases/phase-1.5.3-part3-overview-removal-guide.md` for step-by-step instructions to fully delete all overview code.

### How to Find Deprecated Code
```bash
grep -rn "DEPRECATED(overview)" src/
```

## Notes for Next Team

Phase 1.5.3 Parts 2E and 3 are complete. Overview is DISABLED but not fully removed.

**Next steps:**
- **Part 3 Cleanup**: Follow the removal guide to delete all `DEPRECATED(overview)` code
- **Part 4**: Remove workspace fields from Monitor
- **Part 5**: Remove workspace config and IPC
