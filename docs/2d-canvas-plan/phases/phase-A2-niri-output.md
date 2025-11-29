# Phase A2: Extract niri/output.rs

> **Status**: ⏳ PENDING
> **Estimated Time**: 1 hour
> **Risk Level**: 🟡 Medium (mutations, but well-contained)
> **Prerequisite**: Phase A1 complete

---

## Goal

Extract all output management methods from `niri.rs` into `src/niri/output.rs`.

---

## Work Units

### Unit 1: Create output.rs with Imports (5 min)

Create `src/niri/output.rs`:
```rust
//! Output management for the Niri compositor.

use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::backend::Backend;
use crate::niri::Niri;
use crate::utils::{output_matches_name, output_size};
```

Update `src/niri/mod.rs`:
```rust
mod output;
mod types;

pub use types::*;
// output methods are impl Niri, no re-export needed
```

---

### Unit 2: Extract output_under (10 min)

**Source**: `niri.rs` ~line 3264

```rust
impl Niri {
    pub fn output_under(&self, pos: Point<f64, Logical>) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.global_space.output_under(pos).next()?;
        let pos_within_output = pos
            - self
                .global_space
                .output_geometry(output)
                .unwrap()
                .loc
                .to_f64();
        Some((output, pos_within_output))
    }

    pub fn output_under_cursor(&self) -> Option<Output> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.global_space.output_under(pos).next().cloned()
    }
}
```

---

### Unit 3: Extract Directional Output Methods (15 min)

**Source**: `niri.rs` ~lines 3688-3801

```rust
impl Niri {
    pub fn output_left_of(&self, current: &Output) -> Option<Output> {
        // ... (copy from niri.rs)
    }

    pub fn output_right_of(&self, current: &Output) -> Option<Output> {
        // ...
    }

    pub fn output_up_of(&self, current: &Output) -> Option<Output> {
        // ...
    }

    pub fn output_down_of(&self, current: &Output) -> Option<Output> {
        // ...
    }

    pub fn output_previous_of(&self, current: &Output) -> Option<Output> {
        // ...
    }

    pub fn output_next_of(&self, current: &Output) -> Option<Output> {
        // ...
    }

    // Convenience methods using active output
    pub fn output_left(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_left_of(active)
    }

    pub fn output_right(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_right_of(active)
    }

    pub fn output_up(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_up_of(active)
    }

    pub fn output_down(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_down_of(active)
    }

    pub fn output_previous(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_previous_of(active)
    }

    pub fn output_next(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_next_of(active)
    }
}
```

---

### Unit 4: Extract Output Query Methods (10 min)

**Source**: `niri.rs` ~lines 3829-3864

```rust
impl Niri {
    pub fn output_for_tablet(&self) -> Option<&Output> {
        let config = self.config.borrow();
        let map_to_output = config.input.tablet.map_to_output.as_ref();
        map_to_output.and_then(|name| self.output_by_name_match(name))
    }

    pub fn output_for_touch(&self) -> Option<&Output> {
        let config = self.config.borrow();
        let map_to_output = config.input.touch.map_to_output.as_ref();
        map_to_output
            .and_then(|name| self.output_by_name_match(name))
            .or_else(|| self.global_space.outputs().next())
    }

    pub fn output_by_name_match(&self, target: &str) -> Option<&Output> {
        self.global_space
            .outputs()
            .find(|output| output_matches_name(output, target))
    }

    pub fn output_for_root(&self, root: &WlSurface) -> Option<&Output> {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 5: Extract output_resized (10 min)

**Source**: `niri.rs` ~lines 3194-3242

```rust
impl Niri {
    pub fn output_resized(&mut self, output: &Output) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 6: Extract Monitor Activation (5 min)

**Source**: `niri.rs` ~lines 3244-3262

```rust
impl Niri {
    pub fn deactivate_monitors(&mut self, backend: &mut Backend) {
        if !self.monitors_active {
            return;
        }
        self.monitors_active = false;
        backend.set_monitors_active(false);
    }

    pub fn activate_monitors(&mut self, backend: &mut Backend) {
        if self.monitors_active {
            return;
        }
        self.monitors_active = true;
        backend.set_monitors_active(true);
        self.queue_redraw_all();
    }
}
```

---

### Unit 7: Remove from niri.rs (10 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/output.rs` exists (~500 LOC)
- [ ] All output methods work correctly
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No duplicate method definitions

---

## Methods Extracted

| Method | Type |
|--------|------|
| `output_under` | Query |
| `output_under_cursor` | Query |
| `output_left_of` | Query |
| `output_right_of` | Query |
| `output_up_of` | Query |
| `output_down_of` | Query |
| `output_previous_of` | Query |
| `output_next_of` | Query |
| `output_left` | Query |
| `output_right` | Query |
| `output_up` | Query |
| `output_down` | Query |
| `output_previous` | Query |
| `output_next` | Query |
| `output_for_tablet` | Query |
| `output_for_touch` | Query |
| `output_by_name_match` | Query |
| `output_for_root` | Query |
| `output_resized` | Mutation |
| `deactivate_monitors` | Mutation |
| `activate_monitors` | Mutation |

---

## Next Phase

After completing this phase, proceed to [Phase A3: niri/hit_test.rs](phase-A3-niri-hit-test.md).
