# TEAM_063: FloatingSpace Consolidation into canvas/

## Status: IN PROGRESS

## Task
Phase 2: Consolidate FloatingSpace into `canvas/` directory structure.

## Investigation Summary

### Current Structure

**`src/layout/floating.rs` (1450 LOC)**
- `FloatingSpace<W>` struct definition (lines 35-70)
- `Data` struct for per-tile position data (lines 79-200)
- `FloatingSpaceRenderElement` render element macro (lines 72-77)
- Core implementation:
  - Constructor & config: `new()`, `update_config()`, `update_shaders()` (~40 LOC)
  - Animation: `advance_animations()`, `are_animations_ongoing()`, `are_transitions_ongoing()` (~20 LOC)
  - Tile accessors: `tiles()`, `tiles_mut()`, `tiles_with_offsets()`, etc. (~60 LOC)
  - Operations: `add_tile()`, `add_tile_at()`, `add_tile_above()`, `remove_tile()`, etc. (~150 LOC)
  - Focus/activation: `activate_window()`, `focus_*()` methods (~100 LOC)
  - Movement: `move_*()` methods (~100 LOC)
  - Resize: `set_window_width()`, `set_window_height()`, `toggle_window_*()` (~200 LOC)
  - Interactive resize: `interactive_resize_*()` methods (~80 LOC)
  - Rendering: `render_elements()`, `update_render_elements()` (~60 LOC)
  - Close animation: `start_close_animation_*()` (~40 LOC)
  - Refresh: `refresh()` (~50 LOC)
  - Helpers: `clamp_within_working_area()`, `stored_or_default_tile_pos()`, etc. (~100 LOC)
  - Test helpers: `verify_invariants()`, `snapshot()` (~100 LOC)
- Helper functions: `compute_toplevel_bounds()`, `resolve_preset_size()` (~25 LOC)

**`src/layout/canvas/floating.rs` (293 LOC)**
- Canvas2D methods for floating integration:
  - `toggle_floating_window()`, `toggle_floating_window_by_id()` (~90 LOC)
  - `toggle_floating_focus()`, `focus_tiling()`, `focus_floating()` (~30 LOC)
  - `switch_focus_floating_tiling()` (~15 LOC)
  - `add_window()`, `remove_window()`, `contains_any()` (~50 LOC)
  - `start_close_animation_for_window()`, `start_close_animation_for_tile()` (~50 LOC)

### Dependencies

**FloatingSpace is imported by:**
1. `src/layout/canvas/mod.rs` - Uses `FloatingSpace` and `FloatingSpaceRenderElement`
2. `src/layout/mod.rs` - Re-exports `floating` module
3. `src/layout/types.rs` - References `InteractiveResize<W>` (used by FloatingSpace)

**FloatingSpace imports:**
- `niri_config` types: `PresetSize`, `RelativeTo`, `Border`
- `niri_ipc` types: `PositionChange`, `SizeChange`, `WindowLayout`
- `smithay` types: Various geometry and renderer types
- Internal: `Tile`, `ColumnWidth`, `InteractiveResize`, `ResolvedSize`, `LayoutElement`, etc.

### Proposed Target Structure

```
canvas/floating/
├── mod.rs (~500 LOC)
│   - FloatingSpace struct definition
│   - Data struct
│   - FloatingSpaceRenderElement macro
│   - Constructor, config, animation methods
│   - Basic accessors (tiles, tiles_with_offsets, etc.)
│   - Focus/activation methods
│   - Test helpers (verify_invariants, snapshot)
│
├── operations.rs (~350 LOC)
│   - add_tile(), add_tile_at(), add_tile_above()
│   - remove_tile(), remove_tile_by_idx(), remove_active_tile()
│   - bring_up_descendants_of(), raise_window()
│   - Movement methods (move_to, move_by, move_left/right/up/down, move_window, center_window)
│   - update_window(), descendants_added()
│
├── render.rs (~150 LOC)
│   - render_elements()
│   - update_render_elements()
│   - start_close_animation_for_window()
│   - start_close_animation_for_tile()
│
└── resize.rs (~350 LOC)
    - set_window_width(), set_window_height()
    - toggle_window_width(), toggle_window_height()
    - interactive_resize_begin/update/end()
    - new_window_size(), new_window_toplevel_bounds()
    - Helper functions: compute_toplevel_bounds(), resolve_preset_size()
```

### Migration Strategy

1. **Create `canvas/floating/` directory**
2. **Move FloatingSpace struct to `canvas/floating/mod.rs`**
   - Keep Data struct with it (tightly coupled)
   - Keep FloatingSpaceRenderElement macro
3. **Split implementation into submodules**
   - Use `impl<W: LayoutElement> FloatingSpace<W>` blocks in each file
4. **Merge `canvas/floating.rs` methods** into Canvas2D impl (stays in canvas/floating.rs → renamed)
5. **Update imports** in canvas/mod.rs and layout/mod.rs
6. **Delete old `floating.rs`**

### Risk Assessment: MEDIUM

- FloatingSpace is self-contained with clear boundaries
- No golden test changes expected (pure refactor)
- Main risk: Import path changes may affect external code

## Progress

- [x] Investigation complete
- [ ] Create `canvas/floating/` directory
- [ ] Create `canvas/floating/mod.rs` with struct and core impl
- [ ] Create `canvas/floating/operations.rs`
- [ ] Create `canvas/floating/render.rs`
- [ ] Create `canvas/floating/resize.rs`
- [ ] Merge Canvas2D floating methods
- [ ] Update imports
- [ ] Delete old `floating.rs`
- [ ] Verify compilation
- [ ] Run tests

## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo xtask test-all golden`)
- [ ] Team file complete
