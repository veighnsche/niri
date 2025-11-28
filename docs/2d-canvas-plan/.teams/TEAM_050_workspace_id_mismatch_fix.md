# TEAM_050 — Deep Test Fixes (Workspace ID, Fullscreen View Offset)

## Team Number
TEAM_050

## Objective
Fix deep issues identified in TEAM_049's analysis - focused on workspace ID mismatch and fullscreen view offset preservation.

## Summary
**Fixed 12 test failures** (51 → 39 failures)

## Issues Fixed

### Issue 1: Workspace ID Mismatch (Easy Win)
**Location**: `src/layout/monitor/hit_test.rs:92`

**Root Cause**: Creating synthetic `WorkspaceId::specific(idx as u64)` instead of using actual `ws.id()`.

**Fix**: Use `ws.id()` for workspace identification.

**Impact**: Fixed ~5 interactive move tests.

### Issue 2: Fullscreen/Unfullscreen View Offset (Medium Complexity)
**Location**: Multiple files

**Root Cause**: Three interconnected issues:
1. `Canvas2D::set_fullscreen` was a TODO stub (did nothing)
2. View offset calculation used `sizing_mode()` instead of `pending_sizing_mode()`
3. `Row::update_window` lacked `view_offset_to_restore` save/restore logic

**Fixes Applied**:
1. **Canvas2D operations** (`canvas/operations.rs`): Implemented `set_fullscreen`, `toggle_fullscreen`, `set_maximized`, `toggle_maximized` by delegating to rows
2. **View offset calculation** (`row/view_offset.rs`): Changed to use `pending_sizing_mode()` for correct offset when transitioning to fullscreen
3. **Row update_window** (`row/mod.rs`): Added complete view_offset_to_restore save/restore logic for fullscreen transitions

**Impact**: Fixed ~7 fullscreen-related tests.

## Files Modified
- `src/layout/monitor/hit_test.rs` - Fixed workspace ID (TEAM_050)
- `src/layout/canvas/operations.rs` - Implemented fullscreen methods (TEAM_050)
- `src/layout/row/view_offset.rs` - Use pending_sizing_mode (TEAM_050)
- `src/layout/row/mod.rs` - View offset save/restore in update_window, simplified set_fullscreen (TEAM_050)

## Test Results
- **Before**: 51 failures
- **After**: 39 failures
- **Fixed**: 12 tests

## Remaining Failures (39)
- **Floating tests** (~24): Complex interactions with expected_size, request_size_once
- **Animation tests** (2): Width resize and cancel
- **Window opening tests** (2): Target workspace issues
- **Misc** (~11): Various workspace/layout tests

## Handoff Notes for Next Team

### Priority 1: Floating Tests (~24 failures)
Most failures are in `tests::floating`. These involve complex state management around:
- `expected_size()` - how window expected sizes are calculated
- `request_size_once` state machine
- Size restoration after fullscreen/maximize

### Priority 2: Animation Tests (2 failures)
- `width_resize_and_cancel`
- `width_resize_and_cancel_of_column_to_the_left`
Related to cached ColumnData not updating during animations (per TEAM_049 analysis).

### Priority 3: Window Opening Tests (2 failures)
- `target_output_and_workspaces`
- Related to workspace targeting when opening windows.

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests improved: 51 → 39 failures
- [ ] Golden tests pass (`cargo insta test`) — not checked, layout logic changes may affect
- [x] Team file complete
