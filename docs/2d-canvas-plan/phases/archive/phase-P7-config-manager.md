# Phase P7: Config Reload Refactoring

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🟡 Medium (many interactions)  
> **Prerequisite**: Phase P6 complete  
> **Creates**: Cleaner config reload structure

---

## Goal

Refactor the massive `reload_config()` function (~300 lines) into:
- **Smaller, focused helper methods**
- **Clear separation of config sections**
- **Better use of subsystems** from previous phases

This phase does NOT create a new subsystem, but refactors config handling to use the subsystems we've created.

---

## Why Refactor, Not Extract?

Config reload touches almost everything:
- Output subsystem
- Cursor subsystem
- UI overlays
- Layout
- Input devices
- Keyboard settings

Creating a "ConfigManager" subsystem would just move the problem. Instead, we:
1. Break the monster function into smaller methods
2. Delegate to subsystems where appropriate
3. Keep coordination in `State::reload_config()`

---

## Current State: The Monster Function

```rust
pub fn reload_config(&mut self, config: Result<Config, ()>) {
    // ~300 lines handling:
    // 1. Error notification
    // 2. Named row changes
    // 3. Animation settings
    // 4. Environment variables
    // 5. Cursor settings
    // 6. Keyboard settings (XKB, repeat)
    // 7. Input device settings
    // 8. Output config
    // 9. Binding changes
    // 10. Window/layer rules
    // 11. Custom shaders
    // 12. Cursor inactivity timer
    // 13. MRU config
    // 14. Xwayland satellite
}
```

---

## Target Architecture

### New File: `src/niri/config.rs`

```rust
//! Configuration reloading for the Niri compositor.
//!
//! This module contains the config reload logic, broken into
//! focused helper methods for each config section.

use niri_config::Config;

use super::State;

impl State {
    /// Main config reload entry point.
    pub fn reload_config(&mut self, config: Result<Config, ()>) {
        let _span = tracy_client::span!("State::reload_config");

        // Handle config error
        let config = match self.handle_config_result(config) {
            Some(c) => c,
            None => return,
        };

        // Apply config sections
        self.apply_animation_config(&config);
        self.apply_cursor_config(&config);
        self.apply_keyboard_config(&config);
        self.apply_input_device_config(&config);
        self.apply_output_config_changes(&config);
        self.apply_binding_config(&config);
        self.apply_window_rules(&config);
        self.apply_shader_config(&config);
        self.apply_misc_config(&config);

        // Store the new config
        *self.niri.config.borrow_mut() = config;
    }

    // =========================================================================
    // Config Sections (Private Helpers)
    // =========================================================================

    /// Handles config parse result, shows error notification if needed.
    fn handle_config_result(&mut self, config: Result<Config, ()>) -> Option<Config> {
        match config {
            Ok(config) => {
                self.niri.ui.hide_config_error();
                Some(config)
            }
            Err(()) => {
                self.niri.ui.show_config_error();
                self.niri.outputs.queue_redraw_all();
                None
            }
        }
    }

    /// Applies animation configuration changes.
    fn apply_animation_config(&mut self, config: &Config) {
        // Animation speed, off mode
        self.niri.clock.set_rate(config.animations.rate());
        // ... ~10 lines
    }

    /// Applies cursor configuration changes.
    fn apply_cursor_config(&mut self, config: &Config) {
        // Cursor theme, size
        let cursor_config = &config.cursor;
        if self.niri.cursor.manager().needs_reload(cursor_config) {
            self.niri.cursor.manager_mut().reload(cursor_config);
            self.niri.cursor.texture_cache_mut().clear();
        }

        // Inactivity timer
        self.update_cursor_inactivity_timer(config);
        // ... ~20 lines
    }

    /// Applies keyboard configuration changes.
    fn apply_keyboard_config(&mut self, config: &Config) {
        // XKB settings
        if let Some(xkb) = &config.input.keyboard.xkb {
            self.apply_xkb_config(xkb);
        }

        // Repeat rate/delay
        self.apply_keyboard_repeat(config);
        // ... ~30 lines
    }

    /// Applies XKB configuration.
    fn apply_xkb_config(&mut self, xkb: &niri_config::Xkb) {
        // Convert to smithay XkbConfig and apply
        // ... ~15 lines
    }

    /// Applies keyboard repeat settings.
    fn apply_keyboard_repeat(&mut self, config: &Config) {
        // Repeat rate, delay
        // ... ~10 lines
    }

    /// Applies input device configuration.
    fn apply_input_device_config(&mut self, config: &Config) {
        use crate::input::apply_libinput_settings;
        
        for device in &self.niri.devices {
            apply_libinput_settings(&config.input, device);
        }
        // ... ~10 lines
    }

    /// Detects and applies output configuration changes.
    fn apply_output_config_changes(&mut self, config: &Config) {
        // Check if output config changed
        if self.niri.config_file_output_config != config.outputs {
            self.niri.config_file_output_config = config.outputs.clone();
            self.reload_output_config();
        }
    }

    /// Applies binding/hotkey configuration.
    fn apply_binding_config(&mut self, config: &Config) {
        use crate::input::{mods_with_mouse_binds, mods_with_wheel_binds, mods_with_finger_scroll_binds};
        
        // Update hotkey overlay
        self.niri.ui.hotkey.update(config);
        
        // Update modifier tracking sets
        self.niri.mods_with_mouse_binds = mods_with_mouse_binds(config);
        self.niri.mods_with_wheel_binds = mods_with_wheel_binds(config);
        self.niri.mods_with_finger_scroll_binds = mods_with_finger_scroll_binds(config);
        // ... ~20 lines
    }

    /// Applies window and layer rules.
    fn apply_window_rules(&mut self, config: &Config) {
        // Refresh window rules for all mapped windows
        self.niri.refresh_window_rules();
        
        // Refresh layer surface rules
        for (_, layer) in &mut self.niri.mapped_layer_surfaces {
            layer.recompute_rule(config);
        }
        // ... ~15 lines
    }

    /// Applies custom shader configuration.
    fn apply_shader_config(&mut self, config: &Config) {
        use crate::render_helpers::shaders;
        
        self.backend.with_primary_renderer(|renderer| {
            shaders::reload_custom_shaders(renderer, config);
        });
        // ... ~20 lines
    }

    /// Applies miscellaneous configuration.
    fn apply_misc_config(&mut self, config: &Config) {
        // Environment variables
        // MRU configuration
        // Xwayland satellite
        // ... ~30 lines
    }

    /// Updates cursor inactivity timer based on config.
    fn update_cursor_inactivity_timer(&mut self, config: &Config) {
        // Cancel existing timer if config changed
        // Start new timer if enabled
        // ... ~15 lines
    }
}
```

