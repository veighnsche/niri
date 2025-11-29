//! Keyboard focus state container.
//!
//! Phase P4a: Data container only. Logic extraction happens in P4b-P4d.

use std::collections::{HashMap, HashSet};

use smithay::desktop::LayerSurface;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor;
use smithay::wayland::shell::wlr_layer::Layer;

use super::super::types::KeyboardFocus;

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

/// Keyboard focus state container.
pub struct FocusState {
    /// Current keyboard focus target.
    pub current: KeyboardFocus,
    
    /// Layer surface with on-demand focus (if any).
    pub layer_on_demand: Option<LayerSurface>,
    
    /// Surfaces that are inhibiting idle.
    pub idle_inhibitors: HashSet<WlSurface>,
    
    /// Surfaces that are inhibiting keyboard shortcuts.
    pub shortcut_inhibitors: HashMap<WlSurface, KeyboardShortcutsInhibitor>,
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
    
    /// Cleans up on-demand layer focus if the surface is no longer valid.
    ///
    /// Returns true if the on-demand focus was cleared.
    pub fn cleanup_layer_on_demand<F>(&mut self, is_valid: F) -> bool 
    where
        F: FnOnce(&LayerSurface) -> bool,
    {
        let should_clear = self.layer_on_demand.as_ref().map_or(false, |surface| {
            !is_valid(surface)
        });
        
        if should_clear {
            self.layer_on_demand = None;
            true
        } else {
            false
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::wayland::shell::wlr_layer::Layer;
    
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
            KeyboardFocus::LockScreen { surface: None }
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
    
    #[test]
    fn test_screenshot_ui_priority() {
        let focus = FocusState::new();
        let ctx = FocusContext {
            screenshot_ui_open: true,
            is_locked: false,
            exit_dialog_open: false,
            ..Default::default()
        };
        
        assert!(matches!(
            focus.compute_focus(&ctx),
            KeyboardFocus::ScreenshotUi
        ));
    }
    
    #[test]
    fn test_mru_ui_priority() {
        let focus = FocusState::new();
        let ctx = FocusContext {
            mru_ui_open: true,
            screenshot_ui_open: false,
            is_locked: false,
            exit_dialog_open: false,
            ..Default::default()
        };
        
        assert!(matches!(
            focus.compute_focus(&ctx),
            KeyboardFocus::Mru
        ));
    }
}
