//! Output management subsystem.
//!
//! Owns all state related to physical outputs (monitors) and provides
//! a clean API for output lifecycle management and spatial queries.
//!
//! # Responsibilities
//!
//! - **Global space**: Compositor-wide coordinate space and output positioning
//! - **Output state**: Per-output frame clocks, redraw state, and damage tracking
//! - **Spatial queries**: Find outputs under positions, compute working areas
//! - **Power management**: Monitor power state and lid detection
//!
//! # Example
//!
//! ```ignore
//! // Add a new output
//! niri.outputs.add(output, refresh_interval, vrr, &display_handle, &config);
//!
//! // Find output under cursor
//! if let Some((output, pos)) = niri.outputs.under_position(cursor_pos) {
//!     niri.queue_redraw(&output);
//! }
//!
//! // Update all outputs
//! for (output, state) in niri.outputs.iter() {
//!     state.frame_clock.update_now();
//! }
//!
//! // Check if any monitors are active
//! if !niri.outputs.monitors_active() {
//!     // Handle all monitors off
//! }
//! ```

use std::collections::HashMap;
use std::time::Duration;

use smithay::desktop::Space;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};
use wayland_backend::server::GlobalId;

use crate::frame_clock::FrameClock;
use crate::render_helpers::solid_color::SolidColorBuffer;
use crate::utils::vblank_throttle::VBlankThrottle;

use super::super::types::RedrawState;
use crate::niri::OutputState;

/// Output management subsystem.
///
/// This struct owns all output-related state and encapsulates the logic
/// for adding, removing, repositioning, and querying outputs.
pub struct OutputSubsystem {
    /// Global compositor space containing all outputs.
    space: Space<smithay::desktop::Window>,
    
    /// Outputs sorted by name and position.
    sorted: Vec<Output>,
    
    /// Per-output state (frame clock, redraw state, etc.).
    state: HashMap<Output, OutputState>,
    
    /// Whether monitors are currently active (not powered off for idle).
    monitors_active: bool,
    
    /// Whether the laptop lid is closed.
    lid_closed: bool,
}

impl OutputSubsystem {
    /// Creates a new output subsystem.
    pub fn new() -> Self {
        Self {
            space: Space::default(),
            sorted: Vec::new(),
            state: HashMap::new(),
            monitors_active: true,
            lid_closed: false,
        }
    }
    
    // =========================================================================
    // Lifecycle Management
    // =========================================================================
    
    /// Adds a new output to the compositor.
    pub fn add(
        &mut self,
        output: Output,
        refresh_interval: Option<Duration>,
        vrr: bool,
    ) -> GlobalId {
        // For now, just add to our internal structures
        // The actual global creation will be handled by the caller
        self.sorted.push(output.clone());
        
        // Create a placeholder GlobalId - this will be replaced when we move the full implementation
        GlobalId::new()
    }
    
    /// Removes an output from the compositor.
    pub fn remove(&mut self, output: &Output) {
        // Implementation will be moved from Niri::remove_output
        unimplemented!()
    }
    
    /// Repositions all outputs based on configuration.
    pub fn reposition(&mut self, new_output: Option<&Output>) {
        // Implementation will be moved from Niri::reposition_outputs
        unimplemented!()
    }
    
    // =========================================================================
    // Spatial Queries
    // =========================================================================
    
