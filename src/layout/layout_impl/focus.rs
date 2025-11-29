// TEAM_063: Layout focus and activation operations
//!
//! Methods for window activation and focus management.

use niri_ipc::WindowLayout;
use smithay::output::Output;

use super::super::{
    InteractiveMoveState, Layout, LayoutElement, Monitor, MonitorSet, WorkspaceSwitch,
    row_types::RowId,
};

impl<W: LayoutElement> Layout<W> {
    pub fn activate_window(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return;
        };

        // TEAM_033: Restructure to avoid borrow issues - find first, then operate
        for (monitor_idx, mon) in monitors.iter_mut().enumerate() {
            // First find the workspace with the window (immutable scan)
            let found_ws_idx = mon.canvas.rows().enumerate().find_map(|(idx, (_, ws))| {
                if ws.has_window(window) {
                    Some(idx)
                } else {
                    None
                }
            });

            if let Some(workspace_idx) = found_ws_idx {
                // Now activate the window
                if let Some(ws) = mon.canvas.row_mut(workspace_idx as i32) {
                    if ws.activate_window(window) {
                        *active_monitor_idx = monitor_idx;

                        // If currently in the middle of a vertical swipe between the target workspace
                        // and some other, don't switch the workspace.
                        match &mon.workspace_switch {
                            Some(WorkspaceSwitch::Gesture(gesture))
                                if gesture.current_idx.floor() == workspace_idx as f64
                                    || gesture.current_idx.ceil() == workspace_idx as f64 => {}
                            _ => mon.switch_row(workspace_idx),
                        }

                        return;
                    }
                }
            }
        }
    }

    pub fn activate_window_without_raising(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return;
        };

        for (monitor_idx, mon) in monitors.iter_mut().enumerate() {
            // TEAM_035: Find workspace index first, then switch to avoid double borrow
            let found_ws_idx = mon.canvas.rows_mut()
                .enumerate()
                .find_map(|(workspace_idx, (_, ws))| {
                    if ws.activate_window_without_raising(window) {
                        Some(workspace_idx)
                    } else {
                        None
                    }
                });

            if let Some(workspace_idx) = found_ws_idx {
                *active_monitor_idx = monitor_idx;

                // If currently in the middle of a vertical swipe between the target workspace
                // and some other, don't switch the workspace.
                match &mon.workspace_switch {
                    Some(WorkspaceSwitch::Gesture(gesture))
                        if gesture.current_idx.floor() == workspace_idx as f64
                            || gesture.current_idx.ceil() == workspace_idx as f64 => {}
                    _ => mon.switch_row(workspace_idx),
                }

                return;
            }
        }
    }

    pub fn active_output(&self) -> Option<&Output> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        Some(&monitors[*active_monitor_idx].output)
    }

    pub fn active_row(&self) -> Option<&crate::layout::row::Row<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        let mon = &monitors[*active_monitor_idx];
        mon.canvas.active_row()
    }

    pub fn active_row_mut(&mut self) -> Option<&mut crate::layout::row::Row<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return None;
        };

        let mon = &mut monitors[*active_monitor_idx];
        mon.canvas.active_row_mut()
    }

    pub fn windows_for_output(&self, output: &Output) -> impl Iterator<Item = &W> + '_ {
        let MonitorSet::Normal { monitors, .. } = &self.monitor_set else {
            panic!()
        };

        let moving_window = self
            .interactive_move
            .as_ref()
            .and_then(|x| x.moving())
            .filter(|move_| move_.output == *output)
            .map(|move_| move_.tile.window())
            .into_iter();

        let mon = monitors.iter().find(|mon| &mon.output == output).unwrap();
        let mon_windows = mon.canvas.rows().flat_map(|(_, ws)| ws.windows());

        moving_window.chain(mon_windows)
    }

    pub fn windows_for_output_mut(&mut self, output: &Output) -> impl Iterator<Item = &mut W> + '_ {
        let MonitorSet::Normal { monitors, .. } = &mut self.monitor_set else {
            panic!()
        };

        let moving_window = self
            .interactive_move
            .as_mut()
            .and_then(|x| x.moving_mut())
            .filter(|move_| move_.output == *output)
            .map(|move_| move_.tile.window_mut())
            .into_iter();

        let mon = monitors
            .iter_mut()
            .find(|mon| &mon.output == output)
            .unwrap();
        // TEAM_035: Extract row from tuple
        let mon_windows = mon.canvas.rows_mut().flat_map(|(_, ws)| ws.windows_mut());

        moving_window.chain(mon_windows)
    }

    pub fn with_windows(
        &self,
        mut f: impl FnMut(&W, Option<&Output>, Option<RowId>, WindowLayout),
    ) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            // We don't fill any positions for interactively moved windows.
            let layout = move_.tile.ipc_layout_template();
            f(move_.tile.window(), Some(&move_.output), None, layout);
        }

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows() {
                        for (tile, layout) in ws.tiles_with_ipc_layouts() {
                            f(tile.window(), Some(&mon.output), Some(ws.id()), layout);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Use workspaces() not workspaces_mut() since we have &self
                for (_, ws) in canvas.rows() {
                    for (tile, layout) in ws.tiles_with_ipc_layouts() {
                        f(tile.window(), None, Some(ws.id()), layout);
                    }
                }
            }
        }
    }

    pub fn with_windows_mut(&mut self, mut f: impl FnMut(&mut W, Option<&Output>)) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            f(move_.tile.window_mut(), Some(&move_.output));
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_059: Include floating windows
                    for tile in mon.canvas.floating.tiles_mut() {
                        f(tile.window_mut(), Some(&mon.output));
                    }
                    for (_, ws) in mon.canvas.rows_mut() {
                        for win in ws.windows_mut() {
                            f(win, Some(&mon.output));
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_059: Include floating windows
                for tile in canvas.floating.tiles_mut() {
                    f(tile.window_mut(), None);
                }
                for (_, ws) in canvas.rows_mut() {
                    for win in ws.windows_mut() {
                        f(win, None);
                    }
                }
            }
        }
    }

    pub fn active_monitor_mut(&mut self) -> Option<&mut Monitor<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return None;
        };

        Some(&mut monitors[*active_monitor_idx])
    }

    pub fn active_monitor_ref(&self) -> Option<&Monitor<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        Some(&monitors[*active_monitor_idx])
    }

    pub fn monitors(&self) -> impl Iterator<Item = &Monitor<W>> + '_ {
        let monitors = if let MonitorSet::Normal { monitors, .. } = &self.monitor_set {
            &monitors[..]
        } else {
            &[][..]
        };

        monitors.iter()
    }

    pub fn monitors_mut(&mut self) -> impl Iterator<Item = &mut Monitor<W>> + '_ {
        let monitors = if let MonitorSet::Normal { monitors, .. } = &mut self.monitor_set {
            &mut monitors[..]
        } else {
            &mut [][..]
        };

        monitors.iter_mut()
    }
}
