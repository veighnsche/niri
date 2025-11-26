# Phase 1: Row + Canvas2D

> **Goal**: Create the Row and Canvas2D modules with basic vertical navigation.

---

## Prerequisites

- Phase 0 complete (modular Column, AnimatedValue, clean ScrollingSpace)

### Starting Point After Phase 0
```
src/layout/
├── column/              # Extracted from scrolling.rs
│   ├── mod.rs
│   ├── layout.rs
│   ├── operations.rs
│   └── sizing.rs
├── animated_value/      # New reusable animation primitive
│   └── mod.rs
├── scrolling.rs         # Now ~4000 lines (ScrollingSpace only)
├── tile.rs              # Unchanged
├── workspace.rs         # Unchanged
└── monitor.rs           # Unchanged
```

---

## Step 1.1: Create Row Module

### What is Row?

Row is essentially `ScrollingSpace` renamed and slightly modified:
- A horizontal strip of columns
- Has its own X view offset
- Knows its position in the canvas (row index, y offset)

### File Structure

```
src/layout/row/
├── mod.rs              # Row struct, public interface
├── layout.rs           # Column positioning
├── navigation.rs       # Left/right focus within row
└── operations.rs       # Add/remove columns
```

### Tasks

- [ ] **1.1.1**: Create `row/mod.rs` with `Row<W>` struct
- [ ] **1.1.2**: Port `ScrollingSpace` logic (or wrap it via composition)
- [ ] **1.1.3**: Add row-specific fields (`row_index: i32`, `y_offset: f64`)
- [ ] **1.1.4**: Create `row/layout.rs` — compute column positions
- [ ] **1.1.5**: Create `row/navigation.rs` — focus_left, focus_right
- [ ] **1.1.6**: Create `row/operations.rs` — add/remove columns
- [ ] **1.1.7**: Unit tests for Row

### Interface Design

```rust
// src/layout/row/mod.rs

pub struct Row<W: LayoutElement> {
    columns: Vec<Column<W>>,
    active_column_idx: usize,
    view_offset_x: AnimatedValue<f64>,
    
    // Row-specific
    row_index: i32,
    y_offset: f64,  // Computed from row_index * row_height
    
    options: Rc<Options>,
    clock: Clock,
}

impl<W: LayoutElement> Row<W> {
    // Construction
    pub fn new(row_index: i32, options: Rc<Options>, clock: Clock) -> Self;
    
    // Queries
    pub fn row_index(&self) -> i32;
    pub fn y_offset(&self) -> f64;
    pub fn columns(&self) -> &[Column<W>];
    pub fn active_column(&self) -> Option<&Column<W>>;
    pub fn active_column_idx(&self) -> usize;
    pub fn view_offset_x(&self) -> f64;
    pub fn is_empty(&self) -> bool;
    
    // Configuration
    pub fn set_row_height(&mut self, height: f64);
    pub fn set_working_area(&mut self, area: Rectangle);
    
    // Navigation
    pub fn focus_left(&mut self) -> bool;
    pub fn focus_right(&mut self) -> bool;
    pub fn focus_column(&mut self, idx: usize);
    
    // Operations
    pub fn add_window(&mut self, window: W, target: AddWindowTarget) -> TileRef;
    pub fn remove_window(&mut self, id: &WindowId) -> Option<W>;
    
    // Layout
    pub fn compute_layout(&self) -> RowLayout;
    
    // Rendering
    pub fn render(&self, ...) -> Vec<RowRenderElement>;
    
    // Animation
    pub fn advance_animations(&mut self, now: Duration);
}

pub struct RowLayout {
    pub columns: Vec<ColumnLayout>,
    pub row_y: f64,
    pub row_height: f64,
}
```

### Decision: Composition vs. Refactor

**Option A**: `Row` wraps `ScrollingSpace`
```rust
pub struct Row<W> {
    space: ScrollingSpace<W>,
    row_index: i32,
}
```
- Pro: Less code duplication
- Con: Extra indirection

**Option B**: `Row` replaces `ScrollingSpace`
```rust
pub struct Row<W> {
    columns: Vec<Column<W>>,
    // ... all fields directly
}
```
- Pro: Clean slate, no legacy
- Con: More initial work

