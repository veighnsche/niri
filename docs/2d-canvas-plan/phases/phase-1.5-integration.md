# Phase 1.5: Row Integration

> **Goal**: Complete Row/Canvas2D implementation and wire into the compositor.

**Added by TEAM_007** after analyzing the gap between Phase 1 core work and Phase 2 prerequisites.

---

## Why This Phase?

Phase 1 defined Row + Canvas2D creation, but the actual work revealed these need to be staged:

1. **Phase 1 (Core)** — Create Row and Canvas2D modules with basic functionality ✓
2. **Phase 1.5 (Integration)** — Complete the modules and wire into Monitor ← **YOU ARE HERE**
3. **Phase 2 (Row Spanning)** — Add row span capability

Phase 1.5 bridges the gap between "modules exist" and "modules are usable."

---

## Current State (After TEAM_009)

### What's Done ✓
```
src/layout/
├── row/
│   ├── mod.rs          (305 lines) - Core struct, accessors, animation
│   ├── view_offset.rs  (324 lines) - View offset calculation & animation ✓
│   ├── render.rs       (199 lines) - Rendering + render_above_top_layer ✓
│   ├── operations.rs   (692 lines) - Full add/remove/consume/expel ✓
│   ├── layout.rs       (77 lines)  - Tile positions, config update ✓
│   ├── navigation.rs   (83 lines)  - Focus + activate_column ✓
│   ├── gesture.rs      (445 lines) - Gesture handling ✓
│   └── resize.rs       (151 lines) - Interactive resize ✓
├── canvas/
│   └── mod.rs          (608 lines) - Canvas2D with FloatingSpace ✓
└── animated_value/
    ├── mod.rs          (212 lines) - AnimatedValue enum ✓
    └── gesture.rs      (73 lines)  - ViewGesture ✓
```

### What's Still Missing
1. ~~**FloatingSpace**~~ — ✅ Integrated by TEAM_009
2. **Feature flag** — `canvas-2d` feature not created
3. **Monitor integration** — Canvas2D not wired into compositor
4. **Camera offset** — Deferred to Phase 3 (Camera System)
5. **Config** — `vertical_view_movement` deferred to Phase 3

---

## Step 1.5.1: Complete Row Module ✅ COMPLETE

### Gesture Handling ✅ (TEAM_007)
- [x] **1.5.1.1**: Port `view_offset_gesture_begin`
- [x] **1.5.1.2**: Port `view_offset_gesture_update`
- [x] **1.5.1.3**: Port `view_offset_gesture_end`
- [x] **1.5.1.4**: Port `dnd_scroll_gesture_*` methods

### Interactive Resize ✅ (TEAM_007)
- [x] **1.5.1.5**: Port `interactive_resize_begin`
- [x] **1.5.1.6**: Port `interactive_resize_update`
- [x] **1.5.1.7**: Port `interactive_resize_end`

### Window Operations ✅ (TEAM_008)
- [x] **1.5.1.8**: Port `add_tile_to_column`, `add_tile_right_of`, `activate_column`
- [x] **1.5.1.9**: Port `remove_tile`, `remove_tile_by_idx`, `remove_column_by_idx`
- [x] **1.5.1.10**: Port `consume_or_expel_window_left/right`, `consume_into_column`

### Remaining Rendering ✅ (TEAM_008)
- [x] **1.5.1.11**: Port `render_above_top_layer`

---

## Step 1.5.2: Complete Canvas2D Module ✅ CORE COMPLETE (TEAM_009)

### FloatingSpace Integration ✅
- [x] **1.5.2.1**: Add `floating: FloatingSpace<W>` field
- [x] **1.5.2.2**: Add `floating_is_active: bool` field
- [x] **1.5.2.3**: Implement `toggle_floating_window` and `toggle_floating_focus`
- [x] **1.5.2.4**: Update `render_elements` for floating layer
- [x] **1.5.2.5**: Update animations to include floating

### Camera System (Deferred to Phase 3)
- [ ] **1.5.2.6**: Apply camera offset in `render_elements` — Phase 3
- [ ] **1.5.2.7**: Add `camera_x` tracking for horizontal scroll — Phase 3
- [ ] **1.5.2.8**: Add `vertical_view_movement` config to niri-config — Phase 3

### Window Operations ✅
- [x] **1.5.2.9**: Add `add_window` that routes to correct layer
- [x] **1.5.2.10**: Add `remove_window` that finds across all layers
- [x] **Bonus**: Add `contains_any`, `start_close_animation_for_window`

---

## Step 1.5.3: Feature Flag

### Cargo.toml
```toml
[features]
default = []
canvas-2d = []
```

### Conditional Compilation
- [ ] **1.5.3.1**: Add `canvas-2d` feature to `Cargo.toml`
- [ ] **1.5.3.2**: Conditional `Monitor` code for canvas vs workspaces
- [ ] **1.5.3.3**: Ensure tests pass with feature off
- [ ] **1.5.3.4**: Ensure tests pass with feature on

---

## Step 1.5.4: Monitor Integration

### Replace Workspaces (with feature flag)
```rust
// src/layout/monitor.rs

pub struct Monitor<W: LayoutElement> {
    #[cfg(feature = "canvas-2d")]
    canvas: Canvas2D<W>,
    
    #[cfg(not(feature = "canvas-2d"))]
    workspaces: Vec<Workspace<W>>,
    // ...
}
```

- [ ] **1.5.4.1**: Add Canvas2D field to Monitor (feature-gated)
- [ ] **1.5.4.2**: Wire window operations through Canvas2D
- [ ] **1.5.4.3**: Wire navigation through Canvas2D
- [ ] **1.5.4.4**: Wire rendering through Canvas2D
- [ ] **1.5.4.5**: Update IPC to work with Canvas2D

---

## Success Criteria

- [x] All gesture handling works in Row (TEAM_007)
- [x] Interactive resize works in Row (TEAM_007)
- [ ] FloatingSpace integrated into Canvas2D
- [ ] Feature flag compiles both ways
- [ ] With `canvas-2d` feature: can open windows, navigate, resize
- [ ] Without feature: existing behavior unchanged
- [x] All 251+ tests pass
- [x] All 58 golden tests pass

---

## Estimated Time

1-2 weeks (significant porting work)

---

## Next Phase

→ [Phase 2: Row Spanning](phase-2-row-spanning.md)
