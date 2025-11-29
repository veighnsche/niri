# Phase P7: Create config.rs

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🔴 High (many config interactions)  
> **Prerequisite**: None

---

## Goal

Create new `config.rs` module for configuration reloading.

The `reload_config()` function is **~300 lines** - one of the largest functions.

Total: ~400 lines moved from mod.rs

---

## Functions to Move

All these are `impl State` methods:

### 1. reload_config (mod.rs ~lines 1231-1527) 🔴 HUGE

```rust
pub fn reload_config(&mut self, config: Result<Config, ()>) {
    // ~300 lines!
    // Handles:
    // - Config error notification
    // - Named workspace/row changes
    // - Animation settings
    // - Cursor settings
    // - Keyboard settings (xkb, repeat)
    // - Libinput settings
    // - Output config changes
    // - Window/layer rules
    // - Custom shaders
    // - Bindings/hotkeys
    // - Xwayland satellite
}
```

### 2. reload_output_config (mod.rs ~lines 1528-1623)

```rust
pub fn reload_output_config(&mut self) {
    // ~95 lines - applies output config changes
}
```

### 3. set_xkb_file (mod.rs ~lines 1191-1205)

```rust
fn set_xkb_file(&mut self, xkb_file: String) -> anyhow::Result<()> {
    // ~15 lines - loads XKB keymap from file
}
```

### 4. load_xkb_file (mod.rs ~lines 1206-1214)

```rust
fn load_xkb_file(&mut self) {
    // ~10 lines - loads XKB file on startup
}
```

### 5. set_xkb_config (mod.rs ~lines 1215-1230)

```rust
fn set_xkb_config(&mut self, xkb: XkbConfig) {
    // ~15 lines - applies XKB config
}
```

---

## Work Units

### Unit 1: Create config.rs File

Create `src/niri/config.rs`:

```rust
//! Configuration reloading for the Niri compositor.
//!
//! This module handles applying configuration changes at runtime,
//! including output settings, keyboard settings, window rules, and more.

use std::mem;
use std::path::PathBuf;

use niri_config::{Config, TrackLayout, Xkb};
use smithay::input::keyboard::XkbConfig;

use crate::input::{
    apply_libinput_settings, mods_with_finger_scroll_binds, 
    mods_with_mouse_binds, mods_with_wheel_binds,
};
use crate::render_helpers::shaders;
use crate::utils::spawning::{CHILD_DISPLAY, CHILD_ENV};
use crate::utils::{expand_home, xwayland};

use super::{Niri, State};

// =============================================================================
// Configuration Methods (impl State)
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
mod focus;
// ... other mods
```

---

### Unit 3: Move Keyboard Config Methods First

Start with the simpler keyboard methods:

```rust
impl State {
    /// Loads XKB keymap from a file.
    fn set_xkb_file(&mut self, xkb_file: String) -> anyhow::Result<()> {
        // ... implementation
    }

    /// Loads the xkb_file config option if set.
    fn load_xkb_file(&mut self) {
        // ... implementation
    }

    /// Applies XKB configuration.
    fn set_xkb_config(&mut self, xkb: XkbConfig) {
        // ... implementation
    }
}
```

**Verify**: `cargo check`

---

### Unit 4: Move reload_output_config

```rust
impl State {
    /// Reloads output configuration.
    pub fn reload_output_config(&mut self) {
        // ~95 lines
        // Handles scale, transform, VRR, backdrop color changes
    }
}
```

**Verify**: `cargo check`

---

### Unit 5: Move reload_config

This is the big one. Move section by section:

```rust
impl State {
    /// Reloads the entire configuration.
    pub fn reload_config(&mut self, config: Result<Config, ()>) {
        let _span = tracy_client::span!("State::reload_config");

        // Handle config error
        let mut config = match config {
            Ok(config) => config,
            Err(()) => {
                self.niri.config_error_notification.show();
                self.niri.queue_redraw_all();
                return;
            }
        };

        self.niri.config_error_notification.hide();

        // ... rest of the ~280 lines of config handling
    }
}
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `config.rs` exists with module header
- [ ] `reload_config` moved (~300 lines)
- [ ] `reload_output_config` moved (~95 lines)
- [ ] Keyboard config methods moved
- [ ] Module declared in mod.rs
- [ ] No duplicate definitions
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/config.rs` | +400 lines (new file) |
| `src/niri/mod.rs` | -400 lines, +1 line (mod declaration) |

**Expected mod.rs after P7**: ~1924 lines

---

## Technical Notes

### reload_config Sections

The function has distinct sections that could theoretically be split further:

1. **Config validation** (~20 lines) - Error handling, show/hide notification
2. **Row/workspace changes** (~15 lines) - Named row removal/creation
3. **Animation settings** (~10 lines) - Clock rate, off mode
4. **Environment** (~5 lines) - Child process env
5. **Cursor settings** (~10 lines) - Theme, size reload
6. **Keyboard settings** (~30 lines) - XKB, repeat rate/delay
7. **Input device settings** (~10 lines) - Libinput config
8. **Output settings** (~20 lines) - Detect changes, reload
9. **Binding changes** (~20 lines) - Hotkey overlay, modifier tracking
10. **Window/layer rules** (~15 lines) - Rule recomputation
11. **Shader changes** (~30 lines) - Custom shader reload
12. **Cursor inactivity** (~10 lines) - Timer reset
13. **Recent windows** (~5 lines) - MRU config
14. **Xwayland** (~30 lines) - Satellite setup

### Dependencies

**Backend access**: 
- `self.backend.mod_key(&config)`
- `self.backend.with_primary_renderer()`
- `self.backend.update_ignored_nodes_config()`

**Niri field access** (partial list):
- `config`, `config_error_notification`
- `layout`, `mapped_layer_surfaces`
- `clock`, `cursor_manager`, `cursor_texture_cache`
- `seat`, `devices`, `hotkey_overlay`
- `mods_with_*_binds`, `window_mru_ui`
- `satellite`, `xkb_from_locale1`

---

## Next Phase

After completing this phase, proceed to [Phase P8: Create dbus.rs](phase-P8-dbus.md).
