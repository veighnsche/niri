// TEAM_064: Interactive move and DnD methods extracted from mod.rs
//!
//! This module contains methods for interactive window moving and drag-and-drop operations.

use std::rc::Rc;

use smithay::output::Output;
use smithay::utils::{Logical, Point};

use super::types::{DndData, DndHold, DndHoldTarget, InteractiveMoveData, InteractiveMoveState};
use crate::layout::monitor::{InsertHint, InsertPosition, InsertWorkspace, MonitorAddWindowTarget};
use crate::layout::tile::Tile;
use crate::layout::{
    ActivateWindow, Layout, LayoutElement, MonitorSet, Options, RemovedTile,
    INTERACTIVE_MOVE_ALPHA, INTERACTIVE_MOVE_START_THRESHOLD,
};
use crate::rubber_band::RubberBand;
use crate::utils::{ensure_min_max_size_maybe_zero, output_size};

impl<W: LayoutElement> Layout<W> {
    /// Begins an interactive move for a window.
    ///
    /// Returns true if the move was started successfully.
    pub fn interactive_move_begin(
        &mut self,
        window_id: W::Id,
        output: &Output,
        start_pos_within_output: Point<f64, Logical>,
    ) -> bool {
        if self.interactive_move.is_some() {
            return false;
        }

        // TEAM_059: Rewritten to extract all data upfront to avoid borrow checker issues.
        // We collect needed data into owned values before mutating self.interactive_move.

        // Try to find the window and extract needed data
        enum WindowLocation {
            Tiled {
                is_floating: bool,
                pointer_ratio_within_window: (f64, f64),
            },
            Floating {
                pointer_ratio_within_window: (f64, f64),
            },
            NotFound,
            WrongOutput,
        }

        let location = {
            // First check tiled rows
            let tiled_info = self.monitors().find_map(|mon| {
                mon.workspaces_with_render_geo()
                    .find(|(ws, _)| ws.has_window(&window_id))
                    .map(|(ws, ws_geo)| {
                        let is_correct_output = mon.output() == output;
                        let is_floating = ws.is_floating(&window_id);
                        let tile_info = ws.tiles_with_render_positions()
                            .find(|(tile, _, _)| tile.window().id() == &window_id)
                            .map(|(tile, tile_offset, _)| {
                                let window_offset = tile.window_loc();
                                let tile_pos = ws_geo.loc + tile_offset;
                                let pointer_offset_within_window =
                                    start_pos_within_output - tile_pos - window_offset;
                                let window_size = tile.window_size();
                                (
                                    f64::clamp(pointer_offset_within_window.x / window_size.w, 0., 1.),
                                    f64::clamp(pointer_offset_within_window.y / window_size.h, 0., 1.),
                                )
                            });
                        (is_correct_output, is_floating, tile_info)
                    })
            });

            if let Some((is_correct_output, is_floating, Some(pointer_ratio))) = tiled_info {
                if !is_correct_output {
                    WindowLocation::WrongOutput
                } else {
                    WindowLocation::Tiled {
                        is_floating,
                        pointer_ratio_within_window: pointer_ratio,
                    }
                }
            } else {
                // Check floating space
                let floating_info = self.monitors().find_map(|mon| {
                    if mon.output() != output || !mon.canvas.floating.has_window(&window_id) {
                        return None;
                    }
                    mon.canvas.floating.tiles_with_offsets()
                        .find(|(tile, _)| tile.window().id() == &window_id)
                        .map(|(tile, offset)| {
                            let tile_pos = offset;
                            let window_size = tile.window_size();
                            let window_offset = tile.window_loc();
                            let pointer_offset_within_window =
                                start_pos_within_output - tile_pos - window_offset;
                            (
                                f64::clamp(pointer_offset_within_window.x / window_size.w, 0., 1.),
                                f64::clamp(pointer_offset_within_window.y / window_size.h, 0., 1.),
                            )
                        })
                });

                if let Some(pointer_ratio) = floating_info {
                    WindowLocation::Floating {
                        pointer_ratio_within_window: pointer_ratio,
                    }
                } else {
                    WindowLocation::NotFound
                }
            }
        };

        // Now we can mutate self without borrow issues
        match location {
            WindowLocation::Tiled { is_floating, pointer_ratio_within_window } => {
                self.interactive_move = Some(InteractiveMoveState::Starting {
                    window_id,
                    pointer_delta: Point::from((0., 0.)),
                    pointer_ratio_within_window,
                });

                // Lock the view for scrolling interactive move.
                if !is_floating {
                    for mon in self.monitors_mut() {
                        mon.canvas_mut().dnd_scroll_gesture_begin();
                    }
                }

                true
            }
            WindowLocation::Floating { pointer_ratio_within_window } => {
                self.interactive_move = Some(InteractiveMoveState::Starting {
                    window_id,
                    pointer_delta: Point::from((0., 0.)),
                    pointer_ratio_within_window,
                });

                // Floating windows don't need view locking
                true
            }
            WindowLocation::NotFound | WindowLocation::WrongOutput => false,
        }
    }