**Recommendation**: Option B — clean slate. We're building from the ground up.

---

## Step 1.2: Create Canvas2D Module

### What is Canvas2D?

Canvas2D replaces `Workspace` for 2D mode:
- Contains multiple rows
- Has a camera with (x, y, zoom)
- Manages floating layer
- Coordinates cross-row behavior

### File Structure

```
src/layout/canvas/
├── mod.rs              # Canvas2D struct, public interface
├── layout.rs           # Row positioning
├── navigation.rs       # Up/down/left/right focus
├── operations.rs       # Add/remove rows, add/remove windows
└── render.rs           # Rendering with camera transform
```

### Tasks

- [ ] **1.2.1**: Create `canvas/mod.rs` with `Canvas2D<W>` struct
- [ ] **1.2.2**: Use `BTreeMap<i32, Row<W>>` for sparse row storage
- [ ] **1.2.3**: Port relevant fields from `Workspace` (output, scale, clock, etc.)
- [ ] **1.2.4**: Create `canvas/layout.rs` — compute row positions
- [ ] **1.2.5**: Create `canvas/operations.rs` — add row, add window
- [ ] **1.2.6**: Create `canvas/render.rs` — basic rendering (no zoom yet)
- [ ] **1.2.7**: Unit tests for Canvas2D

### Interface Design

```rust
// src/layout/canvas/mod.rs

pub struct Canvas2D<W: LayoutElement> {
    // Rows indexed by row number (0 = origin, negative = above, positive = below)
    rows: BTreeMap<i32, Row<W>>,
    
    // Active position
    active_row_idx: i32,
    
    // Floating layer (unchanged from Workspace)
    floating: FloatingSpace<W>,
    floating_is_active: bool,
    
    // Camera (simplified for Phase 1, expanded in Phase 3)
    camera_x: AnimatedValue<f64>,
    camera_y: AnimatedValue<f64>,
    // camera_zoom added in Phase 3
    
    // From Workspace
    output: Option<Output>,
    view_size: Size<f64, Logical>,
    working_area: Rectangle<f64, Logical>,
    scale: smithay::output::Scale,
    
    options: Rc<Options>,
    clock: Clock,
}

impl<W: LayoutElement> Canvas2D<W> {
    // Construction
    pub fn new(output: Output, options: Rc<Options>, clock: Clock) -> Self;
    
    // Queries
    pub fn rows(&self) -> impl Iterator<Item = (i32, &Row<W>)>;
    pub fn active_row(&self) -> Option<&Row<W>>;
    pub fn active_row_idx(&self) -> i32;
    pub fn active_tile(&self) -> Option<&Tile<W>>;
    
    // Configuration
    pub fn set_output(&mut self, output: &Output);
    
    // Navigation (basic for Phase 1)
    pub fn focus_up(&mut self) -> bool;
    pub fn focus_down(&mut self) -> bool;
    pub fn focus_left(&mut self) -> bool;
    pub fn focus_right(&mut self) -> bool;
    
    // Operations
    pub fn add_window(&mut self, window: W, ...) -> TileRef;
    pub fn remove_window(&mut self, id: &WindowId) -> Option<W>;
    
    // Floating (delegated)
    pub fn toggle_floating(&mut self);
    
    // Layout
    pub fn compute_layout(&self) -> CanvasLayout;
    
    // Rendering
    pub fn render(&self, ...) -> Vec<CanvasRenderElement>;
    
    // Animation
    pub fn advance_animations(&mut self, now: Duration);
}
```

---

## Step 1.3: Basic Vertical Navigation

### Simple Version First

For Phase 1, "up" and "down" are simple:
- Move to same column index in adjacent row
- If that column doesn't exist, stay put (or find nearest)

Geometric navigation comes in Phase 4.

### Tasks

- [ ] **1.3.1**: Implement `Canvas2D::focus_up()`
- [ ] **1.3.2**: Implement `Canvas2D::focus_down()`
- [ ] **1.3.3**: Update camera Y to follow active row
- [ ] **1.3.4**: Test navigation between rows

### Implementation

