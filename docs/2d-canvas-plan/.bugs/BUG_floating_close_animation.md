# BUG_floating_close_animation

## Symptom
Floating windows disappear instantly when closed instead of playing the close animation.

## Reproduction
1. Open a terminal
2. Press Mod+Shift+Space to make it floating
3. Close the window (click X or Mod+Shift+Q)
4. **Expected**: Window should fade/shrink with animation
5. **Actual**: Window disappears instantly

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_106 | Layout::start_close_animation_for_window doesn't check floating | INCONCLUSIVE | Added floating check but not tested |

## Current Status
INVESTIGATING

## Investigation Notes (TEAM_106)

### What Was Attempted
Modified `Layout::start_close_animation_for_window()` in `src/layout/mod.rs` to check floating space:
```rust
// Check floating first
if mon.canvas.floating.has_window(window) {
    mon.canvas.floating.start_close_animation_for_window(renderer, window, blocker);
    return;
}
```

### Code Path
1. Window close triggers `handlers/xdg_shell.rs` or `handlers/compositor.rs`
2. These call `layout.start_close_animation_for_window()`
3. Original code only checked rows, not floating space
4. Fix adds floating check before row check

### Files Involved
- `src/layout/mod.rs` - start_close_animation_for_window (lines 1764-1797)
- `src/layout/canvas/floating/render.rs` - FloatingSpace::start_close_animation_for_window

### Potential Issues
1. The fix was added but not tested
2. `FloatingSpace::start_close_animation_for_window` might have its own issues
3. The closing_windows list in FloatingSpace might not be rendered

## Recommended Next Steps for CHASE_002
1. Test if the fix works
2. If not, add logging to see if `start_close_animation_for_window` is called
3. Check if `FloatingSpace::closing_windows` is being populated
4. Check if `FloatingSpace::render_elements` includes closing windows
5. Verify the animation parameters are correct
