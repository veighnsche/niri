// TEAM_021: Minimal workspace types remaining after Canvas2D migration
// These types are kept for external system compatibility (IPC, protocols, etc.)

use std::fmt;
use smithay::output::Output;
use smithay::utils::{Logical, Rectangle};

use super::LayoutElement;
use crate::utils::ResizeEdge;

/// Legacy workspace ID for external system compatibility
/// TODO: Eventually remove when external systems are updated to use canvas concepts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub u64);

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WorkspaceId {
    pub fn get(&self) -> u64 {
        self.0
    }
    
    pub fn specific(id: u64) -> Self {
        Self(id)
    }
}

/// Legacy output ID for external system compatibility
/// TODO: Eventually remove when external systems are updated
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputId(pub String);

impl fmt::Display for OutputId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl OutputId {
    pub fn new(output: &Output) -> Self {
        Self(output.name())
    }
}

/// Legacy workspace add window target for external compatibility
/// TODO: Eventually remove when external systems are updated
pub enum WorkspaceAddWindowTarget<'a, W: LayoutElement> {
    AtWindow(&'a W),
    AtEnd,
    Auto,
    NextTo(&'a W),
}

/// Legacy Workspace struct - minimal stub for compatibility
/// TODO: Eventually remove when all external systems are updated
#[derive(Debug)]
pub struct Workspace<W: LayoutElement> {
    // Empty stub - all functionality moved to Canvas2D
    _phantom: std::marker::PhantomData<W>,
}

// Re-export render element for compatibility - using a stub for now
// TODO: Eventually remove when all external systems are updated
#[derive(Debug)]
pub struct WorkspaceRenderElement;

impl<W: LayoutElement> Workspace<W> {
    // Empty stub methods for compatibility
    pub fn has_window(&self, _id: &W::Id) -> bool { false }
    pub fn set_fullscreen(&mut self, _id: &W::Id, _is_fullscreen: bool) {}
    pub fn toggle_fullscreen(&mut self, _id: &W::Id) {}
    pub fn set_maximized(&mut self, _id: &W::Id, _maximize: bool) {}
    pub fn toggle_maximized(&mut self, _id: &W::Id) {}
    pub fn descendants_added(&mut self, _id: &W::Id) -> bool { false }
    pub fn start_open_animation(&mut self, _window: &W::Id) -> bool { false }
    pub fn dnd_scroll_gesture_begin(&mut self) {}
    pub fn dnd_scroll_gesture_end(&mut self) {}
    pub fn interactive_resize_begin(&mut self, _window: W::Id, _edges: ResizeEdge) -> bool { false }
    pub fn interactive_resize_update(&mut self, _window: &W::Id, _delta: smithay::utils::Point<f64, smithay::utils::Logical>) -> bool { false }
    pub fn interactive_resize_end(&mut self, _window: Option<&W::Id>) {}
    pub fn store_unmap_snapshot_if_empty(&mut self, _renderer: &mut crate::render_helpers::renderer::NiriRenderer, _window: &W::Id) {}
    pub fn clear_unmap_snapshot(&mut self, _window: &W::Id) {}
    pub fn start_close_animation_for_window(&mut self, _renderer: &mut crate::render_helpers::renderer::NiriRenderer, _window: &W::Id, _blocker: &mut smithay::wayland::compositor::Blocker) {}
    pub fn start_close_animation_for_tile(&mut self, _renderer: &mut crate::render_helpers::renderer::NiriRenderer, _snapshot: &crate::render_helpers::snapshot::RenderSnapshot, _tile_size: smithay::utils::Size<f64, smithay::utils::Logical>, _tile_pos: smithay::utils::Point<f64, smithay::utils::Logical>) {}
    pub fn is_floating(&self, _window: &W::Id) -> bool { false }
    pub fn current_output(&self) -> Option<Output> { None }
    pub fn windows_mut(&mut self) -> std::vec::IntoIter<&mut W> { Vec::new().into_iter() }
    pub fn windows(&self) -> std::vec::IntoIter<&W> { Vec::new().into_iter() }
    pub fn id(&self) -> WorkspaceId { WorkspaceId(0) }
    pub fn name(&self) -> Option<String> { None }
    pub fn active_window(&self) -> Option<&W> { None }
    pub fn activate_window(&mut self, _window: &W) -> bool { false }
    pub fn refresh(&mut self, _is_active: bool, _is_focused: bool) {}
    pub fn view_offset_gesture_end(&mut self, _config: Option<()>) {}
    pub fn is_urgent(&self) -> bool { false }
}

/// Compute working area for an output
/// This function is still needed by monitor configuration
pub fn compute_working_area(output: &Output) -> Rectangle<f64, Logical> {
    // TEAM_021: Use the layer shell handler directly
    // TODO: This should eventually be moved to a more appropriate location
    Rectangle::from_loc_and_size((0.0, 0.0), output.size().to_f64())
}
