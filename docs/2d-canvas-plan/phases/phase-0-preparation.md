# Phase 0: Preparation — Modular Foundation

> **Goal**: Refactor existing monolithic code into clean, modular components before building new features.

---

## Why This Phase?

The current `scrolling.rs` is a **5586-line monolith** containing:
- `ScrollingSpace<W>` struct + 3500 lines of methods
- `Column<W>` struct + 1520 lines of methods
- Supporting types: `ViewOffset`, `ViewGesture`, `ColumnData`, `TileData`, `ColumnWidth`, `WindowHeight`

Before adding rows and 2D canvas, we need a solid foundation.

---

## Current State Inventory

### What We're Refactoring

| File | Lines | Contains |
|------|-------|----------|
| `scrolling.rs` | 5586 | ScrollingSpace, Column, ViewOffset, all methods |
| `tile.rs` | ~1400 | Tile struct (already modular, keep as-is) |
| `workspace.rs` | ~1800 | Workspace (uses ScrollingSpace) |
| `monitor.rs` | ~2000 | Monitor (uses Workspace) |

### Structs to Extract

1. **Column<W>** (lines 145-216, 1520 lines of methods)
   - Owns: tiles, active_tile_idx, width, display_mode, animations
   - Methods: tile operations, sizing, focus, fullscreen, rendering

2. **ViewOffset** (lines 112-119)
   - Enum: Static, Animation, Gesture
   - Will become `AnimatedValue<f64>`

3. **Supporting types** (lines 104-282)
   - `ColumnData`, `TileData`, `ColumnWidth`, `WindowHeight`, `ScrollDirection`, `MoveAnimation`, `ViewGesture`

---

## Refactoring Strategy: HOW to Split

### Principle: Extract by Ownership

Each struct owns its state. We split by asking: **"What data does this method need?"**

```
Method needs only Column fields? → Goes to column/ module
Method needs ScrollingSpace + Column? → Stays in scrolling.rs, calls Column methods
Method is pure computation? → Goes to a helper module
```

### Column Method Categories (1520 lines → 4 files)

After analyzing all Column methods, here's the split:

#### `column/mod.rs` (~200 lines) — Struct + Core
```rust
// Struct definition with PRIVATE fields
pub struct Column<W: LayoutElement> { ... }

// Construction
fn new_with_tile(...)
fn update_config(...)

// Simple getters
pub fn tiles(&self) -> &[Tile<W>]
pub fn active_tile_idx(&self) -> usize
pub fn width(&self) -> f64
pub fn is_pending_fullscreen(&self) -> bool
pub fn contains(&self, window: &W::Id) -> bool
pub fn position(&self, window: &W::Id) -> Option<usize>
```

#### `column/layout.rs` (~400 lines) — Positioning & Sizing
```rust
// Tile positioning
fn tiles_origin(&self) -> Point
fn tile_offsets(&self) -> impl Iterator
fn tile_offsets_in_render_order(...)
pub fn tiles(&self) -> impl Iterator<Item = (&Tile, Point)>

// Size computation
fn extra_size(&self) -> Size
fn resolve_column_width(&self, width: ColumnWidth) -> f64
fn update_tile_sizes(&mut self, animate: bool)
fn update_tile_sizes_with_transaction(...)
```

#### `column/operations.rs` (~400 lines) — Add/Remove/Focus
```rust
// Tile operations
fn add_tile_at(&mut self, idx: usize, tile: Tile<W>)
fn update_window(&mut self, window: &W::Id)
fn activate_idx(&mut self, idx: usize) -> bool
fn activate_window(&mut self, window: &W::Id)

// Focus movement
fn focus_up(&mut self) -> bool
fn focus_down(&mut self) -> bool
fn focus_index(&mut self, index: u8)

// Tile movement
fn move_up(&mut self) -> bool
fn move_down(&mut self) -> bool
```

#### `column/sizing.rs` (~400 lines) — Width/Height/Fullscreen
```rust
// Width operations
fn toggle_width(&mut self, forwards: bool)
fn toggle_full_width(&mut self)
fn set_column_width(&mut self, change: SizeChange, ...)

// Height operations
fn set_window_height(&mut self, change: SizeChange, ...)
fn reset_window_height(&mut self, tile_idx: Option<usize>)
fn toggle_window_height(&mut self, forwards: bool)
fn convert_heights_to_auto(&mut self)

// Fullscreen/Maximize
fn set_fullscreen(&mut self, is_fullscreen: bool)
fn set_maximized(&mut self, maximize: bool)
fn sizing_mode(&self) -> SizingMode
```

#### `column/render.rs` (~120 lines) — Rendering
```rust
pub fn update_render_elements(&mut self, is_active: bool, view_rect: Rectangle)
pub fn render_offset(&self) -> Point
fn tab_indicator_area(&self) -> Rectangle
pub fn start_open_animation(&mut self, id: &W::Id) -> bool
```

---

## Step 0.1: Create Modular Column Structure

### Current State
```
src/layout/scrolling.rs    # 5586 lines, everything in one file
```

**Problem**: Column is embedded inside scrolling.rs with no module boundary.

### Target State
```
src/layout/
├── column/                      # NEW: Independent module
│   ├── mod.rs                   # Column struct + re-exports
│   ├── layout.rs                # Tile positioning
│   ├── operations.rs            # Add/remove/focus tiles
│   ├── sizing.rs                # Width/height/fullscreen
│   └── render.rs                # Rendering
├── scrolling.rs                 # SMALLER: Only ScrollingSpace now
└── ...
```

