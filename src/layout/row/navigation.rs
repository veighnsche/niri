// TEAM_007: Navigation extracted from mod.rs
//!
//! This module handles column focus navigation within a row.

use super::Row;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Focus navigation
    // =========================================================================

    /// Focuses the column to the left.
    pub fn focus_left(&mut self) -> bool {
        if self.columns.is_empty() || self.active_column_idx == 0 {
            return false;
        }

        self.activate_prev_column_on_removal = None;
        self.view_offset_to_restore = None;

        let new_idx = self.active_column_idx - 1;
        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);
        true
    }

    /// Focuses the column to the right.
    pub fn focus_right(&mut self) -> bool {
        if self.columns.is_empty() || self.active_column_idx + 1 >= self.columns.len() {
            return false;
        }

        self.activate_prev_column_on_removal = None;
        self.view_offset_to_restore = None;

        let new_idx = self.active_column_idx + 1;
        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);
        true
    }

    /// Focuses a specific column by index.
    pub fn focus_column(&mut self, idx: usize) {
        if idx >= self.columns.len() {
            return;
        }

        if idx != self.active_column_idx {
            self.activate_prev_column_on_removal = None;
            self.view_offset_to_restore = None;
        }

        self.active_column_idx = idx;
        self.animate_view_offset_to_column(None, idx, None);
    }
}
