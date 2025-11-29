# Phase P8: Create dbus.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low (isolated handlers)  
> **Prerequisite**: None

---

## Goal

Create new `dbus.rs` module for D-Bus message handlers.

These are callback functions that handle incoming D-Bus messages.

Total: ~200 lines moved from mod.rs

---

## Functions to Move

All these are `impl State` methods, feature-gated with `#[cfg(feature = "dbus")]`:

### 1. on_pw_msg (mod.rs ~lines 1861-1878)

```rust
#[cfg(feature = "xdp-gnome-screencast")]
pub fn on_pw_msg(&mut self, msg: PwToNiri) {
    // ~18 lines - handles PipeWire messages
}
```

### 2. redraw_cast (mod.rs ~lines 1879-1946)

```rust
#[cfg(feature = "xdp-gnome-screencast")]
fn redraw_cast(&mut self, stream_id: usize) {
    // ~65 lines - redraws screencast output
}
```

### 3. set_dynamic_cast_target (mod.rs ~lines 1947-1997)

```rust
#[cfg(not(feature = "xdp-gnome-screencast"))]
pub fn set_dynamic_cast_target(&mut self, _target: CastTarget) {}

#[cfg(feature = "xdp-gnome-screencast")]
pub fn set_dynamic_cast_target(&mut self, target: CastTarget) {
    // ~45 lines - sets dynamic cast target
}
```

### 4. on_screen_cast_msg (mod.rs ~lines 1998-2132)

```rust
#[cfg(feature = "xdp-gnome-screencast")]
pub fn on_screen_cast_msg(&mut self, msg: ScreenCastToNiri) {
    // ~135 lines - handles screen cast D-Bus messages
}
```

### 5. on_screen_shot_msg (mod.rs ~lines 2133-2148)

```rust
#[cfg(feature = "dbus")]
pub fn on_screen_shot_msg(&mut self, ...) {
    // ~15 lines - handles screenshot D-Bus messages
}
```

### 6. handle_take_screenshot (mod.rs ~lines 2149-2189)

```rust
#[cfg(feature = "dbus")]
fn handle_take_screenshot(&mut self, ...) {
    // ~40 lines - implements screenshot capture
}
```

### 7. on_introspect_msg (mod.rs ~lines 2190-2236)

```rust
#[cfg(feature = "dbus")]
pub fn on_introspect_msg(&mut self, ...) {
    // ~45 lines - handles GNOME Shell introspect messages
}
```

### 8. on_login1_msg (mod.rs ~lines 2237-2244)

```rust
#[cfg(feature = "dbus")]
pub fn on_login1_msg(&mut self, msg: Login1ToNiri) {
    // ~8 lines - handles login1 messages (lid state)
}
```

### 9. on_locale1_msg (mod.rs ~lines 2245-2262)

```rust
#[cfg(feature = "dbus")]
pub fn on_locale1_msg(&mut self, msg: Locale1ToNiri) {
    // ~18 lines - handles locale1 messages (XKB changes)
}
```

---

## Work Units

### Unit 1: Create dbus.rs File

Create `src/niri/dbus.rs`:

```rust
//! D-Bus message handlers for the Niri compositor.
//!
//! This module handles incoming D-Bus messages for screencast, screenshot,
//! introspect, login1, and locale1 interfaces.

#[cfg(feature = "xdp-gnome-screencast")]
use crate::dbus::mutter_screen_cast::ScreenCastToNiri;
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::PwToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_locale1::Locale1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_login1::Login1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_introspect::NiriToIntrospect;
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_screenshot::NiriToScreenshot;

use super::{CastTarget, Niri, State};

// =============================================================================
// D-Bus Message Handlers (impl State)
// =============================================================================

impl State {
    // Methods will go here
}
```

---

### Unit 2: Add Module Declaration

In `mod.rs`, add:

```rust
mod config;
mod cursor;
#[cfg(feature = "dbus")]
mod dbus;
mod focus;
// ... other mods
```

---

### Unit 3: Move PipeWire/Screencast Handlers

These are feature-gated with `xdp-gnome-screencast`:

```rust
impl State {
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_pw_msg(&mut self, msg: PwToNiri) {
        // ... implementation
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    fn redraw_cast(&mut self, stream_id: usize) {
        // ... implementation
    }

    #[cfg(not(feature = "xdp-gnome-screencast"))]
    pub fn set_dynamic_cast_target(&mut self, _target: CastTarget) {}

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_dynamic_cast_target(&mut self, target: CastTarget) {
        // ... implementation
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_screen_cast_msg(&mut self, msg: ScreenCastToNiri) {
        // ... implementation
    }
}
```

**Verify**: `cargo check`

---

### Unit 4: Move Screenshot/Introspect Handlers

```rust
impl State {
    #[cfg(feature = "dbus")]
    pub fn on_screen_shot_msg(&mut self, ...) {
        // ... implementation
    }

    #[cfg(feature = "dbus")]
    fn handle_take_screenshot(&mut self, ...) {
        // ... implementation
    }

    #[cfg(feature = "dbus")]
    pub fn on_introspect_msg(&mut self, ...) {
        // ... implementation
    }
}
```

---

### Unit 5: Move login1/locale1 Handlers

```rust
impl State {
    #[cfg(feature = "dbus")]
    pub fn on_login1_msg(&mut self, msg: Login1ToNiri) {
        let Login1ToNiri::LidClosedChanged(is_closed) = msg;
        trace!("login1 lid {}", if is_closed { "closed" } else { "opened" });
        self.set_lid_closed(is_closed);
    }

    #[cfg(feature = "dbus")]
    pub fn on_locale1_msg(&mut self, msg: Locale1ToNiri) {
        // ... implementation
    }
}
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `dbus.rs` exists with module header
- [ ] All D-Bus handlers moved
- [ ] Feature gates preserved
- [ ] Module declared in mod.rs (with feature gate)
- [ ] No duplicate definitions
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/dbus.rs` | +200 lines (new file) |
| `src/niri/mod.rs` | -200 lines, +2 lines (mod declaration with cfg) |

**Expected mod.rs after P8**: ~1724 lines

---

## Technical Notes

### Feature Gates

The D-Bus handlers use two feature gates:
- `#[cfg(feature = "dbus")]` - General D-Bus support
- `#[cfg(feature = "xdp-gnome-screencast")]` - PipeWire screencast

Make sure to:
1. Gate the module import in mod.rs
2. Gate individual functions appropriately
3. Provide stub implementations for disabled features where needed

### Dependencies

Most handlers call other methods:
- `on_pw_msg` → `redraw_cast` → `queue_redraw`
- `on_screen_cast_msg` → `stop_cast`, `do_screen_transition`
- `on_screen_shot_msg` → `handle_take_screenshot` → screenshot methods
- `on_login1_msg` → `set_lid_closed`
- `on_locale1_msg` → `set_xkb_config`

---

## Next Phase

After completing this phase, proceed to [Phase P9: Final Cleanup](phase-P9-cleanup.md).
