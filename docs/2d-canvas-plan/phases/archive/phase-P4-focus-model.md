# Phase P4: Extract FocusModel

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~2 hours  
> **Risk Level**: 🔴 High (complex focus logic)  
> **Prerequisite**: Phase P3 complete  
> **Creates**: `FocusModel` struct

---

## Goal

Extract focus-related state and the complex focus computation logic into a dedicated `FocusModel` that:
- **Owns** focus state (current focus, layer focus, inhibitors)
- **Encapsulates** the focus priority/computation logic
- **Can be tested** in isolation (key goal!)
- **Simplifies** the massive `update_keyboard_focus()` function

---

## Why This Is High Risk

The `update_keyboard_focus()` function is **~260 lines** with:
- Multiple nested closures capturing `self`
- Complex priority logic (layer shell > layout > lock > UI)
- Focus change side effects (MRU timestamps, keyboard layout)
- Popup grab handling

This is the most complex refactor but also the highest value.

---

## Current State Analysis

### Fields to Move from Niri

```rust
pub keyboard_focus: KeyboardFocus,                    // Current focus target
pub layer_shell_on_demand_focus: Option<LayerSurface>, // On-demand layer focus
pub idle_inhibiting_surfaces: HashSet<WlSurface>,     // Surfaces inhibiting idle
pub keyboard_shortcuts_inhibiting_surfaces: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
```

### The Monster Function (mod.rs ~lines 932-1190)

```rust
pub fn update_keyboard_focus(&mut self) {
    // ~260 lines handling:
    // 1. Layer shell on-demand focus cleanup
    // 2. Focus computation with priority:
    //    - Exit confirm dialog
    //    - Lock screen
    //    - Screenshot UI
    //    - Window MRU UI
    //    - Layer shell (exclusive > on-demand)
    //    - Layout windows
    // 3. Focus change handling
    // 4. MRU timestamp tracking
    // 5. Popup grab management
    // 6. Per-window keyboard layout
}
```

---

## Target Architecture

### Focus Priority Model

```rust
/// Focus priority order (highest to lowest):
/// 1. Exit confirm dialog (modal)
/// 2. Lock screen (security)
/// 3. Screenshot UI (modal)
/// 4. Window MRU UI (modal)
/// 5. Layer shell exclusive (overlay > top)
/// 6. Layer shell on-demand
/// 7. Layout windows
/// 8. No focus
pub enum FocusPriority {
    ExitDialog,
    LockScreen,
    ScreenshotUi,
    MruUi,
    LayerExclusive,
    LayerOnDemand,
    Layout,
    None,
}
```

### New File: `src/niri/subsystems/focus.rs`

