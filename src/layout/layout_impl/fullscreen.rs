// TEAM_063: Layout fullscreen/maximize operations
//!
//! Methods for fullscreen and maximize window states.

use super::super::{InteractiveMoveState, Layout, LayoutElement};

impl<W: LayoutElement> Layout<W> {
    pub fn set_fullscreen(&mut self, id: &W::Id, is_fullscreen: bool) {
        // Check if this is a request to unset the windowed fullscreen state.
        if !is_fullscreen {
            let mut handled = false;
            self.with_windows_mut(|window, _| {
                if window.id() == id && window.is_pending_windowed_fullscreen() {
                    window.request_windowed_fullscreen(false);
                    handled = true;
                }
            });
            if handled {
                return;
            }
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.set_fullscreen(id, is_fullscreen);
                return;
            }
        }

        // For MonitorSet::NoOutputs, fallback to workspace iteration
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.set_fullscreen(id, is_fullscreen);
                return;
            }
        }
    }

    pub fn toggle_fullscreen(&mut self, id: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.toggle_fullscreen(id);
                return;
            }
        }

        // For MonitorSet::NoOutputs, fallback to workspace iteration
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.toggle_fullscreen(id);
                return;
            }
        }
    }

    pub fn toggle_windowed_fullscreen(&mut self, id: &W::Id) {
        let (_, window) = self.windows().find(|(_, win)| win.id() == id).unwrap();
        if window.pending_sizing_mode().is_fullscreen() {
            // Remove the real fullscreen.
            // TEAM_021: Try canvas first, then fallback to workspace
            for mon in self.monitors_mut() {
                if mon.canvas.has_window(id) {
                    mon.canvas.set_fullscreen(id, false);
                    break;
                }
            }

            // Canvas2D now manages all rows, workspace fallback removed
        }

        // This will switch is_pending_fullscreen() to false right away.
        self.with_windows_mut(|window, _| {
            if window.id() == id {
                window.request_windowed_fullscreen(!window.is_pending_windowed_fullscreen());
            }
        });
    }

    pub fn set_maximized(&mut self, id: &W::Id, maximize: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.set_maximized(id, maximize);
                return;
            }
        }

        // For MonitorSet::NoOutputs, fallback to workspace iteration
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.set_maximized(id, maximize);
                return;
            }
        }
    }

    pub fn toggle_maximized(&mut self, id: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.toggle_maximized(id);
                return;
            }
        }

        // For MonitorSet::NoOutputs, fallback to workspace iteration
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.toggle_maximized(id);
                return;
            }
        }
    }
}
