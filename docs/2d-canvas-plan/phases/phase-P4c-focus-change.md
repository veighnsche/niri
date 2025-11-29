# Phase P4c: Extract Focus Change Handling

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟡 Medium (side effects)  
> **Prerequisite**: Phase P4b complete  
> **Creates**: `handle_focus_change()` helper method

---

## Goal

Extract the **focus change side effects** (lines 1068-1187 of `update_keyboard_focus()`) into smaller, focused helper methods. This includes:

1. Window focus state updates
2. MRU timestamp tracking with debounce
3. Popup grab management
4. Per-window keyboard layout tracking

---

## Current Code Analysis

When focus changes (lines 1068-1187), the code handles:

### 1. Window Focus State (lines 1076-1126)
```rust
// Tell windows their new focus state
if let KeyboardFocus::Layout { surface: Some(surface) } = &self.niri.keyboard_focus {
    // Set old window as not focused
}
if let KeyboardFocus::Layout { surface: Some(surface) } = &focus {
    // Set new window as focused
    // Handle MRU timestamp with debounce timer
}
```

### 2. Popup Grab Management (lines 1128-1145)
```rust
if let Some(grab) = self.niri.popup_grab.as_mut() {
    if grab.has_keyboard_grab && Some(&grab.root) != focus.surface() {
        // Ungrab popups when focus moves away
    }
}
```

### 3. Per-Window Keyboard Layout (lines 1147-1180)
```rust
if self.niri.config.borrow().input.keyboard.track_layout == TrackLayout::Window {
    // Store current layout for old focus
    // Restore layout for new focus
}
```

### 4. Final Focus Update (lines 1182-1186)
```rust
self.niri.keyboard_focus.clone_from(&focus);
keyboard.set_focus(self, focus.into_surface(), ...);
self.niri.queue_redraw_all();
```

---

## Target Architecture

### Add Helper Methods to State

