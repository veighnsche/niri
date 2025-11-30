// TEAM_063: Layout window lifecycle operations
//!
//! Methods for adding, removing, and updating windows.

use niri_config::PresetSize;
use niri_ipc::SizeChange;
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;

use super::super::row_types::RowAddWindowTarget;
use super::super::{
    ActivateWindow, AddWindowTarget, InteractiveMoveState, Layout, LayoutElement,
    MonitorAddWindowTarget, MonitorSet, RemovedTile, Transaction,
};

impl<W: LayoutElement> Layout<W> {
    /// Adds a new window to the layout.
    ///
    /// Returns an output that the window was added to, if there were any outputs.
    #[allow(clippy::too_many_arguments)]
    pub fn add_window(
        &mut self,
        window: W,
        target: AddWindowTarget<W>,
        width: Option<PresetSize>,
        height: Option<PresetSize>,
        is_full_width: bool,
        is_floating: bool,
        activate: ActivateWindow,
    ) -> Option<&Output> {
        let scrolling_height = height.map(SizeChange::from);
        let id = window.id().clone();

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let (mon_idx, target) = match target {
                    AddWindowTarget::Auto => (*active_monitor_idx, MonitorAddWindowTarget::Auto),
                    AddWindowTarget::Output(output) => {
                        let mon_idx = monitors
                            .iter()
                            .position(|mon| mon.output == *output)
                            .unwrap();

                        (mon_idx, MonitorAddWindowTarget::Auto)
                    }
                    AddWindowTarget::Row(ws_id) => {
                        let mon_idx = monitors
                            .iter()
                            .position(|mon| mon.canvas.rows().any(|(_, ws)| ws.id() == ws_id))
                            .unwrap();

                        (
                            mon_idx,
                            MonitorAddWindowTarget::Row {
                                id: ws_id,
                                column_idx: None,
                            },
                        )
                    }
                    AddWindowTarget::NextTo(next_to) => {
                        if let Some(output) = self
                            .interactive_move
                            .as_ref()
                            .and_then(|move_| {
                                if let InteractiveMoveState::Moving(move_) = move_ {
                                    Some(move_)
                                } else {
                                    None
                                }
                            })
                            .filter(|move_| next_to == move_.tile.window().id())
                            .map(|move_| move_.output.clone())
                        {
                            // The next_to window is being interactively moved.
                            let mon_idx = monitors
                                .iter()
                                .position(|mon| mon.output == output)
                                .unwrap_or(*active_monitor_idx);

                            (mon_idx, MonitorAddWindowTarget::Auto)
                        } else {
                            let mon_idx = monitors
                                .iter()
                                .position(|mon| {
                                    mon.canvas.rows().any(|(_, ws)| ws.has_window(next_to))
                                })
                                .unwrap();
                            (mon_idx, MonitorAddWindowTarget::NextTo(next_to))
                        }
                    }
                };
                let mon = &mut monitors[mon_idx];

                let (ws_key, _) = mon.resolve_add_window_target(&target);
                // TEAM_055: ws_key is a BTreeMap key, not an ordinal index
                let ws = mon.canvas.row(ws_key).expect("row should exist");
                // TEAM_039: resolve_scrolling_width now takes &W and returns ColumnWidth
                let scrolling_width = Some(ws.resolve_scrolling_width(&window, width));

                mon.add_window(
                    window,
                    Some(target),
                    activate,
                    scrolling_width,
                    is_full_width,
                );

                if activate.map_smart(|| false) {
                    *active_monitor_idx = mon_idx;
                }

                // Set the default height for scrolling windows.
                if !is_floating {
                    if let Some(change) = scrolling_height {
                        // TEAM_057: Set window height using Canvas2D method
                        mon.canvas_mut().set_window_height(&id, change);
                    }
                }

                Some(&mon.output)
            }
            MonitorSet::NoOutputs { canvas } => {
                let (ws_idx, _target) = match target {
                    AddWindowTarget::Auto => {
                        // In Canvas2D, we always add to the active row (row 0 by default)
                        (0, RowAddWindowTarget::Auto)
                    }
                    AddWindowTarget::Output(_) => panic!(),
                    AddWindowTarget::Row(ws_id) => {
                        // TEAM_057: Find the row with the given row ID
                        let ws_idx = canvas.find_row_by_id(ws_id).unwrap_or(0);
                        (ws_idx, RowAddWindowTarget::Auto)
                    }
                    AddWindowTarget::NextTo(next_to) => {
                        if self
                            .interactive_move
                            .as_ref()
                            .and_then(|move_| {
                                if let InteractiveMoveState::Moving(move_) = move_ {
                                    Some(move_)
                                } else {
                                    None
                                }
                            })
                            .filter(|move_| next_to == move_.tile.window().id())
                            .is_some()
                        {
                            // The next_to window is being interactively moved. If there are no
                            // other windows, we may have no workspaces at all.
                            // In Canvas2D, we always have at least the origin row
                            (0, RowAddWindowTarget::Auto)
                        } else {
                            // Find the row that contains the next_to window
                            if let Some((ws_idx, _, tile)) = canvas.find_window(&next_to) {
                                (ws_idx, RowAddWindowTarget::NextTo(tile.window()))
                            } else {
                                // Default to origin row if not found
                                (0, RowAddWindowTarget::Auto)
                            }
                        }
                    }
                };
                // TEAM_033: Fixed tile creation and add_tile arguments
                // First ensure the row exists
                let ws = canvas.ensure_row(ws_idx);
                // TEAM_039: resolve_scrolling_width now takes &W and returns ColumnWidth
                let scrolling_width = ws.resolve_scrolling_width(&window, width);

                // Create tile using canvas's make_tile (returns proper Tile<W>)
                let tile = canvas.make_tile(window);
                let activate_bool = activate.map_smart(|| false);

                // Get the row again (ensure_row may have modified the canvas)
                if let Some(ws) = canvas.row_mut(ws_idx) {
                    ws.add_tile(None, tile, activate_bool, scrolling_width, is_full_width);

                    // Set the default height for scrolling windows.
                    if !is_floating {
                        if let Some(change) = scrolling_height {
                            ws.set_window_height(Some(&id), change);
                        }
                    }
                }

                None
            }
        }
    }

    pub fn remove_window(
        &mut self,
        window: &W::Id,
        transaction: Transaction,
    ) -> Option<RemovedTile<W>> {
        if let Some(state) = &self.interactive_move {
            match state {
                InteractiveMoveState::Starting { window_id, .. } => {
                    if window_id == window {
                        self.interactive_move_end(window);
                    }
                }
                InteractiveMoveState::Moving(move_) => {
                    if move_.tile.window().id() == window {
                        let Some(InteractiveMoveState::Moving(move_)) =
                            self.interactive_move.take()
                        else {
                            unreachable!()
                        };

                        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
                        for mon in self.monitors_mut() {
                            mon.dnd_scroll_gesture_end();
                            mon.canvas_mut().dnd_scroll_gesture_end();
                        }

                        return Some(RemovedTile {
                            tile: move_.tile,
                            width: move_.width,
                            is_full_width: move_.is_full_width,
                            is_floating: false,
                            is_maximized: false,
                        });
                    }
                }
            }
        }

        // TEAM_033: Restructured to avoid borrow checker issues
        // TEAM_059: Check floating space first, then rows
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_059: Check floating space first
                    if mon.canvas.floating.has_window(window) {
                        let removed = mon.canvas.floating.remove_tile(window);
                        return Some(removed);
                    }

                    // TEAM_055: First pass: find which row has the window
                    // Use the actual BTreeMap key, not enumeration index
                    let found_row_key = mon.canvas.rows().find_map(|(key, ws)| {
                        if ws.has_window(window) {
                            Some(key)
                        } else {
                            None
                        }
                    });

                    if let Some(row_key) = found_row_key {
                        // Get ordinal position for comparisons
                        let ws_ord_idx = mon
                            .canvas
                            .rows()
                            .position(|(k, _)| k == row_key)
                            .unwrap_or(0);

                        // Now we can mutably access the specific row using its key
                        let removed = mon
                            .canvas
                            .row_mut(row_key)
                            .expect("row should exist")
                            .remove_tile(window, transaction);

                        // Get state we need for cleanup checks
                        let ws_has_windows = mon
                            .canvas
                            .row(row_key)
                            .map(|ws| ws.has_windows_or_name())
                            .unwrap_or(false);
                        let active_ord_idx = mon.active_row_idx();
                        let ws_count = mon.canvas.rows().count();
                        let switch_in_progress = mon.row_switch.is_some();

                        // Clean up empty workspaces that are not active and not last.
                        if !ws_has_windows
                            && ws_ord_idx != active_ord_idx
                            && ws_ord_idx != ws_count - 1
                            && !switch_in_progress
                        {
                            mon.canvas.remove_row(row_key);
                        }

                        // TEAM_055: Adjust active_row_idx if row at lower key was removed
                        // Note: This adjustment only makes sense if we track ordinal position
                        // Since we now use keys, we skip this adjustment - the key is stable

                        // Special case handling when empty_row_above_first is set and all
                        // workspaces are empty.
                        if mon.options.layout.empty_row_above_first
                            && mon.canvas.rows().count() == 2
                            && !switch_in_progress
                        {
                            let keys: Vec<i32> = mon.canvas.rows.keys().copied().collect();
                            if keys.len() == 2 {
                                let ws0_empty = mon
                                    .canvas
                                    .row(keys[0])
                                    .map(|ws| !ws.has_windows_or_name())
                                    .unwrap_or(true);
                                let ws1_empty = mon
                                    .canvas
                                    .row(keys[1])
                                    .map(|ws| !ws.has_windows_or_name())
                                    .unwrap_or(true);
                                if ws0_empty && ws1_empty {
                                    mon.canvas.remove_row(keys[1]);
                                    mon.canvas.active_row_idx = keys[0];
                                }
                            }
                        }
                        return Some(removed);
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // TEAM_059: Check floating space first
                if canvas.floating.has_window(window) {
                    let removed = canvas.floating.remove_tile(window);
                    return Some(removed);
                }

                // TEAM_055: First pass: find which row has the window
                // Use the actual BTreeMap key, not enumeration index
                let found_row_key = canvas.rows().find_map(|(key, ws)| {
                    if ws.has_window(window) {
                        Some(key)
                    } else {
                        None
                    }
                });

                if let Some(row_key) = found_row_key {
                    let removed = canvas
                        .row_mut(row_key)
                        .expect("row should exist")
                        .remove_tile(window, transaction);

                    // Clean up empty workspaces.
                    let ws_has_windows = canvas
                        .row(row_key)
                        .map(|ws| ws.has_windows_or_name())
                        .unwrap_or(false);
                    if !ws_has_windows {
                        canvas.remove_row(row_key);
                    }

                    return Some(removed);
                }
            }
        }

        None
    }

    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn descendants_added(&mut self, id: &W::Id) -> bool {
        // Check canvas rows first
        for mon in self.monitors_mut() {
            if mon.canvas.descendants_added(id) {
                return true;
            }
        }

        // For MonitorSet::NoOutputs, still need workspace iteration
        for ws in self.workspaces_mut() {
            if ws.descendants_added(id) {
                return true;
            }
        }

        false
    }

    pub fn update_window(&mut self, window: &W::Id, serial: Option<Serial>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().id() == window {
                // Do this before calling update_window() so it can get up-to-date info.
                if let Some(serial) = serial {
                    move_.tile.window_mut().on_commit(serial);
                }

                move_.tile.update_window();
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_044: Check floating space first
                    if mon.canvas.floating.has_window(window) {
                        mon.canvas.floating.update_window(window, serial);
                        return;
                    }
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(window) {
                            ws.update_window(window, serial);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // TEAM_044: Check floating space first
                if canvas.floating.has_window(window) {
                    canvas.floating.update_window(window, serial);
                    return;
                }
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(window) {
                        ws.update_window(window, serial);
                        return;
                    }
                }
            }
        }
    }

    pub fn find_window_and_output(&self, wl_surface: &WlSurface) -> Option<(&W, Option<&Output>)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().is_wl_surface(wl_surface) {
                return Some((move_.tile.window(), Some(&move_.output)));
            }
        }

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_054: Use canvas.find_wl_surface which includes floating
                    if let Some(window) = mon.canvas.find_wl_surface(wl_surface) {
                        return Some((window, Some(&mon.output)));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_054: Use canvas.find_wl_surface which includes floating
                if let Some(window) = canvas.find_wl_surface(wl_surface) {
                    return Some((window, None));
                }
            }
        }

        None
    }

    pub fn find_window_and_output_mut(
        &mut self,
        wl_surface: &WlSurface,
    ) -> Option<(&mut W, Option<&Output>)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().is_wl_surface(wl_surface) {
                return Some((move_.tile.window_mut(), Some(&move_.output)));
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_054: Use canvas.find_wl_surface_mut which includes floating
                    if let Some(window) = mon.canvas.find_wl_surface_mut(wl_surface) {
                        return Some((window, Some(&mon.output)));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_054: Use canvas.find_wl_surface_mut which includes floating
                if let Some(window) = canvas.find_wl_surface_mut(wl_surface) {
                    return Some((window, None));
                }
            }
        }

        None
    }
}
