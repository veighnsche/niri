# TEAM_040 — Animation & Test Fix Session

## Status: COMPLETED — ALL 84 GOLDEN TESTS PASS!

## Bug Fixed

### Root Cause Analysis
TEAM_039's animation investigation identified that `Animation::value()` was returning 0. After deeper investigation, I found **TWO bugs**:

### Bug 1: Wrong Animation Parameters (Fixed)
**File**: `src/layout/tile.rs` lines 594-596

`animate_move_y_from_with_config` was using `(0., 1., 0.)` parameters when it should use `(1., 0., 0.)` like `animate_move_x_from_with_config`.

The `render_offset()` formula is `from * anim.value()`:
- **X animation (correct)**: value goes 1→0, so offset = from×1 at start, from×0 at end
- **Y animation (buggy)**: value went 0→1, so offset = from×0 at start, from×1 at end (backwards!)

**Fix**: Changed parameters from `(0., 1., 0.)` to `(1., 0., 0.)`.

### Bug 2: Duplicate Animation Creation (Fixed)
**File**: `src/layout/column/sizing/tile_sizes.rs`

TEAM_039 added code to create move animations in `update_tile_sizes()`, but `Column::update_window()` in `operations.rs` **already handles this** when the tile responds with its new size.

This caused the animation offset to be created **twice**, doubling the visual offset:
- Expected: tile at y=100 (50 actual + 50 animation offset)
- Actual: tile at y=150 (50 actual + 50 + 50 from duplicate animation)

**Fix**: Removed duplicate animation creation from `tile_sizes.rs`.

## Test Results

| Metric | Start | End | Change |
|--------|-------|-----|--------|
| Passed Tests | 187 | 201 | **+14** |
| Failed Tests | 81 | 67 | **-14** |
| Golden Tests | 75/84 | **84/84** | **+9** |

### All Golden Tests Now Pass! ✅

## Additional Fixes Made

### Fix 3: Column Movement Animations in `add_column`
**File**: `src/layout/row/operations/add.rs`

Added missing column movement animations when inserting a new column. This was marked as `TODO(TEAM_006)`.

### Fix 4: Animation Config for `remove_tile_by_idx`
**File**: `src/layout/row/operations/remove.rs`

Added `anim_config` parameter to `remove_tile_by_idx` and created `remove_column_by_idx_with_anim` to match ScrollingSpace's API.

### Fix 5: Floating Toggle via Canvas2D
**File**: `src/layout/mod.rs`, `src/layout/canvas/floating.rs`

Fixed `toggle_window_floating` to use Canvas2D's `toggle_floating_window_by_id` method instead of the Row's no-op method.

### Fix 6: Tabbed Display Toggle
**File**: `src/layout/row/mod.rs`

Implemented `toggle_column_tabbed_display` and `set_column_display` based on ScrollingSpace.

### Fix 7: Focus Column Index Handling
**File**: `src/layout/row/navigation.rs`

Fixed `focus_column` to properly handle 1-based external API vs 0-based internal indexing.

### Fix 8: Focus Window In Column
**File**: `src/layout/row/navigation.rs`

Fixed `focus_window_in_column` to focus a window within the active column (not focus a column by index).

## Files Modified

### `src/layout/tile.rs`
- Fixed `animate_move_y_from_with_config` parameters: `(0.,1.,0.)` → `(1.,0.,0.)`

### `src/layout/column/sizing/tile_sizes.rs`  
- Removed duplicate move animation creation code added by TEAM_039
- Added comment explaining that `Column::update_window()` handles move animations

## Handoff

- [x] Code compiles (`cargo check`)
- [x] Test count improved (187→201 passed)
- [x] **ALL 84 Golden tests pass!**
- [x] Team file complete

## Recommendations for Next Team

1. **Continue fixing remaining 67 tests**: Focus on non-golden tests that are still failing

2. **Check for stub implementations**: Search for `TODO(TEAM_` comments in Row module

3. **Test categories to investigate**:
   - Interactive move tests
   - Fullscreen tests
   - Workspace/row movement tests
   - Named workspace tests

## Technical Notes

The animation system works correctly:
- `Animation::new(clock, from, to, velocity, config)` - creates animation from `from` to `to`
- `Animation::value()` - returns current interpolated value based on clock time
- `tile.render_offset()` - calculates `from * anim.value()` for move animations

For "move from" animations, we want:
- At start: offset = from (tile appears at old position)
- At end: offset = 0 (tile at new position)

Therefore: animation must go from value=1 to value=0, so `from * 1 = from` at start, `from * 0 = 0` at end.
