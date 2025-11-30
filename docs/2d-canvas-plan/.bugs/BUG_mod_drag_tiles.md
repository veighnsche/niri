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

## Current Status
INVESTIGATING

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

## Recommended Next Steps for CHASE_002
1. Add logging to `remove_column_by_idx_with_anim()` to see actual offset values
2. Compare offset values with main branch
3. Check if `dnd_scroll_gesture_begin()` is correctly locking the view
4. Verify column X positions before and after removal
5. Check if animation is being triggered multiple times
