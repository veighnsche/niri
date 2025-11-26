# TEAM_007: Phase 1 Continuation + Modular Refactor

## Status: COMPLETE

## Objective
1. Continue Phase 1: Port key ScrollingSpace methods to Row and Canvas2D
2. Refactor `row/mod.rs` into focused submodules per Rule 7

## Starting Point
Per TEAM_006 handoff:
- Row and Canvas2D modules created with clean-slate design
- Basic column operations work: add, remove, move, focus
- Basic vertical navigation works: focus_up, focus_down
- All 251 tests pass, 58 golden tests pass
- `row/mod.rs` at 858 lines (approaching 1000 max)

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

### Modular Refactor (Rule 7) ✅
Refactored `row/mod.rs` (858 lines) into focused submodules:

```
row/
├── mod.rs          (303 lines) - Core struct, accessors, animation
├── view_offset.rs  (323 lines) - View offset calculation & animation
├── operations.rs   (162 lines) - Add/remove/move columns
├── navigation.rs   (57 lines)  - Focus left/right/column
└── layout.rs       (76 lines)  - Tile positions, config update
```

## Changes Made

### New Files (Modular Refactor)
- `src/layout/row/view_offset.rs` — View offset calculation and animation
- `src/layout/row/navigation.rs` — Focus navigation methods
- `src/layout/row/operations.rs` — Column add/remove/move operations
- `src/layout/row/layout.rs` — Tile position queries and config update

### Modified Files
- `src/layout/row/mod.rs` — Refactored from 858 → 303 lines
  - Added module declarations
  - Kept core struct, accessors, animation methods
  - Moved other methods to submodules
  
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
