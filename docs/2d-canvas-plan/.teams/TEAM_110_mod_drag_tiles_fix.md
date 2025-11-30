# TEAM_110 — Mod+Drag Tiles Fix

## Status: ✅ FIXED

## Summary
Fixing BUG-002: Mod+drag causes all tiles to animate off-screen to the left.

## Root Cause Analysis

Investigating the view offset calculation when a column is removed during an interactive move.

### The Problem

When Mod+drag picks up a tiled window:
1. `dnd_scroll_gesture_begin()` locks the view with `view_offset_x = AnimatedValue::Gesture(...)`
2. Threshold exceeded → `remove_window()` called
3. `remove_column_by_idx_with_anim()` removes the column
4. `activate_column(N)` is called (where N was the removed column's index)
5. View offset adjustment fails because `idx == active_column_idx`
6. `view_pos()` suddenly increases (column_x of new column N is larger)
7. All tiles shift left because `view_off_x = -view_pos()` becomes more negative

### Key Insight

In `animate_view_offset_with_config`:
```rust
let new_col_x = self.column_x(idx);
let old_col_x = self.column_x(self.active_column_idx);
let offset_delta = old_col_x - new_col_x;
```

When the active column is removed and `idx == active_column_idx`:
- Both point to the SAME index after removal
- `offset_delta = 0` (no adjustment!)
- View position jumps by the width of the removed column

## Fix

Added view offset adjustment in `remove_column_by_idx_with_anim()` when the active column is removed:
```rust
if column_idx == self.active_column_idx {
    self.view_offset_x.offset(-offset);
}
```

This compensates for the change in `column_x(active_column_idx)` to keep `view_pos()` stable.

### Files Modified
- `src/layout/row/operations/remove.rs` - Added view_offset_x adjustment

## CHASE_002 Result: DEAD END
The `view_offset_x.offset(-offset)` fix was incorrect.

## CHASE_003 Result: ✅ FIXED

### Root Cause Found
When the interactive move ended (user dropped the window), `view_offset_gesture_end(None)` was called from the render refresh path (`layout_impl/render.rs` line 417). This incorrectly ended the DnD gesture by:
1. Snapping to a column
2. Changing `active_column_idx`
3. Causing `view_pos()` to jump while column animations were still running

### The Fix
In `view_offset_gesture_end()`, detect if the gesture is a DnD gesture and delegate to `dnd_scroll_gesture_end()` which preserves the view position:

```rust
// TEAM_110: If this is a DnD gesture, use the DnD-specific end logic
if gesture.dnd_last_event_time.is_some() {
    self.dnd_scroll_gesture_end();
    return true;
}
```

### Files Modified
- `src/layout/row/gesture.rs` - Added DnD gesture detection in `view_offset_gesture_end()`
- `src/layout/row/view_offset.rs` - Don't cancel DnD gestures in early return path

### Sub-bugs for Next Team
See `.bugs/BUG_mod_drag_subbug.md`:
1. **BUG-002a**: Insert hint bar not showing during drag (blue bar missing)
2. **BUG-002b**: Cannot drag window from right to left (only left→right works)

### Handoff Checklist
- [x] Code compiles (`cargo check`)
- [x] Bug FIXED
- [x] Debug logging cleaned up
- [x] Team file complete