```rust
impl State {
    /// Handles all side effects of a focus change.
    fn handle_focus_change(&mut self, old_focus: &KeyboardFocus, new_focus: &KeyboardFocus) {
        self.update_window_focus_states(old_focus, new_focus);
        self.handle_popup_grab_on_focus_change(new_focus);
        self.handle_keyboard_layout_tracking(old_focus, new_focus);
        self.apply_keyboard_focus(new_focus);
    }
    
    /// Updates window focus states and MRU timestamps.
    fn update_window_focus_states(&mut self, old: &KeyboardFocus, new: &KeyboardFocus) {
        // Unfocus old window
        if let KeyboardFocus::Layout { surface: Some(surface) } = old {
            if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                mapped.set_is_focused(false);
            }
        }
        
        // Focus new window and update MRU
        if let KeyboardFocus::Layout { surface: Some(surface) } = new {
            if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                mapped.set_is_focused(true);
                self.update_mru_timestamp(mapped);
            }
        }
    }
    
    /// Updates MRU timestamp with debounce.
    fn update_mru_timestamp(&mut self, mapped: &mut Mapped) {
        let stamp = get_monotonic_time();
        let debounce = self.niri.config.borrow().recent_windows.debounce_ms;
        let debounce = Duration::from_millis(u64::from(debounce));
        
        if mapped.get_focus_timestamp().is_none() || debounce.is_zero() {
            mapped.set_focus_timestamp(stamp);
        } else {
            self.schedule_mru_commit(mapped.id(), stamp, debounce);
        }
    }
    
    /// Schedules a delayed MRU commit.
    fn schedule_mru_commit(&mut self, id: WindowId, stamp: Duration, debounce: Duration) {
        let timer = Timer::from_duration(debounce);
        let token = self.niri.event_loop
            .insert_source(timer, move |_, _, state| {
                state.niri.mru_apply_keyboard_commit();
                TimeoutAction::Drop
            })
            .unwrap();
        
        if let Some(PendingMruCommit { token: old_token, .. }) =
            self.niri.pending_mru_commit.replace(PendingMruCommit { id, token, stamp })
        {
            self.niri.event_loop.remove(old_token);
        }
    }
    
    /// Handles popup grab when focus changes.
    fn handle_popup_grab_on_focus_change(&mut self, new_focus: &KeyboardFocus) {
        let Some(grab) = self.niri.popup_grab.as_mut() else { return };
        
        if grab.has_keyboard_grab && Some(&grab.root) != new_focus.surface() {
            trace!("grab root {:?} is not the new focus {:?}, ungrabbing", grab.root, new_focus);
            
            let keyboard = self.niri.seat.get_keyboard().unwrap();
            grab.grab.ungrab(PopupUngrabStrategy::All);
            keyboard.unset_grab(self);
            self.niri.seat.get_pointer().unwrap().unset_grab(
                self,
                SERIAL_COUNTER.next_serial(),
                get_monotonic_time().as_millis() as u32,
            );
            self.niri.popup_grab = None;
        }
    }
    
    /// Handles per-window keyboard layout tracking.
    fn handle_keyboard_layout_tracking(&mut self, old: &KeyboardFocus, new: &KeyboardFocus) {
        if self.niri.config.borrow().input.keyboard.track_layout != TrackLayout::Window {
            return;
        }
        
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        let current_layout = keyboard.with_xkb_state(self, |context| {
            context.xkb().lock().unwrap().active_layout()
        });
        
        // Store current layout for old focus surface
        if let Some(surface) = old.surface() {
            with_states(surface, |data| {
                let cell = data.data_map
                    .get_or_insert::<Cell<KeyboardLayout>, _>(Cell::default);
                cell.set(current_layout);
            });
        }
        
        // Restore layout for new focus surface
        let new_layout = new.surface().map_or(current_layout, |surface| {
            with_states(surface, |data| {
                data.data_map
                    .get_or_insert::<Cell<KeyboardLayout>, _>(|| Cell::new(KeyboardLayout::default()))
                    .get()
            })
        });
        
        if new_layout != current_layout && new.surface().is_some() {
            keyboard.set_focus(self, None, SERIAL_COUNTER.next_serial());
            keyboard.with_xkb_state(self, |mut context| {
                context.set_layout(new_layout);
            });
        }
    }
    
    /// Applies the keyboard focus change.
    fn apply_keyboard_focus(&mut self, new_focus: &KeyboardFocus) {
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        self.niri.focus.set_current(new_focus.clone());
        keyboard.set_focus(self, new_focus.clone().into_surface(), SERIAL_COUNTER.next_serial());
        self.niri.queue_redraw_all();
    }
}
```

---

## Work Units

### Unit 1: Extract update_window_focus_states()

Extract the window focus state update logic.

**Verify**: `cargo check`

---

### Unit 2: Extract update_mru_timestamp() and schedule_mru_commit()

Extract the MRU timestamp logic with debounce.

**Verify**: `cargo check`

---

### Unit 3: Extract handle_popup_grab_on_focus_change()

Extract the popup grab handling.

**Verify**: `cargo check`

---

### Unit 4: Extract handle_keyboard_layout_tracking()

Extract the per-window keyboard layout tracking.

**Verify**: `cargo check`

---

### Unit 5: Create handle_focus_change() Coordinator

Create the main coordinator method that calls all helpers.

---

### Unit 6: Refactor update_keyboard_focus()

The final `update_keyboard_focus()` should now be simple:

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

## Verification Checklist

- [ ] `update_window_focus_states()` extracted
- [ ] `update_mru_timestamp()` extracted
- [ ] `schedule_mru_commit()` extracted
- [ ] `handle_popup_grab_on_focus_change()` extracted
- [ ] `handle_keyboard_layout_tracking()` extracted
- [ ] `handle_focus_change()` coordinator created
- [ ] `update_keyboard_focus()` simplified
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/mod.rs` | Refactored into helper methods |

---

## Why This Is Medium Risk

- **Side effects**: These methods have real effects (timers, focus changes)
- **Order matters**: Some operations must happen in sequence
- **Well-isolated**: Each helper has a single responsibility
- **Testable in integration**: Full test suite validates behavior

---

## Next Phase

After completing this phase, proceed to [Phase P4d: Layer On-Demand Cleanup](phase-P4d-layer-cleanup.md).
