// TEAM_064: Extracted from mod.rs as part of row/mod.rs split
//!
//! Extra per-column data cached for performance.

use super::super::column::Column;
use super::super::LayoutElement;

/// Extra per-column data.
///
/// This struct caches computed values from columns to avoid
/// recalculating them on every access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ColumnData {
    /// Cached actual column width.
    pub(super) width: f64,
}

impl ColumnData {
    /// Creates a new ColumnData with default values.
    pub(super) fn new() -> Self {
        Self { width: 0.0 }
    }

    /// Updates the cached data from a column.
    pub(super) fn update<W: LayoutElement>(&mut self, column: &Column<W>) {
        self.width = column.width();
    }
}

impl Default for ColumnData {
    fn default() -> Self {
        Self::new()
    }
}
