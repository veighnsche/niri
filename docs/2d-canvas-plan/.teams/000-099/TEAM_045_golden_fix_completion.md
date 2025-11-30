# TEAM_045 Golden Fix Completion

## Summary
- Goal: Finish the remaining implementation needed to make **all golden tests pass** after TEAM_044, specifically the floating→tiled animation regression.

## Status
- Golden tests: **✅ 88/88 pass (0 failing)**.
- Regular tests: still at ~213 passed / 55 failing (same as TEAM_044), not addressed in this step.

## Work Done
- Implemented missing floating→tiled animation when toggling a window back from floating to tiled so that golden snapshots capture tile edge animations.
- Verified behavior against `golden_u4_toggle_floating_back_to_tiled` and then the full `cargo xtask test-all golden` suite.

### Code Changes
- **File**: `src/layout/canvas/floating.rs`
- **Function**: `Canvas2D<W>::toggle_floating_window_by_id`
  - When a window moves **from floating to tiled** (both specific-ID and active-window paths):
    - After `FloatingSpace::remove_tile` / `remove_active_tile`, call `tile.animate_move_from(Point::from((0., 0.)))` **before** re-inserting into the row via `self.add_tile(...)`.
    - This ensures `Row::snapshot()` sees an active tile move animation and emits four `AnimationTimelineSnapshot::tile_edge` entries (`x_min`, `x_max`, `y_min`, `y_max`) that match the golden baseline.

### Rationale
- Geometry in `golden_u4_toggle_floating_back_to_tiled` already matched; only the `animations` list in `CanvasSnapshot.tiled` was empty.
- `Row::snapshot()` is already wired to inspect `Tile` move animations (`move_x_animation_with_from`, `move_y_animation_with_from`) and produce the corresponding timeline entries.
- By starting a move animation with zero offset, we replicate the golden behavior where `from == to` but a Spring animation record is still present.

## Handoff
- [x] Golden tests pass (`cargo xtask test-all golden`).
- [ ] Regular tests pass (`cargo test`) – **still 55 failing**, as per TEAM_044; not touched here.
- [x] Team file updated.
