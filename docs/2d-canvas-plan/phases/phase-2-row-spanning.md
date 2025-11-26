# Phase 2: Row Spanning

> **Goal**: Windows can span multiple rows, layout adjusts accordingly.

---

## Prerequisites

- Phase 1 complete (Row, Canvas2D, basic navigation) ✅
- Phase 1.5 complete (Integration, feature flag)

### Starting Point After Phase 1.5
```
src/layout/
├── column/              # From Phase 0
│   ├── mod.rs, core.rs, layout.rs, operations.rs
│   ├── render.rs, sizing.rs, tile_data.rs
│   └── tests.rs
├── animated_value/      # From Phase 0
│   ├── mod.rs           # AnimatedValue enum
│   └── gesture.rs       # ViewGesture
├── row/                 # From Phase 1/1.5
│   ├── mod.rs           # Core struct + accessors
│   ├── layout.rs        # Tile positions, config
│   ├── navigation.rs    # Focus left/right/column
│   ├── operations.rs    # Add/remove/move columns
│   ├── render.rs        # Rendering
│   └── view_offset.rs   # View offset animation
├── canvas/              # From Phase 1/1.5
│   └── mod.rs           # Canvas2D (single file, ~400 lines)
├── scrolling.rs         # Still exists for non-2D mode
├── tile.rs
├── workspace.rs
└── monitor.rs           # Updated with feature flag
```

---

## Core Concept

A window with `row_span = 2` occupies the vertical space of 2 rows:

```
─────────────────────────────────────────────────────────────────────
ROW 0   | Column A    | Column B  |        Column C                 |
        | (1× span)   | TileC     |                                 |
        |             |-----------|     "Important App"             |
        |   App A     | TileD     |       row_span = 2              |
        |             |           |                                 |
─────────────────────────────────────────────────────────────────────
ROW 1   |             | Column E  |        ↑↑↑↑↑↑↑                  |
        |   App B     | TileF     |      (same window               |
        |  (1× span)  |           |       continues here)           |
        |             |           |                                 |
─────────────────────────────────────────────────────────────────────
```

The "Important App" is logically in Row 0, but its tile extends into Row 1's space.

---

## Step 2.1: Add Row Span to Tile

### Tasks

- [ ] **2.1.1**: Add `row_span: u8` field to `Tile`
- [ ] **2.1.2**: Default to `1`
- [ ] **2.1.3**: Add getter `Tile::row_span()`
- [ ] **2.1.4**: Add setter `Tile::set_row_span(span: u8)`
- [ ] **2.1.5**: Validate span (minimum 1, maximum configurable)

### Implementation

```rust
// src/layout/tile.rs

pub struct Tile<W: LayoutElement> {
    // ... existing fields
    
    /// Number of rows this tile spans vertically.
    /// 1 = normal, 2 = spans two rows, etc.
    row_span: u8,
}

impl<W: LayoutElement> Tile<W> {
    pub fn new(window: W, ...) -> Self {
        Self {
            // ...
            row_span: 1,
        }
    }
    
    pub fn row_span(&self) -> u8 {
        self.row_span
    }
    
    pub fn set_row_span(&mut self, span: u8) {
        self.row_span = span.max(1); // Minimum 1
    }
}
```

---

## Step 2.2: Row Span in Layout Calculation

### Column Height Calculation

When computing column layout, tile height depends on row span:

```rust
// src/layout/row/layout.rs

impl<W: LayoutElement> Row<W> {
    pub fn compute_layout(&self, row_height: f64) -> RowLayout {
        for column in &self.columns {
            for tile in column.tiles() {
                let tile_height = tile.row_span() as f64 * row_height;
                // Use tile_height in positioning
            }
        }
    }
}
```

### Tasks

- [ ] **2.2.1**: Pass `row_height` to layout computation
- [ ] **2.2.2**: Compute tile height as `row_span * row_height`
- [ ] **2.2.3**: Adjust column layout to handle variable tile heights
- [ ] **2.2.4**: Test with mixed row spans in a column

---

## Step 2.3: Cross-Row Coordination

### The Occupancy Problem

When a tile in Row 0 spans into Row 1:
- That tile's column in Row 1 is "occupied"
- Can't place windows there
- Navigation must understand this

### File Structure

```
src/layout/canvas/
├── spanning.rs         # NEW: Cross-row span coordination
```

### Tasks

- [ ] **2.3.1**: Create `canvas/spanning.rs`
- [ ] **2.3.2**: Implement `is_position_occupied(row, col) -> bool`
- [ ] **2.3.3**: Implement `tile_at(row, col) -> Option<TileRef>` with span awareness
- [ ] **2.3.4**: Update `add_window()` to respect occupied positions
- [ ] **2.3.5**: Update navigation to skip occupied positions

