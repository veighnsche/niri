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

## Current State (After Phase 1)

### What's Done ✓
```
src/layout/
├── row/
│   ├── mod.rs          (307 lines) - Core struct, accessors, animation
│   ├── view_offset.rs  (324 lines) - View offset calculation & animation ✓
│   ├── render.rs       (177 lines) - Rendering ✓
│   ├── operations.rs   (162 lines) - Add/remove/move columns ✓
│   ├── layout.rs       (77 lines)  - Tile positions, config update ✓
│   └── navigation.rs   (57 lines)  - Focus left/right/column ✓
├── canvas/
│   └── mod.rs          (426 lines) - Canvas2D with rendering ✓
└── animated_value/
    ├── mod.rs          (193 lines) - AnimatedValue enum ✓
    └── gesture.rs      (75 lines)  - ViewGesture ✓
```

### What's Missing
1. **Gesture handling** — `view_offset_gesture_begin/update/end` not ported
2. **Interactive resize** — `interactive_resize_begin/update/end` not ported
3. **FloatingSpace** — Not integrated into Canvas2D
4. **Feature flag** — `canvas-2d` feature not created
5. **Monitor integration** — Canvas2D not wired into compositor
6. **Window operations** — `add_window`, `remove_window` wrappers
7. **Camera offset** — Not applied to render elements

---

## Step 1.5.1: Complete Row Module

### Gesture Handling
Port from `scrolling.rs` lines 2852-3005:

- [ ] **1.5.1.1**: Port `view_offset_gesture_begin`
- [ ] **1.5.1.2**: Port `view_offset_gesture_update`
- [ ] **1.5.1.3**: Port `view_offset_gesture_end`
- [ ] **1.5.1.4**: Port `dnd_scroll_gesture_*` methods

### Interactive Resize
Port from `scrolling.rs` lines 3300-3500:

- [ ] **1.5.1.5**: Port `interactive_resize_begin`
- [ ] **1.5.1.6**: Port `interactive_resize_update`
- [ ] **1.5.1.7**: Port `interactive_resize_end`

### Window Operations
Port high-level window operations:

- [ ] **1.5.1.8**: Port `add_window` (routes to add_tile)
- [ ] **1.5.1.9**: Port `remove_window` (finds and removes)
- [ ] **1.5.1.10**: Port `consume_or_expel_window_left/right`

### Remaining Rendering
- [ ] **1.5.1.11**: Port `render_above_top_layer`

---

## Step 1.5.2: Complete Canvas2D Module

### FloatingSpace Integration
- [ ] **1.5.2.1**: Add `floating: FloatingSpace<W>` field
- [ ] **1.5.2.2**: Add `floating_is_active: bool` field
- [ ] **1.5.2.3**: Implement `toggle_floating`
- [ ] **1.5.2.4**: Update `render_elements` for floating layer
- [ ] **1.5.2.5**: Update navigation for floating windows

### Camera System
- [ ] **1.5.2.6**: Apply camera offset in `render_elements`
- [ ] **1.5.2.7**: Add `camera_x` tracking for horizontal scroll
- [ ] **1.5.2.8**: Add `vertical_view_movement` config to niri-config

### Window Operations
- [ ] **1.5.2.9**: Add `add_window` that routes to correct row
- [ ] **1.5.2.10**: Add `remove_window` that finds across rows

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

- [ ] All gesture handling works in Row
- [ ] Interactive resize works in Row
- [ ] FloatingSpace integrated into Canvas2D
- [ ] Feature flag compiles both ways
- [ ] With `canvas-2d` feature: can open windows, navigate, resize
- [ ] Without feature: existing behavior unchanged
- [ ] All 251+ tests pass
- [ ] All 58 golden tests pass

---

## Estimated Time

1-2 weeks (significant porting work)

---

## Next Phase

→ [Phase 2: Row Spanning](phase-2-row-spanning.md)