```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn focus_up(&mut self) -> bool {
        let target_row = self.active_row_idx - 1;
        if self.rows.contains_key(&target_row) {
            let col_idx = self.active_row()
                .map(|r| r.active_column_idx())
                .unwrap_or(0);
            
            self.active_row_idx = target_row;
            if let Some(row) = self.rows.get_mut(&target_row) {
                row.focus_column(col_idx.min(row.columns().len().saturating_sub(1)));
            }
            self.update_camera_y();
            true
        } else {
            false
        }
    }
    
    fn update_camera_y(&mut self) {
        let target_y = self.active_row_idx as f64 * self.row_height();
        self.camera_y.animate_to(target_y, self.options.animations.vertical_view_movement);
    }
}
```

---

## Step 1.4: Feature Flag Integration

### Tasks

- [ ] **1.4.1**: Add `canvas-2d` feature to `Cargo.toml`
- [ ] **1.4.2**: In `Monitor`, conditionally use `Canvas2D` or `Workspace`
- [ ] **1.4.3**: Ensure existing tests pass with feature disabled
- [ ] **1.4.4**: Add new tests that run with feature enabled

### Cargo.toml

```toml
[features]
default = []
canvas-2d = []
```

### Monitor Integration

```rust
// src/layout/monitor.rs

pub struct Monitor<W: LayoutElement> {
    #[cfg(feature = "canvas-2d")]
    canvas: Canvas2D<W>,
    
    #[cfg(not(feature = "canvas-2d"))]
    workspaces: Vec<Workspace<W>>,
    
    // ... other fields
}
```

---

## Checklist Summary

### Step 1.1: Row Module
- [x] Create `row/mod.rs` (TEAM_006)
- [x] Port column management — `add_tile`, `add_column`, `remove_column` (TEAM_006)
- [x] Add row_index, y_offset (TEAM_006)
- [x] Navigation: focus_left, focus_right, focus_column (TEAM_006)
- [x] Operations: add/remove, move_left/right (TEAM_006)
- [x] Port full view offset animation logic (TEAM_007)
- [x] Port render_elements (TEAM_007)
- [ ] **NEXT**: Port column movement animations
- [ ] **NEXT**: Port consume/expel operations
- [ ] **NEXT**: Port gesture handling

### Step 1.2: Canvas2D Module
- [x] Create `canvas/mod.rs` (TEAM_006)
- [x] BTreeMap for rows (TEAM_006)
- [x] Operations: add_tile, add_tile_to_row, contains, find_window (TEAM_006)
- [x] Animate camera_y when changing rows (TEAM_007)
- [x] Port render_elements (TEAM_007)
- [ ] **NEXT**: Integrate FloatingSpace

### Step 1.3: Vertical Navigation
- [x] focus_up, focus_down (TEAM_006)
- [x] Camera Y animation (TEAM_007)
- [ ] Test navigation

### Step 1.4: Feature Flag
- [ ] Add `canvas-2d` feature
- [ ] Conditional Monitor code
- [ ] Tests pass both ways

---

## TEAM_006 Handoff Notes

**What's done:**
- Row and Canvas2D modules created with clean-slate design (Option B)
- Basic column operations work: add, remove, move, focus
- Basic vertical navigation works: focus_up, focus_down
- All 251 tests pass, 58 golden tests pass

**What's next for TEAM_007:**
1. Port column movement animations from `scrolling.rs` (lines 1518-1553)
2. Port `animate_view_offset_to_column` full logic (lines 500-600)
3. Port `render_elements` from ScrollingSpace
4. Integrate FloatingSpace into Canvas2D
5. Add feature flag for conditional compilation

**Key files to reference:**
- `src/layout/scrolling.rs` — source of methods to port
- `docs/2d-canvas-plan/TODO.md` — detailed TODO list with line numbers

---

## Estimated Time: 1-2 Weeks

---

## Success Criteria

- [x] Can create Canvas2D with multiple rows
- [x] Can navigate up/down between rows
- [x] Left/right navigation works within row
- [x] Camera Y updates when changing rows (animation) — TEAM_007
- [ ] Existing behavior preserved with feature flag off

---

## Next Phase
→ [Phase 2: Row Spanning](phase-2-row-spanning.md)
