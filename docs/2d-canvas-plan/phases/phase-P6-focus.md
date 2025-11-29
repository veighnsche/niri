# Phase P6: Create focus.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🔴 High (complex focus logic)  
> **Prerequisite**: None

---

## Goal

Create new `focus.rs` module for keyboard focus management.

The `update_keyboard_focus()` function alone is **~260 lines** and is one of the most complex functions in mod.rs.

Total: ~300 lines moved from mod.rs

---

## Functions to Move

All these are `impl State` methods:

### 1. update_keyboard_focus (mod.rs ~lines 932-1190) 🔴 HUGE

```rust
pub fn update_keyboard_focus(&mut self) {
    // ~260 lines!
    // Complex logic for:
    // - Layer shell focus (exclusive, on-demand)
    // - Layout window focus
    // - Lock screen focus
    // - Screenshot UI focus
    // - Popup grabs
    // - Keyboard layout tracking
    // - Focus timestamps (MRU)
}
```

### 2. focus_default_monitor (mod.rs ~lines 754-778)

```rust
pub fn focus_default_monitor(&mut self) {
    // ~25 lines - focuses first output or configured default
}
```

### 3. focus_window (mod.rs ~lines 779-797)

```rust
pub fn focus_window(&mut self, window: &Window) {
    // ~20 lines - focuses a specific window
}
```

### 4. refresh_popup_grab (mod.rs ~lines 911-931)

```rust
pub fn refresh_popup_grab(&mut self) {
    // ~20 lines - cleans up popup grabs
}
```

---

## Work Units

### Unit 1: Create focus.rs File

Create `src/niri/focus.rs`:

```rust
//! Keyboard focus management for the Niri compositor.
//!
//! This module handles keyboard focus tracking, including layer-shell focus,
//! layout window focus, popup grabs, and keyboard layout per-window tracking.

use std::cell::Cell;
use std::time::Duration;

use smithay::desktop::{layer_map_for_output, WindowSurfaceType};
use smithay::input::keyboard::Layout as KeyboardLayout;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::wlr_layer::{self, Layer};

use crate::utils::get_monotonic_time;
use crate::window::Window;

use super::{KeyboardFocus, Niri, PendingMruCommit, State};

// =============================================================================
// Focus Management Methods (impl State)
// =============================================================================

impl State {
    // Methods will go here
}
```

---

### Unit 2: Add Module Declaration

In `mod.rs`, add:

```rust
mod cursor;
mod focus;
mod frame_callbacks;
// ... other mods
```

---

### Unit 3: Move update_keyboard_focus

This is the big one. It has several nested closures and complex control flow.

**Key sections to preserve**:
1. Layer shell on-demand focus cleanup
2. Focus computation logic (nested closures)
3. Focus change handling
4. MRU timestamp tracking
5. Popup grab management
6. Keyboard layout per-window tracking

```rust
impl State {
    pub fn update_keyboard_focus(&mut self) {
        // Clean up on-demand layer surface focus if necessary.
        if let Some(surface) = &self.niri.layer_shell_on_demand_focus {
            // ... validation logic
        }

        // Compute the current focus.
        let focus = if self.niri.exit_confirm_dialog.is_open() {
            KeyboardFocus::ExitConfirmDialog
        } else if self.niri.is_locked() {
            // ... lock screen focus
        } else if self.niri.screenshot_ui.is_open() {
            // ... screenshot UI focus
        } else if self.niri.window_mru_ui.is_open() {
            // ... MRU UI focus  
        } else if let Some(output) = self.niri.layout.active_output() {
            // ... complex layer/layout focus logic
        } else {
            KeyboardFocus::Layout { surface: None }
        };

        // Handle focus change
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        if self.niri.keyboard_focus != focus {
            // ... focus change handling (~100 lines)
        }
    }
}
```

**Verify**: `cargo check`

---

### Unit 4: Move Supporting Focus Methods

```rust
impl State {
    /// Focuses the default monitor based on config.
    pub fn focus_default_monitor(&mut self) {
        // ... implementation
    }

    /// Focuses a specific window.
    pub fn focus_window(&mut self, window: &Window) {
        // ... implementation
    }

    /// Refreshes popup grab state.
    pub fn refresh_popup_grab(&mut self) {
        // ... implementation
    }
}
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `focus.rs` exists with module header
- [ ] `update_keyboard_focus` moved (~260 lines)
- [ ] `focus_default_monitor` moved
- [ ] `focus_window` moved
- [ ] `refresh_popup_grab` moved
- [ ] Module declared in mod.rs
- [ ] No duplicate definitions
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/focus.rs` | +300 lines (new file) |
| `src/niri/mod.rs` | -300 lines, +1 line (mod declaration) |

**Expected mod.rs after P6**: ~2324 lines

---

## Technical Notes

### update_keyboard_focus Complexity

This function is complex because it handles:

1. **Multiple focus targets**: Layout windows, layer-shell, lock screen, UI overlays
2. **Layer shell priorities**: Overlay > Top > (Bottom/Background depending on mode)
3. **Exclusive vs on-demand**: Layer surfaces can grab focus exclusively or on-demand
4. **Popup grabs**: Menus/popups can capture keyboard
5. **Per-window keyboard layouts**: Track_Layout::Window mode
6. **MRU timestamps**: Focus debouncing for alt-tab

### Nested Closures

The function uses several local closures:
- `grab_on_layer` - check for popup grabs on a layer
- `layout_focus` - get the focused layout window
- `excl_focus_on_layer` - exclusive layer-shell focus
- `on_d_focus_on_layer` - on-demand layer-shell focus
- `focus_on_layer` - combined exclusive + on-demand

These closures capture `self` references, which is why they can't be extracted to separate methods easily.

---

## Next Phase

After completing this phase, proceed to [Phase P7: Create config.rs](phase-P7-config.md).
