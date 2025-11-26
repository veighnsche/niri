# TEAM_008: Documentation Cleanup + Window Operations

## Status: COMPLETE

## Objective
1. Complete the documentation cleanup started by TEAM_007 before their context ran out
2. Port window operations to Row module (Phase 1.5.1.8)

## Starting Point
Per user request:
- TEAM_007 had created `gesture.rs` and `resize.rs` modules
- Modules were declared in `row/mod.rs` 
- TODO.md was updated but stale TODO comments remained in code
- TEAM_007's team file needed completion

## Completed Work

### Documentation Updates ✅
- [x] Updated module structure comment in `row/mod.rs` to include `gesture.rs` and `resize.rs`
- [x] Removed stale TEAM_006 TODO comments for gesture/render/resize (now complete)
- [x] Updated TEAM_007's team file with:
  - Added Priority 4: Gesture Handling (complete)
  - Added Priority 5: Interactive Resize (complete)
  - Updated file listings with new modules
  - Updated remaining work section

### Window Operations (Phase 1.5.1.8) ✅
- [x] Port `add_tile_to_column` from ScrollingSpace — adds tile to existing column
- [x] Port `add_tile_right_of` from ScrollingSpace — adds tile as new column right of window
- [x] Port `activate_column` from ScrollingSpace — activates column with animation

### Remove Operations (Phase 1.5.1.9) ✅
- [x] Port `remove_tile` from ScrollingSpace — removes tile by window ID
- [x] Port `remove_tile_by_idx` from ScrollingSpace — removes tile with full animation support
- [x] Port `remove_active_column` from ScrollingSpace — removes the active column
- [x] Port `remove_column_by_idx` from ScrollingSpace — removes column with animations
- [x] Add `RemovedTile::new` constructor to layout/mod.rs for Row module access

### Consume/Expel Operations (Phase 1.5.1.10) ✅
- [x] Port `consume_or_expel_window_left` from ScrollingSpace — consume into left column or expel
- [x] Port `consume_or_expel_window_right` from ScrollingSpace — consume into right column or expel
- [x] Port `consume_into_column` from ScrollingSpace — consume first tile from right column
- [x] Add `RemovedTile` getters (`tile()`, `width()`, `is_full_width()`, `into_parts()`)

### Rendering (Phase 1.5.1.11) ✅
- [x] Port `render_above_top_layer` from ScrollingSpace — returns true when fullscreen and view stationary

### Refactoring (Rule 7 Compliance) ✅
- [x] Split `row/operations.rs` (692 lines) into submodules:
  - `operations/add.rs` (159 lines)
  - `operations/remove.rs` (246 lines)
  - `operations/move_col.rs` (50 lines)
  - `operations/consume.rs` (250 lines)
  - `operations/mod.rs` (22 lines)
- [x] Split `column/sizing.rs` (566 lines) into submodules:
  - `sizing/tile_sizes.rs` (276 lines)
  - `sizing/height.rs` (160 lines)
  - `sizing/width.rs` (123 lines)
  - `sizing/display.rs` (80 lines)
  - `sizing/mod.rs` (22 lines)
- [x] Split `canvas/mod.rs` (607 lines) into submodules:
  - `mod.rs` (243 lines) - Core struct and accessors
  - `floating.rs` (142 lines) - Floating window operations
  - `operations.rs` (103 lines) - Add/remove/find windows
  - `navigation.rs` (91 lines) - Row/column focus
  - `render.rs` (85 lines) - Rendering
- [x] All files now under 500 lines ✅

## Changes Made

### Modified Files
- `src/layout/row/mod.rs` — Cleaned up stale TODOs, updated module structure doc
- `src/layout/row/operations/` — Refactored into submodules (was 692 lines, now 5 files < 250 lines each)
- `src/layout/row/navigation.rs` — Added `activate_column`
- `src/layout/row/render.rs` — Added `render_above_top_layer`
- `src/layout/column/sizing/` — Refactored into submodules (was 566 lines, now 5 files < 280 lines each)
- `src/layout/canvas/` — Refactored into submodules (was 607 lines, now 5 files < 250 lines each)
- `src/layout/mod.rs` — Added `RemovedTile::new` constructor and getters
- `docs/2d-canvas-plan/.teams/TEAM_007_phase1_continuation.md` — Completed team file
- `docs/2d-canvas-plan/TODO.md` — Updated with completed items
- `docs/2d-canvas-plan/ai-teams-rules.md` — Added Lesson #6 (proper refactoring pattern)

## Remaining Work for Next Team

### Phase 1.5.2: Complete Canvas2D
- [ ] Integrate FloatingSpace
- [ ] Apply camera offset to render elements
- [ ] Add `vertical_view_movement` config to niri-config

### Phase 1.5.3: Feature Flag
- [ ] Add `canvas-2d` feature to Cargo.toml
- [ ] Conditional Monitor code

### Phase 1.5.4: Monitor Integration
- [ ] Wire Canvas2D into Monitor

### Open Questions
See `.questions/TEAM_008_gaps_and_clarifications.md` for questions about:
- Row module testing strategy
- Canvas2D module structure
- Row vs ScrollingSpace API parity
- FloatingSpace priority
- Column animation TODOs

## Handoff
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) — 251 tests
- [x] Golden tests pass (`cargo insta test`) — 58 tests
- [x] Team file complete
- [x] Documentation updated (README, phase-1.5, TODO.md)
- [x] Questions documented
