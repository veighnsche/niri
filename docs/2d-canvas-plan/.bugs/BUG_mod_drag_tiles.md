# BUG_mod_drag_tiles

## Symptom
When using Mod+drag to pick up a tiled window, all other tiles immediately animate off-screen to the left. When dropped, they return. Picking up again causes them to fly left again.

## Reproduction
1. Open 3+ terminal windows (tiled)
2. Press Mod and drag one window
3. **Expected**: Other tiles shift slightly to show drop targets
4. **Actual**: All tiles fly off-screen to the left instantly

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_106 | Animation offset calculation wrong | INCONCLUSIVE | Added logging, needs testing |
| 002 | TEAM_110 | view_offset_x not adjusted on active column removal | DEAD END | Added offset adjustment but didn't fix it |
| 003 | TEAM_110 | Unknown - need more logging | IN PROGRESS | Adding DBG breadcrumbs |

## Current Status
✅ **FIXED** (CHASE_003)

## Investigation Notes (TEAM_106)

### Code Path Analysis
1. `interactive_move_begin()` is called when drag starts
2. For tiled windows, calls `dnd_scroll_gesture_begin()` on all monitors
3. When threshold exceeded, `interactive_move_update()` calls `remove_window()`
4. `remove_window()` triggers `remove_tile()` on the Row
5. `Row::remove_tile()` calls `remove_column_by_idx_with_anim()`
6. This animates OTHER columns with `animate_move_from(offset)`

### Suspected Root Cause
In `src/layout/row/operations/remove.rs` around line 176:
```rust
let offset = Point::from((col_x, 0.));
col.animate_move_from(offset);
```

The `offset` might be calculated incorrectly after the refactor, causing columns to animate from a wrong position.

### Debug Logging Added
- `interactive_move.rs` has BUG002 logging before `remove_window()` call

### Files Involved
- `src/layout/layout_impl/interactive_move.rs` - interactive_move_update (line ~309)
- `src/layout/row/operations/remove.rs` - remove_column_by_idx_with_anim (lines 162-246)
- `src/layout/row/gesture.rs` - dnd_scroll_gesture_begin/end

### Potential Issues
1. `col_x` calculation might be wrong
2. Animation might be triggered multiple times
3. View offset gesture state might interfere
4. The refactor might have changed how column positions are calculated

## Root Cause (TEAM_110)

The bug was in `src/layout/row/operations/remove.rs` in `remove_column_by_idx_with_anim()`.

**Problem**: When the active column is removed:
1. `active_column_idx` stays at the same value N
2. But after removal, index N now points to what was previously column N+1
3. `column_x(N)` increases by `offset` (width of removed column + gap)
4. `view_offset_x` was NOT adjusted to compensate
5. `view_pos() = column_x(active) + view_offset_x` suddenly jumped up
6. `view_off_x = -view_pos()` became more negative
7. All tiles shifted left by the removed column's width!

## Fix (TEAM_110)

Added view offset adjustment when the active column is removed:
```rust
if column_idx == self.active_column_idx {
    self.view_offset_x.offset(-offset);
}
```

This compensates for the change in `column_x(active_column_idx)` to keep `view_pos()` stable.

## Files Modified
- `src/layout/row/operations/remove.rs` - Added view_offset_x adjustment

---

## CHASE_003 Investigation (TEAM_110)

### DBG Breadcrumbs Added
1. `src/layout/row/layout.rs:tiles_with_render_positions` - TRACE[1] - logs extreme view_off_x values
2. `src/layout/row/gesture.rs:dnd_scroll_gesture_begin` - TRACE[2] - logs state before gesture
3. `src/layout/column/render.rs:animate_move_from_with_config` - TRACE[3] - logs extreme animation values

### To Test
Run niri with warning-level logging enabled (to see the CHASE_003 logs):
```bash
RUST_LOG=warn cargo run 2>&1 | tee /tmp/niri-debug.log
```

Then reproduce the bug:
1. Open 3+ terminal windows (tiled)
2. Press Mod and drag one window
3. Stop niri (Ctrl+C)
4. Check the log output:
```bash
grep "DBG\[BUG_mod_drag" /tmp/niri-debug.log
```

### DBG Logging Points Added (CHASE_003)
- **TRACE[1]** `tiles_with_render_positions` - logs EXTREME view_off_x values (>5000)
- **TRACE[2]** `dnd_scroll_gesture_begin` - logs state before gesture starts
- **TRACE[3]** `animate_move_from_with_config` - logs EXTREME animation values (>2000)
- **TRACE[4]** `remove_column START` - logs full state before removal
- **TRACE[5]** `remove_column END` - logs full state after removal

### What We're Looking For
1. **view_pos** before and after removal - does it jump unexpectedly?
2. **view_offset** changes - is the gesture state preserved?
3. **animation offsets** - are they reasonable (should be ~column_width, not huge)?
4. **EXTREME values** - any >2000 or >5000 indicates the bug source

### CHASE_003 Hypothesis 1: Gesture cancellation — DEAD END
The gesture WAS being cancelled by `animate_view_offset_with_config`:
```rust
self.view_offset_x = AnimatedValue::Static(new_view_offset);  // Cancels gesture
```
**Result**: Fixed the cancellation, but bug STILL occurs. Gesture cancellation was NOT the root cause.
**Code marker**: `DBG[BUG_mod_drag_tiles_CHASE_003]: CLEARED` in `view_offset.rs`

### CHASE_003 Hypothesis 2: view_offset_gesture_end called incorrectly — **ROOT CAUSE FOUND**

**Root Cause**: When the interactive move ended (user dropped the window), `view_offset_gesture_end(None)` was called from the render refresh path. This incorrectly ended the DnD gesture by snapping to a column and changing `active_column_idx`, which caused `view_pos()` to jump dramatically while animations were still running.

**Evidence**: TRACE[7] logs showed `view_off` jumping from `16` to `-1372` mid-animation.

**Fix**: In `view_offset_gesture_end()`, detect if the gesture is a DnD gesture (`dnd_last_event_time.is_some()`) and delegate to `dnd_scroll_gesture_end()` which preserves the view position.

**Code**: `src/layout/row/gesture.rs` lines 199-206
