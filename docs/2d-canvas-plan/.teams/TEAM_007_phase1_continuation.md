# TEAM_007: Phase 1 Continuation

## Status: COMPLETE

## Objective
Continue Phase 1: Port key ScrollingSpace methods to Row and Canvas2D.

## Starting Point
Per TEAM_006 handoff:
- Row and Canvas2D modules created with clean-slate design
- Basic column operations work: add, remove, move, focus
- Basic vertical navigation works: focus_up, focus_down
- All 251 tests pass, 58 golden tests pass

## Completed Work

### Priority 1: View Offset Animation (Row) ✅
- [x] Port full `animate_view_offset_to_column` logic from ScrollingSpace
- [x] Port `compute_new_view_offset_fit` and `compute_new_view_offset_centered`
- [x] Port `compute_new_view_offset_for_column` with CenterFocusedColumn logic
- [x] Port `animate_view_offset_with_config` with Animation support
- [x] Port `compute_new_view_offset` helper function
- [x] Add `is_centering_focused_column`, `view_pos`, `target_view_pos` methods

### Priority 2: Camera Animation (Canvas2D) ✅
- [x] Animate camera_y when changing rows using horizontal_view_movement config
- [x] Added TODO for vertical_view_movement config in niri-config

## Changes Made

### Modified Files
- `src/layout/row/mod.rs` — Full view offset animation logic (~200 lines added)
  - Added imports: `min`, `CenterFocusedColumn`, `SizingMode`, `Animation`
  - Added `compute_new_view_offset` helper function
  - Added `compute_new_view_offset_fit`, `compute_new_view_offset_centered`
  - Added `compute_new_view_offset_for_column_fit`, `compute_new_view_offset_for_column_centered`
  - Added `compute_new_view_offset_for_column` with OnOverflow logic
  - Added `animate_view_offset`, `animate_view_offset_with_config`
  - Added `animate_view_offset_to_column_centered`, `animate_view_offset_to_column_with_config`
  - Replaced stub `animate_view_offset_to_column` with full implementation
  
- `src/layout/canvas/mod.rs` — Camera Y animation
  - Added Animation import
  - `update_camera_y` now uses animated transitions

### Documentation Updated
- `docs/2d-canvas-plan/TODO.md` — Marked view offset TODOs as complete
- `docs/2d-canvas-plan/phases/phase-1-row-and-canvas.md` — Updated checklist

## Remaining Work for Next Team

### Priority 3: Rendering
- [ ] Port `render_elements` from ScrollingSpace to Row
- [ ] Add `render_elements` to Canvas2D

### Priority 4: Interactive Resize
- [ ] Port `interactive_resize_begin/update/end`

### Priority 5: Feature Flag
- [ ] Add `canvas-2d` feature to Cargo.toml
- [ ] Conditional code in Monitor

### Other
- [ ] Port gesture handling (`view_offset_gesture_begin`, etc.)
- [ ] Port consume/expel operations
- [ ] Add `vertical_view_movement` config to niri-config

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`./scripts/verify-golden.sh`) — 58 tests
- [x] Team file complete
