# Phase P4a: Extract Focus State Container

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low (mechanical extraction)  
> **Prerequisite**: Phase P3 complete  
> **Creates**: `FocusState` struct (data only)

---

## Goal

Extract focus-related **fields only** into a `FocusState` container. This is a low-risk mechanical extraction that does NOT touch the complex `update_keyboard_focus()` logic yet.

---

## Fields to Move from Niri

```rust
// Focus state (mod.rs)
pub keyboard_focus: KeyboardFocus,
pub layer_shell_on_demand_focus: Option<LayerSurface>,
pub idle_inhibiting_surfaces: HashSet<WlSurface>,
pub keyboard_shortcuts_inhibiting_surfaces: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
```

---

## Target Architecture

### New File: `src/niri/subsystems/focus.rs`

```rust
//! Keyboard focus state container.
//!
//! Phase P4a: Data container only. Logic extraction happens in P4b-P4d.

use std::collections::{HashMap, HashSet};

use smithay::desktop::LayerSurface;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor;

use super::super::types::KeyboardFocus;

/// Keyboard focus state container.
pub struct FocusState {
    /// Current keyboard focus target.
    current: KeyboardFocus,
    
    /// Layer surface with on-demand focus (if any).
    layer_on_demand: Option<LayerSurface>,
    
    /// Surfaces that are inhibiting idle.
    idle_inhibitors: HashSet<WlSurface>,
    
    /// Surfaces that are inhibiting keyboard shortcuts.
    shortcut_inhibitors: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
}

impl FocusState {
    pub fn new() -> Self {
        Self {
            current: KeyboardFocus::Layout { surface: None },
            layer_on_demand: None,
            idle_inhibitors: HashSet::new(),
            shortcut_inhibitors: HashMap::new(),
        }
    }
    
    // =========================================================================
    // Current Focus (simple accessors)
    // =========================================================================
    
    pub fn current(&self) -> &KeyboardFocus {
        &self.current
    }
    
    pub fn current_mut(&mut self) -> &mut KeyboardFocus {
        &mut self.current
    }
    
    pub fn set_current(&mut self, focus: KeyboardFocus) {
        self.current = focus;
    }
    
    // =========================================================================
    // Layer On-Demand Focus
    // =========================================================================
    
    pub fn layer_on_demand(&self) -> Option<&LayerSurface> {
        self.layer_on_demand.as_ref()
    }
    
    pub fn layer_on_demand_mut(&mut self) -> &mut Option<LayerSurface> {
        &mut self.layer_on_demand
    }
    
    pub fn set_layer_on_demand(&mut self, surface: Option<LayerSurface>) {
        self.layer_on_demand = surface;
    }
    
    // =========================================================================
    // Idle Inhibitors
    // =========================================================================
    
    pub fn idle_inhibitors(&self) -> &HashSet<WlSurface> {
        &self.idle_inhibitors
    }
    
    pub fn idle_inhibitors_mut(&mut self) -> &mut HashSet<WlSurface> {
        &mut self.idle_inhibitors
    }
    
    pub fn is_idle_inhibited(&self) -> bool {
        !self.idle_inhibitors.is_empty()
    }
    
    // =========================================================================
    // Shortcut Inhibitors
    // =========================================================================
    
    pub fn shortcut_inhibitors(&self) -> &HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &self.shortcut_inhibitors
    }
    
    pub fn shortcut_inhibitors_mut(&mut self) -> &mut HashMap<WlSurface, KeyboardShortcutsInhibitor> {
        &mut self.shortcut_inhibitors
    }
    
    pub fn are_shortcuts_inhibited(&self, surface: &WlSurface) -> bool {
        self.shortcut_inhibitors.contains_key(surface)
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Work Units

### Unit 1: Create focus.rs with FocusState

Create `src/niri/subsystems/focus.rs` with the struct and accessors above.

**Verify**: `cargo check`

---

### Unit 2: Add to subsystems/mod.rs

```rust
mod focus;
pub use focus::FocusState;
```

---

### Unit 3: Move Fields from Niri

1. Remove the 4 focus fields from `Niri` struct
2. Add `pub focus: FocusState` field
3. Update `Niri::new()` to create `FocusState::new()`

---

### Unit 4: Update Access Patterns

Simple find-and-replace:

```rust
// Before
self.niri.keyboard_focus
self.niri.layer_shell_on_demand_focus
self.niri.idle_inhibiting_surfaces
self.niri.keyboard_shortcuts_inhibiting_surfaces

// After
self.niri.focus.current()       // or current_mut()
self.niri.focus.layer_on_demand()  // or layer_on_demand_mut()
self.niri.focus.idle_inhibitors()  // or idle_inhibitors_mut()
self.niri.focus.shortcut_inhibitors()  // or shortcut_inhibitors_mut()
```

**Verify**: `cargo check` && `cargo test`

---

## Verification Checklist

- [ ] `FocusState` struct exists with private fields
- [ ] All 4 focus fields removed from `Niri`
- [ ] `Niri.focus: FocusState` field added
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/focus.rs` | **NEW** ~110 lines |
| `src/niri/subsystems/mod.rs` | +2 lines |
| `src/niri/mod.rs` | -4 fields, +1 field |
| Various files | Updated access patterns |

---

## Why This Is Low Risk

- **No logic changes**: Just moving fields into a container
- **Mechanical refactor**: Find-and-replace access patterns
- **Compiler-guided**: Missing updates will cause compile errors
- **No behavior change**: Same data, different location

---

## Next Phase

After completing this phase, proceed to [Phase P4b: Focus Computation Logic](phase-P4b-focus-computation.md).
