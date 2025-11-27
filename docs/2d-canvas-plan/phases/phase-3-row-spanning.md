# Phase 3: Row Spanning

> **Status**: ⏳ PENDING (after Phase 2)
> **Goal**: Windows can span multiple rows, layout adjusts accordingly

---

## Overview

Based on USER clarifications:
- Windows can span across multiple rows (completely new feature)
- The 0,0 point (top-left corner) determines which row a window belongs to
- This enables "important" windows to take up more vertical space

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

The "Important App" is logically in Row 0 (its 0,0 point), but extends into Row 1's space.

---

## Step 3.1: Add Row Span to Tile

### Tasks

- [ ] **3.1.1**: Add `row_span: u8` field to `Tile`
- [ ] **3.1.2**: Default to `1`
- [ ] **3.1.3**: Add getter `Tile::row_span()`
- [ ] **3.1.4**: Add setter `Tile::set_row_span(span: u8)`
- [ ] **3.1.5**: Validate span (minimum 1, maximum configurable)

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
    pub fn row_span(&self) -> u8 {
        self.row_span
    }
    
    pub fn set_row_span(&mut self, span: u8) {
        self.row_span = span.max(1); // Minimum 1
    }
}
```

---

## Step 3.2: Layout Calculation

### Tasks

- [ ] **3.2.1**: Pass `row_height` to layout computation
- [ ] **3.2.2**: Compute tile height as `row_span * row_height`
- [ ] **3.2.3**: Adjust column layout to handle variable tile heights
- [ ] **3.2.4**: Test with mixed row spans in a column

---

## Step 3.3: Cross-Row Coordination

### The Occupancy Problem

When a tile in Row 0 spans into Row 1:
- That tile's column position in Row 1 is "occupied"
- Can't place windows there
- Navigation must understand this

### Tasks

- [ ] **3.3.1**: Create `canvas/spanning.rs`
- [ ] **3.3.2**: Implement `is_position_occupied(row, col) -> bool`
- [ ] **3.3.3**: Implement `tile_at(row, col) -> Option<TileRef>` with span awareness
- [ ] **3.3.4**: Update `add_window()` to respect occupied positions
- [ ] **3.3.5**: Update navigation to skip occupied positions

---

## Step 3.4: Row Span Commands

### New Actions

| Action | Effect |
|--------|--------|
| `increase-row-span` | Current tile spans one more row |
| `decrease-row-span` | Current tile spans one fewer row |
| `set-row-span N` | Set tile's row span to N |

### Tasks

- [ ] **3.4.1**: Add actions to `niri-config/src/actions.rs`
- [ ] **3.4.2**: Implement handlers in layout
- [ ] **3.4.3**: Validate: can't increase if would overlap another tile
- [ ] **3.4.4**: Add keybind suggestions to default config

---

## Step 3.5: Rendering Spanning Tiles

### Tasks

- [ ] **3.5.1**: Render spanning tiles from their origin row only
- [ ] **3.5.2**: Skip occupied positions when iterating rows
- [ ] **3.5.3**: Test visual appearance of spanning tiles

---

## Success Criteria

- [ ] Can set a tile to span 2+ rows
- [ ] Spanning tile occupies space in all spanned rows
- [ ] Can't place window in occupied position
- [ ] Navigation skips/handles occupied positions
- [ ] Spanning tile renders correctly (once, not duplicated)

---

*Phase 3 - Row Spanning*
