# TEAM_040 — Animation Bug Fix

## Status: COMPLETED (Partial - Fixed Y Animation Duplicate Bug)

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

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Passed Tests | 162 | 187 | **+25** |
| Failed Tests | 106 | 81 | **-25** |

### Animation Tests Now Passing
- `height_resize_animates_next_y` ✅
- `height_resize_and_back` ✅  
- `height_resize_and_cancel` ✅
- And others...

## Remaining Issues (Not Caused by This Fix)

### Golden Test Failures (14 tests)
The golden tests show missing X move animations in operations like:
- `consume_into_column` - missing `column_X_move_x` animation
- `toggle_tabbed` - likely similar issue
- Various focus operations

These are **pre-existing issues** with the Row/Canvas2D refactor, not regressions from my fix.

### Root Cause of Remaining Issues
The Row implementation may be missing animation triggers that ScrollingSpace had. Operations like:
- `consume_into_column` - should animate consumed window's X position
- `expel_from_column` - should animate expelled window's X position
- Column movement operations

## Files Modified

### `src/layout/tile.rs`
- Fixed `animate_move_y_from_with_config` parameters: `(0.,1.,0.)` → `(1.,0.,0.)`

### `src/layout/column/sizing/tile_sizes.rs`  
- Removed duplicate move animation creation code added by TEAM_039
- Added comment explaining that `Column::update_window()` handles move animations

## Handoff

- [x] Code compiles (`cargo check`)
- [x] Test count improved (162→187 passed)
- [ ] Golden tests - 14 still failing (pre-existing Row animation issues)
- [x] Team file complete

## Recommendations for Next Team

1. **Compare Row animation triggers with ScrollingSpace**: Operations like consume/expel need to create X move animations that are currently missing

2. **Check `consume_into_column` in Row**: Should call `tile.animate_move_x_from()` when moving a window from one column to another

3. **Animation audit**: Run through the animation checklist to find all missing triggers in Row operations

## Technical Notes

The animation system works correctly:
- `Animation::new(clock, from, to, velocity, config)` - creates animation from `from` to `to`
- `Animation::value()` - returns current interpolated value based on clock time
- `tile.render_offset()` - calculates `from * anim.value()` for move animations

For "move from" animations, we want:
- At start: offset = from (tile appears at old position)
- At end: offset = 0 (tile at new position)

Therefore: animation must go from value=1 to value=0, so `from * 1 = from` at start, `from * 0 = 0` at end.
