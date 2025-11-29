// TEAM_065: Row management operations extracted from operations.rs
//!
//! This module handles row lifecycle operations in the canvas.

use crate::layout::canvas::Canvas2D;
use crate::layout::row::Row;
use crate::layout::row_types::RowId;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Canvas2D<W> {
    // =========================================================================
    // Row Management
    // =========================================================================

    /// Creates a new row at the specified index if it doesn't exist.
    /// TEAM_057: Fixed row ID generation to avoid collisions across canvases.
    /// Each canvas gets a unique base ID from Layout, and we use a large stride
    /// to ensure IDs don't collide when multiple canvases create rows.
    pub fn ensure_row(&mut self, row_idx: i32) -> &mut Row<W> {
        self.rows.entry(row_idx).or_insert_with(|| {
            // TEAM_057: Generate unique row ID for new row
            // Use a large stride (1000) to avoid collisions with other canvases
            // Each canvas starts with a unique base from Layout.next_row_id()
            self.row_id_counter += 1000;
            let row_id = RowId(self.row_id_counter);

            Row::new(
                row_idx,
                row_id,
                self.view_size,
                self.parent_area,
                self.scale,
                self.clock.clone(),
                self.options.clone(),
            )
        })
    }

    /// Removes empty unnamed rows.
    /// TEAM_055: Fixed to match original clean_up_workspaces behavior - only keep rows with windows
    /// or names
    pub fn cleanup_empty_rows(&mut self) {
        // Keep at least one row - if all rows would be removed, keep row 0
        let has_non_empty = self
            .rows
            .values()
            .any(|row| row.has_windows() || row.name().is_some());
        self.rows.retain(|&idx, row| {
            row.has_windows() || row.name().is_some() || (!has_non_empty && idx == 0)
        });
    }

    /// Renumbers rows to maintain contiguous indices starting from 0.
    /// This should be called after cleanup_empty_rows to ensure workspaces are contiguous.
    /// TEAM_059: Added to fix move_to_workspace_by_idx_does_not_leave_empty_workspaces test
    pub fn renumber_rows(&mut self) {
        // Take all rows out of the map
        let old_rows = std::mem::take(&mut self.rows);

        // Collect and sort by index
        let mut sorted_rows: Vec<(i32, Row<W>)> = old_rows.into_iter().collect();
        sorted_rows.sort_by_key(|(idx, _)| *idx);

        // Re-insert with contiguous indices starting from 0
        for (new_idx, (old_idx, mut row)) in sorted_rows.into_iter().enumerate() {
            let new_idx = new_idx as i32;
            if old_idx != new_idx {
                row.set_idx(new_idx);
            }
            self.rows.insert(new_idx, row);
        }

        // Update active_row_idx to point to a valid row
        if !self.rows.contains_key(&self.active_row_idx) {
            self.active_row_idx = self.rows.keys().next().copied().unwrap_or(0);
        }
    }

    /// Cleans up empty rows and renumbers remaining rows to be contiguous.
    /// TEAM_059: Combined cleanup and renumber for convenience
    pub fn cleanup_and_renumber_rows(&mut self) {
        self.cleanup_empty_rows();
        self.renumber_rows();
    }

    /// TEAM_057: Find a row by its workspace/row ID.
    /// Returns the row index if found.
    pub fn find_row_by_id(&self, ws_id: RowId) -> Option<i32> {
        self.rows
            .iter()
            .find(|(_, row)| row.id() == ws_id)
            .map(|(&idx, _)| idx)
    }
}
