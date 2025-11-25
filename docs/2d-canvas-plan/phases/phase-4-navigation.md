# Phase 4: Navigation + Polish

> **Goal**: Geometric navigation, origin-based leading edges, spawn direction, animation refinement.

---

## Prerequisites

- Phase 3 complete (camera with zoom works)

---

## Step 4.1: Geometric Navigation

### Current (Phase 1) Behavior
"Down" moves to same column index in row below.

### Target Behavior
"Down" finds the window whose top edge is closest to the current window's bottom edge center.

```
┌─────────────┐
│   Current   │
│   Window    │
└──────●──────┘  ← probe point (bottom edge center)
       ↓
       ↓ find nearest below
       ↓
    ┌──●────────────┐
    │  Nearest Tile │
    └───────────────┘
```

### File Structure

```
src/layout/canvas/
├── navigation.rs       # Refactored for geometric
```

### Tasks

- [ ] **4.1.1**: Define `Direction` enum (Up, Down, Left, Right)
- [ ] **4.1.2**: Implement `probe_point(tile_rect, direction) -> Point`
- [ ] **4.1.3**: Implement `tiles_in_direction(probe, direction) -> Iterator`
- [ ] **4.1.4**: Implement `distance_to_tile(probe, tile, direction) -> f64`
- [ ] **4.1.5**: Implement `find_nearest_in_direction(direction) -> Option<TileRef>`
- [ ] **4.1.6**: Wire up to focus_up/down/left/right
- [ ] **4.1.7**: Test edge cases (no tile in direction, spanning tiles)

### Implementation

```rust
// src/layout/canvas/navigation.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns true if this direction is horizontal.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::Left | Direction::Right)
    }
}

impl<W: LayoutElement> Canvas2D<W> {
    /// Get the probe point for navigation from a tile rect.
    fn probe_point(&self, rect: Rectangle<f64, Logical>, dir: Direction) -> Point<f64, Logical> {
        match dir {
            Direction::Up => Point::from((
                rect.loc.x + rect.size.w / 2.0,  // center X
                rect.loc.y,                       // top edge
            )),
            Direction::Down => Point::from((
                rect.loc.x + rect.size.w / 2.0,  // center X
                rect.loc.y + rect.size.h,        // bottom edge
            )),
            Direction::Left => Point::from((
                rect.loc.x,                       // left edge
                rect.loc.y + rect.size.h / 2.0,  // center Y
            )),
            Direction::Right => Point::from((
                rect.loc.x + rect.size.w,        // right edge
                rect.loc.y + rect.size.h / 2.0,  // center Y
            )),
        }
    }
    
    /// Check if a tile is in the given direction from the probe point.
    fn is_in_direction(
        &self,
        probe: Point<f64, Logical>,
        tile_rect: Rectangle<f64, Logical>,
        dir: Direction,
    ) -> bool {
        match dir {
            Direction::Up => tile_rect.loc.y + tile_rect.size.h <= probe.y,
            Direction::Down => tile_rect.loc.y >= probe.y,
            Direction::Left => tile_rect.loc.x + tile_rect.size.w <= probe.x,
            Direction::Right => tile_rect.loc.x >= probe.x,
        }
    }
    
    /// Compute distance from probe to tile (for ranking candidates).
    fn distance_to_tile(
        &self,
        probe: Point<f64, Logical>,
        tile_rect: Rectangle<f64, Logical>,
        dir: Direction,
    ) -> f64 {
        // Primary: perpendicular distance (how far in the direction)
        // Secondary: lateral distance (how far off-center)
        
        let (primary, secondary) = match dir {
            Direction::Up => {
                let edge_y = tile_rect.loc.y + tile_rect.size.h;
                let center_x = tile_rect.loc.x + tile_rect.size.w / 2.0;
                (probe.y - edge_y, (probe.x - center_x).abs())
            }
            Direction::Down => {
                let edge_y = tile_rect.loc.y;
                let center_x = tile_rect.loc.x + tile_rect.size.w / 2.0;
                (edge_y - probe.y, (probe.x - center_x).abs())
            }
            Direction::Left => {
                let edge_x = tile_rect.loc.x + tile_rect.size.w;
                let center_y = tile_rect.loc.y + tile_rect.size.h / 2.0;
                (probe.x - edge_x, (probe.y - center_y).abs())
            }
            Direction::Right => {
                let edge_x = tile_rect.loc.x;
                let center_y = tile_rect.loc.y + tile_rect.size.h / 2.0;
                (edge_x - probe.x, (probe.y - center_y).abs())
            }
        };
        
        // Combine: primary matters more, secondary breaks ties
        primary + secondary * 0.01
    }
    
    /// Find the nearest tile in a direction.
    pub fn find_nearest_in_direction(&self, dir: Direction) -> Option<TileRef> {
        let current = self.active_tile_ref()?;
        let current_rect = self.tile_rect_for_ref(&current);
        let probe = self.probe_point(current_rect, dir);
        
        let mut best: Option<(TileRef, f64)> = None;
        
        for (row_idx, row) in &self.rows {
            for (col_idx, column) in row.columns().iter().enumerate() {
                let tile_ref = TileRef { row: *row_idx, col: col_idx };
                if tile_ref == current {
                    continue;
                }
                
                let rect = self.tile_rect_for_ref(&tile_ref);
                if !self.is_in_direction(probe, rect, dir) {
                    continue;
                }
                
                let dist = self.distance_to_tile(probe, rect, dir);
                if best.map_or(true, |(_, d)| dist < d) {
                    best = Some((tile_ref, dist));
                }
            }
        }
        
        best.map(|(r, _)| r)
    }
    
    pub fn focus_direction(&mut self, dir: Direction) -> bool {
        if let Some(tile_ref) = self.find_nearest_in_direction(dir) {
            self.focus_tile_ref(tile_ref);
            true
        } else {
            false
        }
    }
}
```

