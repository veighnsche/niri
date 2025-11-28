// TEAM_022: Minimal row types remaining after Canvas2D migration
// TEAM_055: Renamed from workspace_types.rs to row_types.rs
// These types are kept ONLY for external system compatibility (IPC, protocols, etc.)
// Most functionality has been moved to Canvas2D/Row.

use std::fmt;
use smithay::output::Output;
use smithay::utils::{Logical, Rectangle};

// TEAM_055: Renamed from WorkspaceId to RowId
/// Legacy row ID for external system compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowId(pub u64);

impl fmt::Display for RowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RowId {
    pub fn get(&self) -> u64 {
        self.0
    }
    
    pub fn specific(id: u64) -> Self {
        Self(id)
    }
    
    /// TEAM_024: Create a RowId from a row index
    /// This maps row indices to row IDs for compatibility
    pub fn from_row_index(row_index: i32) -> Self {
        // Map row index to row ID
        // Row 0 (origin) -> row ID 1
        // Row 1 -> row ID 2, etc.
        // Row -1 -> row ID 0, Row -2 -> row ID 99, etc.
        if row_index >= 0 {
            Self((row_index + 1) as u64)
        } else {
            // Negative rows map to high row IDs
            Self(((-row_index) as u64).wrapping_mul(10))
        }
    }
}

/// Legacy output ID for external system compatibility
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

// TEAM_055: Renamed from WorkspaceAddWindowTarget to RowAddWindowTarget
/// Legacy row add window target for external compatibility
/// TEAM_022: Still needed by layout/mod.rs callers
#[derive(Debug, Clone, Copy)]
pub enum RowAddWindowTarget<'a, W: super::LayoutElement> {
    AtWindow(&'a W),
    AtEnd,
    Auto,
    NextTo(&'a W),
}

use crate::utils::output_size;

/// Compute working area for an output
/// This function is still needed by monitor configuration
pub fn compute_working_area(output: &Output) -> Rectangle<f64, Logical> {
    // TEAM_022: Use the layer shell handler directly
    Rectangle::from_loc_and_size((0.0, 0.0), output_size(output))
}
