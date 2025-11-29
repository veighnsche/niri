# Phase 4: Camera System

> **Status**: ⏳ PENDING (after Phase 3)
> **Goal**: Camera with position (x, y) and zoom for 2D canvas navigation

---

## Related TODOs from Codebase

These TODOs will be resolved by this phase:
- `TODO(TEAM_007): Apply camera offset to render elements` (canvas/render.rs:25)
- `TODO(TEAM_007): Add vertical_view_movement config` (canvas/navigation.rs:79)

---

## Overview

Based on USER clarifications:
- Camera tracks (x, y, zoom) position on the canvas
- Can zoom out to see multiple rows at once
- Zoom out gradually (smooth animation)
- This is a completely new feature for Canvas2D

---

## Core Concept

The camera controls what the user sees:
- **Position (x, y)**: What part of the canvas is centered
- **Zoom**: How much of the canvas is visible (1.0 = normal, 0.5 = see twice as much)

When focus changes to a spanning tile:
- If tile spans 1 row → zoom to 1.0 (fit 1 row)
- If tile spans 2 rows → zoom to 0.5 (fit 2 rows)

---

## Step 4.1: Create Camera Module

### File Structure

```
src/layout/camera/
├── mod.rs              # Camera struct, public interface
├── position.rs         # X/Y position handling
├── zoom.rs             # Zoom level handling
└── following.rs        # Auto-follow logic
```

### Tasks

- [ ] **4.1.1**: Create `camera/mod.rs` with `Camera` struct
- [ ] **4.1.2**: Create `camera/position.rs` with position logic
- [ ] **4.1.3**: Create `camera/zoom.rs` with zoom logic
- [ ] **4.1.4**: Unit tests for camera

### Implementation

```rust
pub struct Camera {
    /// X position (canvas coordinates)
    x: AnimatedValue<f64>,
    /// Y position (canvas coordinates)  
    y: AnimatedValue<f64>,
    /// Zoom level (1.0 = normal, 0.5 = see twice as much)
    zoom: AnimatedValue<f64>,
    
    /// Whether camera is in manual mode (user override)
    manual_override: bool,
}

impl Camera {
    pub fn x(&self) -> f64 { self.x.current() }
    pub fn y(&self) -> f64 { self.y.current() }
    pub fn zoom(&self) -> f64 { self.zoom.current() }
    
    pub fn animate_to(&mut self, x: f64, y: f64, zoom: f64, config: &CameraAnimConfig) {
        self.x.animate_to(x, &config.position);
        self.y.animate_to(y, &config.position);
        self.zoom.animate_to(zoom, &config.zoom);
    }
}
```

---

## Step 4.2: Auto-Zoom for Row Span

### Logic

When focus changes to a tile with row_span N:
1. Compute camera Y to center the tile vertically
2. Compute zoom to fit N rows in the viewport
3. Animate to new position + zoom

### Tasks

- [ ] **4.2.1**: Create `camera/following.rs`
- [ ] **4.2.2**: Implement `compute_camera_for_tile(tile, row_height, viewport)`
- [ ] **4.2.3**: Call from `Canvas2D` when focus changes
- [ ] **4.2.4**: Skip auto-adjust if `manual_override` is true

---

## Step 4.3: Zoom Gestures

### User Controls

| Input | Action |
|-------|--------|
| `Mod+ScrollDown` | Zoom out (see more) |
| `Mod+ScrollUp` | Zoom in (see less) |
| Pinch gesture | Zoom in/out |

### Tasks

- [ ] **4.3.1**: Add zoom keybinds to config
- [ ] **4.3.2**: Implement `Canvas2D::zoom_in()` and `zoom_out()`
- [ ] **4.3.3**: Handle pinch gesture (if supported)
- [ ] **4.3.4**: Clamp zoom to reasonable range (0.1 to 2.0)
- [ ] **4.3.5**: Manual zoom disables auto-adjust until focus changes

---

## Step 4.4: Rendering with Zoom

### Transform Pipeline

1. Compute visible region based on camera position + zoom
2. Cull tiles outside visible region
3. Transform tile positions: `screen_pos = (canvas_pos - camera_pos) * zoom`
4. Scale tile renders by zoom factor

### Tasks

- [ ] **4.4.1**: Compute visible region from camera state
- [ ] **4.4.2**: Cull non-visible rows/columns
- [ ] **4.4.3**: Apply transform to render positions
- [ ] **4.4.4**: Scale render elements by zoom
- [ ] **4.4.5**: Transform input coordinates (clicks) inversely

---

## Step 4.5: Animation Configuration

### New Config Options

```kdl
animations {
    camera-movement {
        duration-ms 250
        curve "ease-out-expo"
    }
    
    camera-zoom {
        duration-ms 200
        curve "ease-out-quad"
    }
}
```

### Tasks

- [ ] **4.5.1**: Add `camera_movement` config
- [ ] **4.5.2**: Add `camera_zoom` config
- [ ] **4.5.3**: Use configs in Camera animation
- [ ] **4.5.4**: Update default-config.kdl

---

## Success Criteria

- [ ] Camera has x, y, zoom
- [ ] Focus on 2-row tile → camera zooms out
- [ ] Focus on 1-row tile → camera zooms in
- [ ] Manual zoom via Mod+Scroll works
- [ ] Rendering correctly transforms positions
- [ ] Clicking works at any zoom level
- [ ] Smooth animations between zoom levels

---

*Phase 4 - Camera System*
