// TEAM_008: Move operations split from operations.rs
//!
//! This module handles moving columns within a row.

use crate::layout::row::Row;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Row<W> {
    /// Moves the active column to the left.
    pub fn move_left(&mut self) -> bool {
        if self.active_column_idx == 0 {
            return false;
        }
        self.move_column_to(self.active_column_idx - 1);
        true
    }

    /// Moves the active column to the right.
    pub fn move_right(&mut self) -> bool {
        if self.active_column_idx >= self.columns.len().saturating_sub(1) {
            return false;
        }
        self.move_column_to(self.active_column_idx + 1);
        true
    }

    /// Moves the active column to a specific index.
    pub fn move_column_to_index(&mut self, index: usize) {
        self.move_column_to(index.saturating_sub(1).min(self.columns.len() - 1));
    }

    /// Moves the active column to a specific index.
    pub(crate) fn move_column_to(&mut self, new_idx: usize) {
        if self.active_column_idx == new_idx {
            return;
        }

        let current_col_x = self.column_x(self.active_column_idx);

        let column = self.columns.remove(self.active_column_idx);
        let data = self.data.remove(self.active_column_idx);
        self.columns.insert(new_idx, column);
        self.data.insert(new_idx, data);

        // Preserve the camera position when moving.
        let view_offset_delta = -self.column_x(self.active_column_idx) + current_col_x;
        self.view_offset_x.offset(view_offset_delta);

        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);

        // TODO(TEAM_006): Animate column movement (port from ScrollingSpace)
    }
}
