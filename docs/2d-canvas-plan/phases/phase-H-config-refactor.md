# Phase H: Config Reload Refactor

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1.5 hours  
> **Risk Level**: 🟡 Medium  
> **Prerequisite**: Phase G complete  
> **Creates**: Cleaner config reload using subsystems

---

## Goal

Refactor `reload_config()` (~300 lines) to use the new subsystems instead of directly accessing 50+ fields.

---

## Current State

```rust
pub fn reload_config(&mut self, config: Result<Config, ()>) {
    // ~300 lines touching:
    // - cursor_manager, cursor_texture_cache
    // - output_state, monitors_active
    // - mods_with_mouse_binds, mods_with_wheel_binds
    // - screenshot_ui, hotkey_overlay
    // - And 40+ more fields directly
}
```

---

## Target Architecture

```rust
pub fn reload_config(&mut self, config: Result<Config, ()>) {
    let config = match config {
        Ok(c) => c,
        Err(()) => {
            self.niri.ui.config_error.show();
            self.niri.outputs.queue_redraw_all();
            return;
        }
    };
    
    self.niri.ui.config_error.hide();
    
    // Delegate to subsystems
    self.niri.cursor.apply_config(&config);
    self.niri.outputs.apply_config(&config);
    self.niri.input.update_from_config(&config);
    self.niri.ui.hotkey.update(&config);
    
    // Core state that remains in Niri
    self.niri.clock.set_rate(config.animations.rate());
    self.apply_keyboard_config(&config);
    self.apply_shader_config(&config);
    
    *self.niri.config.borrow_mut() = config;
}
```

---

## Work Units

### Unit 1: Add apply_config to CursorSubsystem

```rust
impl CursorSubsystem {
    pub fn apply_config(&mut self, config: &Config) {
        if self.manager.needs_reload(&config.cursor) {
            self.manager.reload(&config.cursor);
            self.texture_cache.clear();
        }
    }
}
```

### Unit 2: Add apply_config to OutputSubsystem

```rust
impl OutputSubsystem {
    pub fn apply_config(&mut self, config: &Config) {
        // Output-specific config application
    }
}
```

### Unit 3: Add update_from_config to InputTracking

Already done in Phase G.

### Unit 4: Refactor reload_config

Break into smaller helper methods:
- `apply_animation_config()`
- `apply_keyboard_config()`
- `apply_shader_config()`
- etc.

### Unit 5: Verify

---

## Verification Checklist

- [ ] Subsystems have `apply_config()` methods
- [ ] `reload_config()` delegates to subsystems
- [ ] Helper methods for remaining config
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/cursor.rs` | +apply_config() |
| `src/niri/subsystems/outputs.rs` | +apply_config() |
| `src/niri/mod.rs` or config handling | Refactored |

---

## Next Phase

After completing this phase, proceed to [Phase I: Final Cleanup](phase-I-final-cleanup.md).
