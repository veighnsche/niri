# TEAM_047 — Alacritty Invisible Debugging

## Context
- Branch: 2d-canvas
- Backend: winit
- Symptom: Pressing Mod+T spawns Alacritty according to logs, but no terminal is visible.
- Bug ID: `BUG_alacritty_invisible`

## Work Log

### Step 1 — Confirm spawn and mapping
- Verified `Action::Spawn(["alacritty"])` runs from keybinding.
- Confirmed `utils::spawning::spawn_sync` executes `"alacritty"` with niri's `WAYLAND_DISPLAY`.
- Added logging in `CompositorHandler::commit` to log `app_id` and `title` for newly mapped toplevels.
- Observed `mapped new toplevel window app_id="Alacritty" title="Alacritty"` in runtime logs.

### Step 2 — Confirm layout insertion and camera visibility
- Added logging in `Monitor::add_window`:
  - Row placement relative to camera (`row_idx`, `row_y`, `row_h`, `camera_y`, `view_h`, `row_visible`).
  - Per-window render-space tile position via `Canvas2D::tiles_with_render_positions()`.
- Runtime logs show for the first Alacritty window:
  - `row_idx=0`, `row_y=0.0`, `row_h=1378.0`, `camera_y=0.0`, `view_h=1378.0`, `row_visible=true`.
  - `x=16.0`, `y=16.0`, `w=707.0`, `h=1346.0`, `is_active_tile=true`.
- Subsequent spawns are placed at `x=739.0` and `x=1462.0`, as expected for multiple columns.
- Conclusion: layout math and camera positioning claim that at least one Alacritty tile is fully on-screen.

### Step 3 — Compare with golden snapshots
- Reviewed `src/layout/snapshot.rs` and `src/layout/tests/golden.rs`.
- Golden snapshots operate on `Layout<TestWindow>` with synthetic outputs and do not involve winit, smithay clients, or real rendering.
- Golden tests verify:
  - Column/tile geometry and `view_offset`.
  - Animation timelines.
- They do **not** exercise:
  - Real Wayland clients (no Alacritty).
  - The compositor → layout → render → winit pipeline.
- Therefore golden snapshots can legitimately pass while this runtime integration bug still exists.

## Findings
- `BUG_alacritty_invisible_CHASE_001`: Layout misplacement hypothesis is a **DEAD END**.
  - Row 0 is visible (`row_visible=true`).
  - The first Alacritty tile has a large, sensible rect `(x≈16, y≈16, w≈707, h≈1346)`.
  - Multiple spawns create additional columns at expected X positions.
- New working hypothesis `CHASE_002`:
  - The render pipeline is failing to draw the tile contents, despite correct layout geometry.
  - Candidates:
    - Canvas2D/Row/Column render element wiring.
    - Damage tracking / cropping in `backend/winit.rs`.
    - Some interaction with Alacritty's buffer (e.g. fully transparent, same color as backdrop).

## Next Steps (for future teams)
- Trace rendering for the active monitor and canvas:
  - Inspect `Monitor::render` and `Canvas2D` render methods.
  - Add `DBG[BUG_alacritty_invisible_CHASE_002]` breadcrumbs around where row/tiles are turned into render elements.
- Verify that the tile geometry we logged (`x,y,w,h`) is reflected in the rendered elements’ rectangles.
- Check damage regions and clear colors in `backend/winit.rs`:
  - Ensure the whole tile rect is within the damage region.
  - Confirm we are not drawing the backdrop **over** the window.
- If needed, create a dedicated repro test case (similar to golden tests) that runs through the render path for a single large tile at `(16,16)`.

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) — **not explicitly run in this chase**
- [ ] Golden tests pass (`cargo insta test`) — **not rerun in this chase; assumed green from previous teams**
- [x] Bug file created: `docs/2d-canvas-plan/.bugs/BUG_alacritty_invisible.md`
- [x] Breadcrumb-capable logging added in layout and compositor.
