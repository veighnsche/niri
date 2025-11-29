// TEAM_063: Layout navigation operations
//!
//! Methods for focus movement, scrolling, and window movement.

use niri_ipc::ColumnDisplay;
use smithay::output::Output;
use smithay::utils::{Logical, Point};

use super::super::{
    ActivateWindow, HitType, InteractiveMoveState, Layout, LayoutElement, MonitorSet, ResizeEdge,
    ScrollDirection,
};

impl<W: LayoutElement> Layout<W> {
    pub fn move_left(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_left();
        }
    }

    pub fn move_right(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_right();
        }
    }

    pub fn move_column_to_first(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_column_to_first();
        }
    }

    pub fn move_column_to_last(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_column_to_last();
        }
    }

    pub fn move_column_left_or_to_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.move_left() {
                return false;
            }
        }

        self.move_column_to_output(output, None, true);
        true
    }

    pub fn move_column_right_or_to_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.move_right() {
                return false;
            }
        }

        self.move_column_to_output(output, None, true);
        true
    }

    pub fn move_column_to_index(&mut self, index: usize) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.move_column_to_index(index);
    }

    pub fn move_down(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.move_down();
    }

    pub fn move_up(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.move_up();
    }

    // TEAM_012: Renamed from move_down_or_to_workspace_down
    pub fn move_down_or_to_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_down_or_to_workspace_down();
    }

    // TEAM_012: Renamed from move_up_or_to_workspace_up
    pub fn move_up_or_to_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_up_or_to_workspace_up();
    }

    pub fn consume_or_expel_window_left(&mut self, window: Option<&W::Id>) {
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
        workspace.consume_or_expel_window_left(window);
    }

    pub fn consume_or_expel_window_right(&mut self, window: Option<&W::Id>) {
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
        workspace.consume_or_expel_window_right(window);
    }

    pub fn focus_left(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_left();
    }

    pub fn focus_right(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_right();
    }

    pub fn focus_column_first(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_column_first();
    }

    pub fn focus_column_last(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_column_last();
    }

    pub fn focus_column_right_or_first(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_column_right_or_first();
    }

    pub fn focus_column_left_or_last(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_column_left_or_last();
    }

    pub fn focus_column(&mut self, index: usize) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_column(index);
    }

    pub fn focus_window_up_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.focus_up() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_window_down_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.focus_down() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_column_left_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.focus_left() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_column_right_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_row_mut() {
            if workspace.focus_right() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_window_in_column(&mut self, index: u8) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        // TEAM_040: Fixed - pass u8 directly
        workspace.focus_window_in_column(index);
    }

    pub fn focus_down(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_down();
    }

    pub fn focus_up(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_up();
    }

    pub fn focus_down_or_left(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_down_or_left();
    }

    pub fn focus_down_or_right(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_down_or_right();
    }

    pub fn focus_up_or_left(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_up_or_left();
    }

    pub fn focus_up_or_right(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_up_or_right();
    }

    // TEAM_012: Renamed from focus_window_or_workspace_down
    pub fn focus_window_or_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.focus_window_or_workspace_down();
    }

    // TEAM_012: Renamed from focus_window_or_workspace_up
    pub fn focus_window_or_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.focus_window_or_workspace_up();
    }

    pub fn focus_window_top(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_window_top();
    }

    pub fn focus_window_bottom(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_window_bottom();
    }

    pub fn focus_window_down_or_top(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_window_down_or_top();
    }

    pub fn focus_window_up_or_bottom(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.focus_window_up_or_bottom();
    }

    // TEAM_012: Renamed from move_to_workspace_up
    pub fn move_to_row_up(&mut self, focus: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_to_workspace_up(focus);
    }

    // TEAM_012: Renamed from move_to_workspace_down
    pub fn move_to_row_down(&mut self, focus: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_to_workspace_down(focus);
    }

    pub fn move_to_row(&mut self, window: Option<&W::Id>, idx: usize, activate: ActivateWindow) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        let monitor = if let Some(window) = window {
            match &mut self.monitor_set {
                MonitorSet::Normal { monitors, .. } => monitors
                    .iter_mut()
                    .find(|mon| mon.has_window(window))
                    .unwrap(),
                MonitorSet::NoOutputs { .. } => {
                    return;
                }
            }
        } else {
            let Some(monitor) = self.active_monitor() else {
                return;
            };
            monitor
        };
        monitor.move_to_workspace(window, idx, activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace_up
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_column_to_row_up(&mut self, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_column_to_row_up(activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace_down
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_column_to_row_down(&mut self, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_column_to_row_down(activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace
    pub fn move_column_to_row(&mut self, idx: usize, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_column_to_row(idx, activate);
    }

    // TEAM_012: Renamed from switch_row_up
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn focus_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().focus_row_up();
    }

    // TEAM_012: Renamed from switch_row_down
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn focus_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().focus_row_down();
    }

    // TEAM_012: Renamed from switch_row
    pub fn focus_row(&mut self, idx: usize) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_row(idx);
    }

    // TEAM_012: Renamed from switch_row_auto_back_and_forth
    pub fn focus_row_auto_back_and_forth(&mut self, idx: usize) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_row_auto_back_and_forth(idx);
    }

    // TEAM_012: Renamed from switch_row_previous
    pub fn focus_previous_position(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_row_previous();
    }

    pub fn consume_into_column(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.consume_into_column();
    }

    pub fn expel_from_column(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.expel_from_column();
    }

    pub fn swap_window_in_direction(&mut self, direction: ScrollDirection) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.swap_window_in_direction(direction);
    }

    pub fn toggle_column_tabbed_display(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.toggle_column_tabbed_display();
    }

    pub fn set_column_display(&mut self, display: ColumnDisplay) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.set_column_display(display);
    }

    pub fn center_column(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.center_column();
    }

    pub fn center_window(&mut self, id: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if id.is_none() || id == Some(move_.tile.window().id()) {
                return;
            }
        }

        // Find the monitor that contains the window and call canvas.center_window
        if let Some(id) = id {
            for monitor in self.monitors_mut() {
                if monitor.canvas().has_window(id) {
                    monitor.canvas_mut().center_window(Some(id));
                    break;
                }
            }
        } else {
            // Center active window on active monitor
            if let Some(monitor) = self.active_monitor() {
                monitor.canvas_mut().center_window(None);
            }
        }
    }

    pub fn center_visible_columns(&mut self) {
        let Some(workspace) = self.active_row_mut() else {
            return;
        };
        workspace.center_visible_columns();
    }

    pub fn focus(&self) -> Option<&W> {
        self.focus_with_output().map(|(win, _out)| win)
    }

    pub fn focus_with_output(&self) -> Option<(&W, &Output)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            return Some((move_.tile.window(), &move_.output));
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        let mon = &monitors[*active_monitor_idx];
        mon.active_window().map(|win| (win, &mon.output))
    }

    pub fn interactive_moved_window_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&W, HitType)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.output == *output {
                // TEAM_014: Removed overview zoom handling (Part 3)
                let tile_pos = move_.tile_render_location(1.);
                HitType::hit_tile(&move_.tile, tile_pos, pos_within_output)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the window under the cursor and the hit type.
    pub fn window_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&W, HitType)> {
        let mon = self.monitor_for_output(output)?;
        mon.window_under(pos_within_output)
    }

    pub fn resize_edges_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<ResizeEdge> {
        let mon = self.monitor_for_output(output)?;
        mon.resize_edges_under(pos_within_output)
    }

    pub fn workspace_under(
        &self,
        extended_bounds: bool,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<&crate::layout::row::Row<W>> {
        if self
            .interactive_moved_window_under(output, pos_within_output)
            .is_some()
        {
            return None;
        }

        let mon = self.monitor_for_output(output)?;
        if extended_bounds {
            mon.row_under(pos_within_output).map(|(ws, _)| ws)
        } else {
            mon.row_under_narrow(pos_within_output)
        }
    }
}