    /// Updates an ongoing interactive move.
    ///
    /// Returns true if the update was processed successfully.
    pub fn interactive_move_update(
        &mut self,
        window: &W::Id,
        delta: Point<f64, Logical>,
        output: Output,
        pointer_pos_within_output: Point<f64, Logical>,
    ) -> bool {
        let Some(state) = self.interactive_move.take() else {
            return false;
        };

        match state {
            InteractiveMoveState::Starting {
                window_id,
                mut pointer_delta,
                pointer_ratio_within_window,
            } => {
                if window_id != *window {
                    self.interactive_move = Some(InteractiveMoveState::Starting {
                        window_id,
                        pointer_delta,
                        pointer_ratio_within_window,
                    });
                    return false;
                }

                // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

                pointer_delta += delta;

                let (cx, cy) = (pointer_delta.x, pointer_delta.y);
                let sq_dist = cx * cx + cy * cy;

                let factor = RubberBand {
                    stiffness: 1.0,
                    limit: 0.5,
                }
                .band(sq_dist / INTERACTIVE_MOVE_START_THRESHOLD);

                // TEAM_059: Check tiled rows first, then floating space
                let found_in_rows = self
                    .workspaces_mut()
                    .find(|ws| ws.has_window(&window_id))
                    .map(|ws| {
                        let workspace_config = ws.layout_config().map(|c| (ws.id(), c.clone()));
                        let is_floating = ws.is_floating(&window_id);
                        let tile = ws.tiles_mut()
                            .find(|tile| *tile.window().id() == window_id)
                            .unwrap();
                        tile.interactive_move_offset = pointer_delta.upscale(factor);
                        (is_floating, workspace_config)
                    });

                let (is_floating, workspace_config) = if let Some((is_floating, workspace_config)) = found_in_rows {
                    (is_floating, workspace_config)
                } else {
                    // Check floating space
                    let found_floating = self.monitors_mut().find_map(|mon| {
                        mon.canvas.floating.tiles_mut()
                            .find(|tile| *tile.window().id() == window_id)
                            .map(|tile| {
                                tile.interactive_move_offset = pointer_delta.upscale(factor);
                            })
                    });
                    if found_floating.is_none() {
                        // Window not found anywhere, return false
                        self.interactive_move = Some(InteractiveMoveState::Starting {
                            window_id,
                            pointer_delta,
                            pointer_ratio_within_window,
                        });
                        return false;
                    }
                    (true, None) // Floating windows don't have workspace config
                };

                // Put it back to be able to easily return.
                self.interactive_move = Some(InteractiveMoveState::Starting {
                    window_id: window_id.clone(),
                    pointer_delta,
                    pointer_ratio_within_window,
                });

                if !is_floating && sq_dist < INTERACTIVE_MOVE_START_THRESHOLD {
                    return true;
                }

                let output_config = self
                    .monitors()
                    .find(|mon| mon.output() == &output)
                    .and_then(|mon| mon.layout_config().cloned());

                // If the pointer is currently on the window's own output, then we can animate the
                // window movement from its current (rubberbanded and possibly moved away) position
                // to the pointer. Otherwise, we just teleport it as the layout code is not aware
                // of monitor positions.
                //
                // FIXME: when and if the layout code knows about monitor positions, this will be
                // potentially animatable.
                let mut tile_pos = None;
                if let Some((mon, (ws, ws_geo))) = self.monitors().find_map(|mon| {
                    mon.workspaces_with_render_geo()
                        .find(|(ws, _)| ws.has_window(window))
                        .map(|rv| (mon, rv))
                }) {
                    if mon.output() == &output {
                        let (_, tile_offset, _) = ws
                            .tiles_with_render_positions()
                            .find(|(tile, _, _)| tile.window().id() == window)
                            .unwrap();

                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
                        tile_pos = Some((ws_geo.loc + tile_offset, 1.0));
                    }
                }

                // Clear it before calling remove_window() to avoid running interactive_move_end()
                // in the middle of interactive_move_update() and the confusion that causes.
                self.interactive_move = None;

                // Unset fullscreen before removing the tile. This will restore its size properly,
                // and move it to floating if needed, so we don't have to deal with that here.
                // TEAM_059: Only needed for tiled windows (floating can't be fullscreen/maximized)
                let should_restore_to_floating = if !is_floating {
                    if let Some(ws) = self.workspaces_mut().find(|ws| ws.has_window(&window_id)) {
                        ws.set_fullscreen(window, false) || ws.set_maximized(window, false)
                    } else {
                        false
                    }
                } else {
                    false
                };

                let RemovedTile {
                    mut tile,
                    width,
                    is_full_width,
                    is_floating: was_floating,
                    is_maximized,
                } = self.remove_window(window, crate::utils::transaction::Transaction::new()).unwrap();
                
                // TEAM_059: If window should restore to floating, treat it as floating
                let is_floating = was_floating || should_restore_to_floating;

                tile.stop_move_animations();
                tile.interactive_move_offset = Point::from((0., 0.));
                tile.window().output_enter(&output);
                tile.window().set_preferred_scale_transform(
                    output.current_scale(),
                    output.current_transform(),
                );

                let view_size = output_size(&output);
                let scale = output.current_scale().fractional_scale();
                let options = Options::clone(&self.options)
                    .with_merged_layout(output_config.as_ref())
                    .with_merged_layout(workspace_config.as_ref().map(|(_, c)| c))
                    .adjusted_for_scale(scale);
                tile.update_config(view_size, scale, Rc::new(options));

                if is_floating {
                    // TEAM_021: Unlock view using canvas instead of workspace iteration
                    for mon in self.monitors_mut() {
                        mon.canvas_mut().dnd_scroll_gesture_end();
                    }
                    
                    // TEAM_059: Only request the floating size when restoring to floating from 
                    // fullscreen/maximize. For windows that were already floating, don't request
                    // a size - they'll keep their current size during the move.
                    if should_restore_to_floating {
                        let floating_size = tile.floating_window_size;
                        let win = tile.window_mut();
                        let mut size =
                            floating_size.unwrap_or_else(|| win.expected_size().unwrap_or_default());
                        
                        // Apply min/max size window rules
                        let min_size = win.min_size();
                        let max_size = win.max_size();
                        size.w = ensure_min_max_size_maybe_zero(size.w, min_size.w, max_size.w);
                        size.h = ensure_min_max_size_maybe_zero(size.h, min_size.h, max_size.h);
                        
                        win.request_size_once(size, true);
                    }
                } else {
                    // Animate to semitransparent.
                    tile.animate_alpha(
                        1.,
                        INTERACTIVE_MOVE_ALPHA,
                        self.options.animations.window_movement.0,
                    );
                    tile.hold_alpha_animation_after_done();
                }

                let mut data = InteractiveMoveData {
                    tile,
                    output,
                    pointer_pos_within_output,
                    width,
                    is_full_width,
                    is_floating,
                    pointer_ratio_within_window,
                    output_config,
                    workspace_config,
                };

                if let Some((tile_pos, zoom)) = tile_pos {
                    let new_tile_pos = data.tile_render_location(zoom);
                    data.tile
                        .animate_move_from((tile_pos - new_tile_pos).downscale(zoom));
                }

                self.interactive_move = Some(InteractiveMoveState::Moving(data));
            }
            InteractiveMoveState::Moving(mut move_) => {
                if window != move_.tile.window().id() {
                    self.interactive_move = Some(InteractiveMoveState::Moving(move_));
                    return false;
                }

                let mut ws_id = None;
                if let Some(mon) = self.monitor_for_output(&output) {
                    let (insert_ws, _) = mon.insert_position(move_.pointer_pos_within_output);
                    if let InsertWorkspace::Existing(id) = insert_ws {
                        ws_id = Some(id);
                    }
                }

                // If moved over a different workspace, reset the config override.
                let mut update_config = false;
                if let Some((id, _)) = &move_.workspace_config {
                    if Some(*id) != ws_id {
                        move_.workspace_config = None;
                        update_config = true;
                    }
                }

                if output != move_.output {
                    move_.tile.window().output_leave(&move_.output);
                    move_.tile.window().output_enter(&output);
                    move_.tile.window().set_preferred_scale_transform(
                        output.current_scale(),
                        output.current_transform(),
                    );
                    move_.output = output.clone();
                    self.focus_output(&output);

                    move_.output_config = self
                        .monitor_for_output(&output)
                        .and_then(|mon| mon.layout_config().cloned());

                    update_config = true;
                }

                if update_config {
                    let view_size = output_size(&output);
                    let scale = output.current_scale().fractional_scale();
                    let options = Options::clone(&self.options)
                        .with_merged_layout(move_.output_config.as_ref())
                        .with_merged_layout(move_.workspace_config.as_ref().map(|(_, c)| c))
                        .adjusted_for_scale(scale);
                    move_.tile.update_config(view_size, scale, Rc::new(options));
                }

                move_.pointer_pos_within_output = pointer_pos_within_output;

                self.interactive_move = Some(InteractiveMoveState::Moving(move_));
            }
        }

        true
    }

