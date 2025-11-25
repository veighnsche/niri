# 2D Canvas — Feasibility Study

> This document analyzes what exists today and what would need to change for 2D support.

---

## Current Architecture Summary

```
Layout (global)
└── Monitor (per-output)
    └── Workspace[] (vertical stack, switchable)
        ├── ScrollingSpace (1D horizontal strip)
        │   └── Column[] (each is a vertical stack of tiles)
        │       └── Tile[] (actual windows)
        └── FloatingSpace (free-positioned windows)
```

### Key Assumptions Baked Into Current Design

| Assumption | Where It's Baked In |
|------------|---------------------|
| Windows arranged on 1D horizontal strip | `column_x(idx)` returns single f64 |
| View offset is scalar (X only) | `ViewOffset` is `f64`, not `Point` |
| Tiles stack vertically within column | `Column.tiles[]` indexed by Y position |
| No zoom — fixed scale | Rendering assumes `scale = output.scale` |
| Focus moves left/right at column level | `focus_column_left()` / `focus_column_right()` |
| Workspaces are separate, vertically stacked | `Monitor.workspaces[]`, switch is animated Y |

---

## Code Locations That Would Need Changes

### 1. Positioning System

**Current**: Position is computed from column index.

```rust
// scrolling/space/queries.rs (or similar)
pub fn column_x(&self, idx: usize) -> f64 {
    // Sum of widths + gaps for all columns before idx
    // Returns single X coordinate
}
```

**For 2D**: Would need `(x, y)` coordinates per window/cell.

```rust
pub fn window_pos(&self, idx: WindowId) -> Point<f64, Logical> {
    // Full 2D coordinate
}
```

**Files affected**:
- `scrolling/space/queries.rs`
- `scrolling/column/positioning.rs`
- `scrolling/utils.rs` (`compute_new_view_offset`)

---

### 2. View Offset / Camera

**Current**: Single dimension.

```rust
// scrolling/types.rs
pub enum ViewOffset {
    Static(f64),           // X offset only
    Animation(Animation),  // Animates X only
    Gesture(ViewGesture),  // Tracks X delta only
}
```

**For 2D**: Would need 2D camera position + zoom.

```rust
pub struct Camera {
    offset: Point<f64, Logical>,  // (X, Y)
    zoom: f64,                     // 1.0 = normal, 0.5 = zoomed out
    // ... animations for each
}
```

**Files affected**:
- `scrolling/types.rs`
- `scrolling/space/view_offset.rs`
- `scrolling/gestures.rs`

---

### 3. Column Concept

**Current**: `Column` is a vertical stack of tiles at a fixed X position.

```rust
// scrolling/column/mod.rs
pub struct Column<W> {
    tiles: Vec<Tile<W>>,  // Vertically stacked
    width: ColumnWidth,
    // ...
}
```

**For 2D**: Options:
- **Keep columns**: They become "groups" that can be placed anywhere
- **Remove columns**: Each window is independent with its own `(x, y)`
- **Grid cells**: Replace columns with cells that can contain windows

**Files affected**:
- Entire `scrolling/column/` directory
- `scrolling/manipulation/` (add/remove logic)

---

### 4. Rendering with Zoom

**Current**: Windows rendered at output scale, no canvas zoom.

```rust
// scrolling/render.rs
for (col, col_x) in self.columns_with_x() {
    // Render at col_x, no scale adjustment
}
```

**For 2D with zoom**: Would need to apply camera transform.

```rust
// Pseudocode
let camera_transform = Transform {
    translate: -self.camera.offset,
    scale: self.camera.zoom,
};

for window in self.windows() {
    let screen_pos = camera_transform.apply(window.pos());
    // Render at screen_pos, scaled by camera.zoom
}
```

**Files affected**:
- `scrolling/render.rs`
- `layout/workspace.rs` (calls render)
- `layout/monitor.rs` (overview already does some of this)

---

### 5. Input / Navigation

**Current**: Left/right at column level, up/down within column.

```rust
// layout/mod.rs
pub fn focus_column_left(&mut self) { ... }
pub fn focus_column_right(&mut self) { ... }
pub fn focus_up(&mut self) { ... }    // Within column
pub fn focus_down(&mut self) { ... }  // Within column
```

**For 2D**: Unified directional navigation based on geometry.

```rust
pub fn focus_direction(&mut self, dir: Direction) {
    // Find nearest window in direction based on (x, y) positions
}
```

**Files affected**:
- `layout/mod.rs`
- `layout/workspace.rs`
- `scrolling/manipulation/movement.rs`

---

### 6. Gestures

**Current**: Horizontal-only gestures with `SwipeTracker`.

```rust
// scrolling/gestures.rs
pub fn view_gesture_begin(&mut self, is_touchpad: bool) { ... }
// Tracks X delta only
```

**For 2D**: Would need 2D gesture tracking.

```rust
pub fn pan_gesture_update(&mut self, delta: Point<f64, Logical>) { ... }
pub fn zoom_gesture_update(&mut self, scale_delta: f64) { ... }
```

**Files affected**:
- `scrolling/gestures.rs`
- `input/swipe_tracker.rs` (may need 2D variant)

---

## Reusable Components (What Can Stay)

| Component | Reusable? | Notes |
|-----------|-----------|-------|
| `Tile<W>` | ✅ Yes | Window wrapper, position-agnostic |
| `Animation` | ✅ Yes | Already generic |
| `Clock` | ✅ Yes | Timing system |
| `Options` / config | ✅ Mostly | May need new config fields |
| `LayoutElement` trait | ✅ Yes | Core abstraction |
| Rendering pipeline | ⚠️ Partial | Needs zoom transform added |
| `FloatingSpace` | ⚠️ Partial | Already has (x, y), could be starting point |

---

## Three Possible Approaches

### Approach A: Extend Current Model

Keep columns, add Y positioning for columns themselves.

```
Canvas
└── Column[] with (x, y) position each
    └── Tile[] (vertical stack within column)
```

- **Pros**: Minimal changes, preserves column semantics
- **Cons**: Still feels 1.5D, not true 2D freedom
- **Effort**: 2-4 weeks

### Approach B: True 2D Canvas

Replace columns with free-positioned windows.

```
Canvas
└── Window[] with (x, y, width, height) each
```

- **Pros**: Maximum flexibility, clean model
- **Cons**: Loses auto-tiling, more like floating
- **Effort**: 1-2 months

### Approach C: Grid-Based 2D

Windows snap to a logical grid.

```
Canvas
└── Grid cells at (row, col)
    └── Each cell contains 0-1 windows
```

- **Pros**: Predictable navigation, easy zoom math
- **Cons**: Less flexible than floating
- **Effort**: 1-2 months

---

## Questions That Affect Feasibility

1. **Do you want auto-tiling?** (windows automatically fill space)
2. **Do you want overlapping windows?** (floating-style)
3. **Is zoom purely visual, or can you interact while zoomed?**
4. **Is this replacing ScrollingSpace, or adding a new mode?**

---

## Next Step

Fill out `2D_CANVAS_QUESTIONNAIRE.md` and I'll map your answers to one of these approaches with a concrete implementation plan.
