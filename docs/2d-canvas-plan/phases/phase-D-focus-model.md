# Phase D: Extract FocusModel

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~2 hours  
> **Risk Level**: 🔴 High (complex focus logic)  
> **Prerequisite**: Phase C complete  
> **Creates**: `FocusModel` struct

---

## Goal

Extract focus-related state and the complex `update_keyboard_focus()` logic into a `FocusModel` that:
- **Owns** focus state
- **Encapsulates** focus priority rules
- **Can be unit tested** without full compositor

---

## Why This Is High Risk

The `update_keyboard_focus()` function is:
- **~260 lines** of complex logic
- Handles **6+ focus targets** with priority rules
- Uses **nested closures** that capture `self`
- **Tightly coupled** to layer-shell, layout, lock screen, UI overlays

Careful refactoring is required.

---

## Fields to Move from Niri

```rust
// Focus state (mod.rs lines ~348-352)
pub keyboard_focus: KeyboardFocus,
pub layer_shell_on_demand_focus: Option<LayerSurface>,
pub idle_inhibiting_surfaces: HashSet<WlSurface>,
pub is_fdo_idle_inhibited: Arc<AtomicBool>,
pub keyboard_shortcuts_inhibiting_surfaces: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
```

---

## Target Architecture

### New File: `src/niri/subsystems/focus.rs`

```rust
//! Focus management subsystem.
//!
//! Handles keyboard focus computation with priority rules:
//! 1. Exit confirm dialog
//! 2. Lock screen
//! 3. Screenshot UI
//! 4. MRU window switcher
//! 5. Layer shell (exclusive → on-demand)
//! 6. Layout windows

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use smithay::desktop::LayerSurface;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor;

use crate::niri::types::KeyboardFocus;

/// Focus computation context.
///
/// Provides the data needed to compute focus without accessing all of Niri.
pub struct FocusContext<'a> {
    pub exit_dialog_open: bool,
    pub is_locked: bool,
    pub screenshot_ui_open: bool,
    pub mru_ui_open: bool,
    pub active_output: Option<&'a smithay::output::Output>,
    pub layout_focus: Option<WlSurface>,
    pub layer_map: Option<&'a smithay::desktop::LayerMap>,
}

/// Focus management subsystem.
pub struct FocusModel {
    /// Current keyboard focus.
    current: KeyboardFocus,
    
    /// Layer-shell surface with on-demand focus.
    layer_on_demand: Option<LayerSurface>,
    
    /// Surfaces inhibiting idle.
    idle_inhibiting: HashSet<WlSurface>,
    
    /// FreeDesktop idle inhibition flag.
    fdo_idle_inhibited: Arc<AtomicBool>,
    
    /// Surfaces inhibiting keyboard shortcuts.
    shortcuts_inhibiting: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
}

impl FocusModel {
    /// Creates a new focus model.
    pub fn new() -> Self {
        Self {
            current: KeyboardFocus::Layout { surface: None },
            layer_on_demand: None,
            idle_inhibiting: HashSet::new(),
            fdo_idle_inhibited: Arc::new(AtomicBool::new(false)),
            shortcuts_inhibiting: HashMap::new(),
        }
    }
    
    // =========================================================================
    // Current Focus
    // =========================================================================
    
    /// Returns the current keyboard focus.
    pub fn current(&self) -> &KeyboardFocus {
        &self.current
    }
    
    /// Sets the current keyboard focus.
    pub fn set_current(&mut self, focus: KeyboardFocus) {
        self.current = focus;
    }
    
    // =========================================================================
    // Focus Computation
    // =========================================================================
    
    /// Computes what should be focused based on priority rules.
    ///
    /// Priority order:
    /// 1. Exit confirm dialog
    /// 2. Lock screen
    /// 3. Screenshot UI
    /// 4. MRU window switcher
    /// 5. Layer shell (exclusive keyboards)
    /// 6. Layer shell (on-demand)
    /// 7. Layout windows
    pub fn compute(&self, ctx: &FocusContext<'_>) -> KeyboardFocus {
        // Priority 1: Exit confirm dialog
        if ctx.exit_dialog_open {
            return KeyboardFocus::ExitConfirmDialog;
        }
        
        // Priority 2: Lock screen
        if ctx.is_locked {
            return KeyboardFocus::LockScreen;
        }
        
        // Priority 3: Screenshot UI
        if ctx.screenshot_ui_open {
            return KeyboardFocus::ScreenshotUi;
        }
        
        // Priority 4: MRU window switcher
        if ctx.mru_ui_open {
            return KeyboardFocus::WindowMruUi;
        }
        
        // Priority 5-7: Layer shell and layout
        if let Some(output) = ctx.active_output {
            if let Some(layer_map) = ctx.layer_map {
                // Check exclusive layer shell focus
                if let Some(focus) = self.exclusive_layer_focus(layer_map) {
                    return focus;
                }
                
                // Check on-demand layer focus
                if let Some(ref layer) = self.layer_on_demand {
                    if layer.alive() {
                        return KeyboardFocus::LayerShell {
                            surface: layer.wl_surface().clone(),
                        };
                    }
                }
            }
        }
        
        // Priority 7: Layout windows
        KeyboardFocus::Layout {
            surface: ctx.layout_focus.clone(),
        }
    }
    
    /// Finds exclusive layer-shell focus.
    fn exclusive_layer_focus(&self, layer_map: &smithay::desktop::LayerMap) -> Option<KeyboardFocus> {
        use smithay::wayland::shell::wlr_layer::Layer;
        
        // Check layers in priority order: Overlay, Top, Bottom, Background
        for layer in [Layer::Overlay, Layer::Top, Layer::Bottom, Layer::Background] {
            for surface in layer_map.layers_on(layer) {
                let data = surface.cached_state();
                if data.keyboard_interactivity == 
                   smithay::wayland::shell::wlr_layer::KeyboardInteractivity::Exclusive 
                {
                    return Some(KeyboardFocus::LayerShell {
                        surface: surface.wl_surface().clone(),
                    });
                }
            }
        }
        None
    }
    
    // =========================================================================
    // Layer On-Demand Focus
    // =========================================================================
    
    /// Returns the on-demand layer focus.
    pub fn layer_on_demand(&self) -> Option<&LayerSurface> {
        self.layer_on_demand.as_ref()
    }
    
    /// Sets on-demand layer focus.
    pub fn set_layer_on_demand(&mut self, surface: Option<LayerSurface>) {
        self.layer_on_demand = surface;
    }
    
    /// Clears on-demand focus if the surface is dead.
    pub fn cleanup_dead_on_demand(&mut self) {
        if let Some(ref surface) = self.layer_on_demand {
            if !surface.alive() {
                self.layer_on_demand = None;
            }
        }
    }
    
    // =========================================================================
    // Idle Inhibition
    // =========================================================================
    
    /// Returns idle inhibiting surfaces.
    pub fn idle_inhibiting(&self) -> &HashSet<WlSurface> {
        &self.idle_inhibiting
    }
    
    /// Returns mutable idle inhibiting surfaces.
    pub fn idle_inhibiting_mut(&mut self) -> &mut HashSet<WlSurface> {
        &mut self.idle_inhibiting
    }
    
    /// Returns the FreeDesktop idle inhibited flag.
    pub fn fdo_idle_inhibited(&self) -> &Arc<AtomicBool> {
        &self.fdo_idle_inhibited
    }
    
    // =========================================================================
    // Keyboard Shortcuts Inhibition
    // =========================================================================
    
    /// Returns shortcuts inhibiting surfaces.
    pub fn shortcuts_inhibiting(&self) -> &HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &self.shortcuts_inhibiting
    }
    
    /// Returns mutable shortcuts inhibiting surfaces.
    pub fn shortcuts_inhibiting_mut(&mut self) -> &mut HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &mut self.shortcuts_inhibiting
    }
}

impl Default for FocusModel {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Work Units

### Unit 1: Add FocusModel to subsystems/mod.rs

```rust
mod cursor;
mod focus;
mod outputs;