    /// Returns the output under the given global position.
    pub fn under_position(&self, pos: Point<f64, Logical>) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.space.output_under(pos).next()?;
        let geo = self.space.output_geometry(output)?;
        Some((output, pos - geo.loc.to_f64()))
    }
    
    /// Returns the output to the left of the given output.
    pub fn left_of(&self, current: &Output) -> Option<&Output> {
        // Implementation will be moved from Niri::output_left_of
        unimplemented!()
    }
    
    /// Returns the output to the right of the given output.
    pub fn right_of(&self, current: &Output) -> Option<&Output> {
        // Implementation will be moved from Niri::output_right_of
        unimplemented!()
    }
    
    /// Returns the output above the given output.
    pub fn above(&self, current: &Output) -> Option<&Output> {
        // Implementation will be moved from Niri::output_up_of
        unimplemented!()
    }
    
    /// Returns the output below the given output.
    pub fn below(&self, current: &Output) -> Option<&Output> {
        // Implementation will be moved from Niri::output_down_of
        unimplemented!()
    }
    
    // =========================================================================
    // State Access
    // =========================================================================
    
    /// Returns an iterator over all outputs.
    pub fn iter(&self) -> impl Iterator<Item = &Output> {
        self.sorted.iter()
    }
    
    /// Returns the global space (read-only).
    pub fn space(&self) -> &Space<smithay::desktop::Window> {
        &self.space
    }
    
    /// Returns a mutable reference to the global space.
    pub fn space_mut(&mut self) -> &mut Space<smithay::desktop::Window> {
        &mut self.space
    }
    
    /// Returns the state for a specific output.
    pub fn state(&self, output: &Output) -> Option<&OutputState> {
        self.state.get(output)
    }
    
    /// Returns mutable state for a specific output.
    pub fn state_mut(&mut self, output: &Output) -> Option<&mut OutputState> {
        self.state.get_mut(output)
    }
    
    /// Returns an iterator over all (output, state) pairs.
    pub fn state_iter(&self) -> impl Iterator<Item = (&Output, &OutputState)> {
        self.state.iter()
    }
    
    /// Returns a mutable iterator over all (output, state) pairs.
    pub fn state_iter_mut(&mut self) -> impl Iterator<Item = (&Output, &mut OutputState)> {
        self.state.iter_mut()
    }
    
    /// Inserts state for an output. Returns previous state if present.
    pub fn insert_state(&mut self, output: Output, state: OutputState) -> Option<OutputState> {
        self.state.insert(output, state)
    }
    
    /// Removes and returns state for an output.
    pub fn remove_state(&mut self, output: &Output) -> Option<OutputState> {
        self.state.remove(output)
    }
    
    /// Returns true if the output has state.
    pub fn has_state(&self, output: &Output) -> bool {
        self.state.contains_key(output)
    }
    
    /// Returns the number of outputs with state.
    pub fn state_count(&self) -> usize {
        self.state.len()
    }
    
    /// Returns all output states as a slice-like iterator for values.
    pub fn states(&self) -> impl Iterator<Item = &OutputState> {
        self.state.values()
    }
    
    /// Returns mutable access to all output states.
    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut OutputState> {
        self.state.values_mut()
    }
    
    /// Adds an output to the sorted list.
    pub fn add_sorted(&mut self, output: Output) {
        self.sorted.push(output);
    }
    
    /// Clears the sorted outputs list.
    pub fn clear_sorted(&mut self) {
        self.sorted.clear();
    }
    
    /// Sets the sorted outputs list.
    pub fn set_sorted(&mut self, outputs: Vec<Output>) {
        self.sorted = outputs;
    }
    
    /// Returns whether monitors are active.
    pub fn monitors_active(&self) -> bool {
        self.monitors_active
    }
    
    /// Sets whether monitors are active.
    pub fn set_monitors_active(&mut self, active: bool) {
        self.monitors_active = active;
    }
    
    /// Returns whether the lid is closed.
    pub fn lid_closed(&self) -> bool {
        self.lid_closed
    }
    
    /// Sets the lid closed state.
    pub fn set_lid_closed(&mut self, closed: bool) {
        self.lid_closed = closed;
    }
    
    // =========================================================================
    // Redraw Management
    // =========================================================================
    
    /// Queues a redraw for a specific output.
    pub fn queue_redraw(&mut self, output: &Output) {
        if let Some(state) = self.state.get_mut(output) {
            state.redraw_state = std::mem::take(&mut state.redraw_state).queue_redraw();
        }
    }
    
    /// Queues a redraw for all outputs.
    pub fn queue_redraw_all(&mut self) {
        for state in self.state.values_mut() {
            state.redraw_state = std::mem::take(&mut state.redraw_state).queue_redraw();
        }
    }
}

impl Default for OutputSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
