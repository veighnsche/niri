# TEAM_003: Golden Snapshot Strategy

## Context
User wants to ensure behavioral correctness during the refactor by creating "golden" snapshots that capture the layout state from the original main branch code.

## User's Requirements
- Capture window/tile positions (x, y, corners)
- Active column and tile indices
- Camera/view offset position
- Working area
- Compare refactored code against original behavior

## Current Testing Infrastructure

The existing tests already have:
- `verify_invariants()` on Layout, ScrollingSpace, Column, Tile, FloatingSpace, Workspace, Monitor
- Property-based tests with proptest
- `TestWindow` mock implementation

But these verify **constraints** (invariants), not **exact positions**.

## Proposed Solution: Golden Snapshot Tests

### Step 1: Create Snapshot Types

```rust
// src/layout/snapshot.rs

use serde::{Deserialize, Serialize};

/// Complete snapshot of layout state for golden testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    pub scrolling: ScrollingSnapshot,
    pub active_column_idx: usize,
    pub view_offset: f64,
    pub working_area: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollingSnapshot {
    pub columns: Vec<ColumnSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSnapshot {
    /// Visual X position of column left edge
    pub x: f64,
    /// Visual width of column
    pub width: f64,
    /// Tiles from top to bottom
    pub tiles: Vec<TileSnapshot>,
    /// Active tile index
    pub active_tile_idx: usize,
    /// Is full width
    pub is_full_width: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileSnapshot {
    /// Visual bounds (x, y, width, height)
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
```

### Step 2: Add snapshot() Methods

Add to ScrollingSpace, Column, etc.:

```rust
impl<W: LayoutElement> ScrollingSpace<W> {
    #[cfg(test)]
    pub fn snapshot(&self) -> ScrollingSnapshot {
        ScrollingSnapshot {
            columns: self.columns.iter().enumerate().map(|(i, col)| {
                let x = self.column_x(i);
                col.snapshot(x)
            }).collect(),
        }
    }
}

impl<W: LayoutElement> Column<W> {
    #[cfg(test)]
    pub fn snapshot(&self, x: f64) -> ColumnSnapshot {
        ColumnSnapshot {
            x,
            width: self.width(),
            tiles: self.tiles_with_offsets()
                .map(|(tile, offset)| TileSnapshot {
                    x: x + offset.x,
                    y: offset.y,
                    width: tile.tile_width(),
                    height: tile.tile_height(),
                })
                .collect(),
            active_tile_idx: self.active_tile_idx,
            is_full_width: self.is_full_width,
        }
    }
}
```

### Step 3: Generate Golden Data from Main Branch

```bash
# On main branch, run test that outputs JSON snapshots
cargo test --package niri generate_golden_snapshots -- --nocapture > golden.json
```

### Step 4: Snapshot Comparison Tests

```rust
#[test]
fn golden_single_column_positions() {
    let ops = [
        Op::AddOutput(1),
        Op::AddWindow { params: TestWindowParams::new(0) },
    ];
    
    let layout = check_ops(ops);
    let snapshot = layout.snapshot();
    
    // Compare against golden
    let expected = LayoutSnapshot {
        scrolling: ScrollingSnapshot {
            columns: vec![ColumnSnapshot {
                x: 8.0,  // gap
                width: 1264.0,  // 1280 - 2*8
                tiles: vec![TileSnapshot {
                    x: 8.0,
                    y: 8.0,
                    width: 1264.0,
                    height: 704.0,  // 720 - 2*8
                }],
                active_tile_idx: 0,
                is_full_width: false,
            }],
        },
        active_column_idx: 0,
        view_offset: 0.0,
        working_area: Rect { x: 0.0, y: 0.0, w: 1280.0, h: 720.0 },
    };
    
    assert_eq!(snapshot, expected);
}
```

## Implementation Plan

1. **Create `src/layout/snapshot.rs`** with snapshot types
2. **Add `snapshot()` methods** to ScrollingSpace, Column, Tile
3. **Create golden test file** with key scenarios:
   - Single window
   - Two columns
   - Column with multiple tiles
   - Full-width column
   - After focus changes
   - After resize
4. **Generate expected values** from current main branch behavior
5. **Run snapshot tests** after each refactor step

## Key Scenarios to Capture

| Scenario | What to verify |
|----------|---------------|
| Single window | Column X, width, tile position |
| Two columns side by side | Gap between columns, equal widths |
| Three tiles in column | Vertical gaps, heights |
| Focus left/right | View offset changes correctly |
| Full-width toggle | Column expands to fill |
| Window resize | Tile size changes, others adjust |
| Fullscreen | View offset, column position |

## Precision

Use `approx` crate for floating point comparison:
```rust
assert_abs_diff_eq!(snapshot.view_offset, expected.view_offset, epsilon = 0.001);
```

## Questions for USER

1. **Should I implement this now?** It would add ~500 lines but provide strong regression protection.
Let the next team focus on this

2. **Which scenarios are most critical?** I can start with a minimal set.
Well for example
0) empty workspace
1) add 1/2 app, snapshot
2) add another 1/2 app, snapshot
3) add a third 1/2 app, snapshot
4) make the second app active, (mod+left), snapshot
5) resize the second app (active one) to 2/3, (by pressing the function that is associated to mod+r), snapshot
6) make the first app active, (mod+left), snapshot
7) resize the first app (active one) to 2/3, (mod+r), snapshot
8) resize the first app (active one) again so that it turns 1/3, (mod+r), snapshot
9) select the second app (mod+right), snapshot
10) select the third app (mod+right), snapshot
11) resize the third app (active one) to 2/3, (mod+r), snapshot
12) resize the third app (active one) again so that it turns 1/3, (mod+r), snapshot
14) make the second app active (mod+left), snapshot
15) open a new 1/2 app, snapshot (should be added to the right of the second app)
16) center the active app, (mod+c), snapshot
17) make the second app active (mod+left), snapshot
18) make the first app active (mod+left), snapshot
19) center the first app (active one), (mod+c), snapshot
20) resize the first app (active one), (mod+r), snapshot
21) make the second app active (mod+right), snapshot
22) make the second app full-width (mod+f), snapshot
23) make the fourth app active (mod+right), snapshot (remember that the order is 1243)
24) make this active app full-width (mod+f), snapshot
25) resize to a preset column width (mod+r), snapshot
26) now press (mod+[) which will put the window to the left making 2 tiles in one column, snapshot
27) now make the first app active (mod+left), snapshot
28) now press (mod+]) which will put the window to the right making 3 tiles in one column, snapshot
29) now make the second app active (mod+right), snapshot
30) now press (mod+up) which will move the window up, snapshot
31) press (mod+r) to resize the entire column, snapshot
32) press (mod+shift+f) to fullscreen the active app, snapshot
33) press (mod+shift+f) to turn off fullscreen the active app, snapshot
34) We need to look at `/home/vince/Projects/niri/resources/default-config.kdl` to see all the other movement functions that we need to make snapshots off.

3. **JSON files or inline expected values?** JSON is more readable but inline is self-contained.

**Answer**: YAML format (via `insta`) — it's parsable for LTR→RTL mirroring.

For RTL support, we need to:
- Parse the snapshot YAML
- Negate all X values (mirror around origin 0)
- Compare against RTL golden snapshots

This means snapshot format must have explicit `x` fields, not computed values.

---

## Status: RESOLVED

All questions answered. Comprehensive test scenarios created.

See **[golden-test-scenarios.md](../phases/golden-test-scenarios.md)** for the complete list (~150 snapshots across 23 test groups A-W covering all layout-affecting operations).