pub use cursor::CursorSubsystem;
pub use focus::{FocusModel, FocusContext};
pub use outputs::{OutputSubsystem, OutputState};
```

---

### Unit 2: Create FocusModel struct

Create basic struct with fields and accessors.

**Verify**: `cargo check`

---

### Unit 3: Implement compute() method

Extract the focus priority logic from `update_keyboard_focus()`.

---

### Unit 4: Move fields from Niri

1. Remove focus-related fields from `Niri` struct
2. Add `pub focus: FocusModel` field
3. Update `Niri::new()`

---

### Unit 5: Refactor update_keyboard_focus()

In `State` (or wherever it lives), refactor to use FocusModel:

```rust
pub fn update_keyboard_focus(&mut self) {
    // Clean up dead on-demand focus
    self.niri.focus.cleanup_dead_on_demand();
    
    // Build context
    let ctx = FocusContext {
        exit_dialog_open: self.niri.ui.is_exit_dialog_open(),
        is_locked: self.niri.is_locked(),
        screenshot_ui_open: self.niri.ui.is_screenshot_open(),
        mru_ui_open: self.niri.ui.is_mru_open(),
        active_output: self.niri.layout.active_output(),
        layout_focus: self.niri.layout.focus_surface(),
        layer_map: self.niri.layout.active_output()
            .map(|o| layer_map_for_output(o)),
    };
    
    // Compute new focus
    let new_focus = self.niri.focus.compute(&ctx);
    
    // Handle focus change
    if self.niri.focus.current() != &new_focus {
        // ... focus change handling
        self.niri.focus.set_current(new_focus);
    }
}
```

---

### Unit 6: Add unit tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exit_dialog_priority() {
        let focus = FocusModel::new();
        let ctx = FocusContext {
            exit_dialog_open: true,
            is_locked: false,
            // ...
        };
        assert!(matches!(focus.compute(&ctx), KeyboardFocus::ExitConfirmDialog));
    }
    
    #[test]
    fn test_lock_screen_priority() {
        // ...
    }
}
```

---

### Unit 7: Verify

```bash
cargo check
cargo test
```

---

## Verification Checklist

- [ ] `FocusModel` struct with private fields
- [ ] `FocusContext` for testable computation
- [ ] `compute()` method implements priority rules
- [ ] Focus fields removed from Niri
- [ ] `update_keyboard_focus()` refactored
- [ ] Unit tests for focus priority
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/focus.rs` | **NEW** ~350 lines |
| `src/niri/subsystems/mod.rs` | +2 lines |
| `src/niri/mod.rs` | -5 fields |
| Focus handling code | Refactored |

---

## Benefits

1. **-5 fields** from Niri struct
2. **Testable** focus logic
3. **Clear priority rules** in one place
4. **Decoupled** from UI overlays

---

## Next Phase

After completing this phase, proceed to [Phase E: StreamingSubsystem](phase-E-streaming-subsystem.md).
