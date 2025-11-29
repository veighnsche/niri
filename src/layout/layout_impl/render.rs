// TEAM_064: Render and animation methods extracted from mod.rs
//!
//! This module contains methods for rendering, animations, and render element updates.

use std::rc::Rc;
use std::time::Duration;

use niri_config::CornerRadius;
use smithay::backend::renderer::element::utils::RescaleRenderElement;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Scale};

use super::types::{DndHold, DndHoldTarget, InteractiveMoveState};
use crate::layout::monitor::{InsertHint, InsertPosition, InsertWorkspace};
use crate::layout::tile::TileRenderElement;
use crate::layout::{Layout, LayoutElement, MonitorSet, Options};
use crate::niri_render_elements;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::RenderTarget;
use crate::utils::output_size;

impl<W: LayoutElement> Layout<W> {
    /// Advances all animations in the layout.
    pub fn advance_animations(&mut self) {
        let _span = tracy_client::span!("Layout::advance_animations");

        let mut dnd_scroll = None;
        let mut is_dnd = false;
        if let Some(dnd) = &self.dnd {
            dnd_scroll = Some((dnd.output().clone(), dnd.pointer_pos_within_output(), true));
            is_dnd = true;
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            move_.tile.advance_animations();

            if dnd_scroll.is_none() {
                dnd_scroll = Some((
                    move_.output.clone(),
                    move_.pointer_pos_within_output,
                    !move_.is_floating,
                ));
            }
        }

        // TEAM_014: Removed is_overview_open (Part 3)

        // Scroll the view if needed.
        if let Some((output, pos_within_output, is_scrolling)) = dnd_scroll {
            if let Some(mon) = self.monitor_for_output_mut(&output) {
                let mut scrolled = false;

                // TEAM_014: Removed overview zoom (Part 3) - always 1.0
                scrolled |= mon.dnd_scroll_gesture_scroll(pos_within_output, 1.);

                if is_scrolling {
                    if let Some((ws, geo)) = mon.row_under(pos_within_output) {
                        let ws_id = ws.id();
                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon
                            .canvas.rows_mut()
                            .find(|(_, ws)| ws.id() == ws_id)
                            .unwrap();
                        // As far as the DnD scroll gesture is concerned, the workspace spans across
                        // the whole monitor horizontally.
                        let ws_pos = Point::from((0., geo.loc.y));
                        // TEAM_014: Removed overview zoom (Part 3) - always 1.0
                        // TEAM_035: Row's dnd_scroll_gesture_scroll only takes delta.x
                        let delta = pos_within_output - ws_pos;
                        scrolled |= ws.dnd_scroll_gesture_scroll(delta.x);
                    }
                }

                if scrolled {
                    // Don't trigger DnD hold while scrolling.
                    if let Some(dnd) = &mut self.dnd {
                        *dnd.hold_mut() = None;
                    }
                } else if is_dnd {
                    let target = mon
                        .window_under(pos_within_output)
                        .map(|(win, _)| DndHoldTarget::Window(win.id().clone()))
                        .or_else(|| {
                            mon.row_under_narrow(pos_within_output)
                                .map(|ws| DndHoldTarget::Workspace(ws.id()))
                        });

                    let dnd = self.dnd.as_mut().unwrap();
                    if let Some(target) = target {
                        let now = self.clock.now_unadjusted();
                        let start_time = if let Some(hold) = dnd.hold_mut() {
                            if *hold.target() != target {
                                *hold = DndHold::new(now, target);
                            }
                            hold.start_time()
                        } else {
                            let hold = dnd.hold_mut().insert(DndHold::new(now, target));
                            hold.start_time()
                        };

                        // Delay copied from gnome-shell.
                        let delay = Duration::from_millis(750);
                        if delay <= now.saturating_sub(start_time) {
                            let hold = dnd.hold_mut().take().unwrap();

                            // TEAM_014: Removed overview animation sync (Part 3)
                            let config = None;

                            let mon = self.monitor_for_output_mut(&output).unwrap();

                            let ws_idx = match hold.target() {
                                // TEAM_035: Extract row from tuple
                                DndHoldTarget::Window(id) => mon
                                    .canvas
                                    .rows_mut()
                                    .position(|(_, ws)| ws.activate_window(id))
                                    .unwrap(),
                                DndHoldTarget::Workspace(id) => {
                                    mon.canvas.rows().position(|(idx, ws)| ws.id() == *id).unwrap()
                                }
                            };

                            mon.dnd_scroll_gesture_end();
                            mon.activate_workspace_with_anim_config(ws_idx, config);

                            self.focus_output(&output);

                            // TEAM_014: Removed close_overview call (Part 3)
                        }
                    } else {
                        // No target, reset the hold timer.
                        *dnd.hold_mut() = None;
                    }
                }
            }
        }

        // TEAM_014: Removed overview_progress animation handling (Part 3)

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_014: Removed set_overview_progress call (Part 3)
                    mon.advance_animations();
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    ws.advance_animations();
                }
            }
        }
    }

    /// Returns whether any animations are ongoing for the given output.
    pub fn are_animations_ongoing(&self, output: Option<&Output>) -> bool {
        // Keep advancing animations if we might need to scroll the view.
        if let Some(dnd) = &self.dnd {
            if output.map_or(true, |output| *output == *dnd.output()) {
                return true;
            }
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if output.map_or(true, |output| *output == move_.output) {
                if move_.tile.are_animations_ongoing() {
                    return true;
                }

                // Keep advancing animations if we might need to scroll the view.
                if !move_.is_floating {
                    return true;
                }
            }
        }

        // TEAM_014: Removed overview_progress animation check (Part 3)

        for mon in self.monitors() {
            if output.is_some_and(|output| mon.output != *output) {
                continue;
            }

            if mon.are_animations_ongoing() {
                return true;
            }
        }

        false
    }

    /// Updates render elements for the given output.
    pub fn update_render_elements(&mut self, output: Option<&Output>) {
        let _span = tracy_client::span!("Layout::update_render_elements");

        self.update_render_elements_time = self.clock.now();

        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if output.map_or(true, |output| move_.output == *output) {
                let pos_within_output = move_.tile_render_location(1.);
                let view_rect =
                    Rectangle::new(pos_within_output.upscale(-1.), output_size(&move_.output));
                move_.tile.update_render_elements(true, view_rect);
            }
        }

        self.update_insert_hint(output);

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            tracing::error!("update_render_elements called with no monitors");
            return;
        };

        for (idx, mon) in monitors.iter_mut().enumerate() {
            if output.map_or(true, |output| mon.output == *output) {
                let is_active = self.is_active
                    && idx == *active_monitor_idx
                    && !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_)));
                // TEAM_014: Removed set_overview_progress call (Part 3)
                mon.update_render_elements(is_active);
            }
        }
    }

    /// Updates shaders for all tiles.
    pub fn update_shaders(&mut self) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            move_.tile.update_shaders();
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    mon.update_shaders();
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    ws.update_shaders();
                }
            }
        }
    }

    /// Updates the insert hint for interactive move.
    fn update_insert_hint(&mut self, output: Option<&Output>) {
        let _span = tracy_client::span!("Layout::update_insert_hint");

        for mon in self.monitors_mut() {
            mon.insert_hint = None;
        }

        if !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_))) {
            return;
        }
        let Some(InteractiveMoveState::Moving(move_)) = self.interactive_move.take() else {
            unreachable!()
        };
        if output.is_some_and(|out| &move_.output != out) {
            self.interactive_move = Some(InteractiveMoveState::Moving(move_));
            return;
        }

        let _span = tracy_client::span!("Layout::update_insert_hint::update");

        if let Some(mon) = self.monitor_for_output_mut(&move_.output) {
            // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
            let (insert_ws, geo) = mon.insert_position(move_.pointer_pos_within_output);
            match insert_ws {
                InsertWorkspace::Existing(ws_id) => {
                    // TEAM_035: Extract row from tuple
                    let (_, ws) = mon
                        .canvas.rows_mut()
                        .find(|(_, ws)| ws.id() == ws_id)
                        .unwrap();
                    let pos_within_workspace = move_.pointer_pos_within_output - geo.loc;
                    let position = if move_.is_floating {
                        InsertPosition::Floating
                    } else {
                        ws.scrolling_insert_position(pos_within_workspace)
                    };

                    let rules = move_.tile.window().rules();
                    let border_width = move_.tile.effective_border_width().unwrap_or(0.);
                    let corner_radius = rules
                        .geometry_corner_radius
                        .map_or(CornerRadius::default(), |radius| {
                            radius.expanded_by(border_width as f32)
                        });
                    mon.insert_hint = Some(InsertHint {
                        workspace: insert_ws,
                        position,
                        corner_radius,
                    });
                }
                InsertWorkspace::NewAt(_) => {
                    let position = if move_.is_floating {
                        InsertPosition::Floating
                    } else {
                        InsertPosition::NewColumn(0)
                    };
                    mon.insert_hint = Some(InsertHint {
                        workspace: insert_ws,
                        position,
                        corner_radius: CornerRadius::default(),
                    });
                }
            }
        }

        self.interactive_move = Some(InteractiveMoveState::Moving(move_));
    }

    /// Renders the interactive move tile for the given output.
    pub fn render_interactive_move_for_output<'a, R: NiriRenderer + 'a>(
        &'a self,
        renderer: &mut R,
        output: &Output,
        target: RenderTarget,
    ) -> impl Iterator<Item = RescaleRenderElement<TileRenderElement<R>>> + 'a {
        if self.update_render_elements_time != self.clock.now() {
            tracing::error!("clock moved between updating render elements and rendering");
        }

        let mut rv = None;

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if &move_.output == output {
                let scale = Scale::from(move_.output.current_scale().fractional_scale());
                // Overview mode has been removed, zoom is always 1.0
                let zoom = 1.0;
                let location = move_.tile_render_location(zoom);
                let iter = move_
                    .tile
                    .render(renderer, location, true, target)
                    .map(move |elem| {
                        RescaleRenderElement::from_element(
                            elem,
                            location.to_physical_precise_round(scale),
                            zoom,
                        )
                    });
                rv = Some(iter);
            }
        }

        rv.into_iter().flatten()
    }

    /// Refreshes the layout state.
    pub fn refresh(&mut self, is_active: bool) {
        let _span = tracy_client::span!("Layout::refresh");

        self.is_active = is_active;

        let mut ongoing_scrolling_dnd = self.dnd.is_some().then_some(true);

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            let win = move_.tile.window_mut();

            win.set_active_in_column(true);
            win.set_floating(move_.is_floating);
            win.set_activated(true);

            win.set_interactive_resize(None);

            win.set_bounds(output_size(&move_.output).to_i32_round());

            win.send_pending_configure();
            win.refresh();

            ongoing_scrolling_dnd.get_or_insert(!move_.is_floating);
        } else if let Some(InteractiveMoveState::Starting { window_id, .. }) =
            &self.interactive_move
        {
            ongoing_scrolling_dnd.get_or_insert_with(|| {
                let (_, _, ws) = self
                    .workspaces()
                    .find(|(_, _, ws)| ws.has_window(window_id))
                    .unwrap();
                !ws.is_floating(window_id)
            });
        }

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                for (idx, mon) in monitors.iter_mut().enumerate() {
                    let is_active = self.is_active
                        && idx == *active_monitor_idx
                        && !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_)));

                    // DEPRECATED(overview): Removed overview_open checks
                    // Overview is no longer supported, so always end DnD scroll gesture
                    mon.dnd_scroll_gesture_end();

                    // TEAM_043: Refresh all rows in the canvas
                    let active_row_idx = mon.canvas().active_row_idx();
                    let floating_is_active = mon.canvas().floating_is_active;
                    
                    for (row_idx, row) in mon.canvas_mut().rows_mut() {
                        let is_focused = is_active && row_idx == active_row_idx && !floating_is_active;
                        row.refresh(is_active, is_focused);
                        row.view_offset_gesture_end(ongoing_scrolling_dnd);
                    }
                    
                    // TEAM_043: Refresh floating space
                    let is_floating_focused = is_active && floating_is_active;
                    mon.canvas_mut().floating.refresh(is_active, is_floating_focused);
                    
                    if let Some(is_scrolling) = ongoing_scrolling_dnd {
                        // Lock or unlock the view for scrolling interactive move.
                        if is_scrolling {
                            // Canvas equivalent: dnd_scroll_gesture_begin on active row
                            if let Some(row) = mon.canvas_mut().active_row_mut() {
                                row.dnd_scroll_gesture_begin();
                            }
                        } else {
                            mon.canvas_mut().dnd_scroll_gesture_end();
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                let floating_is_active = canvas.floating_is_active;
                for (_, ws) in canvas.rows_mut() {
                    ws.refresh(false, !floating_is_active);
                    ws.view_offset_gesture_end(None);
                }
                // TEAM_043: Refresh floating space
                canvas.floating.refresh(false, floating_is_active);
            }
        }
    }

    /// Updates the layout configuration.
    ///
    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn update_config(&mut self, config: &niri_config::Config) {
        // Update canvas config for all monitors
        let options = Options::from_config(config);
        for mon in self.monitors_mut() {
            mon.canvas.update_config(Rc::new(options.clone()));
        }

        // Canvas2D now manages all rows, workspace config removed
        // Named row configs will be handled by Canvas2D in future phases

        self.update_options(Options::from_config(config));
    }

    /// Updates the layout options.
    pub(crate) fn update_options(&mut self, options: Options) {
        let options = Rc::new(options);

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            let view_size = output_size(&move_.output);
            let scale = move_.output.current_scale().fractional_scale();
            let options = Options::clone(&options)
                .with_merged_layout(move_.output_config.as_ref())
                .with_merged_layout(move_.workspace_config.as_ref().map(|(_, c)| c))
                .adjusted_for_scale(scale);
            move_.tile.update_config(view_size, scale, Rc::new(options));
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    mon.update_config(options.clone());
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Use canvas properties for update_config
                let view_size = canvas.view_size;
                let parent_area = canvas.parent_area;
                let scale = canvas.scale;
                for (_, ws) in canvas.rows_mut() {
                    ws.update_config(view_size, parent_area, scale, options.clone());
                }
            }
        }

        self.options = options;
    }
}
