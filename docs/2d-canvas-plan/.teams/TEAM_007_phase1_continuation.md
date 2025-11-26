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
├── mod.rs          (306 lines) - Core struct, accessors, animation
├── view_offset.rs  (322 lines) - View offset calculation & animation
├── operations.rs   (162 lines) - Add/remove/move columns
├── navigation.rs   (57 lines)  - Focus left/right/column
├── layout.rs       (76 lines)  - Tile positions, config update
└── render.rs       (170 lines) - Rendering
```

### Priority 3: Rendering ✅
- [x] Create `row/render.rs` module
- [x] Define `RowRenderElement` type
- [x] Port `columns_in_render_order` from ScrollingSpace
- [x] Port `render_elements` from ScrollingSpace
- [x] Port `update_render_elements` from ScrollingSpace
- [x] Define `Canvas2DRenderElement` type
- [x] Add `render_elements` to Canvas2D
- [x] Add `update_render_elements` to Canvas2D

## Changes Made

### New Files (Modular Refactor)
- `src/layout/row/view_offset.rs` — View offset calculation and animation
- `src/layout/row/navigation.rs` — Focus navigation methods
- `src/layout/row/operations.rs` — Column add/remove/move operations
- `src/layout/row/layout.rs` — Tile position queries and config update
- `src/layout/row/render.rs` — Rendering with RowRenderElement

### Modified Files
- `src/layout/row/mod.rs` — Refactored from 858 → 303 lines
  - Added module declarations
  - Kept core struct, accessors, animation methods
  - Moved other methods to submodules
  
- `src/layout/canvas/mod.rs` — Camera Y animation + rendering
  - Added Animation import
  - `update_camera_y` now uses animated transitions
  - Added `Canvas2DRenderElement` type
  - Added `render_elements` and `update_render_elements` methods

### Documentation Updated
- `docs/2d-canvas-plan/TODO.md` — Marked view offset and render TODOs as complete
- `docs/2d-canvas-plan/phases/phase-1-row-and-canvas.md` — Updated checklist
- `docs/2d-canvas-plan/phases/phase-1.5-integration.md` — **Created new phase**
- `docs/2d-canvas-plan/phases/phase-2-row-spanning.md` — Updated prerequisites
- `docs/2d-canvas-plan/README.md` — Updated progress tracking, added Phase 1.5
- `docs/2d-canvas-plan/ai-teams-rules.md` — Added Lessons Learned section
- `docs/2d-canvas-plan/.teams/TEAM_006_row_and_canvas.md` — Marked as COMPLETE

## Remaining Work for Next Team

### Priority 4: Interactive Resize
- [ ] Port `interactive_resize_begin/update/end`

### Priority 5: Feature Flag
- [ ] Add `canvas-2d` feature to Cargo.toml
- [ ] Conditional code in Monitor

### Other
- [ ] Port gesture handling (`view_offset_gesture_begin`, etc.)
- [ ] Port consume/expel operations
- [ ] Add `vertical_view_movement` config to niri-config
- [ ] Apply camera offset to Canvas2D render elements
- [ ] Port `render_above_top_layer` from ScrollingSpace

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`./scripts/verify-golden.sh`) — 58 tests
- [x] Team file complete
