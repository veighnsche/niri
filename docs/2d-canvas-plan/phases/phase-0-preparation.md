# Phase 0: Preparation — Modular Foundation

> **Goal**: Refactor existing monolithic code into clean, modular components before building new features.

---

## Why This Phase?

The current `scrolling.rs` (and the partial refactor in `scrolling/`) has issues:
- Deep import chains (`super::super::super::tile::Tile`)
- `pub(super)` exposing internals across the module
- Methods spread across files but still tightly coupled
- No clear ownership boundaries

Before adding rows and 2D canvas, we need a solid foundation.

---

## Step 0.1: Create Modular Column Structure

### Current State
```
src/layout/scrolling/
├── column/
│   ├── mod.rs        # Column struct with pub(super) fields
│   ├── core.rs       # Construction, updates
│   ├── operations.rs # Add/remove tiles
│   ├── positioning.rs
│   └── sizing.rs
```

**Problem**: Column's fields are `pub(super)`, so any file in `scrolling/` can reach in.

### Target State
```
src/layout/column/           # Moved up, independent module
├── mod.rs                   # Column struct, PRIVATE fields, public methods only
├── layout.rs                # compute_tile_positions() -> Vec<TilePosition>
├── resize.rs                # handle_resize_begin/update/end
└── operations.rs            # add_tile, remove_tile, swap_tiles
```

### Tasks

- [ ] **0.1.1**: Move `scrolling/column/` to `layout/column/`
- [ ] **0.1.2**: Make all `Column` fields private (remove `pub(super)`)
- [ ] **0.1.3**: Add getter methods for anything external code needs
- [ ] **0.1.4**: Remove deep imports — Column should only import from `layout::tile`
- [ ] **0.1.5**: Update all callers to use public interface
- [ ] **0.1.6**: Add unit tests for Column in isolation

### Interface Design

```rust
// src/layout/column/mod.rs

pub struct Column<W: LayoutElement> {
    // ALL PRIVATE
    tiles: Vec<Tile<W>>,
    active_tile_idx: usize,
    width: ColumnWidth,
    is_fullscreen: bool,
    move_animation: Option<MoveAnimation>,
    // ...
}

impl<W: LayoutElement> Column<W> {
    // Construction
    pub fn new(options: Rc<Options>, ...) -> Self;
    
    // Queries (read-only)
    pub fn tiles(&self) -> &[Tile<W>];
    pub fn active_tile(&self) -> Option<&Tile<W>>;
    pub fn active_tile_idx(&self) -> usize;
    pub fn width(&self) -> &ColumnWidth;
    pub fn is_empty(&self) -> bool;
    
    // Layout
    pub fn compute_positions(&self, working_area: Rectangle) -> ColumnLayout;
    
    // Operations (mutating)
    pub fn add_tile(&mut self, tile: Tile<W>, ...);
    pub fn remove_tile(&mut self, idx: usize) -> Tile<W>;
    pub fn focus_tile(&mut self, idx: usize);
    
    // Resize
    pub fn resize_begin(&mut self, ...);
    pub fn resize_update(&mut self, ...);
    pub fn resize_end(&mut self);
}

/// Result of layout computation — positions without mutating Column
pub struct ColumnLayout {
    pub tile_positions: Vec<TilePosition>,
    pub total_height: f64,
}

pub struct TilePosition {
    pub idx: usize,
    pub y: f64,
    pub height: f64,
}
```

### Success Criteria
- [ ] `Column` compiles as independent module
- [ ] No `pub(super)` in column module
- [ ] All existing tests pass
- [ ] New unit tests for Column isolation

---

## Step 0.2: Extract View Offset into Reusable Component

### Current State
```rust
// In scrolling/types.rs
pub enum ViewOffset {
    Static(f64),
    Animation(Animation),
    Gesture(ViewGesture),
}
```

This is a 1D offset. We'll need 2D, but the pattern (static/animation/gesture) is reusable.

### Target State
```
src/layout/animated_value/
├── mod.rs                   # AnimatedValue<T> generic
└── gesture.rs               # Gesture handling
```

### Tasks