    /// Ends an interactive move for a window.
    pub fn interactive_move_end(&mut self, window: &W::Id) {
        let Some(move_) = &self.interactive_move else {
            return;
        };

        let move_ = match move_ {
            InteractiveMoveState::Starting { window_id, .. } => {
                if window_id != window {
                    return;
                }

                let Some(InteractiveMoveState::Starting { window_id, .. }) =
                    self.interactive_move.take()
                else {
                    unreachable!()
                };

                // TEAM_021: Use canvas to find and update window instead of workspace iteration
                for mon in self.monitors_mut() {
                    mon.dnd_scroll_gesture_end();
                    
                    // Try canvas first for window operations
                    if let Some((_row_idx, tile)) = mon.canvas_mut().find_window_mut(&window_id) {
                        let offset = tile.interactive_move_offset;
                        tile.interactive_move_offset = Point::from((0., 0.));
                        tile.animate_move_from(offset);
                        
                        // Unlock view
                        mon.canvas_mut().dnd_scroll_gesture_end();
                        return;
                    }
                }

                // Fallback to workspace iteration for compatibility
                // TEAM_035: Capture is_active before mutable borrow
                let is_active = self.is_active;
                for mon in self.monitors_mut() {
                    let active_ws_idx = mon.active_row_idx() as i32;
                    for (ws_idx, ws) in mon.canvas.rows_mut() {
                        let is_focused = is_active && ws_idx == active_ws_idx;
                        ws.refresh(is_active, is_focused);
                    }
                }

                for ws in self.workspaces_mut() {
                    if let Some(tile) = ws.tiles_mut().find(|tile| *tile.window().id() == window_id)
                    {
                        let offset = tile.interactive_move_offset;
                        tile.interactive_move_offset = Point::from((0., 0.));
                        tile.animate_move_from(offset);
                    }

                    // Unlock the view on the workspaces, but if the moved window was active,
                    // preserve that.
                    let moved_tile_was_active =
                        ws.active_window().is_some_and(|win| *win.id() == window_id);

                    ws.dnd_scroll_gesture_end();

                    if moved_tile_was_active {
                        ws.activate_window(&window_id);
                    }
                }

                return;
            }
            InteractiveMoveState::Moving(move_) => move_,
        };

        if window != move_.tile.window().id() {
            return;
        }

        let Some(InteractiveMoveState::Moving(mut move_)) = self.interactive_move.take() else {
            unreachable!()
        };

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.dnd_scroll_gesture_end();
        }

