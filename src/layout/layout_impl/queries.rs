// TEAM_063: Layout query methods (is_*, has_*, should_*)
//!
//! Read-only query methods for the Layout struct.

use smithay::utils::{Logical, Rectangle, Size};

use super::super::{
    output_size, InteractiveMoveState, Layout, LayoutElement, MonitorSet, RowSwitch,
};

impl<W: LayoutElement> Layout<W> {
    /// Computes the window-geometry-relative target rect for popup unconstraining.
    ///
    /// We will try to fit popups inside this rect.
    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn popup_target_rect(&self, window: &W::Id) -> Rectangle<f64, Logical> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                // Follow the scrolling layout logic and fit the popup horizontally within the
                // window geometry.
                let width = move_.tile.window_size().w;
                let height = output_size(&move_.output).h;
                let mut target = Rectangle::from_size(Size::from((width, height)));
                // FIXME: ideally this shouldn't include the tile render offset, but the code
                // duplication would be a bit annoying for this edge case.
                target.loc.y -= move_.tile_render_location(1.).y;
                return target;
            }
        }

        // Try canvas first
        for mon in self.monitors() {
            if let Some(window_ref) = mon.canvas.find_window(window) {
                return mon.canvas.popup_target_rect(window_ref.2.window());
            }
        }

        // For MonitorSet::NoOutputs, fallback to workspace iteration
        self.workspaces()
            .find_map(|(_, _, ws)| ws.popup_target_rect(window))
            .unwrap()
    }

    pub fn scroll_amount_to_activate(&self, window: &W::Id) -> f64 {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return 0.;
            }
        }

        for mon in self.monitors() {
            for (_, ws) in mon.canvas.rows() {
                if ws.has_window(window) {
                    return ws.scroll_amount_to_activate(window);
                }
            }
        }

        0.
    }

    pub fn should_trigger_focus_follows_mouse_on(&self, window: &W::Id) -> bool {
        // During an animation, it's easy to trigger focus-follows-mouse on the previous workspace,
        // especially when clicking to switch workspace on a bar of some kind. This cancels the
        // workspace switch, which is annoying and not intended.
        //
        // This function allows focus-follows-mouse to trigger only on the animation target
        // workspace.
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return true;
            }
        }

        let MonitorSet::Normal { monitors, .. } = &self.monitor_set else {
            return true;
        };

        let (mon, ws_idx) = monitors
            .iter()
            .find_map(|mon| {
                mon.canvas
                    .rows()
                    .position(|(_, ws)| ws.has_window(window))
                    .map(|ws_idx| (mon, ws_idx))
            })
            .unwrap();

        // During a gesture, focus-follows-mouse does not cause any unintended row switches.
        if let Some(RowSwitch::Gesture(_)) = mon.row_switch {
            return true;
        }

        ws_idx == mon.active_row_idx()
    }

    /// Returns a canvas snapshot including both tiled and floating state.
    ///
    /// Used for golden tests that need to verify floating window behavior.
    #[cfg(test)]
    pub fn canvas_snapshot(&self) -> Option<crate::layout::snapshot::CanvasSnapshot> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        let mon = &monitors[*active_monitor_idx];
        Some(mon.canvas.canvas_snapshot())
    }

    pub fn has_window(&self, window: &W::Id) -> bool {
        self.windows().any(|(_, win)| win.id() == window)
    }
}
