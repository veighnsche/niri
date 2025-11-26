# Porting Snapshot Infrastructure to Main Branch

> **Purpose**: Step-by-step guide to add snapshot infrastructure to main branch.
> **Created by**: TEAM_011

---

## Overview

The snapshot infrastructure captures layout state for golden testing.
Main branch doesn't have this — we need to add it to generate true golden snapshots.

---

## Files to Copy

```
src/layout/snapshot/
├── mod.rs          # Module root
├── types.rs        # Snapshot types (ScrollingSnapshot, etc.)
└── PORTING.md      # This file (optional)
```

---

## Step 1: Copy Snapshot Module

```bash
# From 2d-canvas branch
cp -r src/layout/snapshot/ /tmp/snapshot-port/

# Switch to main
git checkout main

# Copy to main
cp -r /tmp/snapshot-port/ src/layout/snapshot/
```

---

## Step 2: Update src/layout/mod.rs

Add near the top with other module declarations:

```rust
// Golden snapshot infrastructure
#[cfg(test)]
pub mod snapshot;
```

---

## Step 3: Update Cargo.toml

Ensure insta has yaml feature:

```toml
[dev-dependencies]
insta = { version = "1.42.2", features = ["yaml"] }
```

---

## Step 4: Add Accessor Methods to Animation

In `src/animation/mod.rs`, add these methods to `impl Animation`:

```rust
/// Returns the easing curve name if this is an easing animation.
#[cfg(test)]
pub fn easing_curve_name(&self) -> Option<&'static str> {
    match &self.kind {
        AnimationKind::Easing { curve, .. } => Some(match curve {
            Curve::Linear => "Linear",
            Curve::EaseOutQuad => "EaseOutQuad",
            Curve::EaseOutCubic => "EaseOutCubic",
            Curve::EaseOutExpo => "EaseOutExpo",
            // Add other curves as needed
        }),
        _ => None,
    }
}

/// Returns spring parameters if this is a spring animation.
#[cfg(test)]
pub fn spring_params(&self) -> Option<&SpringParams> {
    match &self.kind {
        AnimationKind::Spring(params) => Some(params),
        _ => None,
    }
}

/// Returns (initial_velocity, deceleration_rate) if this is a deceleration animation.
#[cfg(test)]
pub fn deceleration_params(&self) -> Option<(f64, f64)> {
    match &self.kind {
        AnimationKind::Deceleration { initial_velocity, deceleration_rate, .. } => {
            Some((*initial_velocity, *deceleration_rate))
        }
        _ => None,
    }
}
```

---

## Step 5: Add Accessor Methods to Tile

In `src/layout/tile.rs`, add these methods to `impl<W: LayoutElement> Tile<W>`:

```rust
/// Returns resize animation from sizes for golden testing.
#[cfg(test)]
pub fn resize_animation_from_sizes(&self) -> Option<(Size<f64, Logical>, Size<f64, Logical>)> {
    self.resize_animation.as_ref().map(|r| (r.size_from, r.tile_size_from))
}

/// Returns move_x animation with from value for golden testing.
#[cfg(test)]
pub fn move_x_animation_with_from(&self) -> Option<(&Animation, f64)> {
    self.move_x_animation.as_ref().map(|m| (&m.anim, m.from))
}

/// Returns move_y animation with from value for golden testing.
#[cfg(test)]
pub fn move_y_animation_with_from(&self) -> Option<(&Animation, f64)> {
    self.move_y_animation.as_ref().map(|m| (&m.anim, m.from))
}
```

---

## Step 6: Add Accessor Methods to Column

In `src/layout/scrolling.rs`, add a `#[cfg(test)]` impl block for Column:

```rust
#[cfg(test)]
impl<W: LayoutElement> Column<W> {
    /// Create a snapshot of this column for golden testing.
    pub fn snapshot(&self, column_x: f64) -> crate::layout::snapshot::ColumnSnapshot {
        use crate::layout::snapshot::{ColumnSnapshot, TileSnapshot};

        let tiles: Vec<TileSnapshot> = self
            .tiles()
            .enumerate()
            .map(|(idx, tile)| {
                let offset = self.tile_offset(idx);
                let tile_size = tile.tile_size();
                TileSnapshot {
                    x: column_x + offset.x,
                    y: offset.y,
                    width: tile_size.w,
                    height: tile_size.h,
                }
            })
            .collect();

        ColumnSnapshot {
            x: column_x,
            width: self.width(),
            tiles,
            active_tile_idx: self.active_tile_idx,
            is_full_width: self.is_full_width,
            is_fullscreen: self.sizing_mode().is_fullscreen(),
        }
    }

    /// Returns the column's move animation if present (animation, from_offset).
    pub fn move_animation(&self) -> Option<(&crate::animation::Animation, f64)> {
        self.move_animation.as_ref().map(|m| (&m.anim, m.from))
    }

    /// Returns iterator over tiles with their indices for golden testing.
    pub fn tiles_with_animations(&self) -> impl Iterator<Item = (&Tile<W>, usize)> {
        self.tiles.iter().enumerate().map(|(idx, tile)| (tile, idx))
    }
}
```

**NOTE**: Main's `Column::tiles()` returns `Iterator<Item = &Tile>`, not `Iterator<Item = (&Tile, Point)>`.
Adjust the snapshot method to use `self.tile_offset(idx)` instead.

---

## Step 7: Add snapshot() to ScrollingSpace

In `src/layout/scrolling.rs`, add to `impl<W: LayoutElement> ScrollingSpace<W>`:

```rust
#[cfg(test)]
pub fn snapshot(&self) -> crate::layout::snapshot::ScrollingSnapshot {
    use crate::layout::snapshot::{
        AnimationTimelineSnapshot, RectSnapshot, ScrollingSnapshot, SizeSnapshot,
    };

    let columns = self
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let col_x = self.column_x(idx);
            col.snapshot(col_x)
        })
        .collect();

    // Capture view_offset animation
    let mut animations = Vec::new();
    
    // Main uses ViewOffset enum, not AnimatedValue
    if let ViewOffset::Animation(anim) = &self.view_offset {
        let kind = Self::extract_animation_kind(anim);
        animations.push(AnimationTimelineSnapshot::view_offset(
            anim.from(),
            anim.to(),
            kind,
            anim.duration().as_millis() as u64,
        ));
    }

    // Capture column and tile animations
    for (col_idx, column) in self.columns.iter().enumerate() {
        let col_x = self.column_x(col_idx);

        if let Some((anim, from_offset)) = column.move_animation() {
            let kind = Self::extract_animation_kind(anim);
            animations.push(AnimationTimelineSnapshot {
                target: format!("column_{col_idx}_move_x"),
                from: from_offset,
                to: 0.0,
                kind,
                duration_ms: anim.duration().as_millis() as u64,
                pinned_edge: None,
            });
        }

        for (tile, tile_idx) in column.tiles_with_animations() {
            let tile_offset = column.tile_offset(tile_idx);
            let tile_size = tile.tile_size();

            if let Some(anim) = tile.resize_animation() {
                if let Some((_, tile_size_from)) = tile.resize_animation_from_sizes() {
                    let kind = Self::extract_animation_kind(anim);

                    if (tile_size_from.w - tile_size.w).abs() > 0.1 {
                        animations.push(AnimationTimelineSnapshot::tile_edge(
                            col_idx, tile_idx, "x_max",
                            col_x + tile_offset.x + tile_size_from.w,
                            col_x + tile_offset.x + tile_size.w,
                            kind.clone(),
                            anim.duration().as_millis() as u64,
                        ));
                    }

                    if (tile_size_from.h - tile_size.h).abs() > 0.1 {
                        animations.push(AnimationTimelineSnapshot::tile_edge(
                            col_idx, tile_idx, "y_max",
                            tile_offset.y + tile_size_from.h,
                            tile_offset.y + tile_size.h,
                            kind,
                            anim.duration().as_millis() as u64,
                        ));
                    }
                }
            }

            if let Some((anim, from_x)) = tile.move_x_animation_with_from() {
                let kind = Self::extract_animation_kind(anim);
                let current_x = col_x + tile_offset.x;
                let from_abs_x = current_x + from_x;

                animations.push(AnimationTimelineSnapshot::tile_edge(
                    col_idx, tile_idx, "x_min",
                    from_abs_x, current_x,
                    kind.clone(),
                    anim.duration().as_millis() as u64,
                ));
                animations.push(AnimationTimelineSnapshot::tile_edge(
                    col_idx, tile_idx, "x_max",
                    from_abs_x + tile_size.w, current_x + tile_size.w,
                    kind,
                    anim.duration().as_millis() as u64,
                ));
            }

            if let Some((anim, from_y)) = tile.move_y_animation_with_from() {
                let kind = Self::extract_animation_kind(anim);
                let current_y = tile_offset.y;
                let from_abs_y = current_y + from_y;

                animations.push(AnimationTimelineSnapshot::tile_edge(
                    col_idx, tile_idx, "y_min",
                    from_abs_y, current_y,
                    kind.clone(),
                    anim.duration().as_millis() as u64,
                ));
                animations.push(AnimationTimelineSnapshot::tile_edge(
                    col_idx, tile_idx, "y_max",
                    from_abs_y + tile_size.h, current_y + tile_size.h,
                    kind,
                    anim.duration().as_millis() as u64,
                ));
            }
        }
    }

    // Get current view offset value
    let view_offset_value = match &self.view_offset {
        ViewOffset::Static(v) => *v,
        ViewOffset::Animation(anim) => anim.value(),
        ViewOffset::Gesture(gesture) => gesture.current_view_offset,
    };

    ScrollingSnapshot {
        columns,
        active_column_idx: self.active_column_idx,
        view_offset: view_offset_value,
        working_area: RectSnapshot::from(self.working_area),
        view_size: SizeSnapshot::from(self.view_size),
        animations,
    }
}

#[cfg(test)]
fn extract_animation_kind(anim: &Animation) -> crate::layout::snapshot::AnimationKindSnapshot {
    use crate::layout::snapshot::AnimationKindSnapshot;

    if let Some(curve_name) = anim.easing_curve_name() {
        return AnimationKindSnapshot::Easing {
            curve: curve_name.to_string(),
            duration_ms: anim.duration().as_millis() as u64,
        };
    }

    if let Some(params) = anim.spring_params() {
        let damping_ratio = params.damping / (2.0 * params.stiffness.sqrt());
        return AnimationKindSnapshot::Spring {
            damping_ratio: (damping_ratio * 100.0).round() / 100.0,
            stiffness: (params.stiffness * 10.0).round() / 10.0,
        };
    }

    if let Some((initial_velocity, deceleration_rate)) = anim.deceleration_params() {
        return AnimationKindSnapshot::Deceleration {
            initial_velocity: (initial_velocity * 10.0).round() / 10.0,
            deceleration_rate: (deceleration_rate * 1000.0).round() / 1000.0,
        };
    }

    AnimationKindSnapshot::Easing {
        curve: "Unknown".to_string(),
        duration_ms: anim.duration().as_millis() as u64,
    }
}
```

---

## Step 8: Copy Test File

Copy `src/layout/tests/golden.rs` to main and add `mod golden;` to `src/layout/tests.rs`.

---

## Step 9: Generate Golden Snapshots

```bash
# On main branch
cargo insta test --accept

# Lock snapshots
chmod 444 src/layout/tests/snapshots/*.snap

# Commit
git add src/layout/
git commit -m "Add golden snapshot infrastructure"
```

---

## Step 10: Port Back to 2d-canvas

```bash
# Copy snapshots to 2d-canvas
git checkout 2d-canvas
cp /path/to/main/src/layout/tests/snapshots/*.snap src/layout/tests/snapshots/

# Test
cargo insta test

# If failures → regressions found!
```

---

## Key Differences: Main vs 2d-canvas

| Component | Main | 2d-canvas |
|-----------|------|-----------|
| View offset | `ViewOffset` enum | `AnimatedValue` enum |
| Column location | `scrolling.rs` | `column/mod.rs` |
| `Column::tiles()` | `Iterator<Item = &Tile>` | `Iterator<Item = (&Tile, Point)>` |

The porting guide accounts for these differences.