- [ ] **0.2.1**: Create `AnimatedValue<f64>` that handles static/animation/gesture
- [ ] **0.2.2**: Create `AnimatedPoint` for 2D (x, y) with same pattern
- [ ] **0.2.3**: Refactor `ViewOffset` to use `AnimatedValue<f64>`
- [ ] **0.2.4**: Test that existing view offset behavior is preserved

### Interface Design

```rust
// src/layout/animated_value/mod.rs

pub enum AnimatedValue<T> {
    Static(T),
    Animation { from: T, to: T, anim: Animation },
    Gesture { start: T, current: T, tracker: GestureTracker },
}

impl AnimatedValue<f64> {
    pub fn current(&self) -> f64;
    pub fn target(&self) -> f64;
    pub fn is_animating(&self) -> bool;
    pub fn set_static(&mut self, value: f64);
    pub fn animate_to(&mut self, target: f64, anim_config: AnimConfig);
    pub fn advance_animations(&mut self, now: Duration);
}

// Convenience type for 2D
pub struct AnimatedPoint {
    pub x: AnimatedValue<f64>,
    pub y: AnimatedValue<f64>,
}
```

### Success Criteria
- [ ] `AnimatedValue` works for f64
- [ ] Existing view offset behavior unchanged
- [ ] Ready to extend for Camera (x, y, zoom)

---

## Step 0.3: Clean Up ScrollingSpace Dependencies

### Current State

`ScrollingSpace` imports from many places and exposes internals:
```rust
use super::super::tile::Tile;
use super::super::workspace::...;
// etc.
```

### Target State

`ScrollingSpace` (which will become `Row`) should:
- Only depend on `Column` and common types
- Not know about `Workspace`, `Monitor`, or `Layout`
- Expose clean interface for parent to call

### Tasks

- [ ] **0.3.1**: List all imports in `scrolling/space/`
- [ ] **0.3.2**: Identify which are truly needed vs. coupling
- [ ] **0.3.3**: Move shared types to `layout/types.rs` or similar
- [ ] **0.3.4**: Remove upward dependencies (no `super::super`)
- [ ] **0.3.5**: Define clean interface for ScrollingSpace

### Interface Design

```rust
// What ScrollingSpace should look like after cleanup

pub struct ScrollingSpace<W: LayoutElement> {
    // Private
    columns: Vec<Column<W>>,
    active_column_idx: usize,
    view_offset: AnimatedValue<f64>,
    // ...
}

impl<W: LayoutElement> ScrollingSpace<W> {
    // Parent provides what's needed
    pub fn new(options: Rc<Options>, clock: Clock) -> Self;
    
    // Parent tells it about output changes
    pub fn set_working_area(&mut self, area: Rectangle);
    
    // Clean query interface
    pub fn columns(&self) -> &[Column<W>];
    pub fn active_column(&self) -> Option<&Column<W>>;
    pub fn view_offset(&self) -> f64;
    
    // Operations parent can request
    pub fn add_window(&mut self, window: W, ...) -> WindowId;
    pub fn remove_window(&mut self, id: &WindowId) -> Option<W>;
    pub fn focus_left(&mut self);
    pub fn focus_right(&mut self);
    
    // Layout computation
    pub fn compute_layout(&self) -> ScrollingLayout;
    
    // Rendering (returns elements, doesn't know about actual rendering)
    pub fn render(&self, ...) -> Vec<RenderElement>;
}
```

### Success Criteria
- [ ] No `super::super` imports in scrolling module
- [ ] ScrollingSpace doesn't import from workspace/monitor
- [ ] Interface is parent-agnostic (could be used by Row or Canvas2D)

---

## Checklist Summary

### Step 0.1: Modular Column
- [ ] Move to `layout/column/`
- [ ] Private fields, public methods
- [ ] Remove deep imports
- [ ] Unit tests

### Step 0.2: AnimatedValue
- [ ] Create generic `AnimatedValue<T>`
- [ ] Refactor ViewOffset
- [ ] Add AnimatedPoint for 2D

### Step 0.3: Clean ScrollingSpace
- [ ] Remove upward dependencies
- [ ] Clean interface
- [ ] Ready to become Row

---

## Estimated Time: 1 Week

This phase is about cleanup and doesn't add features, but it makes Phase 1-5 much cleaner.

---

## Next Phase
→ [Phase 1: Row + Canvas2D](phase-1-row-and-canvas.md)
