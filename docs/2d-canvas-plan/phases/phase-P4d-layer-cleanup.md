# Phase P4d: Layer On-Demand Cleanup

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~15 minutes  
> **Risk Level**: 🟢 Low (simple extraction)  
> **Prerequisite**: Phase P4c complete  
> **Creates**: `cleanup_layer_on_demand_focus()` method

---

## Goal

Extract the **layer on-demand cleanup logic** (lines 933-953 of `update_keyboard_focus()`) into a dedicated method on `FocusState`. This is the final piece of the P4 refactor.

---

## Current Code Analysis

```rust
// Clean up on-demand layer surface focus if necessary.
if let Some(surface) = &self.niri.layer_shell_on_demand_focus {
    // Still alive and has on-demand interactivity.
    let mut good = surface.alive()
        && surface.cached_state().keyboard_interactivity
            == wlr_layer::KeyboardInteractivity::OnDemand;

    if let Some(mapped) = self.niri.mapped_layer_surfaces.get(surface) {
        // Check if it moved to the overview backdrop.
        if mapped.place_within_backdrop() {
            good = false;
        }
    } else {
        // The layer surface is alive but it got unmapped.
        good = false;
    }

    if !good {
        self.niri.layer_shell_on_demand_focus = None;
    }
}
```

---

## Target Architecture

### Add to FocusState

```rust
impl FocusState {
    /// Cleans up on-demand layer focus if the surface is no longer valid.
    ///
    /// Returns true if the on-demand focus was cleared.
    pub fn cleanup_layer_on_demand<F>(&mut self, is_valid: F) -> bool 
    where
        F: FnOnce(&LayerSurface) -> bool,
    {
        let should_clear = self.layer_on_demand.as_ref().map_or(false, |surface| {
            !is_valid(surface)
        });
        
        if should_clear {
            self.layer_on_demand = None;
            true
        } else {
            false
        }
    }
}
```

### Add Helper to State

```rust
impl State {
    /// Cleans up stale on-demand layer focus.
    fn cleanup_layer_on_demand_focus(&mut self) {
        self.niri.focus.cleanup_layer_on_demand(|surface| {
            // Must be alive
            if !surface.alive() {
                return false;
            }
            
            // Must have on-demand interactivity
            if surface.cached_state().keyboard_interactivity
                != wlr_layer::KeyboardInteractivity::OnDemand
            {
                return false;
            }
            
            // Must be mapped and not in backdrop
            match self.niri.mapped_layer_surfaces.get(surface) {
                Some(mapped) => !mapped.place_within_backdrop(),
                None => false, // Unmapped
            }
        });
    }
}
```

---

## Work Units

### Unit 1: Add cleanup_layer_on_demand() to FocusState

Add the method that takes a validation closure.

**Verify**: `cargo check`

---

### Unit 2: Add cleanup_layer_on_demand_focus() to State

Add the helper that provides the validation logic.

**Verify**: `cargo check`

---

### Unit 3: Update update_keyboard_focus()

Replace the inline cleanup with the method call:

```rust
pub fn update_keyboard_focus(&mut self) {
    // Clean up stale on-demand focus
    self.cleanup_layer_on_demand_focus();
    
    // Compute new focus
    let ctx = self.build_focus_context();
    let new_focus = self.niri.focus.compute_focus(&ctx);
    
    // Handle focus change if different
    let old_focus = self.niri.focus.current().clone();
    if old_focus != new_focus {
        trace!("keyboard focus changed from {:?} to {:?}", old_focus, new_focus);
        self.handle_focus_change(&old_focus, &new_focus);
    }
}
```

**Verify**: `cargo check` && `cargo test`

---

## Final update_keyboard_focus()

After all P4 phases, the function should be ~15 lines:

```rust
pub fn update_keyboard_focus(&mut self) {
    // Clean up stale on-demand focus
    self.cleanup_layer_on_demand_focus();
    
    // Compute new focus
    let ctx = self.build_focus_context();
    let new_focus = self.niri.focus.compute_focus(&ctx);
    
    // Handle focus change if different
    let old_focus = self.niri.focus.current().clone();
    if old_focus != new_focus {
        trace!("keyboard focus changed from {:?} to {:?}", old_focus, new_focus);
        self.handle_focus_change(&old_focus, &new_focus);
    }
}
```

Down from **~256 lines** to **~15 lines**!

---

## Verification Checklist

- [ ] `cleanup_layer_on_demand()` added to FocusState
- [ ] `cleanup_layer_on_demand_focus()` added to State
- [ ] `update_keyboard_focus()` uses the new methods
- [ ] `update_keyboard_focus()` is now ~15 lines
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/focus.rs` | +15 lines |
| `src/niri/mod.rs` | Simplified cleanup |

---

## P4 Summary

After completing P4a-P4d:

| Metric | Before | After |
|--------|--------|-------|
| `update_keyboard_focus()` | 256 lines | ~15 lines |
| Focus fields in Niri | 4 | 0 (moved to FocusState) |
| Testable focus logic | No | Yes (compute_focus has unit tests) |
| Helper methods | 0 | 6+ focused methods |

### New Structure

```
FocusState (subsystems/focus.rs)
├── current: KeyboardFocus
├── layer_on_demand: Option<LayerSurface>
├── idle_inhibitors: HashSet<WlSurface>
├── shortcut_inhibitors: HashMap<...>
├── compute_focus(&FocusContext) -> KeyboardFocus
└── cleanup_layer_on_demand()

State helpers (mod.rs)
├── build_focus_context() -> FocusContext
├── cleanup_layer_on_demand_focus()
├── handle_focus_change()
├── update_window_focus_states()
├── update_mru_timestamp()
├── schedule_mru_commit()
├── handle_popup_grab_on_focus_change()
└── handle_keyboard_layout_tracking()
```

---

## Next Phase

After completing this phase, proceed to [Phase P5: StreamingSubsystem](phase-P5-streaming.md).
