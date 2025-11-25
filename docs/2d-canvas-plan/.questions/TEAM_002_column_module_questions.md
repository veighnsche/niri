# TEAM_002: Questions After Column Extraction

## Status: RESOLVED

## Context
Completed Step 0.1 (Column extraction). These questions arose during implementation.

---

## Question 1: Re-export Strategy ✅ RESOLVED

**USER Decision**: Option B — Update all import sites to use `column::` directly

**Action**: Add to Step 0.3 tasks — update all imports from `scrolling::{Column, ColumnWidth}` to `column::{Column, ColumnWidth}`

---

## Question 2: Field Visibility ✅ RESOLVED

**USER Decision**: Option B — During Phase 1 when building Row/Canvas2D

**Rationale**: We'll need to design the proper API when building Row anyway. No action needed now.

---

## Question 3: TileData & Shared Types ✅ RESOLVED

### USER Feedback
> "In the plans... a tile is a window basically right??? but tiles can extend to multiple rows... meaning that it should be shared type no??? can we plan this beforehand better?"

### Analysis

Looking at the 2D Canvas vision:
- **Tile** = a window wrapper (already in `tile.rs`)
- **TileData** = cached per-tile metadata (height, size, resize state)
- **Row spanning** = tiles can span 1-2 rows (new `row_span: u8` field on Tile)

**Key insight**: In 2D Canvas, the hierarchy is:
```
Canvas2D
└── Row[]
    └── Column[]
        └── Tile[] (with row_span)
```

A tile that spans 2 rows is still **owned by one Column** in one Row, but its height extends into the row below. The tile doesn't move between rows — it just takes up more vertical space.

### Decision: Shared Types Module

Create `src/layout/types.rs` for types used across multiple modules:

```rust
// src/layout/types.rs

/// Cached per-tile data within a Column
pub struct TileData {
    pub height: WindowHeight,
    pub size: Size<f64, Logical>,
    pub interactively_resizing_by_left_edge: bool,
}

/// How a window's height is determined
pub enum WindowHeight {
    Auto { weight: f64 },
    Fixed(f64),
    Preset(usize),
}

/// How a column's width is determined  
pub enum ColumnWidth {
    Proportion(f64),
    Fixed(f64),
}

/// Direction for horizontal operations
pub enum ScrollDirection {
    Left,
    Right,
}
```

**Action**: Add to Phase 0 plan — create `layout/types.rs` and move shared types there.

---

## Question 4: Test Organization ✅ RESOLVED

### USER Feedback
> "Yeah let's make a serious consistency plan for tests first... please write down what is best for our future structure."

### Test Strategy for 2D Canvas

#### Current State
- `src/layout/tests.rs` — 3873 lines of integration tests
- `src/layout/column/tests.rs` — just `verify_invariants` (test helper)
- Tests use `TestWindow` mock and `Op` enum for operations

#### Recommended Structure

```
src/layout/
├── tests.rs                    # Integration tests (keep as-is)
│                               # Tests full Layout behavior
│
├── column/
│   ├── tests.rs               # Column unit tests
│   │   - verify_invariants()  # Already exists
│   │   - test_add_tile()      # NEW: isolated column ops
│   │   - test_focus_movement()
│   │   - test_sizing()
│   │
├── types.rs                   # Shared types (no tests needed)
│
├── animated_value/            # Phase 0.2
│   └── tests.rs               # AnimatedValue unit tests
│       - test_static_value()
│       - test_animation()
│       - test_gesture()
│
├── row/                       # Phase 1 (future)
│   └── tests.rs               # Row unit tests
│
└── canvas/                    # Phase 1 (future)
    └── tests.rs               # Canvas2D unit tests
```

#### Testing Principles

1. **Unit tests per module**: Each module (`column/`, `row/`, `canvas/`) has its own `tests.rs` for isolated testing

2. **Integration tests stay central**: `layout/tests.rs` tests the full system with `TestWindow`

3. **`verify_invariants()` pattern**: Each struct has a `#[cfg(test)]` method that asserts internal consistency. Called after every operation in integration tests.

4. **Property-based tests**: Use `proptest` for random operation sequences (already in place)

5. **Test helpers in module**: `tests.rs` can define helpers specific to that module

#### Example: Column Unit Tests

```rust
// src/layout/column/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_column() -> Column<TestWindow> {
        // Create a column with test fixtures
    }
    
    #[test]
    fn add_tile_increases_count() {
        let mut col = test_column();
        let initial = col.tiles().len();
        col.add_tile_at(0, test_tile());
        assert_eq!(col.tiles().len(), initial + 1);
    }
    
    #[test]
    fn focus_wraps_at_boundaries() {
        let mut col = test_column_with_3_tiles();
        col.focus_down();
        col.focus_down();
        col.focus_down(); // Should not panic
        assert_eq!(col.active_tile_idx(), 2); // Stays at last
    }
}
```

**Action**: Add test strategy to Phase 0 plan. Unit tests can be added incrementally.

---

## Summary of Actions

| Question | Decision | Action |
|----------|----------|--------|
| Q1: Re-exports | Option B | Add to Step 0.3: update all imports |
| Q2: Field visibility | Option B | Defer to Phase 1 |
| Q3: Shared types | Create `types.rs` | Add Step 0.3.x: create shared types module |
| Q4: Tests | Unit + Integration | Document test strategy in Phase 0 |
