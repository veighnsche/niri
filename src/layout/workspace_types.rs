// TEAM_022: Minimal workspace types remaining after Canvas2D migration
// These types are kept ONLY for external system compatibility (IPC, protocols, etc.)
// Most functionality has been moved to Canvas2D/Row.

use std::fmt;
use smithay::output::Output;
use smithay::utils::{Logical, Rectangle};

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
    
    /// TEAM_024: Create a WorkspaceId from a row index
    /// This maps row indices to workspace IDs for compatibility
    pub fn from_row_index(row_index: i32) -> Self {
        // Map row index to workspace ID
        // Row 0 (origin) -> workspace ID 1
        // Row 1 -> workspace ID 2, etc.
        // Row -1 -> workspace ID 0, Row -2 -> workspace ID 99, etc.
        if row_index >= 0 {
            Self((row_index + 1) as u64)
        } else {
            // Negative rows map to high workspace IDs
            Self(((-row_index) as u64).wrapping_mul(10))
        }
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
/// TEAM_022: Still needed by layout/mod.rs callers
#[derive(Debug, Clone, Copy)]
pub enum WorkspaceAddWindowTarget<'a, W: super::LayoutElement> {
    AtWindow(&'a W),
    AtEnd,
    Auto,
    NextTo(&'a W),
}

/// Compute working area for an output
/// This function is still needed by monitor configuration
pub fn compute_working_area(output: &Output) -> Rectangle<f64, Logical> {
    // TEAM_022: Use the layer shell handler directly
    // TODO: This should eventually be moved to a more appropriate location
    Rectangle::from_loc_and_size((0.0, 0.0), output.size().to_f64())
}
