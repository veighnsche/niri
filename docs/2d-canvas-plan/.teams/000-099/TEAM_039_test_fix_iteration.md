# TEAM_039 — Test Fix Iteration

## Mission
Run all tests (including golden tests) and fix code until all tests pass.
This is a continuous process — iterate until 100% test pass rate.

## Status: IN PROGRESS

## Approach
1. Run `cargo test` to identify failing tests
2. Analyze failures to understand root causes
3. Fix product code (NOT tests) to match expected behavior
4. Repeat until all tests pass

## Test Run Log

### Run 1: Initial Assessment
- **Started**: Session start
- **Status**: 105 passed, 163 failed

### Run 2: After configure_new_window fix
- **Status**: 106 passed, 162 failed
- **Fixed**: `Row::configure_new_window()` - properly sends scale/transform and sets size/bounds

### Run 3: After resolve_scrolling_width fix
- **Status**: 106 passed, 162 failed (simple_no_workspaces now passes)
- **Fixed**: `Row::resolve_scrolling_width()` - returns ColumnWidth based on preset or window size
- **Fixed**: Default canvas size for NoOutputs (1280x720 instead of 1920x1080)

### Run 4: After view_offset and ActivateWindow fixes
- **Status**: 133 passed, 135 failed (+27 tests!)
- **Fixed**: View offset initialization when adding first window
- **Fixed**: `ActivateWindow::Smart` handling in `Monitor::add_window`

## Changes Made

### src/layout/row/mod.rs
- Implemented `configure_new_window()` - sends scale/transform, sets size/bounds
- Implemented `new_window_size()` - computes window size with min/max constraints
- Implemented `new_window_toplevel_bounds()` - computes toplevel bounds
- Implemented `scrolling_new_window_size()` - helper for scrolling window size
- Fixed `resolve_default_width()` and `resolve_default_height()` signatures
- Fixed `resolve_scrolling_width()` - now takes `&W` and returns `ColumnWidth`

### src/layout/row/operations/add.rs
- Fixed view offset initialization for first window (set static offset immediately)

### src/layout/monitor/mod.rs
- Fixed `ActivateWindow::Smart` handling - use `map_smart(|| true)` instead of `== Yes`

### src/layout/mod.rs
- Fixed default canvas size for NoOutputs (1280x720)
- Updated `resolve_scrolling_width` calls to pass `&window` instead of `window.id()`

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) - 174/268 passing (65% pass rate, up from 39%)
- [ ] Golden tests pass (`cargo insta test`) — 71/84 passing (85% pass rate)
- [x] Team file complete

## Summary
Started with 105 passed, 163 failed. Ended with 174 passed, 94 failed.
**+69 tests fixed** in this session.

### Key Fixes
1. **Window configuration**: Implemented `configure_new_window`, `new_window_size`, `new_window_toplevel_bounds`
2. **Scrolling width**: Fixed `resolve_scrolling_width` to return ColumnWidth and use window size
3. **Default canvas size**: Fixed NoOutputs to use 1280x720 instead of 1920x1080
4. **View offset**: Fixed initialization for first window and animation for subsequent windows
5. **ActivateWindow**: Fixed Smart handling to activate by default
6. **Navigation**: Fixed `focus_left`, `focus_right`, `focus_column` to pass prev_idx
7. **Center column**: Implemented `center_column` and `animate_view_offset_to_column_centered`
8. **Index conversion**: Fixed `focus_column` to use 1-based index like original

### Remaining Issues
- 94 tests still failing
- Many stub implementations in Row need to be completed
- Floating window support needs work
- Tabbed display mode needs implementation
