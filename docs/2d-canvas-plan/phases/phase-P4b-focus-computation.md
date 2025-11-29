# Phase P4b: Extract Focus Computation Logic

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium (logic extraction)  
> **Prerequisite**: Phase P4a complete  
> **Creates**: `FocusComputer` with `compute()` method

---

## Goal

Extract the **focus computation logic** (lines 955-1066 of `update_keyboard_focus()`) into a separate, testable component. This is the "what should have focus?" logic.

---

## Current Code Analysis

The focus computation (lines 955-1066) determines focus priority:

```rust
let focus = if self.niri.exit_confirm_dialog.is_open() {
    KeyboardFocus::ExitConfirmDialog
} else if self.niri.is_locked() {
    KeyboardFocus::LockScreen { surface: ... }
} else if self.niri.screenshot_ui.is_open() {
    KeyboardFocus::ScreenshotUi
} else if self.niri.window_mru_ui.is_open() {
    KeyboardFocus::Mru
} else if let Some(output) = self.niri.layout.active_output() {
    // Complex layer shell priority logic (~80 lines)
    // - Layer grabs
    // - Exclusive focus on overlay/top/bottom/background
    // - On-demand focus
    // - Layout focus
} else {
    KeyboardFocus::Layout { surface: None }
};
```

---

## Target Architecture

### Add to `src/niri/subsystems/focus.rs`

```rust
/// Context for focus computation.
///
/// Contains all the state needed to compute focus, without requiring
/// access to the full Niri/State structs.
#[derive(Default)]
pub struct FocusContext<'a> {
    /// Is the exit confirm dialog open?
    pub exit_dialog_open: bool,
    
    /// Is the screen locked?
    pub is_locked: bool,
    
    /// Lock screen focus surface (if locked).
    pub lock_surface: Option<WlSurface>,
    
    /// Is the screenshot UI open?
    pub screenshot_ui_open: bool,
    
    /// Is the MRU UI open?
    pub mru_ui_open: bool,
    
    /// Current popup grab root surface and layer (if any).
    pub popup_grab: Option<(WlSurface, Layer)>,
    
    /// Layer surfaces on each layer that can receive keyboard focus.
    /// Tuple: (surface, is_exclusive, is_on_demand_focused, is_in_backdrop)
    pub layer_surfaces: Vec<LayerFocusCandidate<'a>>,
    
    /// Whether layout renders above top layer (fullscreen).
    pub layout_above_top: bool,
    
    /// Layout focus surface (if any).
    pub layout_focus: Option<WlSurface>,
}

/// A layer surface candidate for focus.
pub struct LayerFocusCandidate<'a> {
    pub surface: &'a LayerSurface,
    pub layer: Layer,
    pub is_exclusive: bool,
    pub is_on_demand_focused: bool,
    pub is_in_backdrop: bool,
}

impl FocusState {
    /// Computes what should have keyboard focus based on current state.
    ///
    /// This is the pure computation logic extracted from update_keyboard_focus().
    /// It does NOT apply the focus change - that's done by the caller.
    pub fn compute_focus(&self, ctx: &FocusContext) -> KeyboardFocus {
        // Priority 1: Exit confirm dialog (modal)
        if ctx.exit_dialog_open {
            return KeyboardFocus::ExitConfirmDialog;
        }
        
        // Priority 2: Lock screen (security)
        if ctx.is_locked {
            return KeyboardFocus::LockScreen {
                surface: ctx.lock_surface.clone(),
            };
        }
        
        // Priority 3: Screenshot UI (modal)
        if ctx.screenshot_ui_open {
            return KeyboardFocus::ScreenshotUi;
        }
        
        // Priority 4: MRU UI (modal)
        if ctx.mru_ui_open {
            return KeyboardFocus::Mru;
        }
        
        // Priority 5+: Layer shell and layout focus
        self.compute_layer_and_layout_focus(ctx)
    }
    
    /// Computes focus among layer shells and layout.
    fn compute_layer_and_layout_focus(&self, ctx: &FocusContext) -> KeyboardFocus {
        // Helper: check for grab on a specific layer
        let grab_on_layer = |layer: Layer| -> Option<KeyboardFocus> {
            ctx.popup_grab.as_ref().and_then(|(surface, grab_layer)| {
                if *grab_layer == layer {
                    Some(KeyboardFocus::LayerShell { surface: surface.clone() })
                } else {
                    None
                }
            })
        };
        
        // Helper: exclusive focus on a layer
        let excl_focus_on_layer = |layer: Layer| -> Option<KeyboardFocus> {
            ctx.layer_surfaces.iter().find_map(|candidate| {
                if candidate.layer == layer 
                    && candidate.is_exclusive 
                    && !candidate.is_in_backdrop 
                {
                    Some(KeyboardFocus::LayerShell {
                        surface: candidate.surface.wl_surface().clone(),
                    })
                } else {
                    None
                }
            })
        };
        
        // Helper: on-demand focus on a layer
        let on_demand_focus_on_layer = |layer: Layer| -> Option<KeyboardFocus> {
            ctx.layer_surfaces.iter().find_map(|candidate| {
                if candidate.layer == layer && candidate.is_on_demand_focused {
                    Some(KeyboardFocus::LayerShell {
                        surface: candidate.surface.wl_surface().clone(),
                    })
                } else {
                    None
                }
            })
        };
        
        // Helper: any focus on layer (exclusive or on-demand)
        let focus_on_layer = |layer: Layer| -> Option<KeyboardFocus> {
            excl_focus_on_layer(layer).or_else(|| on_demand_focus_on_layer(layer))
        };
        
        // Helper: layout focus
        let layout_focus = || -> KeyboardFocus {
            KeyboardFocus::Layout {
                surface: ctx.layout_focus.clone(),
            }
        };
        
        // Check grabs first (all layers)
        let mut focus = grab_on_layer(Layer::Overlay);
        focus = focus.or_else(|| grab_on_layer(Layer::Top));
        focus = focus.or_else(|| grab_on_layer(Layer::Bottom));
        focus = focus.or_else(|| grab_on_layer(Layer::Background));
        
        // Overlay layer always has priority
        focus = focus.or_else(|| focus_on_layer(Layer::Overlay));
        
        if ctx.layout_above_top {
            // Fullscreen: layout > top > bottom > background
            focus = focus.or_else(|| Some(layout_focus()));
            focus = focus.or_else(|| focus_on_layer(Layer::Top));
            focus = focus.or_else(|| focus_on_layer(Layer::Bottom));
            focus = focus.or_else(|| focus_on_layer(Layer::Background));
        } else {
            // Normal: top > on-demand bottom/bg > layout > exclusive bottom/bg
            focus = focus.or_else(|| focus_on_layer(Layer::Top));
            focus = focus.or_else(|| on_demand_focus_on_layer(Layer::Bottom));
            focus = focus.or_else(|| on_demand_focus_on_layer(Layer::Background));
            focus = focus.or_else(|| Some(layout_focus()));
            focus = focus.or_else(|| excl_focus_on_layer(Layer::Bottom));
            focus = focus.or_else(|| excl_focus_on_layer(Layer::Background));
        }
        
        focus.unwrap_or(KeyboardFocus::Layout { surface: None })
    }
}
```

