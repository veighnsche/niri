# TEAM_044 — Test Fix Continuation

## Mission
Continue fixing failing tests from TEAM_043's work. Target: 100% pass rate.

## Status: IN PROGRESS

## Starting Point
- **Tests**: 210 passed, 58 failed (78%)
- **Golden Tests**: ✅ 84/84 pass

## Current Status
- **Tests**: 213 passed, 55 failed (79.5%)
- **Golden Tests**: ✅ 84/84 pass

## Fixes Applied

### 1. Fixed Layout::update_window to check floating space ✅
- **File**: `src/layout/mod.rs` (lines 1275-1305)
- **Issue**: Floating windows weren't getting `on_commit` called because `update_window` only checked rows
- **Fix**: Added floating space check before row check, pass serial to both

### 2. Fixed Row::update_window to accept serial parameter ✅
- **File**: `src/layout/row/mod.rs` (lines 1343-1384)
- **Issue**: `on_commit` wasn't being called for windows in rows
- **Fix**: Added serial parameter and call `tile.window_mut().on_commit(serial)` before update

### 3. Added Serial import to row module ✅
- **File**: `src/layout/row/mod.rs` (line 48)

## Analysis of Remaining Failures

### Floating Tests (~22 failing)
The floating tests are failing because of complex interactions between:
1. `expected_size()` returning pending tiled size instead of current window size
2. `request_size_once` state machine transitions
3. Configure throttling logic

**Root cause investigation**:
- When a window is toggled from tiled to floating, `add_tile_at` calls `win.expected_size()`
- `expected_size()` may return the pending tiled size if there's an uncommitted configure
- This causes the floating window to be configured with the tiled size

**Attempted fix** (reverted):
- Changed `win.expected_size()` to `win.size()` in `add_tile_at`
- This broke more tests because some tests rely on `expected_size()` behavior

**Needs further investigation**:
- The issue is in the interaction between tiled refresh and floating toggle
- May need to clear pending size when moving to floating

### Other Failing Categories
- **Animation tests**: ~10 failing (view offset issues)
- **Fullscreen tests**: ~5 failing (view offset preservation)
- **Window opening tests**: ~10 failing (workspace targeting)
- **Interactive move tests**: ~8 failing

## Files Modified
- `src/layout/mod.rs` - Added floating space check in update_window
- `src/layout/row/mod.rs` - Added serial parameter to update_window

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) - 213/268 passing (79.5%)
- [x] Golden tests pass (`cargo insta test`) - 84/84 passing
- [x] Team file complete

## Next Steps for TEAM_045
1. Investigate floating test failures more deeply
2. Consider clearing pending size when toggling to floating
3. Fix animation view offset issues
4. Fix fullscreen view offset preservation
5. Fix window opening workspace targeting
