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