---

## Work Units

### Unit 1: Add FocusContext and LayerFocusCandidate

Add the context structs to `focus.rs`.

**Verify**: `cargo check`

---

### Unit 2: Implement compute_focus()

Add the `compute_focus()` and `compute_layer_and_layout_focus()` methods.

**Verify**: `cargo check`

---

### Unit 3: Add Context Builder Helper

Add a helper in `State` to build the `FocusContext`:

```rust
impl State {
    fn build_focus_context(&self) -> FocusContext<'_> {
        // Collect all the state needed for focus computation
        FocusContext {
            exit_dialog_open: self.niri.exit_confirm_dialog.is_open(),
            is_locked: self.niri.is_locked(),
            lock_surface: self.niri.lock_surface_focus(),
            screenshot_ui_open: self.niri.screenshot_ui.is_open(),
            mru_ui_open: self.niri.window_mru_ui.is_open(),
            popup_grab: self.build_popup_grab_info(),
            layer_surfaces: self.collect_layer_focus_candidates(),
            layout_above_top: self.layout_renders_above_top(),
            layout_focus: self.niri.layout.focus()
                .map(|win| win.toplevel().wl_surface().clone()),
        }
    }
}
```

---

### Unit 4: Update update_keyboard_focus() - Part 1

Replace the focus computation section (lines 955-1066) with:

```rust
pub fn update_keyboard_focus(&mut self) {
    // Clean up on-demand layer surface focus if necessary.
    // ... (keep existing cleanup code for now)
    
    // Compute the new focus using the subsystem
    let ctx = self.build_focus_context();
    let focus = self.niri.focus.compute_focus(&ctx);
    
    // Handle focus change (keep existing code for now)
    // ... (lines 1068-1187 unchanged)
}
```

**Verify**: `cargo check` && `cargo test`

---

### Unit 5: Add Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exit_dialog_has_highest_priority() {
        let focus = FocusState::new();
        let ctx = FocusContext {
            exit_dialog_open: true,
            is_locked: true,  // Even if locked
            screenshot_ui_open: true,  // Even if screenshot open
            ..Default::default()
        };
        
        assert!(matches!(
            focus.compute_focus(&ctx),
            KeyboardFocus::ExitConfirmDialog
        ));
    }
    
    #[test]
    fn test_lock_screen_priority() {
        let focus = FocusState::new();
        let ctx = FocusContext {
            is_locked: true,
            lock_surface: None,
            ..Default::default()
        };
        
        assert!(matches!(
            focus.compute_focus(&ctx),
            KeyboardFocus::LockScreen { .. }
        ));
    }
    
    #[test]
    fn test_layout_focus_when_nothing_special() {
        let focus = FocusState::new();
        let ctx = FocusContext::default();
        
        assert!(matches!(
            focus.compute_focus(&ctx),
            KeyboardFocus::Layout { surface: None }
        ));
    }
}
```

---

## Verification Checklist

- [ ] `FocusContext` struct added
- [ ] `LayerFocusCandidate` struct added
- [ ] `compute_focus()` implemented
- [ ] `build_focus_context()` helper added
- [ ] Focus computation in `update_keyboard_focus()` uses subsystem
- [ ] Unit tests pass
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/focus.rs` | +150 lines |
| `src/niri/mod.rs` | Refactored computation section |

---

## Why This Is Medium Risk

- **Logic extraction**: Moving complex priority logic
- **Many edge cases**: Layer shell priority is subtle
- **Testable**: But we can now write unit tests!
- **Isolated change**: Only touches computation, not side effects

---

## Next Phase

After completing this phase, proceed to [Phase P4c: Focus Change Handling](phase-P4c-focus-change.md).
