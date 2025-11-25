# Phase 3: Camera System

> **Goal**: Camera with position (x, y) and zoom, auto-adjusts based on focused window's row span.

---

## Prerequisites

- Phase 2 complete (row spanning works)

### Starting Point After Phase 2
```
src/layout/
├── column/              # From Phase 0
├── animated_value/      # From Phase 0
├── row/                 # From Phase 1
├── canvas/              # From Phase 1, updated in Phase 2
│   ├── mod.rs
│   ├── layout.rs
│   ├── navigation.rs
│   ├── operations.rs
│   ├── render.rs
│   └── spanning.rs      # NEW in Phase 2
├── tile.rs              # Updated: has row_span field
└── ...
```

---

## Core Concept

The camera controls what the user sees:
- **Position (x, y)**: What part of the canvas is centered
- **Zoom**: How much of the canvas is visible

When focus changes:
- If new tile spans 1 row → zoom to 1.0 (fit 1 row)
- If new tile spans 2 rows → zoom to 0.5 (fit 2 rows)

The spanning tile stays at "full size" from the user's perspective, but other tiles appear smaller because the camera is zoomed out.

---

## Step 3.1: Create Camera Module

### File Structure

```
src/layout/camera/
├── mod.rs              # Camera struct, public interface
├── position.rs         # X/Y position handling
├── zoom.rs             # Zoom level handling
└── following.rs        # Auto-follow logic
```

### Tasks

- [ ] **3.1.1**: Create `camera/mod.rs` with `Camera` struct
- [ ] **3.1.2**: Create `camera/position.rs` with position logic
- [ ] **3.1.3**: Create `camera/zoom.rs` with zoom logic
- [ ] **3.1.4**: Unit tests for camera

### Implementation

```rust
// src/layout/camera/mod.rs

mod position;
mod zoom;
mod following;

pub use position::*;
pub use zoom::*;

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
    pub fn new() -> Self {
        Self {
            x: AnimatedValue::Static(0.0),
            y: AnimatedValue::Static(0.0),
            zoom: AnimatedValue::Static(1.0),
            manual_override: false,
        }
    }
    
    // Queries
    pub fn x(&self) -> f64 { self.x.current() }
    pub fn y(&self) -> f64 { self.y.current() }
    pub fn zoom(&self) -> f64 { self.zoom.current() }
    pub fn position(&self) -> Point<f64, Logical> {
        Point::from((self.x(), self.y()))
    }
    
    // Target (for rendering smooth animations)
    pub fn target_x(&self) -> f64 { self.x.target() }
    pub fn target_y(&self) -> f64 { self.y.target() }
    pub fn target_zoom(&self) -> f64 { self.zoom.target() }
    
    // Animate to new values
    pub fn animate_to(&mut self, x: f64, y: f64, zoom: f64, config: &CameraAnimConfig) {
        self.x.animate_to(x, &config.position);
        self.y.animate_to(y, &config.position);
        self.zoom.animate_to(zoom, &config.zoom);
        self.manual_override = false;
    }
    
    // Manual control
    pub fn set_zoom_manual(&mut self, zoom: f64) {
        self.zoom.set_static(zoom.clamp(0.1, 2.0));
        self.manual_override = true;
    }
    
    pub fn adjust_zoom(&mut self, delta: f64) {
        let new_zoom = (self.zoom.current() + delta).clamp(0.1, 2.0);
        self.zoom.set_static(new_zoom);
        self.manual_override = true;
    }
    
    // Animation tick
    pub fn advance_animations(&mut self, now: Duration) {
        self.x.advance(now);
        self.y.advance(now);
        self.zoom.advance(now);
    }
    
    pub fn is_animating(&self) -> bool {
        self.x.is_animating() || self.y.is_animating() || self.zoom.is_animating()
    }
}
```

---

## Step 3.2: Auto-Zoom for Row Span

### Logic