---

## Work Units

### Unit 1: Create config.rs File

Create `src/niri/config.rs` with module declaration in `mod.rs`.

---

### Unit 2: Extract handle_config_result

First, extract the simple error handling:

```rust
fn handle_config_result(&mut self, config: Result<Config, ()>) -> Option<Config> {
    match config {
        Ok(config) => {
            self.niri.ui.hide_config_error();
            Some(config)
        }
        Err(()) => {
            self.niri.ui.show_config_error();
            self.niri.outputs.queue_redraw_all();
            None
        }
    }
}
```

**Verify**: `cargo check`

---

### Unit 3: Extract apply_animation_config

```rust
fn apply_animation_config(&mut self, config: &Config) {
    self.niri.clock.set_rate(config.animations.rate());
    // ...
}
```

---

### Unit 4: Extract apply_cursor_config

This uses the `CursorSubsystem` from Phase P3:

```rust
fn apply_cursor_config(&mut self, config: &Config) {
    let cursor_config = &config.cursor;
    if self.niri.cursor.manager().needs_reload(cursor_config) {
        self.niri.cursor.manager_mut().reload(cursor_config);
        self.niri.cursor.texture_cache_mut().clear();
    }
    // ...
}
```

---

### Unit 5: Extract Remaining Sections

Continue extracting each section:
1. `apply_keyboard_config`
2. `apply_input_device_config`
3. `apply_output_config_changes`
4. `apply_binding_config`
5. `apply_window_rules`
6. `apply_shader_config`
7. `apply_misc_config`

---

### Unit 6: Move XKB Helper Methods

Move the existing XKB methods to config.rs:
- `set_xkb_file`
- `load_xkb_file`
- `set_xkb_config`

---

### Unit 7: Move reload_output_config

Move `reload_output_config()` to config.rs as well.

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `config.rs` exists with helper methods
- [ ] `reload_config` refactored to call helpers
- [ ] Each config section in its own method
- [ ] Uses subsystems (`ui`, `outputs`, `cursor`) appropriately
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/config.rs` | +400 lines (new) |
| `src/niri/mod.rs` | -350 lines, +1 line (mod declaration) |

---

## Benefits Achieved

1. **Readable**: Each config section clearly named
2. **Testable**: Individual sections can be tested
3. **Maintainable**: Easy to add new config options
4. **Uses subsystems**: Delegates to proper owners
5. **Clear flow**: Main function shows the order of operations

---

## Next Phase

After completing this phase, proceed to [Phase P8: State Context Pattern](phase-P8-state-context.md).
