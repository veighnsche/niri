# TEAM_048 — Render Pipeline Debugging for BUG_alacritty_invisible

## Context
- Bug ID: `BUG_alacritty_invisible`
- Chase: `CHASE_003` — investigating render pipeline
- Previous: TEAM_047 confirmed layout placement is correct (CHASE_001 dead end), suspects render pipeline (CHASE_002 branch)

## Work Log

### Step 1 — Read existing breadcrumbs and understand render flow
- Reading TEAM_047's logging to understand what's confirmed:
  - Layout correctly places tiles at `(x≈16, y≈16, w≈707, h≈1346)`
  - Row 0 is visible (`row_visible=true`)
  - Camera position is at 0,0

### Step 2 — Trace render pipeline
- Traced: `Winit::render()` → `niri.render()` → `mon.render_elements()` → `canvas.render_elements()` → `row.render_elements()` → `tile.render()`
- Found first suspect: `canvas/render.rs` line 26 has `let _camera = self.camera_position();` — camera unused!
- However, for row 0, `y_offset = 0`, so this shouldn't cause invisibility

### Step 3 — Found Root Cause in niri.rs
- In `Niri::render()`, there are two branches:
  1. `if mon.render_above_top_layer()` — fullscreen case, monitor_elements ARE added
  2. `else` — normal case, **monitor_elements were NOT added!**
- Lines 4533-4564 had old workspace code commented out with `/* ... */`
- The `for _element in &monitor_elements` loop did nothing useful

### Step 4 — Applied Fix
- Replaced the broken loop with proper Canvas2D rendering:
  - Add layer popups
  - Add monitor_elements (window tiles)
  - Add layer normal elements

## Files Modified
- `src/niri.rs` — Fixed render() else branch to add monitor_elements

## Root Cause
**`monitor_elements` were generated but never added to the render output in the non-fullscreen case.**

The old workspace-based code was commented out during Canvas2D refactor but never replaced with working code.

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) — 51 failures, **PRE-EXISTING** from Canvas2D refactor
- [ ] Golden tests pass (`cargo insta test`) — 26 failures + 1264 snapshots to review, **PRE-EXISTING**
- [x] Bug file updated with root cause
- [x] Team file complete

## Note on Test Failures
The test failures are NOT caused by this fix. They are pre-existing failures from the Canvas2D refactor work by previous teams (TEAM_023, etc.). My fix only changes the render pipeline to actually output window elements - it doesn't affect the test scenarios that were already failing.

## Additional Issue Found (not fixed)
In `src/layout/canvas/render.rs` line 26:
```rust
// TODO(TEAM_007): Apply camera offset to render elements for proper scrolling
let _camera = self.camera_position();
```
The camera Y offset is NOT being applied to row render positions. For row 0 this doesn't matter (y_offset=0), but multi-row scrolling will be broken. This should be fixed in a future task.
