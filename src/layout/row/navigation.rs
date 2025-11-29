// TEAM_007: Navigation extracted from mod.rs
// TEAM_008: Added activate_column
// TEAM_064: Added activate_window, activate_window_without_raising
//!
//! This module handles column focus navigation within a row.

use super::Row;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Window activation
    // =========================================================================

    /// Activates a window by ID, focusing both the window within its column
    /// and the column within the row.
    pub fn activate_window(&mut self, window: &W::Id) -> bool {
        // Find the column containing this window
        let column_idx = self.columns.iter().position(|col| col.contains(window));
        let Some(column_idx) = column_idx else {
            return false;
        };
        let column = &mut self.columns[column_idx];

        // Activate the window within its column
        column.activate_window(window);
        // Activate the column within the row
        self.activate_column(column_idx);

        true
    }

    /// Activate window without raising it in the stacking order.
    /// TEAM_025: Stub implementation - rows don't have stacking order
    pub fn activate_window_without_raising(&mut self, _window: &W::Id) -> bool {
        // TEAM_025: TODO - implement activation without raising
        // For rows, this is a no-op since rows don't have stacking order
        false
    }

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
        // TEAM_039: Pass old active column index for proper view offset calculation
        let prev_idx = self.active_column_idx;
        self.animate_view_offset_to_column(None, new_idx, Some(prev_idx));
        self.active_column_idx = new_idx;
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
        // TEAM_039: Pass old active column index for proper view offset calculation
        let prev_idx = self.active_column_idx;
        self.animate_view_offset_to_column(None, new_idx, Some(prev_idx));
        self.active_column_idx = new_idx;
        true
    }

    /// Focuses a specific column by index (1-based, for external API compatibility).
    // TEAM_040: Matches ScrollingSpace - external API uses 1-based index
    pub fn focus_column(&mut self, index: usize) {
        if self.columns.is_empty() {
            return;
        }

        // Convert 1-based index to 0-based, clamped to valid range
        let idx = index.saturating_sub(1).min(self.columns.len() - 1);
        self.focus_column_idx(idx);
    }

    /// Focuses a specific column by 0-based index (internal use).
    // TEAM_040: Internal method for 0-based indexing
    fn focus_column_idx(&mut self, idx: usize) {
        if self.columns.is_empty() {
            return;
        }

        let idx = idx.min(self.columns.len() - 1);

        // TEAM_039: Pass old active column index for proper view offset calculation
        let prev_idx = self.active_column_idx;
        
        if idx != self.active_column_idx {
            self.activate_prev_column_on_removal = None;
            self.view_offset_to_restore = None;
        }

        self.animate_view_offset_to_column(None, idx, Some(prev_idx));
        self.active_column_idx = idx;
    }

    /// Focuses the first column in the row.
    pub fn focus_column_first(&mut self) -> bool {
        if self.columns.is_empty() {
            return false;
        }
        self.focus_column_idx(0);
        true
    }

    /// Focuses the last column in the row.
    pub fn focus_column_last(&mut self) -> bool {
        if self.columns.is_empty() {
            return false;
        }
        self.focus_column_idx(self.columns.len() - 1);
        true
    }

    /// Focuses the column to the right, or wraps to first if at end.
    pub fn focus_column_right_or_first(&mut self) -> bool {
        if self.focus_right() {
            true
        } else {
            self.focus_column_first()
        }
    }

    /// Focuses the column to the left, or wraps to last if at start.
    pub fn focus_column_left_or_last(&mut self) -> bool {
        if self.focus_left() {
            true
        } else {
            self.focus_column_last()
        }
    }

    /// Focuses a window within the active column by index.
    // TEAM_040: Fixed - this focuses a window within the column, not a column
    pub fn focus_window_in_column(&mut self, index: u8) {
        if self.columns.is_empty() {
            return;
        }

        self.columns[self.active_column_idx].focus_index(index);
    }

    /// Focuses down and left (diagonal navigation).
    pub fn focus_down_or_left(&mut self) -> bool {
        // Try to focus down first, then left if not possible
        self.focus_down() || self.focus_left()
    }

    /// Focuses down and right (diagonal navigation).
    pub fn focus_down_or_right(&mut self) -> bool {
        // Try to focus down first, then right if not possible
        self.focus_down() || self.focus_right()
    }

    /// Focuses up and left (diagonal navigation).
    pub fn focus_up_or_left(&mut self) -> bool {
        // Try to focus up first, then left if not possible
        self.focus_up() || self.focus_left()
    }

    /// Focuses up and right (diagonal navigation).
    pub fn focus_up_or_right(&mut self) -> bool {
        // Try to focus up first, then right if not possible
        self.focus_up() || self.focus_right()
    }

    /// Focuses the top window in the active column.
    pub fn focus_window_top(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.focus_top();
            true
        } else {
            false
        }
    }

    /// Focuses the bottom window in the active column.
    pub fn focus_window_bottom(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.focus_bottom();
            true
        } else {
            false
        }
    }

    /// Focuses down or to top if cannot move down.
    pub fn focus_window_down_or_top(&mut self) -> bool {
        if self.focus_down() {
            true
        } else {
            self.focus_window_top()
        }
    }

    /// Focuses up or to bottom if cannot move up.
    pub fn focus_window_up_or_bottom(&mut self) -> bool {
        if self.focus_up() {
            true
        } else {
            self.focus_window_bottom()
        }
    }
}