### Implementation

```rust
// src/layout/canvas/spanning.rs

impl<W: LayoutElement> Canvas2D<W> {
    /// Check if a position is occupied by a spanning tile from a row above.
    pub fn is_occupied_by_span(&self, row_idx: i32, col_idx: usize) -> bool {
        // Check rows above
        for span_offset in 1..=MAX_ROW_SPAN {
            let upper_row_idx = row_idx - span_offset as i32;
            if let Some(upper_row) = self.rows.get(&upper_row_idx) {
                if let Some(column) = upper_row.column(col_idx) {
                    if let Some(tile) = column.active_tile() {
                        if tile.row_span() > span_offset {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    /// Get the tile at a position, accounting for row spans.
    /// Returns the actual tile even if it's from a row above.
    pub fn effective_tile_at(&self, row_idx: i32, col_idx: usize) -> Option<TileRef> {
        // First check if there's a tile directly here
        if let Some(row) = self.rows.get(&row_idx) {
            if let Some(col) = row.column(col_idx) {
                if col.active_tile().is_some() {
                    return Some(TileRef { row: row_idx, col: col_idx });
                }
            }
        }
        
        // Check if a spanning tile from above covers this position
        for span_offset in 1..=MAX_ROW_SPAN {
            let upper_row_idx = row_idx - span_offset as i32;
            if let Some(upper_row) = self.rows.get(&upper_row_idx) {
                if let Some(column) = upper_row.column(col_idx) {
                    if let Some(tile) = column.active_tile() {
                        if tile.row_span() > span_offset {
                            return Some(TileRef { row: upper_row_idx, col: col_idx });
                        }
                    }
                }
            }
        }
        
        None
    }
}
```

---

## Step 2.4: Row Span Commands

### New Actions

| Action | Effect |
|--------|--------|
| `increase-row-span` | Current tile spans one more row |
| `decrease-row-span` | Current tile spans one fewer row |
| `set-row-span N` | Set tile's row span to N |

### Tasks

- [ ] **2.4.1**: Add actions to `niri-config/src/actions.rs`
- [ ] **2.4.2**: Implement handlers in layout
- [ ] **2.4.3**: Validate: can't increase if would overlap another tile
- [ ] **2.4.4**: Add keybind suggestions to default config
- [ ] **2.4.5**: Update camera zoom when span changes (Phase 3 prep)

### Implementation

```rust
// In Canvas2D

pub fn increase_row_span(&mut self) -> bool {
    let Some(tile_ref) = self.active_tile_ref() else { return false };
    let Some(tile) = self.tile_mut(&tile_ref) else { return false };
    
    let new_span = tile.row_span() + 1;
    
    // Check if the row below is available
    let target_row = tile_ref.row + new_span as i32 - 1;
    if self.is_occupied_by_another_tile(target_row, tile_ref.col, &tile_ref) {
        return false; // Can't expand, something in the way
    }
    
    tile.set_row_span(new_span);
    self.update_layout();
    true
}
```

### Config

```kdl
// Example keybinds
binds {
    Mod+Shift+Up { increase-row-span; }
    Mod+Shift+Down { decrease-row-span; }
}
```

---

## Step 2.5: Rendering Spanning Tiles

### Clipping / Visibility

A spanning tile should:
- Render across the rows it spans
- Be clipped to visible viewport
- Not double-render in each row

### Tasks

- [ ] **2.5.1**: Render spanning tiles from their origin row only
- [ ] **2.5.2**: Skip occupied positions when iterating rows
- [ ] **2.5.3**: Test visual appearance of spanning tiles

---

## Checklist Summary

### Step 2.1: Tile Row Span
- [ ] Add `row_span` field
- [ ] Getter and setter
- [ ] Default to 1

### Step 2.2: Layout Calculation
- [ ] Tile height = span × row_height
- [ ] Column handles variable heights

### Step 2.3: Cross-Row Coordination
- [ ] `is_occupied_by_span()`
- [ ] `effective_tile_at()`
- [ ] Update add/remove logic
- [ ] Update navigation

### Step 2.4: Commands
- [ ] `increase-row-span` action
- [ ] `decrease-row-span` action
- [ ] Validation
- [ ] Keybinds

### Step 2.5: Rendering
- [ ] Render from origin row
- [ ] Skip occupied in iteration
- [ ] Visual test

---

## Estimated Time: 1 Week

---

## Success Criteria

- [ ] Can set a tile to span 2 rows
- [ ] Spanning tile occupies space in both rows
- [ ] Can't place window in occupied position
- [ ] Navigation skips/handles occupied positions
- [ ] Spanning tile renders correctly

---

## Next Phase
→ [Phase 3: Camera System](phase-3-camera.md)
