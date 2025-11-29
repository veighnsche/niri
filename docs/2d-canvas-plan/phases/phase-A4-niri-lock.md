# Phase A4: Extract niri/lock.rs

> **Status**: ⏳ PENDING
> **Estimated Time**: 45 minutes
> **Risk Level**: 🟡 Medium (state machine, but self-contained)
> **Prerequisite**: Phase A3 complete

---

## Goal

Extract all session lock methods from `niri.rs` into `src/niri/lock.rs`.

The lock system is a self-contained state machine with clear boundaries.

---

## Work Units

### Unit 1: Create lock.rs with Imports (5 min)

Create `src/niri/lock.rs`:
```rust
//! Session lock management for the Niri compositor.
//!
//! Handles the ext-session-lock-v1 protocol for screen locking.

use std::mem;
use std::time::Duration;

use calloop::{timer::Timer, LoopHandle};
use smithay::output::Output;
use smithay::wayland::session_lock::{LockSurface, SessionLocker};

use crate::niri::{LockState, Niri};
use crate::utils::is_mapped;
```

Update `src/niri/mod.rs`:
```rust
mod hit_test;
mod lock;
mod output;
mod types;

pub use types::*;
```

---

### Unit 2: Extract is_locked (5 min)

**Source**: `niri.rs` ~line 5917

```rust
impl Niri {
    pub fn is_locked(&self) -> bool {
        match self.lock_state {
            LockState::Unlocked | LockState::WaitingForSurfaces { .. } => false,
            LockState::Locking(_) | LockState::Locked(_) => true,
        }
    }
}
```

---

### Unit 3: Extract lock (15 min)

**Source**: `niri.rs` ~lines 5924-5984

```rust
impl Niri {
    pub fn lock(&mut self, confirmation: SessionLocker) {
        // Check if another client is in the process of locking.
        if matches!(
            self.lock_state,
            LockState::WaitingForSurfaces { .. } | LockState::Locking(_)
        ) {
            info!("refusing lock as another client is currently locking");
            return;
        }

        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 4: Extract maybe_continue_to_locking (10 min)

**Source**: `niri.rs` ~lines 5986-6007

```rust
impl Niri {
    pub fn maybe_continue_to_locking(&mut self) {
        if !matches!(self.lock_state, LockState::WaitingForSurfaces { .. }) {
            return;
        }

        // Check if there are any outputs whose lock surfaces had not had a commit yet.
        for state in self.output_state.values() {
            let Some(surface) = &state.lock_surface else {
                return;
            };

            if !is_mapped(surface.wl_surface()) {
                return;
            }
        }

        trace!("lock surfaces are ready, continuing");
        self.continue_to_locking();
    }
}
```

---

### Unit 5: Extract continue_to_locking (10 min)

**Source**: `niri.rs` ~lines 6009-6038

```rust
impl Niri {
    fn continue_to_locking(&mut self) {
        match mem::take(&mut self.lock_state) {
            LockState::WaitingForSurfaces {
                confirmation,
                deadline_token,
            } => {
                self.event_loop.remove(deadline_token);
                // ... (copy full method from niri.rs)
            }
            other => {
                error!("continue_to_locking() called with wrong lock state: {other:?}");
                self.lock_state = other;
            }
        }
    }
}
```

---

### Unit 6: Extract unlock (5 min)

**Source**: `niri.rs` ~lines 6040-6052

```rust
impl Niri {
    pub fn unlock(&mut self) {
        info!("unlocking session");

        let prev = mem::take(&mut self.lock_state);
        if let LockState::WaitingForSurfaces { deadline_token, .. } = prev {
            self.event_loop.remove(deadline_token);
        }

        for output_state in self.output_state.values_mut() {
            output_state.lock_surface = None;
        }
        self.queue_redraw_all();
    }
}
```

---

### Unit 7: Extract update_locked_hint (10 min)

**Source**: `niri.rs` ~lines 6054-6131 (feature-gated)

```rust
impl Niri {
    #[cfg(feature = "dbus")]
    fn update_locked_hint(&mut self) {
        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 8: Extract new_lock_surface (5 min)

**Source**: `niri.rs` ~lines 6133-6155

```rust
impl Niri {
    pub fn new_lock_surface(&mut self, surface: LockSurface, output: &Output) {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 9: Extract lock_surface_focus (5 min)

**Source**: `niri.rs` ~lines 3866-3875

```rust
impl Niri {
    pub fn lock_surface_focus(&self) -> Option<WlSurface> {
        let output_under_cursor = self.output_under_cursor();
        let output = output_under_cursor
            .as_ref()
            .or_else(|| self.layout.active_output())
            .or_else(|| self.global_space.outputs().next())?;

        let state = self.output_state.get(output)?;
        state.lock_surface.as_ref().map(|s| s.wl_surface()).cloned()
    }
}
```

---

### Unit 10: Remove from niri.rs (5 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/lock.rs` exists (~350 LOC)
- [ ] Lock state machine works correctly
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No duplicate method definitions

---

## Methods Extracted

| Method | Description |
|--------|-------------|
| `is_locked` | Check if session is locked |
| `lock` | Start locking session |
| `maybe_continue_to_locking` | Check if ready to continue |
| `continue_to_locking` | Continue lock process |
| `unlock` | Unlock session |
| `update_locked_hint` | Update logind hint (#[cfg(dbus)]) |
| `new_lock_surface` | Handle new lock surface |
| `lock_surface_focus` | Get focused lock surface |

---

## Next Phase

After completing this phase, proceed to [Phase A5: niri/render.rs](phase-A5-niri-render.md).
