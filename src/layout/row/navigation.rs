// TEAM_007: Navigation extracted from mod.rs
// TEAM_008: Added activate_column
//!
//! This module handles column focus navigation within a row.

use super::Row;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Column activation
    // =========================================================================

    /// Activates a column by index, animating the view offset.
    pub(crate) fn activate_column(&mut self, idx: usize) {
        if self.active_column_idx == idx
            // During a DnD scroll, animate even when activating the same window, for DnD hold.
            && (self.columns.is_empty() || !self.view_offset_x.is_dnd_scroll())
        {
            return;
        }

        self.animate_view_offset_to_column(None, idx, Some(self.active_column_idx));

        if self.active_column_idx != idx {
            self.active_column_idx = idx;

            // A different column was activated; reset the flag.
            self.activate_prev_column_on_removal = None;
            self.view_offset_to_restore = None;
            self.interactive_resize = None;
        }
    }

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