        // Unlock the view on the workspaces.
        if !move_.is_floating {
            // TEAM_021: Use canvas for DND gestures instead of workspace iteration
            for mon in self.monitors_mut() {
                mon.canvas_mut().dnd_scroll_gesture_end();
            }

            // Also animate the tile back to opaque.
            move_.tile.animate_alpha(
                INTERACTIVE_MOVE_ALPHA,
                1.,
                self.options.animations.window_movement.0,
            );
        }

        // TEAM_014: Removed overview check (Part 3) - always allow workspace activation
        let allow_to_activate_workspace = true;

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let (mon, insert_ws, position, offset, zoom) =
                    if let Some(mon) = monitors.iter_mut().find(|mon| mon.output == move_.output) {
                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

                        let (insert_ws, geo) = mon.insert_position(move_.pointer_pos_within_output);
                        let (position, offset) = match insert_ws {
                            InsertWorkspace::Existing(ws_id) => {
                                // TEAM_035: Extract row from tuple
                                let ws_idx = mon
                                    .canvas.rows_mut()
                                    .position(|(_, ws)| ws.id() == ws_id)
                                    .unwrap();

                                let position = if move_.is_floating {
                                    InsertPosition::Floating
                                } else {
                                    let pos_within_workspace =
                                        move_.pointer_pos_within_output - geo.loc;
                                    // TEAM_035: Extract row from tuple
                                    let (_, ws) = mon.canvas.rows_mut().nth(ws_idx).unwrap();
                                    ws.scrolling_insert_position(pos_within_workspace)
                                };

                                (position, Some(geo.loc))
                            }
                            InsertWorkspace::NewAt(_) => {
                                let position = if move_.is_floating {
                                    InsertPosition::Floating
                                } else {
                                    InsertPosition::NewColumn(0)
                                };

                                (position, None)
                            }
                        };

                        // TEAM_014: zoom always 1.0 (Part 3)
                        (mon, insert_ws, position, offset, 1.0)
                    } else {
                        let mon = &mut monitors[*active_monitor_idx];
                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
                        // No point in trying to use the pointer position on the wrong output.
                        let ws = &mon.canvas.rows().nth(0).unwrap().1;
                        let ws_geo = mon.workspaces_render_geo().next().unwrap();

                        let position = if move_.is_floating {
                            InsertPosition::Floating
                        } else {
                            ws.scrolling_insert_position(Point::from((0., 0.)))
                        };

                        let insert_ws = InsertWorkspace::Existing(ws.id());
                        // TEAM_014: zoom always 1.0 (Part 3)
                        (mon, insert_ws, position, Some(ws_geo.loc), 1.0)
                    };