When focus changes to a tile with row_span N:
1. Compute camera Y to center the tile vertically
2. Compute zoom to fit N rows in the viewport
3. Animate to new position + zoom

### Tasks

- [ ] **3.2.1**: Create `camera/following.rs`
- [ ] **3.2.2**: Implement `compute_camera_for_tile(tile, row_height, viewport)`
- [ ] **3.2.3**: Call from `Canvas2D` when focus changes
- [ ] **3.2.4**: Skip auto-adjust if `manual_override` is true

### Implementation

```rust
// src/layout/camera/following.rs

pub struct CameraTarget {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Camera {
    /// Compute ideal camera position/zoom to focus on a tile.
    pub fn compute_target_for_tile(
        &self,
        tile_rect: Rectangle<f64, Logical>,
        row_span: u8,
        row_height: f64,
        viewport: Size<f64, Logical>,
    ) -> CameraTarget {
        // Zoom to fit the tile's row span
        let zoom = 1.0 / row_span as f64;
        
        // Viewport size at this zoom level
        let visible_width = viewport.w / zoom;
        let visible_height = viewport.h / zoom;
        
        // Center the tile in the viewport
        let tile_center_x = tile_rect.loc.x + tile_rect.size.w / 2.0;
        let tile_center_y = tile_rect.loc.y + tile_rect.size.h / 2.0;
        
        // Camera position is top-left of visible area
        let camera_x = tile_center_x - visible_width / 2.0;
        let camera_y = tile_center_y - visible_height / 2.0;
        
        CameraTarget {
            x: camera_x,
            y: camera_y,
            zoom,
        }
    }
}

// In Canvas2D
impl<W: LayoutElement> Canvas2D<W> {
    fn update_camera_for_focus(&mut self) {
        if self.camera.manual_override {
            return; // User is controlling zoom
        }
        
        let Some(tile) = self.active_tile() else { return };
        let tile_rect = self.tile_rect(tile);
        let row_span = tile.row_span();
        
        let target = self.camera.compute_target_for_tile(
            tile_rect,
            row_span,
            self.row_height(),
            self.viewport_size(),
        );
        
        self.camera.animate_to(
            target.x,
            target.y,
            target.zoom,
            &self.options.animations.camera,
        );
    }
}
```

---

## Step 3.3: Zoom Gestures

### User Controls

| Input | Action |
|-------|--------|
| `Mod+ScrollDown` | Zoom out (see more) |
| `Mod+ScrollUp` | Zoom in (see less) |
| Pinch gesture | Zoom in/out |

### Tasks

- [ ] **3.3.1**: Add zoom keybinds to config
- [ ] **3.3.2**: Implement `Canvas2D::zoom_in()` and `zoom_out()`
- [ ] **3.3.3**: Handle pinch gesture (if supported)
- [ ] **3.3.4**: Clamp zoom to reasonable range (0.1 to 2.0)
- [ ] **3.3.5**: Manual zoom disables auto-adjust until focus changes

### Implementation

```rust
// In Canvas2D

pub fn zoom_out(&mut self) {
    self.camera.adjust_zoom(-0.1);
    // Don't update_camera_for_focus — let user control
}

pub fn zoom_in(&mut self) {
    self.camera.adjust_zoom(0.1);
}

pub fn reset_zoom(&mut self) {
    self.camera.manual_override = false;
    self.update_camera_for_focus();
}
```

### Config

```kdl
// Example keybinds
binds {
    Mod+WheelScrollDown { zoom-out; }
    Mod+WheelScrollUp { zoom-in; }
    Mod+0 { reset-zoom; }  // Reset to auto-zoom
}
```

---

## Step 3.4: Rendering with Zoom

### Transform Pipeline

1. Compute visible region based on camera position + zoom
2. Cull tiles outside visible region
3. Transform tile positions: `screen_pos = (canvas_pos - camera_pos) * zoom`
4. Scale tile renders by zoom factor

### Tasks

