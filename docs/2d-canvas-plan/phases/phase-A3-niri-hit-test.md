# Phase A3: Extract niri/hit_test.rs

> **Status**: ⏳ PENDING
> **Estimated Time**: 45 minutes
> **Risk Level**: 🟢 Low (pure queries, no mutations)
> **Prerequisite**: Phase A2 complete

---

## Goal

Extract all hit testing and content-under methods from `niri.rs` into `src/niri/hit_test.rs`.

These are all read-only queries (`&self`) that determine what's under a given point.

---

## Work Units

### Unit 1: Create hit_test.rs with Imports (5 min)

Create `src/niri/hit_test.rs`:
```rust
//! Hit testing and content queries for the Niri compositor.
//!
//! All methods in this module are pure queries that don't mutate state.

use smithay::desktop::{layer_map_for_output, LayerSurface, WindowSurfaceType};
use smithay::output::Output;
use smithay::utils::{Logical, Point};
use smithay::wayland::shell::wlr_layer::Layer;

use crate::layout::monitor::HitType;
use crate::layout::row::Row;
use crate::niri::{Niri, PointContents};
use crate::window::Mapped;
```

Update `src/niri/mod.rs`:
```rust
mod hit_test;
mod output;
mod types;

pub use types::*;
```

---

### Unit 2: Extract is_inside_hot_corner (5 min)

**Source**: `niri.rs` ~lines 3277-3318

```rust
impl Niri {
    fn is_inside_hot_corner(&self, output: &Output, pos: Point<f64, Logical>) -> bool {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 3: Extract is_sticky_obscured_under (10 min)

**Source**: `niri.rs` ~lines 3320-3372

```rust
impl Niri {
    pub fn is_sticky_obscured_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> bool {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 4: Extract is_layout_obscured_under (10 min)

**Source**: `niri.rs` ~lines 3374-3417

```rust
impl Niri {
    pub fn is_layout_obscured_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> bool {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 5: Extract row_under Methods (10 min)

**Source**: `niri.rs` ~lines 3419-3454

```rust
impl Niri {
    /// Returns the row under the position to be activated.
    pub fn row_under(
        &self,
        extended_bounds: bool,
        pos: Point<f64, Logical>,
    ) -> Option<(Output, &Row<Mapped>)> {
        // ... (copy from niri.rs)
    }

    pub fn row_under_cursor(
        &self,
        extended_bounds: bool,
    ) -> Option<(Output, &Row<Mapped>)> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.row_under(extended_bounds, pos)
    }
}
```

---

### Unit 6: Extract window_under Methods (10 min)

**Source**: `niri.rs` ~lines 3456-3497

```rust
impl Niri {
    /// Returns the window under the position to be activated.
    pub fn window_under(&self, pos: Point<f64, Logical>) -> Option<&Mapped> {
        // ... (copy from niri.rs)
    }

    /// Returns the window under the cursor to be activated.
    pub fn window_under_cursor(&self) -> Option<&Mapped> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.window_under(pos)
    }
}
```

---

### Unit 7: Extract contents_under (15 min)

**Source**: `niri.rs` ~lines 3499-3681

This is the largest method — comprehensive hit testing.

```rust
impl Niri {
    /// Returns contents under the given point.
    pub fn contents_under(&self, pos: Point<f64, Logical>) -> PointContents {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 8: Remove from niri.rs (5 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/hit_test.rs` exists (~450 LOC)
- [ ] All hit test methods work correctly
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No duplicate method definitions

---

## Methods Extracted

| Method | Description |
|--------|-------------|
| `is_inside_hot_corner` | Check if point is in hot corner |
| `is_sticky_obscured_under` | Check if sticky layer obscures point |
| `is_layout_obscured_under` | Check if layout is obscured at point |
| `row_under` | Get row under point |
| `row_under_cursor` | Get row under cursor |
| `window_under` | Get window under point |
| `window_under_cursor` | Get window under cursor |
| `contents_under` | Comprehensive hit test |

---

## Next Phase

After completing this phase, proceed to [Phase A4: niri/lock.rs](phase-A4-niri-lock.md).