                let win_id = move_.tile.window().id().clone();
                let tile_render_loc = move_.tile_render_location(zoom);

                // TEAM_035: Use canvas.rows() instead of mon.workspaces
                let ws_idx = match insert_ws {
                    InsertWorkspace::Existing(ws_id) => mon
                        .canvas
                        .rows()
                        .position(|(_, ws)| ws.id() == ws_id)
                        .unwrap(),
                    InsertWorkspace::NewAt(ws_idx) => {
                        if mon.options.layout.empty_row_above_first && ws_idx == 0 {
                            // Reuse the top empty workspace.
                            0
                        } else if mon.canvas.rows().count() - 1 <= ws_idx {
                            // Reuse the bottom empty workspace.
                            mon.canvas.rows().count() - 1
                        } else {
                            // TEAM_035: Use canvas.ensure_row instead of add_workspace_at
                            mon.canvas.ensure_row(ws_idx as i32);
                            ws_idx
                        }
                    }
                };

                match position {
                    InsertPosition::NewColumn(column_idx) => {
                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon.canvas.rows().nth(ws_idx).unwrap();
                        let ws_id = ws.id();
                        mon.add_tile(
                            move_.tile,
                            MonitorAddWindowTarget::Workspace {
                                id: ws_id,
                                column_idx: Some(column_idx),
                            },
                            ActivateWindow::Yes,
                            allow_to_activate_workspace,
                            move_.width,
                            move_.is_full_width,
                            false,
                        );
                    }
                    InsertPosition::InColumn(column_idx, _tile_idx) => {
                        // TEAM_035: Use canvas row directly instead of mon.add_tile_to_column
                        // Just add the tile directly to the row
                        if let Some(row) = mon.canvas.row_mut(ws_idx as i32) {
                            row.add_tile(Some(column_idx), move_.tile, true, move_.width, move_.is_full_width);
                        }
                    }
                    InsertPosition::Floating => {
                        let tile_render_loc = move_.tile_render_location(zoom);

                        let mut tile = move_.tile;
                        tile.floating_pos = None;

                        match insert_ws {
                            InsertWorkspace::Existing(_) => {
                                if let Some(offset) = offset {
                                    let pos = (tile_render_loc - offset).downscale(zoom);
                                    // TEAM_035: Extract row from tuple and call method
                                    let (_, ws) = mon.canvas.rows().nth(ws_idx).unwrap();
                                    let pos = ws.floating_logical_to_size_frac(pos);
                                    tile.floating_pos = Some(pos);
                                } else {
                                    tracing::error!(
                                        "offset unset for inserting a floating tile \
                                         to existing workspace"
                                    );
                                }
                            }
                            InsertWorkspace::NewAt(_) => {
                                // When putting a floating tile on a new workspace, we don't really
                                // have a good pre-existing position.
                            }
                        }

                        // Set the floating size so it takes into account any window resizing that
                        // took place during the move. Use the actual committed size, not the
                        // expected size, because the client may have resized itself during the move.
                        let size = tile.window().size();
                        tile.floating_window_size = Some(size);

                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon.canvas.rows().nth(ws_idx).unwrap();
                        let ws_id = ws.id();
                        mon.add_tile(
                            tile,
                            MonitorAddWindowTarget::Workspace {
                                id: ws_id,
                                column_idx: None,
                            },
                            ActivateWindow::Yes,
                            allow_to_activate_workspace,
                            move_.width,
                            move_.is_full_width,
                            true,
                        );
                    }
                }

