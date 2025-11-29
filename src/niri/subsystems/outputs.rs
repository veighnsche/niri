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

use smithay::desktop::Space;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::niri::OutputState;

/// Helper to compute center of a rectangle.
fn center(geo: Rectangle<i32, Logical>) -> Point<i32, Logical> {
    Point::from((geo.loc.x + geo.size.w / 2, geo.loc.y + geo.size.h / 2))
}

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

    /// Unmaps an output from the space and removes from sorted list.
    /// Returns the removed OutputState for cleanup by caller.
    pub fn unmap_and_remove(&mut self, output: &Output) -> Option<OutputState> {
        // Unmap from space
        self.space.unmap_output(output);

        // Remove from sorted list
        self.sorted.retain(|o| o != output);

        // Remove and return state
        self.state.remove(output)
    }

    /// Maps an output at the given position.
    pub fn map_output(&mut self, output: &Output, position: Point<i32, Logical>) {
        self.space.map_output(output, position);
    }

    /// Unmaps an output from the space (but keeps in sorted list).
    pub fn unmap_output(&mut self, output: &Output) {
        self.space.unmap_output(output);
    }

    /// Returns the geometry of an output in global space.
    pub fn output_geometry(&self, output: &Output) -> Option<Rectangle<i32, Logical>> {
        self.space.output_geometry(output)
    }

    // =========================================================================
    // Spatial Queries
    // =========================================================================

    /// Returns the output under the given global position.
    pub fn under_position(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.space.output_under(pos).next()?;
        let geo = self.space.output_geometry(output)?;
        Some((output, pos - geo.loc.to_f64()))
    }

    /// Returns the output to the left of the given output.
    pub fn left_of(&self, current: &Output) -> Option<&Output> {
        let current_geo = self.space.output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((i32::MIN / 2, current_geo.loc.y)),
            Size::from((i32::MAX, current_geo.size.h)),
        );

        self.space
            .outputs()
            .map(|output| (output, self.space.output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).x < center(current_geo).x && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(current_geo).x - center(*geo).x)
            .map(|(output, _)| output)
    }

    /// Returns the output to the right of the given output.
    pub fn right_of(&self, current: &Output) -> Option<&Output> {
        let current_geo = self.space.output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((i32::MIN / 2, current_geo.loc.y)),
            Size::from((i32::MAX, current_geo.size.h)),
        );

        self.space
            .outputs()
            .map(|output| (output, self.space.output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).x > center(current_geo).x && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(*geo).x - center(current_geo).x)
            .map(|(output, _)| output)
    }

    /// Returns the output above the given output.
    pub fn above(&self, current: &Output) -> Option<&Output> {
        let current_geo = self.space.output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((current_geo.loc.x, i32::MIN / 2)),
            Size::from((current_geo.size.w, i32::MAX)),
        );

        self.space
            .outputs()
            .map(|output| (output, self.space.output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).y < center(current_geo).y && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(current_geo).y - center(*geo).y)
            .map(|(output, _)| output)
    }

    /// Returns the output below the given output.
    pub fn below(&self, current: &Output) -> Option<&Output> {
        let current_geo = self.space.output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((current_geo.loc.x, i32::MIN / 2)),
            Size::from((current_geo.size.w, i32::MAX)),
        );

        self.space
            .outputs()
            .map(|output| (output, self.space.output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).y > center(current_geo).y && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(*geo).y - center(current_geo).y)
            .map(|(output, _)| output)
    }

    /// Returns the previous output in sorted order.
    pub fn previous_of(&self, current: &Output) -> Option<&Output> {
        let mut iter = self.sorted.iter();
        let mut prev = None;
        for output in iter.by_ref() {
            if output == current {
                return prev
                    .or_else(|| self.sorted.last())
                    .filter(|&o| o != current);
            }
            prev = Some(output);
        }
        None
    }

    /// Returns the next output in sorted order.
    pub fn next_of(&self, current: &Output) -> Option<&Output> {
        let mut iter = self.sorted.iter();
        while let Some(output) = iter.next() {
            if output == current {
                return iter
                    .next()
                    .or_else(|| self.sorted.first())
                    .filter(|&o| o != current);
            }
        }
        None
    }

    /// Returns all outputs from the space.
    pub fn all(&self) -> impl Iterator<Item = &Output> {
        self.space.outputs()
    }

    /// Returns the first output (if any).
    pub fn first(&self) -> Option<&Output> {
        self.sorted.first()
    }

    /// Returns the last output (if any).
    pub fn last(&self) -> Option<&Output> {
        self.sorted.last()
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

    // =========================================================================
    // Behavior Methods (encapsulated logic, not raw accessors)
    // =========================================================================

    /// Refreshes the global space.
    pub fn refresh_space(&mut self) {
        self.space.refresh();
    }

    /// Maps an output at a position in the space.
    pub fn map_output_at(&mut self, output: &Output, position: Point<i32, Logical>) {
        self.space.map_output(output, position);
    }

    /// Returns outputs from the space.
    pub fn space_outputs(&self) -> impl Iterator<Item = &Output> {
        self.space.outputs()
    }

    /// Returns the lock surface for an output (if any).
    pub fn lock_surface(
        &self,
        output: &Output,
    ) -> Option<&smithay::wayland::session_lock::LockSurface> {
        self.state.get(output).and_then(|s| s.lock_surface.as_ref())
    }

    /// Checks if all outputs with state have lock surfaces ready.
    pub fn all_lock_surfaces_ready(&self) -> bool {
        self.state.values().all(|s| s.lock_surface.is_some())
    }

    /// Finds an output that needs redraw and returns it (cloned).
    /// This avoids holding a borrow while allowing mutation elsewhere.
    pub fn find_output_needing_redraw(
        &self,
        check: impl Fn(&Output, &super::super::OutputState) -> bool,
    ) -> Option<Output> {
        self.state
            .iter()
            .find(|(output, state)| check(output, state))
            .map(|(output, _)| output.clone())
    }

    /// Collects all outputs as a Vec (for iteration without holding borrow).
    pub fn collect_outputs(&self) -> Vec<Output> {
        self.sorted.clone()
    }

    /// Updates backdrop color for an output if different.
    /// Returns true if color was changed.
    pub fn update_backdrop_color(
        &mut self,
        output: &Output,
        color: smithay::backend::renderer::Color32F,
    ) -> bool {
        if let Some(state) = self.state.get_mut(output) {
            if state.backdrop_buffer.color() != color {
                state.backdrop_buffer.set_color(color);
                return true;
            }
        }
        false
    }
}

impl Default for OutputSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
