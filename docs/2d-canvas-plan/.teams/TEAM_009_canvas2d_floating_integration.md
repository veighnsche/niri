# TEAM_009: Canvas2D FloatingSpace Integration

## Status: COMPLETE

## Objective
Complete Phase 1.5.2: Integrate FloatingSpace into Canvas2D module.

## Starting Point
Per TEAM_008 handoff:
- Row module is feature-complete (Phase 1.5.1 ✅)
- Canvas2D has basic row operations and rendering
- FloatingSpace NOT YET integrated into Canvas2D
- All 251 tests pass, 58 golden tests pass

## Completed Work

### Phase 1.5.2.1-2: FloatingSpace Integration ✅
- [x] Added `floating: FloatingSpace<W>` field
- [x] Added `floating_is_active: bool` field
- [x] Added FloatingSpace constructor in `Canvas2D::new`

### Phase 1.5.2.3-5: Floating Operations ✅
- [x] Implemented `toggle_floating_window` - moves window between layers
- [x] Implemented `toggle_floating_focus` - switches focus between layers
- [x] Updated `render_elements` for floating layer
- [x] Updated `update_render_elements` for floating layer
- [x] Updated `advance_animations` to include floating
- [x] Updated `are_animations_ongoing` to include floating
- [x] Added accessor methods: `floating()`, `floating_mut()`, `floating_is_active()`, `has_floating_windows()`

### Phase 1.5.2.6-8: Camera System
- [ ] Apply camera offset in `render_elements` — Deferred to Phase 3
- [ ] Add `camera_x` tracking for horizontal scroll — Deferred to Phase 3
- [ ] Add `vertical_view_movement` config to niri-config — Deferred to Phase 3

**Decision**: Camera offset and vertical_view_movement are part of the full camera system (Phase 3). The current implementation uses `horizontal_view_movement` config for Y animations as a reasonable fallback.

### Phase 1.5.2.9-10: Window Operations ✅
- [x] Added `add_window` - routes to floating or tiled layer
- [x] Added `remove_window` - finds window across all layers
- [x] Added `contains_any` - checks both floating and tiled
- [x] Added `start_close_animation_for_window` - handles floating close animations
- [x] Added `start_close_animation_for_tile` - wrapper for floating close animations

### Row Module Addition ✅
- [x] Added `remove_active_tile` method to Row (operations/remove.rs)

## Changes Made

### Modified Files
- `src/layout/canvas/mod.rs` — Full FloatingSpace integration (426 → 608 lines)
  - Added FloatingSpace and related imports
  - Added `floating` and `floating_is_active` fields
  - Added accessor methods
  - Updated `render_elements` with floating layer rendering
  - Updated `update_render_elements` for floating
  - Updated animation methods
  - Added toggle_floating methods
  - Added add_window and remove_window methods
  - Added close animation methods
- `src/layout/row/operations/remove.rs` — Added `remove_active_tile` method

### Documentation Updated
- `docs/2d-canvas-plan/TODO.md` — Marked FloatingSpace integration complete
- `docs/2d-canvas-plan/phases/phase-1.5-integration.md` — Updated with TEAM_009 progress

## Remaining Work for Next Team

### Phase 1.5.3: Feature Flag
- [ ] Add `canvas-2d` feature to Cargo.toml
- [ ] Conditional Monitor code (use Canvas2D vs Workspaces based on feature)

### Phase 1.5.4: Monitor Integration
- [ ] Wire Canvas2D into Monitor
- [ ] Route window operations through Canvas2D
- [ ] Route navigation through Canvas2D
- [ ] Update IPC

### Phase 3: Camera System
- [ ] Apply camera offset to render elements
- [ ] Add camera_x tracking for horizontal scroll
- [ ] Add vertical_view_movement config

## Handoff
- [x] Code compiles (`cargo check`) — 1 dead code warning (expected)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`cargo insta test`) — 58 tests
- [x] Team file complete