---

## Step 4.2: Origin-Based Leading Edge

### Recap

The origin (0,0) determines which edge is "leading" for animations:

| Position | Horizontal Leading | Vertical Leading |
|----------|-------------------|------------------|
| Right of origin | Right edge | — |
| Left of origin | Left edge | — |
| Above origin | — | Top edge |
| Below origin | — | Bottom edge |

### Tasks

- [ ] **4.2.1**: Add `Canvas2D::quadrant_of(tile) -> Quadrant`
- [ ] **4.2.2**: Define `Quadrant { h: HorizontalSide, v: VerticalSide }`
- [ ] **4.2.3**: Use quadrant in resize animations
- [ ] **4.2.4**: Use quadrant in window movement animations
- [ ] **4.2.5**: Test: resize animation pins correct edge

### Implementation

```rust
// src/layout/canvas/mod.rs

#[derive(Debug, Clone, Copy)]
pub enum HorizontalSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalSide {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Quadrant {
    pub horizontal: HorizontalSide,
    pub vertical: VerticalSide,
}

impl<W: LayoutElement> Canvas2D<W> {
    /// Determine which quadrant a tile is in relative to origin.
    pub fn quadrant_of(&self, tile_ref: &TileRef) -> Quadrant {
        let rect = self.tile_rect_for_ref(tile_ref);
        let center = Point::from((
            rect.loc.x + rect.size.w / 2.0,
            rect.loc.y + rect.size.h / 2.0,
        ));
        
        Quadrant {
            horizontal: if center.x >= 0.0 {
                HorizontalSide::Right  // LTR behavior
            } else {
                HorizontalSide::Left   // RTL behavior
            },
            vertical: if center.y >= 0.0 {
                VerticalSide::Bottom   // Leading is bottom
            } else {
                VerticalSide::Top      // Leading is top
            },
        }
    }
    
    /// Get the leading horizontal edge for resize animation.
    pub fn leading_horizontal_edge(&self, tile_ref: &TileRef) -> HorizontalSide {
        self.quadrant_of(tile_ref).horizontal
    }
    
    /// Get the leading vertical edge for resize animation.
    pub fn leading_vertical_edge(&self, tile_ref: &TileRef) -> VerticalSide {
        self.quadrant_of(tile_ref).vertical
    }
}
```

---

## Step 4.3: Spawn Direction

### Logic