### Tasks

- [ ] **0.1.1**: Create `src/layout/column/mod.rs` with Column struct (copy from scrolling.rs lines 145-216)
- [ ] **0.1.2**: Create `column/layout.rs` — move tile positioning methods
- [ ] **0.1.3**: Create `column/operations.rs` — move add/remove/focus methods
- [ ] **0.1.4**: Create `column/sizing.rs` — move width/height/fullscreen methods
- [ ] **0.1.5**: Create `column/render.rs` — move rendering methods
- [ ] **0.1.6**: Move supporting types (`ColumnWidth`, `WindowHeight`, `TileData`, `MoveAnimation`) to column module
- [ ] **0.1.7**: Update `scrolling.rs` to `use crate::layout::column::Column`
- [ ] **0.1.8**: Verify all tests pass after extraction

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
// In scrolling.rs (lines 112-142)
pub(super) enum ViewOffset {
    Static(f64),
    Animation(Animation),
    Gesture(ViewGesture),
}

pub(super) struct ViewGesture {
    current_view_offset: f64,
    animation: Option<Animation>,
    tracker: SwipeTracker,
    delta_from_tracker: f64,
    stationary_view_offset: f64,
    is_touchpad: bool,
    dnd_last_event_time: Option<Duration>,
    dnd_nonzero_start_time: Option<Duration>,
}
```

This is a 1D offset. We'll need 2D for Camera, but the pattern (static/animation/gesture) is reusable.

### Target State
```
src/layout/animated_value/
├── mod.rs                   # AnimatedValue<f64> + AnimatedPoint
└── gesture.rs               # ViewGesture (extracted from scrolling.rs)
```

### Tasks

- [ ] **0.2.1**: Create `src/layout/animated_value/mod.rs` with `AnimatedValue<f64>` enum
- [ ] **0.2.2**: Move `ViewGesture` to `animated_value/gesture.rs`
- [ ] **0.2.3**: Create `AnimatedPoint` for 2D (x, y) — used by Camera later
- [ ] **0.2.4**: Replace `ViewOffset` in scrolling.rs with `AnimatedValue<f64>`
- [ ] **0.2.5**: Verify all gesture/animation behavior unchanged

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

`ScrollingSpace` (lines 34-94, 3500 lines of methods) imports from:
```rust
// scrolling.rs lines 1-28
use super::closing_window::{ClosingWindow, ClosingWindowRenderElement};
use super::monitor::InsertPosition;
use super::tab_indicator::{TabIndicator, TabIndicatorRenderElement, TabInfo};
use super::tile::{Tile, TileRenderElement, TileRenderSnapshot};
use super::workspace::{InteractiveResize, ResolvedSize};
use super::{ConfigureIntent, HitType, InteractiveResizeData, LayoutElement, Options, RemovedTile};
```

**Problem**: ScrollingSpace knows about `workspace::InteractiveResize` — this couples it upward.

### Target State

After Column extraction, `ScrollingSpace` (which will become `Row`) should:
- Import from `column/` module (downward dependency ✓)
- Import from `tile/` module (sibling dependency ✓)
- NOT import from `workspace/` or `monitor/` (upward dependency ✗)
- Own its own `InteractiveResize` or receive it as a parameter

### Tasks

- [ ] **0.3.1**: Audit all imports in scrolling.rs after Column extraction
- [ ] **0.3.2**: Move `InteractiveResize` to scrolling.rs (or make it a parameter)
- [ ] **0.3.3**: Move shared types (`InsertPosition`, etc.) to `layout/types.rs`
- [ ] **0.3.4**: Ensure no `super::workspace` or `super::monitor` imports remain
- [ ] **0.3.5**: Document the clean public interface for ScrollingSpace

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

### Step 0.1: Modular Column (Estimated: 2-3 days)
- [ ] Create `layout/column/` module structure
- [ ] Extract Column struct + 1520 lines of methods
- [ ] Split into: mod.rs, layout.rs, operations.rs, sizing.rs, render.rs
- [ ] Move supporting types (ColumnWidth, WindowHeight, TileData, MoveAnimation)
- [ ] Update scrolling.rs to use new column module
- [ ] All tests pass

### Step 0.2: AnimatedValue (Estimated: 1 day)
- [ ] Create `layout/animated_value/` module
- [ ] Create `AnimatedValue<f64>` enum
- [ ] Move ViewGesture to gesture.rs
- [ ] Replace ViewOffset with AnimatedValue
- [ ] Create AnimatedPoint for 2D (Camera prep)

### Step 0.3: Clean ScrollingSpace (Estimated: 1-2 days)
- [ ] Audit and fix imports after Column extraction
- [ ] Move InteractiveResize out of workspace.rs
- [ ] Create layout/types.rs for shared types
- [ ] No upward dependencies (workspace, monitor)
- [ ] Document public interface

---

## Execution Order

1. **Step 0.1 first** — Column extraction is the biggest piece
2. **Step 0.3 second** — Clean up the mess from extraction
3. **Step 0.2 last** — AnimatedValue is independent, can be done anytime

---

## Estimated Time: 1 Week

This phase is about cleanup and doesn't add features, but it makes Phase 1-5 much cleaner.

---

## Next Phase
→ [Phase 1: Row + Canvas2D](phase-1-row-and-canvas.md)
