# Phase A7: Extract niri/pointer.rs + rules.rs

> **Status**: ✅ COMPLETE (TEAM_068)
> **Estimated Time**: 45 minutes
> **Risk Level**: 🟢 Low (small, focused modules)
> **Prerequisite**: Phase A6 complete

---

## Goal

Extract pointer/cursor management and window/layer rules into two files:
- `src/niri/pointer.rs` — Pointer constraints, inactivity, focus-follows-mouse
- `src/niri/rules.rs` — Window and layer rule recomputation

User: Remember that when a item is too big of a refactor than planned. that I want you to make it smaller and do it in multiple steps. Write it down in this folder as broken down steps in the phase file...

---

## Work Units

### Unit 1: Create pointer.rs (5 min)

Create `src/niri/pointer.rs`:
```rust
//! Pointer and cursor management for the Niri compositor.

use std::time::Duration;

use calloop::timer::Timer;
use smithay::input::pointer::PointerHandle;
use smithay::wayland::pointer_constraints::with_pointer_constraint;

use crate::niri::{Niri, PointerVisibility, PointContents};
```

Update `src/niri/mod.rs`:
```rust
mod frame_callbacks;
mod hit_test;
mod lock;
mod output;
mod pointer;
mod render;
mod rules;
mod screencast;
mod screencopy;
mod screenshot;
mod types;

pub use types::*;
```

---

### Unit 2: Extract maybe_activate_pointer_constraint (10 min)

**Source**: `niri.rs` ~lines 6157-6188

```rust
impl Niri {
    /// Activates the pointer constraint if necessary according to the current pointer contents.
    ///
    /// Make sure the pointer location and contents are up to date before calling this.
    pub fn maybe_activate_pointer_constraint(&self) {
        let Some((surface, surface_loc)) = &self.pointer_contents.surface else {
            return;
        };

        let pointer = self.seat.get_pointer().unwrap();
        if Some(surface) != pointer.current_focus().as_ref() {
            return;
        }

        with_pointer_constraint(surface, &pointer, |constraint| {
            let Some(constraint) = constraint else { return };

            if constraint.is_active() {
                return;
            }

            // Constraint does not apply if not within region.
            if let Some(region) = constraint.region() {
                let pointer_pos = pointer.current_location();
                let pos_within_surface = pointer_pos - *surface_loc;
                if !region.contains(pos_within_surface.to_i32_round()) {
                    return;
                }
            }

            constraint.activate();
        });
    }
}
```

---

### Unit 3: Extract reset_pointer_inactivity_timer (10 min)

**Source**: `niri.rs` ~lines 6462-6497

```rust
impl Niri {
    pub fn reset_pointer_inactivity_timer(&mut self) {
        if self.pointer_inactivity_timer_got_reset {
            return;
        }

        let _span = tracy_client::span!("Niri::reset_pointer_inactivity_timer");

        if let Some(token) = self.pointer_inactivity_timer.take() {
            self.event_loop.remove(token);
        }

        let Some(timeout_ms) = self.config.borrow().cursor.hide_after_inactive_ms else {
            return;
        };

        let duration = Duration::from_millis(timeout_ms as u64);
        let timer = Timer::from_duration(duration);
        let token = self
            .event_loop
            .insert_source(timer, move |_, _, state| {
                state.niri.pointer_inactivity_timer = None;

                if state.niri.pointer_visibility.is_visible() {
                    state.niri.pointer_visibility = PointerVisibility::Hidden;
                    state.niri.queue_redraw_all();
                }

                TimeoutAction::Drop
            })
            .unwrap();
        self.pointer_inactivity_timer = Some(token);

        self.pointer_inactivity_timer_got_reset = true;
    }
}
```

---

### Unit 4: Extract notify_activity (5 min)

**Source**: `niri.rs` ~lines 6499-6510 (approximately)