```rust
//! Keyboard focus management subsystem.
//!
//! Handles focus computation, tracking, and the various focus priorities
//! (layer shell, layout windows, modal dialogs, etc.).

use std::collections::{HashMap, HashSet};

use smithay::desktop::LayerSurface;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor;

use super::super::types::KeyboardFocus;

/// Keyboard focus management subsystem.
///
/// Tracks the current keyboard focus, computes focus based on priority,
/// and manages focus-related state like idle inhibitors.
pub struct FocusModel {
    /// Current keyboard focus target.
    current: KeyboardFocus,
    
    /// Layer surface with on-demand focus (if any).
    layer_on_demand: Option<LayerSurface>,
    
    /// Surfaces that are inhibiting idle.
    idle_inhibitors: HashSet<WlSurface>,
    
    /// Surfaces that are inhibiting keyboard shortcuts.
    shortcut_inhibitors: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
}

impl FocusModel {
    /// Creates a new focus model.
    pub fn new() -> Self {
        Self {
            current: KeyboardFocus::Layout { surface: None },
            layer_on_demand: None,
            idle_inhibitors: HashSet::new(),
            shortcut_inhibitors: HashMap::new(),
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
    
    /// Returns true if focus changed.
    pub fn update_if_changed(&mut self, new_focus: KeyboardFocus) -> bool {
        if self.current != new_focus {
            self.current = new_focus;
            true
        } else {
            false
        }
    }
    
    // =========================================================================
    // Focus Computation
    // =========================================================================
    
    /// Computes what should have keyboard focus based on current state.
    ///
    /// This encapsulates the focus priority logic that was in update_keyboard_focus().
    /// The caller provides the state needed for the computation.
    pub fn compute_focus(&self, ctx: &FocusContext) -> KeyboardFocus {
        // Priority 1: Exit confirm dialog
        if ctx.exit_dialog_open {
            return KeyboardFocus::ExitConfirmDialog;
        }
        
        // Priority 2: Lock screen
        if ctx.is_locked {
            return ctx.lock_focus.clone();
        }
        
        // Priority 3: Screenshot UI
        if ctx.screenshot_ui_open {
            return KeyboardFocus::ScreenshotUi;
        }
        
        // Priority 4: MRU UI
        if ctx.mru_ui_open {
            return KeyboardFocus::WindowMruUi;
        }
        
        // Priority 5: Layer shell exclusive focus
        if let Some(focus) = self.exclusive_layer_focus(ctx) {
            return focus;
        }
        
        // Priority 6: Layer shell on-demand focus
        if let Some(ref surface) = self.layer_on_demand {
            if surface.alive() {
                return KeyboardFocus::LayerShell {
                    surface: surface.wl_surface().clone(),
                };
            }
        }
        
        // Priority 7: Layout focus
        ctx.layout_focus.clone()
    }
    
    /// Finds exclusive layer shell focus (overlay, then top layer).
    fn exclusive_layer_focus(&self, ctx: &FocusContext) -> Option<KeyboardFocus> {
        // Check overlay layer first
        for layer in &ctx.overlay_layers {
            if layer.can_receive_keyboard_focus() {
                return Some(KeyboardFocus::LayerShell {
                    surface: layer.wl_surface().clone(),
                });
            }
        }
        
        // Then top layer
        for layer in &ctx.top_layers {
            if layer.can_receive_keyboard_focus() {
                return Some(KeyboardFocus::LayerShell {
                    surface: layer.wl_surface().clone(),
                });
            }
        }
        
        None
    }
    
    // =========================================================================
    // Layer Shell On-Demand Focus
    // =========================================================================
    
    /// Returns the current on-demand layer focus.
    pub fn layer_on_demand(&self) -> Option<&LayerSurface> {
        self.layer_on_demand.as_ref()
    }
    
    /// Sets on-demand layer focus.
    pub fn set_layer_on_demand(&mut self, surface: Option<LayerSurface>) {
        self.layer_on_demand = surface;
    }
    
    /// Clears on-demand focus if the surface is no longer valid.
    pub fn cleanup_layer_on_demand(&mut self) {
        if let Some(ref surface) = self.layer_on_demand {
            if !surface.alive() {
                self.layer_on_demand = None;
            }
        }
    }
    
    // =========================================================================
    // Idle Inhibitors
    // =========================================================================
    
    /// Returns whether idle is inhibited.
    pub fn is_idle_inhibited(&self) -> bool {
        !self.idle_inhibitors.is_empty()
    }
    
    /// Adds an idle inhibitor surface.
    pub fn add_idle_inhibitor(&mut self, surface: WlSurface) {
        self.idle_inhibitors.insert(surface);
    }
    
    /// Removes an idle inhibitor surface.
    pub fn remove_idle_inhibitor(&mut self, surface: &WlSurface) {
        self.idle_inhibitors.remove(surface);
    }
    
    /// Returns the idle inhibitors set.
    pub fn idle_inhibitors(&self) -> &HashSet<WlSurface> {
        &self.idle_inhibitors
    }
    
    /// Returns mutable access to idle inhibitors.
    pub fn idle_inhibitors_mut(&mut self) -> &mut HashSet<WlSurface> {
        &mut self.idle_inhibitors
    }
    
    // =========================================================================
    // Keyboard Shortcuts Inhibitors
    // =========================================================================
    
    /// Returns whether keyboard shortcuts are inhibited for a surface.
    pub fn are_shortcuts_inhibited(&self, surface: &WlSurface) -> bool {
        self.shortcut_inhibitors.contains_key(surface)
    }
    
    /// Adds a shortcuts inhibitor.
    pub fn add_shortcut_inhibitor(&mut self, surface: WlSurface, inhibitor: KeyboardShortcutsInhibitor) {
        self.shortcut_inhibitors.insert(surface, inhibitor);
    }
    
    /// Removes a shortcuts inhibitor.
    pub fn remove_shortcut_inhibitor(&mut self, surface: &WlSurface) {
        self.shortcut_inhibitors.remove(surface);
    }
    
    /// Returns the shortcuts inhibitors map.
    pub fn shortcut_inhibitors(&self) -> &HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &self.shortcut_inhibitors
    }
    
    /// Returns mutable access to shortcuts inhibitors.
    pub fn shortcut_inhibitors_mut(&mut self) -> &mut HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &mut self.shortcut_inhibitors
    }
}

impl Default for FocusModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Context needed for focus computation.
///
/// This is passed to `FocusModel::compute_focus()` to avoid the subsystem
/// needing access to the entire Niri state.
pub struct FocusContext<'a> {
    pub exit_dialog_open: bool,
    pub is_locked: bool,
    pub lock_focus: KeyboardFocus,
    pub screenshot_ui_open: bool,
    pub mru_ui_open: bool,
    pub overlay_layers: Vec<&'a LayerSurface>,
    pub top_layers: Vec<&'a LayerSurface>,
    pub layout_focus: KeyboardFocus,
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
pub use outputs::OutputSubsystem;
```

