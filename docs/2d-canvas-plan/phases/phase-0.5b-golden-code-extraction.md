# Phase 0.5.B: Golden Code Extraction

> **Goal**: Extract original main branch code and add snapshot methods.
> **Estimated Time**: 4-6 hours
> **Dependency**: Phase 0.5.A complete

---

## Tasks

### 1. Extract Main Branch Files

```bash
cd /home/vince/Projects/niri

# Extract original files
git show main:src/layout/scrolling.rs > src/layout/golden/scrolling_original.rs
git show main:src/layout/workspace.rs > src/layout/golden/workspace_original.rs
git show main:src/layout/floating.rs > src/layout/golden/floating_original.rs
```

---

### 2. Create Minimal Golden ScrollingSpace

Create `src/layout/golden/scrolling.rs` with:

1. **Rename** `ScrollingSpace` → `GoldenScrollingSpace`
2. **Rename** `Column` → `GoldenColumn` (if embedded)
3. **Keep only** fields needed for snapshots:
   - `columns`, `active_column_idx`, `view_offset`
   - `working_area`, `view_size`
4. **Remove** render methods, animations, gestures
5. **Add** `snapshot()` method

```rust
// src/layout/golden/scrolling.rs

use crate::layout::snapshot::{ScrollingSnapshot, ColumnSnapshot, ...};

impl GoldenScrollingSpace {
    #[cfg(test)]
    pub fn snapshot(&self) -> ScrollingSnapshot {
        ScrollingSnapshot {
            columns: self.columns.iter().enumerate()
                .map(|(i, col)| col.snapshot(self.column_x(i)))
                .collect(),
            active_column_idx: self.active_column_idx,
            view_offset: self.view_offset.current(),
            working_area: RectSnapshot::from(self.working_area),
            view_size: SizeSnapshot::from(self.view_size),
        }
    }
}
```

---

### 3. Add snapshot() to Refactored Code

Add to `src/layout/scrolling.rs`:

```rust
#[cfg(test)]
impl<W: LayoutElement> ScrollingSpace<W> {
    pub fn snapshot(&self) -> crate::layout::snapshot::ScrollingSnapshot {
        use crate::layout::snapshot::*;
        
        ScrollingSnapshot {
            columns: self.columns.iter().enumerate()
                .map(|(i, col)| col.snapshot(self.column_x(i)))
                .collect(),
            active_column_idx: self.active_column_idx,
            view_offset: self.view_offset.current(),
            working_area: RectSnapshot::from(self.working_area),
            view_size: SizeSnapshot::from(self.view_size),
        }
    }
}
```

Add to `src/layout/column/mod.rs`:

```rust
#[cfg(test)]
impl<W: LayoutElement> Column<W> {
    pub fn snapshot(&self, x: f64) -> crate::layout::snapshot::ColumnSnapshot {
        use crate::layout::snapshot::*;
        
        ColumnSnapshot {
            x,
            width: self.width(),
            tiles: self.tiles_with_offsets()
                .map(|(tile, offset)| TileSnapshot {
                    x: x + offset.x,
                    y: offset.y,
                    width: tile.tile_width(),
                    height: tile.tile_height(),
                    window_id: tile.window().id(),  // Adjust based on actual API
                })
                .collect(),
            active_tile_idx: self.active_tile_idx,
            is_full_width: self.is_full_width,
            is_fullscreen: self.is_fullscreen(),
        }
    }
}
```

---

### 4. Verify Identical Output

Create a simple test to verify both produce the same snapshot:

```rust
// src/layout/tests/golden.rs

#[test]
fn golden_and_refactored_match_empty() {
    // Create identical initial states
    let golden = GoldenScrollingSpace::new(...);
    let refactored = ScrollingSpace::new(...);
    
    assert_eq!(golden.snapshot(), refactored.snapshot());
}
```

---

## Success Criteria

- [ ] `src/layout/golden/scrolling.rs` compiles with `GoldenScrollingSpace`
- [ ] `GoldenScrollingSpace::snapshot()` returns `ScrollingSnapshot`
- [ ] `ScrollingSpace::snapshot()` returns `ScrollingSnapshot`
- [ ] `Column::snapshot()` returns `ColumnSnapshot`
- [ ] Basic test shows both produce identical snapshots

---

## Notes

- Golden code only needs to compile in `#[cfg(test)]` context
- Strip out all render, animation, and unused methods
- Focus on layout state: positions, sizes, indices

---

## Handoff

After completing this phase:
- Next: **Phase 0.5.C** (Core golden tests A-L)