- [ ] **3.4.1**: Compute visible region from camera state
- [ ] **3.4.2**: Cull non-visible rows/columns
- [ ] **3.4.3**: Apply transform to render positions
- [ ] **3.4.4**: Scale render elements by zoom
- [ ] **3.4.5**: Transform input coordinates (clicks) inversely

### Implementation

```rust
// src/layout/canvas/render.rs

impl<W: LayoutElement> Canvas2D<W> {
    pub fn visible_region(&self) -> Rectangle<f64, Logical> {
        let zoom = self.camera.zoom();
        let pos = self.camera.position();
        
        // Visible region in canvas coordinates
        Rectangle::new(
            pos,
            Size::from((
                self.viewport_size().w / zoom,
                self.viewport_size().h / zoom,
            )),
        )
    }
    
    pub fn render(&self, ...) -> Vec<CanvasRenderElement> {
        let visible = self.visible_region();
        let zoom = self.camera.zoom();
        let camera_pos = self.camera.position();
        
        let mut elements = Vec::new();
        
        for (row_idx, row) in &self.rows {
            // Skip rows outside visible region
            let row_rect = self.row_rect(*row_idx);
            if !visible.overlaps(&row_rect) {
                continue;
            }
            
            for element in row.render(...) {
                // Transform: canvas coords → screen coords
                let screen_pos = (element.position() - camera_pos) * zoom;
                let scaled = element
                    .with_position(screen_pos)
                    .with_scale(zoom);
                elements.push(scaled);
            }
        }
        
        elements
    }
    
    /// Transform screen coordinates to canvas coordinates (for input).
    pub fn screen_to_canvas(&self, screen_pos: Point<f64, Logical>) -> Point<f64, Logical> {
        let zoom = self.camera.zoom();
        let camera_pos = self.camera.position();
        screen_pos / zoom + camera_pos
    }
}
```

---

## Step 3.5: Animation Configuration

### New Config Options

```rust
// niri-config/src/animations.rs

pub struct Animations {
    // ... existing
    
    /// Camera position movement (X and Y)
    pub camera_movement: CameraMovementAnim,
    
    /// Camera zoom animation
    pub camera_zoom: CameraZoomAnim,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraMovementAnim {
    pub off: bool,
    pub curve: AnimationCurve,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraZoomAnim {
    pub off: bool,
    pub curve: AnimationCurve,
    pub duration_ms: u32,
}
```

### Tasks

- [ ] **3.5.1**: Add `camera_movement` config
- [ ] **3.5.2**: Add `camera_zoom` config
- [ ] **3.5.3**: Use configs in Camera animation
- [ ] **3.5.4**: Update default-config.kdl

---

## Checklist Summary

### Step 3.1: Camera Module
- [ ] `camera/mod.rs` with Camera struct
- [ ] `camera/position.rs`
- [ ] `camera/zoom.rs`
- [ ] Unit tests

### Step 3.2: Auto-Zoom
- [ ] `compute_target_for_tile()`
- [ ] Call on focus change
- [ ] Respect manual override

### Step 3.3: Zoom Gestures
- [ ] `zoom_in()`, `zoom_out()`
- [ ] Keybind config
- [ ] Clamp range
- [ ] Manual override flag

### Step 3.4: Rendering
- [ ] Visible region calculation
- [ ] Cull non-visible
- [ ] Transform positions
- [ ] Scale elements
- [ ] Inverse transform for input

### Step 3.5: Config
- [ ] `camera_movement` anim
- [ ] `camera_zoom` anim
- [ ] Default config

---

## Estimated Time: 1-2 Weeks

---

## Success Criteria

- [ ] Camera has x, y, zoom
- [ ] Focus on 2-row tile → camera zooms out
- [ ] Focus on 1-row tile → camera zooms in
- [ ] Manual zoom via Mod+Scroll works
- [ ] Rendering correctly transforms positions
- [ ] Clicking works at any zoom level

---

## Next Phase
→ [Phase 4: Navigation + Polish](phase-4-navigation.md)