---

### Unit 2: Create FocusModel Struct

Create `src/niri/subsystems/focus.rs` with:
1. `FocusModel` struct with private fields
2. `FocusContext` struct for computation inputs
3. Basic accessors

**Verify**: `cargo check`

---

### Unit 3: Move Fields from Niri

1. Remove focus-related fields from `Niri` struct
2. Add `pub focus: FocusModel` field
3. Update `Niri::new` to create `FocusModel`

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Implement compute_focus()

Implement the focus priority logic in `FocusModel::compute_focus()`.

This is the core logic extraction from `update_keyboard_focus()`.

---

### Unit 5: Refactor update_keyboard_focus()

Transform the monster function to use the subsystem:

```rust
// Before: 260 lines of interleaved logic
impl State {
    pub fn update_keyboard_focus(&mut self) {
        // ... huge function
    }
}

// After: Coordination using subsystem
impl State {
    pub fn update_keyboard_focus(&mut self) {
        // Cleanup stale state
        self.niri.focus.cleanup_layer_on_demand();
        
        // Build context for focus computation
        let ctx = FocusContext {
            exit_dialog_open: self.niri.ui.exit_dialog.is_open(),
            is_locked: self.niri.is_locked(),
            lock_focus: self.compute_lock_focus(),
            screenshot_ui_open: self.niri.ui.screenshot.is_open(),
            mru_ui_open: self.niri.ui.mru.is_open(),
            overlay_layers: self.collect_overlay_layers(),
            top_layers: self.collect_top_layers(),
            layout_focus: self.compute_layout_focus(),
        };
        
        // Compute new focus
        let new_focus = self.niri.focus.compute_focus(&ctx);
        
        // Handle focus change
        if self.niri.focus.update_if_changed(new_focus) {
            self.handle_focus_change();
        }
    }
}
```

---

### Unit 6: Add Unit Tests

Key benefit: Focus logic is now testable!

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exit_dialog_takes_priority() {
        let focus = FocusModel::new();
        let ctx = FocusContext {
            exit_dialog_open: true,
            is_locked: false,
            // ... other fields
        };
        
        assert_eq!(focus.compute_focus(&ctx), KeyboardFocus::ExitConfirmDialog);
    }
    
    #[test]
    fn test_lock_screen_priority() {
        let focus = FocusModel::new();
        let ctx = FocusContext {
            exit_dialog_open: false,
            is_locked: true,
            lock_focus: KeyboardFocus::LockScreen { surface: None },
            // ...
        };
        
        assert_eq!(focus.compute_focus(&ctx), KeyboardFocus::LockScreen { surface: None });
    }
}
```

---

## Verification Checklist

- [ ] `FocusModel` struct exists with private fields
- [ ] `FocusContext` struct for computation inputs
- [ ] Focus fields removed from `Niri`
- [ ] `Niri.focus: FocusModel` field added
- [ ] `compute_focus()` implements priority logic
- [ ] `update_keyboard_focus()` refactored to use subsystem
- [ ] Unit tests for focus priority
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/focus.rs` | +350 lines (new) |
| `src/niri/subsystems/mod.rs` | +3 lines |
| `src/niri/mod.rs` | -10 lines (fields), +2 lines (field) |
| `src/niri/mod.rs` | Refactored `update_keyboard_focus()` |

---

## Benefits Achieved

1. **Testable focus logic**: Can test priority without full compositor
2. **Clear priority model**: `FocusContext` makes inputs explicit
3. **Encapsulation**: Focus state owned by subsystem
4. **Simplified monster function**: Coordination only, not computation
5. **Reduced Niri complexity**: 4 fewer fields

---

## Next Phase

After completing this phase, proceed to [Phase P5: StreamingSubsystem](phase-P5-streaming.md).
