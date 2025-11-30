# BUG_floating_close_no_focus

## Symptom
When a floating window is closed, no window becomes selected/focused. The tiled window that was previously active should receive focus.

## Reproduction
1. Open terminal 1 (tiled)
2. Open terminal 2 (tiled)
3. Select terminal 2
4. Press Mod+Shift+Space to make terminal 2 floating
5. Close the floating terminal 2 (click X button)
6. **Expected**: Terminal 1 should be focused
7. **Actual**: No window is focused, no focus ring visible

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_106 | floating_is_active not updated on remove | INCONCLUSIVE | Added update_focus_after_removing() but not tested |

## Current Status
**FIXED** by TEAM_108

## Investigation Notes (TEAM_106)

### What Was Attempted
Added `Canvas2D::update_focus_after_removing()` in `canvas_floating.rs`:
```rust
fn update_focus_after_removing(&mut self, removed_from_floating: bool) {
    if removed_from_floating {
        if self.floating.is_empty() {
            self.floating_is_active = false;
        }
    } else {
        let tiled_empty = self.rows.values().all(|row| row.is_empty());
        if tiled_empty && !self.floating.is_empty() {
            self.floating_is_active = true;
        }
    }
}
```

### Why It Might Not Work
1. `floating_is_active = false` only switches the MODE, it doesn't ACTIVATE a tiled window
2. Main branch might have additional logic to activate the previously active tiled window
3. The active window in the row might need to be explicitly focused

### Main Branch Behavior
Main branch has `update_focus_floating_tiling_after_removing()` which:
- Sets `floating_is_active = FloatingActive::No` when floating becomes empty
- Sets `floating_is_active = FloatingActive::Yes` when tiled becomes empty

But this is just the flag - there might be additional activation logic elsewhere.

### Files Involved
- `src/layout/canvas/canvas_floating.rs` - remove_window, update_focus_after_removing
- `src/layout/layout_impl/window_ops.rs` - Layout::remove_window
- `src/niri.rs` - window close handling

## Root Cause (TEAM_108)

The bug was in `Layout::remove_window()` (`src/layout/layout_impl/window_ops.rs`).

**Problem**: This method directly calls `mon.canvas.floating.remove_tile(window)` bypassing `Canvas2D::remove_window()` which has the `update_focus_after_removing()` logic.

**Code path**:
1. Window closes → Layout::remove_window() is called
2. `floating.remove_tile()` called directly ❌
3. `update_focus_after_removing()` NOT called
4. `floating_is_active` stays `true` even though floating is empty
5. Focus queries check empty floating space → no focus

## Fix (TEAM_108)

1. Made `Canvas2D::update_focus_after_removing()` public
2. Added call to `mon.canvas.update_focus_after_removing(true)` after floating removal in `Layout::remove_window()`
3. Fixed both `MonitorSet::Normal` and `MonitorSet::NoOutputs` branches

## Files Modified
- `src/layout/canvas/canvas_floating.rs` - Made `update_focus_after_removing` public
- `src/layout/layout_impl/window_ops.rs` - Added focus update calls