                // needed because empty_row_above_first could have modified the idx
                // TEAM_035: Use tiles_mut() to get mutable reference for animate_move_from
                // TEAM_059: Skip animation for floating tiles (they're in floating space, not rows)
                if let InsertPosition::Floating = position {
                    // Floating tiles are handled by FloatingSpace and don't need this animation
                } else {
                    let (tile, tile_offset, ws_geo) = mon
                        .workspaces_with_render_geo_mut(false)
                        .find_map(|(ws, geo)| {
                            ws.tiles_mut()
                                .find(|tile| tile.window().id() == &win_id)
                                .map(|tile| (tile, Point::from((0.0, 0.0)), geo))
                        })
                        .unwrap();
                    let new_tile_render_loc = ws_geo.loc + tile_offset.upscale(zoom);

                    tile.animate_move_from((tile_render_loc - new_tile_render_loc).downscale(zoom));
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // TEAM_024: Use canvas instead of creating workspaces
                // Ensure we have at least the origin row (row 0)
                let row = canvas.ensure_row(0);
                
                // No point in trying to use the pointer position without outputs.
                // TEAM_035: add_tile takes (col_idx, tile, activate, width, is_full_width)
                row.add_tile(
                    None,
                    move_.tile,
                    true,
                    move_.width,
                    move_.is_full_width,
                );
            }
        }
    }

    /// Returns whether an interactive move is currently moving a window above the given output.
    pub fn interactive_move_is_moving_above_output(&self, output: &Output) -> bool {
        let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move else {
            return false;
        };

        move_.output == *output
    }

    /// Updates the drag-and-drop state with a new pointer position.
    pub fn dnd_update(&mut self, output: Output, pointer_pos_within_output: Point<f64, Logical>) {
        let begin_gesture = self.dnd.is_none();

        self.dnd = Some(DndData::new(output, pointer_pos_within_output));

        if begin_gesture {
            // dnd_scroll_gesture_begin removed - was overview-only

            for ws in self.workspaces_mut() {
                ws.dnd_scroll_gesture_begin();
            }
        }
    }

    /// Ends the current drag-and-drop operation.
    pub fn dnd_end(&mut self) {
        if self.dnd.is_none() {
            return;
        }

        self.dnd = None;

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.dnd_scroll_gesture_end();
        }

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.canvas_mut().dnd_scroll_gesture_end();
        }
    }
}
