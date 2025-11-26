// TEAM_013: Navigation methods extracted from monitor.rs
// TEAM_022: Updated to use Canvas2D row navigation instead of workspaces
//!
//! This module contains row navigation methods.
//! Canvas2D rows replace workspaces.

use std::cmp::min;

use crate::layout::monitor::{Monitor, WorkspaceSwitch};
use crate::layout::LayoutElement;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // Row switching (was workspace switching)
    // =========================================================================

    // TEAM_022: Row count for navigation
    fn row_count(&self) -> usize {
        self.canvas.rows().count().max(1)
    }

    pub fn switch_workspace_up(&mut self) {
        let current_idx = self.canvas.active_row_idx();
        let new_idx = match &self.workspace_switch {
            Some(WorkspaceSwitch::Gesture(gesture)) if gesture.dnd_last_event_time.is_some() => {
                let current = gesture.current_idx;
                let new = current.ceil() - 1.;
                new.clamp(0., (self.row_count() - 1) as f64) as i32
            }
            _ => current_idx.saturating_sub(1),
        };

        self.activate_row(new_idx);
    }

    pub fn switch_workspace_down(&mut self) {
        let current_idx = self.canvas.active_row_idx();
        let new_idx = match &self.workspace_switch {
            Some(WorkspaceSwitch::Gesture(gesture)) if gesture.dnd_last_event_time.is_some() => {
                let current = gesture.current_idx;
                let new = current.floor() + 1.;
                new.clamp(0., (self.row_count() - 1) as f64) as i32
            }
            _ => min(current_idx + 1, (self.row_count() - 1) as i32),
        };

        self.activate_row(new_idx);
    }

    pub fn switch_workspace(&mut self, idx: usize) {
        self.activate_row(min(idx, self.row_count() - 1) as i32);
    }

    pub fn switch_workspace_auto_back_and_forth(&mut self, idx: usize) {
        let idx = min(idx, self.row_count() - 1);
        let current = self.canvas.active_row_idx() as usize;

        if idx == current {
            // TODO(TEAM_022): Implement previous row tracking if needed
            // For now, just stay on current row
        } else {
            self.switch_workspace(idx);
        }
    }

    pub fn switch_workspace_previous(&mut self) {
        // TODO(TEAM_022): Implement previous row tracking
        // For now, this is a no-op
    }

    // TEAM_022: Activate a specific row
    fn activate_row(&mut self, row_idx: i32) {
        self.canvas.focus_row(row_idx);
    }

    // =========================================================================
    // Combined window/row navigation
    // =========================================================================

    pub fn move_down_or_to_workspace_down(&mut self) {
        // TEAM_022: Try to move down within column, else move to next row
        if let Some(row) = self.canvas.active_row_mut() {
            if !row.move_down() {
                // Can't move down in column, try next row
                self.switch_workspace_down();
            }
        }
    }

    pub fn move_up_or_to_workspace_up(&mut self) {
        // TEAM_022: Try to move up within column, else move to previous row
        if let Some(row) = self.canvas.active_row_mut() {
            if !row.move_up() {
                // Can't move up in column, try previous row
                self.switch_workspace_up();
            }
        }
    }

    pub fn focus_window_or_workspace_down(&mut self) {
        // TEAM_022: Try to focus down within column, else switch to next row
        if let Some(row) = self.canvas.active_row_mut() {
            if !row.focus_down() {
                self.switch_workspace_down();
            }
        }
    }

    pub fn focus_window_or_workspace_up(&mut self) {
        // TEAM_022: Try to focus up within column, else switch to previous row
        if let Some(row) = self.canvas.active_row_mut() {
            if !row.focus_up() {
                self.switch_workspace_up();
            }
        }
    }

    // =========================================================================
    // Move window to row (was workspace)
    // =========================================================================

    pub fn move_to_workspace_up(&mut self, _focus: bool) {
        // TEAM_022: Move window to row above
        // TODO: Implement move window between rows in Canvas2D
    }

    pub fn move_to_workspace_down(&mut self, _focus: bool) {
        // TEAM_022: Move window to row below
        // TODO: Implement move window between rows in Canvas2D
    }

    pub fn move_to_workspace(
        &mut self,
        _window: Option<&W::Id>,
        _idx: usize,
        _activate: crate::layout::ActivateWindow,
    ) {
        // TEAM_022: Move window to specific row
        // TODO: Implement move window between rows in Canvas2D
    }

    // =========================================================================
    // Move column to row (was workspace)
    // =========================================================================

    pub fn move_column_to_workspace_up(&mut self, _activate: bool) {
        // TEAM_022: Move column to row above
        // TODO: Implement move column between rows in Canvas2D
    }

    pub fn move_column_to_workspace_down(&mut self, _activate: bool) {
        // TEAM_022: Move column to row below
        // TODO: Implement move column between rows in Canvas2D
    }

    pub fn move_column_to_workspace(&mut self, _idx: usize, _activate: bool) {
        // TEAM_022: Move column to specific row
        // TODO: Implement move column between rows in Canvas2D
    }

    // =========================================================================
    // Row render index
    // =========================================================================

    pub fn workspace_render_idx(&self) -> f64 {
        // TEAM_022: Returns the current row index for rendering
        if let Some(switch) = &self.workspace_switch {
            switch.current_idx()
        } else {
            self.canvas.active_row_idx() as f64
        }
    }
}