When spawning a new window "next to" focused:
- Right of origin → spawn to the right
- Left of origin → spawn to the left
- Below origin → spawn below (in same column)
- Above origin → spawn above (in same column)

### Tasks

- [ ] **4.3.1**: Modify `add_window()` to consider quadrant
- [ ] **4.3.2**: Implement horizontal spawn direction (left/right of current column)
- [ ] **4.3.3**: Implement vertical spawn direction (new row above/below)
- [ ] **4.3.4**: Handle edge case: spawning at origin
- [ ] **4.3.5**: Test spawn direction in each quadrant

### Implementation

```rust
impl<W: LayoutElement> Canvas2D<W> {
    pub fn add_window(&mut self, window: W, target: AddWindowTarget) -> TileRef {
        match target {
            AddWindowTarget::NextToFocused => {
                let focused = self.active_tile_ref();
                if let Some(focused) = focused {
                    let quadrant = self.quadrant_of(&focused);
                    self.add_window_next_to(window, &focused, quadrant)
                } else {
                    self.add_window_at_origin(window)
                }
            }
            // ... other targets
        }
    }
    
    fn add_window_next_to(
        &mut self,
        window: W,
        focused: &TileRef,
        quadrant: Quadrant,
    ) -> TileRef {
        let focused_tile = self.tile(&focused).unwrap();
        let row_span = focused_tile.row_span();
        
        // Horizontal direction: add new column left or right
        let col_offset = match quadrant.horizontal {
            HorizontalSide::Right => 1,  // Add to the right
            HorizontalSide::Left => -1,  // Add to the left (insert before)
        };
        
        let row = self.rows.get_mut(&focused.row).unwrap();
        let new_col_idx = if col_offset > 0 {
            focused.col + 1
        } else {
            focused.col  // Insert before, current moves right
        };
        
        // Create tile with same row span as focused
        let tile = Tile::new(window).with_row_span(row_span);
        row.insert_column(new_col_idx, Column::new_with_tile(tile));
        
        TileRef { row: focused.row, col: new_col_idx }
    }
}
```

---

## Step 4.4: Animation Refinement

### Tasks

- [ ] **4.4.1**: Ensure all animations respect leading edge
- [ ] **4.4.2**: Add `vertical_view_movement` config (if not reusing camera_movement)
- [ ] **4.4.3**: Fine-tune animation curves for zoom
- [ ] **4.4.4**: Test animations in all quadrants
- [ ] **4.4.5**: Add option to disable 2D canvas animations

### Animation Review

| Animation | Phase 1 | Phase 4 |
|-----------|---------|---------|
| Horizontal view movement | Uses existing | Verify leading edge |
| Vertical view movement | New | Add config, verify |
| Zoom | New in Phase 3 | Tune curve |
| Resize | Existing | Verify quadrant-based edge |
| Window spawn | Existing | Verify direction |

---

## Checklist Summary

### Step 4.1: Geometric Navigation
- [ ] Direction enum
- [ ] probe_point()
- [ ] is_in_direction()
- [ ] distance_to_tile()
- [ ] find_nearest_in_direction()
- [ ] Wire to focus actions
- [ ] Tests

### Step 4.2: Origin-Based Edges
- [ ] Quadrant struct
- [ ] quadrant_of()
- [ ] Use in resize
- [ ] Use in movement
- [ ] Tests

### Step 4.3: Spawn Direction
- [ ] Modify add_window()
- [ ] Horizontal spawn
- [ ] Vertical spawn (new row)
- [ ] Origin edge case
- [ ] Tests

### Step 4.4: Animation Refinement
- [ ] Leading edge in all animations
- [ ] vertical_view_movement config
- [ ] Tune zoom curve
- [ ] Test all quadrants
- [ ] Disable option

---

## Estimated Time: 1 Week

---

## Success Criteria

- [ ] "Down" finds geometrically nearest window below
- [ ] Resize pins correct edge based on quadrant
- [ ] New windows spawn in correct direction
- [ ] Animations feel smooth and consistent
- [ ] Works correctly in all four quadrants

---

## Next Phase
→ [Phase 5: Integration](phase-5-integration.md)
