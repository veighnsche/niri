// TEAM_063: Layout resize operations
//!
//! Methods for width and height manipulation.

use niri_ipc::SizeChange;
use smithay::utils::{Logical, Point};

use super::super::{InteractiveMoveState, Layout, LayoutElement, MonitorSet, ResizeEdge};

impl<W: LayoutElement> Layout<W> {
    pub fn toggle_width(&mut self, forwards: bool) {
        // TEAM_106: Handle floating windows like main branch
        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let mon = &mut monitors[*active_monitor_idx];
                if mon.canvas.floating_is_active {
                    mon.canvas.floating.toggle_window_width(None, forwards);
                } else if let Some(row) = mon.canvas.active_row_mut() {
                    row.toggle_width(forwards);
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if canvas.floating_is_active {
                    canvas.floating.toggle_window_width(None, forwards);
                } else if let Some(row) = canvas.active_row_mut() {
                    row.toggle_width(forwards);
                }
            }
        }
    }

    pub fn toggle_window_width(&mut self, window: Option<&W::Id>, forwards: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        let workspace = if let Some(window) = window {
            Some(
                self.workspaces_mut()
                    .find(|ws| ws.has_window(window))
                    .unwrap(),
            )
        } else {
            self.active_row_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.toggle_window_width(window, forwards);
    }

    pub fn toggle_window_height(&mut self, window: Option<&W::Id>, forwards: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        let workspace = if let Some(window) = window {
            Some(
                self.workspaces_mut()
                    .find(|ws| ws.has_window(window))
                    .unwrap(),
            )
        } else {
            self.active_row_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.toggle_window_height(window, forwards);
    }

    pub fn toggle_full_width(&mut self) {
        // TEAM_106: Like main branch, do nothing if floating is active
        match &self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                if monitors[*active_monitor_idx].canvas.floating_is_active {
                    return;
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if canvas.floating_is_active {
                    return;
                }
            }
        }
        
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.toggle_full_width();
    }

    pub fn set_column_width(&mut self, change: SizeChange) {
        // TEAM_043: Handle floating windows
        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let mon = &mut monitors[*active_monitor_idx];
                if mon.canvas.floating_is_active {
                    mon.canvas.floating.set_window_width(None, change, true);
                } else if let Some(row) = mon.canvas.active_row_mut() {
                    row.set_column_width(change);
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if canvas.floating_is_active {
                    canvas.floating.set_window_width(None, change, true);
                } else if let Some(row) = canvas.active_row_mut() {
                    row.set_column_width(change);
                }
            }
        }
    }

    pub fn set_window_width(&mut self, window: Option<&W::Id>, change: SizeChange) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        // TEAM_054: Handle floating windows properly
        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                // If a specific window is given, find it
                if let Some(win_id) = window {
                    for mon in monitors.iter_mut() {
                        if mon.canvas.floating.has_window(win_id) {
                            mon.canvas
                                .floating
                                .set_window_width(Some(win_id), change, true);
                            return;
                        }
                        for (_, row) in mon.canvas.rows_mut() {
                            if row.has_window(win_id) {
                                row.set_window_width(Some(win_id), change);
                                return;
                            }
                        }
                    }
                } else {
                    // No specific window - use active
                    let mon = &mut monitors[*active_monitor_idx];
                    if mon.canvas.floating_is_active {
                        mon.canvas.floating.set_window_width(None, change, true);
                    } else if let Some(row) = mon.canvas.active_row_mut() {
                        row.set_window_width(None, change);
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if let Some(win_id) = window {
                    if canvas.floating.has_window(win_id) {
                        canvas.floating.set_window_width(Some(win_id), change, true);
                        return;
                    }
                    for (_, row) in canvas.rows_mut() {
                        if row.has_window(win_id) {
                            row.set_window_width(Some(win_id), change);
                            return;
                        }
                    }
                } else {
                    if canvas.floating_is_active {
                        canvas.floating.set_window_width(None, change, true);
                    } else if let Some(row) = canvas.active_row_mut() {
                        row.set_window_width(None, change);
                    }
                }
            }
        }
    }

    pub fn set_window_height(&mut self, window: Option<&W::Id>, change: SizeChange) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        // TEAM_054: Handle floating windows properly
        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                // If a specific window is given, find it
                if let Some(win_id) = window {
                    for mon in monitors.iter_mut() {
                        if mon.canvas.floating.has_window(win_id) {
                            mon.canvas
                                .floating
                                .set_window_height(Some(win_id), change, true);
                            return;
                        }
                        for (_, row) in mon.canvas.rows_mut() {
                            if row.has_window(win_id) {
                                row.set_window_height(Some(win_id), change);
                                return;
                            }
                        }
                    }
                } else {
                    // No specific window - use active
                    let mon = &mut monitors[*active_monitor_idx];
                    if mon.canvas.floating_is_active {
                        mon.canvas.floating.set_window_height(None, change, true);
                    } else if let Some(row) = mon.canvas.active_row_mut() {
                        row.set_window_height(None, change);
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if let Some(win_id) = window {
                    if canvas.floating.has_window(win_id) {
                        canvas
                            .floating
                            .set_window_height(Some(win_id), change, true);
                        return;
                    }
                    for (_, row) in canvas.rows_mut() {
                        if row.has_window(win_id) {
                            row.set_window_height(Some(win_id), change);
                            return;
                        }
                    }
                } else {
                    if canvas.floating_is_active {
                        canvas.floating.set_window_height(None, change, true);
                    } else if let Some(row) = canvas.active_row_mut() {
                        row.set_window_height(None, change);
                    }
                }
            }
        }
    }

    pub fn reset_window_height(&mut self, window: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        let workspace = if let Some(window) = window {
            Some(
                self.workspaces_mut()
                    .find(|ws| ws.has_window(window))
                    .unwrap(),
            )
        } else {
            self.active_row_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.reset_window_height(window);
    }

    pub fn expand_column_to_available_width(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.expand_column_to_available_width();
    }

    pub fn interactive_resize_begin(&mut self, window: W::Id, edges: ResizeEdge) -> bool {
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(&window) {
                            return ws.interactive_resize_begin(window, edges);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(&window) {
                        return ws.interactive_resize_begin(window, edges);
                    }
                }
            }
        }

        false
    }

    pub fn interactive_resize_update(
        &mut self,
        window: &W::Id,
        delta: Point<f64, Logical>,
    ) -> bool {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return false;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(window) {
                            return ws.interactive_resize_update(window, delta);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(window) {
                        return ws.interactive_resize_update(window, delta);
                    }
                }
            }
        }

        false
    }

    pub fn interactive_resize_end(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(window) {
                            ws.interactive_resize_end(Some(window));
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(window) {
                        ws.interactive_resize_end(Some(window));
                        return;
                    }
                }
            }
        }
    }
}