```rust
impl Niri {
    pub fn notify_activity(&mut self) {
        if self.notified_activity_this_iteration {
            return;
        }
        // ... (copy rest from niri.rs)
    }
}
```

---

### Unit 5: Extract handle_focus_follows_mouse (10 min)

**Source**: `niri.rs` ~lines 6269-6327

```rust
impl Niri {
    pub fn handle_focus_follows_mouse(&mut self, new_focus: &PointContents) {
        let Some(ffm) = self.config.borrow().input.focus_follows_mouse else {
            return;
        };

        let pointer = &self.seat.get_pointer().unwrap();
        if pointer.is_grabbed() {
            return;
        }

        // ... (copy full method from niri.rs)
    }
}
```

---

### Unit 6: Extract focus_layer_surface_if_on_demand (5 min)

**Source**: `niri.rs` ~lines 6190-6213

```rust
impl Niri {
    pub fn focus_layer_surface_if_on_demand(&mut self, surface: Option<LayerSurface>) {
        // ... (copy from niri.rs)
    }
}
```

---

### Unit 7: Create rules.rs (5 min)

Create `src/niri/rules.rs`:
```rust
//! Window and layer rule recomputation for the Niri compositor.

use crate::niri::Niri;
use crate::window::{ResolvedWindowRules, WindowRef};
```

---

### Unit 8: Extract recompute_window_rules (10 min)

**Source**: `niri.rs` ~lines 6404-6438

```rust
impl Niri {
    pub fn recompute_window_rules(&mut self) {
        let _span = tracy_client::span!("Niri::recompute_window_rules");

        let changed = {
            let window_rules = &self.config.borrow().window_rules;

            for unmapped in self.unmapped_windows.values_mut() {
                let new_rules = ResolvedWindowRules::compute(
                    window_rules,
                    WindowRef::Unmapped(unmapped),
                    self.is_at_startup,
                );
                if let InitialConfigureState::Configured { rules, .. } = &mut unmapped.state {
                    *rules = new_rules;
                }
            }

            let mut windows = vec![];
            self.layout.with_windows_mut(|mapped, _| {
                if mapped.recompute_window_rules(window_rules, self.is_at_startup) {
                    windows.push(mapped.window.clone());
                }
            });
            let changed = !windows.is_empty();
            for win in windows {
                self.layout.update_window(&win, None);
            }
            changed
        };

        if changed {
            self.queue_redraw_all();
        }
    }
}
```

---

### Unit 9: Extract recompute_layer_rules (10 min)

**Source**: `niri.rs` ~lines 6440-6460

```rust
impl Niri {
    pub fn recompute_layer_rules(&mut self) {
        let _span = tracy_client::span!("Niri::recompute_layer_rules");

        let mut changed = false;
        {
            let config = self.config.borrow();
            let rules = &config.layer_rules;

            for mapped in self.mapped_layer_surfaces.values_mut() {
                if mapped.recompute_layer_rules(rules, self.is_at_startup) {
                    changed = true;
                    mapped.update_config(&config);
                }
            }
        }

        if changed {
            self.queue_redraw_all();
        }
    }
}
```

---

### Unit 10: Remove from niri.rs (5 min)

Delete all the extracted methods from `niri.rs`.

**Verify**: `cargo check` and `cargo test`

---

## Verification Checklist

- [ ] `src/niri/pointer.rs` exists (~350 LOC)
- [ ] `src/niri/rules.rs` exists (~150 LOC)
- [ ] Pointer constraints work
- [ ] Inactivity timer works
- [ ] Focus-follows-mouse works
- [ ] Window/layer rules recompute correctly
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Created

| File | LOC | Description |
|------|-----|-------------|
| `niri/pointer.rs` | ~350 | Pointer/cursor management |
| `niri/rules.rs` | ~150 | Window/layer rules |

---

## Next Phase

After completing this phase, proceed to [Phase A8: Constructor Extraction](phase-A8-niri-init.md).
