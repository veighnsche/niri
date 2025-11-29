//! Hit testing and content queries for the Niri compositor.
//!
//! All methods in this module are pure queries that determine what's under a given point.

use niri_config::OutputName;
use smithay::desktop::utils::under_from_surface_tree;
use smithay::desktop::{layer_map_for_output, WindowSurfaceType};
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};
use smithay::wayland::shell::wlr_layer::Layer;

use crate::layout::HitType;
use crate::window::Mapped;

use super::{Niri, PointContents};

// =============================================================================
// Hit Testing Methods
// =============================================================================

impl Niri {
    /// Checks if the given position is inside a hot corner.
    pub(super) fn is_inside_hot_corner(&self, output: &Output, pos: Point<f64, Logical>) -> bool {
        let config = self.config.borrow();
        let hot_corners = output
            .user_data()
            .get::<OutputName>()
            .and_then(|name| config.outputs.find(name))
            .and_then(|c| c.hot_corners)
            .unwrap_or(config.gestures.hot_corners);

        if hot_corners.off {
            return false;
        }

        // Use size from the ceiled output geometry, since that's what we currently use for pointer
        // motion clamping.
        let geom = self.outputs.global_space.output_geometry(output).unwrap();
        let size = geom.size.to_f64();

        let contains = move |corner: Point<f64, Logical>| {
            Rectangle::new(corner, Size::new(1., 1.)).contains(pos)
        };

        if hot_corners.top_right && contains(Point::new(size.w - 1., 0.)) {
            return true;
        }
        if hot_corners.bottom_left && contains(Point::new(0., size.h - 1.)) {
            return true;
        }
        if hot_corners.bottom_right && contains(Point::new(size.w - 1., size.h - 1.)) {
            return true;
        }

        // If the user didn't explicitly set any corners, we default to top-left.
        if (hot_corners.top_left
            || !(hot_corners.top_right || hot_corners.bottom_right || hot_corners.bottom_left))
            && contains(Point::new(0., 0.))
        {
            return true;
        }

        false
    }

    /// Checks if a sticky layer surface obscures the given position.
    pub fn is_sticky_obscured_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> bool {
        // The ordering here must be consistent with the ordering in render() so that input is
        // consistent with the visuals.

        // Check if some layer-shell surface is on top.
        let layers = layer_map_for_output(output);
        let layer_surface_under = |layer, popup| {
            layers
                .layers_on(layer)
                .rev()
                .find_map(|layer| {
                    let mapped = self.mapped_layer_surfaces.get(layer)?;

                    let mut layer_pos_within_output =
                        layers.layer_geometry(layer).unwrap().loc.to_f64();
                    layer_pos_within_output += mapped.bob_offset();

                    let surface_type = if popup {
                        WindowSurfaceType::POPUP
                    } else {
                        WindowSurfaceType::TOPLEVEL
                    } | WindowSurfaceType::SUBSURFACE;
                    layer.surface_under(pos_within_output - layer_pos_within_output, surface_type)
                })
                .is_some()
        };

        let layer_toplevel_under = |layer| layer_surface_under(layer, false);
        let layer_popup_under = |layer| layer_surface_under(layer, true);

        if layer_popup_under(Layer::Overlay) || layer_toplevel_under(Layer::Overlay) {
            return true;
        }

        let mon = self.layout.monitor_for_output(output).unwrap();
        if mon.render_above_top_layer() {
            return false;
        }

        if self.is_inside_hot_corner(output, pos_within_output) {
            return true;
        }

        if layer_popup_under(Layer::Top) || layer_toplevel_under(Layer::Top) {
            return true;
        }

        false
    }

    /// Checks if the layout is obscured at the given position.
    pub fn is_layout_obscured_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> bool {
        // Check if some layer-shell surface is on top.
        let layers = layer_map_for_output(output);
        let layer_popup_under = |layer| {
            layers
                .layers_on(layer)
                .rev()
                .find_map(|layer_surface| {
                    let mapped = self.mapped_layer_surfaces.get(layer_surface)?;
                    if mapped.place_within_backdrop() {
                        return None;
                    }

                    let mut layer_pos_within_output =
                        layers.layer_geometry(layer_surface).unwrap().loc.to_f64();
                    layer_pos_within_output += mapped.bob_offset();

                    // Background and bottom layers move together with the rows.
                    let mon = self.layout.monitor_for_output(output)?;
                    let (_, geo) = mon.row_under(pos_within_output)?;
                    layer_pos_within_output += geo.loc;

                    let surface_type = WindowSurfaceType::POPUP | WindowSurfaceType::SUBSURFACE;
                    layer_surface
                        .surface_under(pos_within_output - layer_pos_within_output, surface_type)
                })
                .is_some()
        };

        if layer_popup_under(Layer::Bottom) || layer_popup_under(Layer::Background) {
            return true;
        }

        false
    }

    /// Returns the row under the position to be activated.
    ///
    /// The return value is an output and a row on it.
    pub fn row_under(
        &self,
        extended_bounds: bool,
        pos: Point<f64, Logical>,
    ) -> Option<(Output, &crate::layout::row::Row<Mapped>)> {
        let _ = extended_bounds; // Currently unused but kept for API compatibility

        if self.ui.exit_dialog.is_open() || self.is_locked() || self.ui.screenshot.is_open() {
            return None;
        }

        let (output, pos_within_output) = self.output_under(pos)?;

        if self.is_sticky_obscured_under(output, pos_within_output) {
            return None;
        }

        if self.is_layout_obscured_under(output, pos_within_output) {
            return None;
        }

        let (ws, _) = self
            .layout
            .monitor_for_output(&output)?
            .row_under(pos_within_output)?;
        Some((output.clone(), ws))
    }

    /// Returns the row under the cursor.
    pub fn row_under_cursor(
        &self,
        extended_bounds: bool,
    ) -> Option<(Output, &crate::layout::row::Row<Mapped>)> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.row_under(extended_bounds, pos)
    }

    /// Returns the window under the position to be activated.
    ///
    /// The cursor may be inside the window's activation region, but not within the window's input
    /// region.
    pub fn window_under(&self, pos: Point<f64, Logical>) -> Option<&Mapped> {
        if self.ui.exit_dialog.is_open()
            || self.is_locked()
            || self.ui.screenshot.is_open()
            || self.ui.mru.is_open()
        {
            return None;
        }

        let (output, pos_within_output) = self.output_under(pos)?;

        if self.is_sticky_obscured_under(output, pos_within_output) {
            return None;
        }

        if let Some((window, _loc)) = self
            .layout
            .interactive_moved_window_under(output, pos_within_output)
        {
            return Some(window);
        }

        if self.is_layout_obscured_under(output, pos_within_output) {
            return None;
        }

        let (window, _loc) = self.layout.window_under(output, pos_within_output)?;
        Some(window)
    }

    /// Returns the window under the cursor to be activated.
    ///
    /// The cursor may be inside the window's activation region, but not within the window's input
    /// region.
    pub fn window_under_cursor(&self) -> Option<&Mapped> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.window_under(pos)
    }

    /// Returns contents under the given point.
    ///
    /// We don't have a proper global space for all windows, so this function converts window
    /// locations to global space according to where they are rendered.
    ///
    /// This function does not take pointer or touch grabs into account.
    pub fn contents_under(&self, pos: Point<f64, Logical>) -> PointContents {
        let mut rv = PointContents::default();

        let Some((output, pos_within_output)) = self.output_under(pos) else {
            return rv;
        };
        rv.output = Some(output.clone());
        let output_pos_in_global_space = self.outputs.global_space.output_geometry(output).unwrap().loc;

        // The ordering here must be consistent with the ordering in render() so that input is
        // consistent with the visuals.

        if self.ui.exit_dialog.is_open() {
            return rv;
        } else if self.is_locked() {
            rv.layer_surface = layer_map_for_output(output)
                .layer_under(
                    pos_within_output,
                    WindowSurfaceType::ALL,
                )
                .map(|(surface, pos_within_output)| {
                    (
                        surface,
                        (pos_within_output + output_pos_in_global_space).to_f64(),
                    )
                });

            return rv;
        }

        if self.ui.screenshot.is_open() || self.ui.mru.is_open() {
            return rv;
        }

        let layers = layer_map_for_output(output);
        let layer_surface_under = |layer, popup| {
            layers
                .layers_on(layer)
                .rev()
                .find_map(|layer_surface| {
                    let mapped = self.mapped_layer_surfaces.get(layer_surface)?;
                    if mapped.place_within_backdrop() {
                        return None;
                    }

                    let mut layer_pos_within_output =
                        layers.layer_geometry(layer_surface).unwrap().loc.to_f64();
                    layer_pos_within_output += mapped.bob_offset();

                    // Background and bottom layers move together with the rows.
                    if matches!(layer, Layer::Background | Layer::Bottom) {
                        let mon = self.layout.monitor_for_output(output)?;
                        let (_, geo) = mon.row_under(pos_within_output)?;
                        layer_pos_within_output += geo.loc;
                        // Don't need to deal with zoom here because in the overview background and
                        // bottom layers don't receive input.
                    }

                    let surface_type = if popup {
                        WindowSurfaceType::POPUP
                    } else {
                        WindowSurfaceType::TOPLEVEL
                    } | WindowSurfaceType::SUBSURFACE;

                    layer_surface
                        .surface_under(pos_within_output - layer_pos_within_output, surface_type)
                        .map(|(surface, pos_within_layer)| {
                            (
                                (surface, pos_within_layer.to_f64() + layer_pos_within_output),
                                layer_surface,
                            )
                        })
                })
                .map(|(s, l)| (Some(s), (None, Some(l.clone()))))
        };

        let layer_toplevel_under = |layer| layer_surface_under(layer, false);
        let layer_popup_under = |layer| layer_surface_under(layer, true);

        let mapped_hit_data = |(mapped, hit): (&Mapped, HitType)| {
            let window = &mapped.window;
            let surface_and_pos = if let HitType::Input { win_pos } = hit {
                let win_pos_within_output = win_pos;
                window
                    .surface_under(
                        pos_within_output - win_pos_within_output,
                        WindowSurfaceType::ALL,
                    )
                    .map(|(s, pos_within_window)| {
                        (s, pos_within_window.to_f64() + win_pos_within_output)
                    })
            } else {
                None
            };
            (surface_and_pos, (Some((window.clone(), hit)), None))
        };

        let interactive_moved_window_under = || {
            self.layout
                .interactive_moved_window_under(output, pos_within_output)
                .map(mapped_hit_data)
        };
        let window_under = || {
            self.layout
                .window_under(output, pos_within_output)
                .map(mapped_hit_data)
        };

        let mon = self.layout.monitor_for_output(output).unwrap();

        let mut under =
            layer_popup_under(Layer::Overlay).or_else(|| layer_toplevel_under(Layer::Overlay));

        // Overview mode has been removed, this is always false
        let is_overview_open = false;

        // When rendering above the top layer, we put the regular monitor elements first.
        // Otherwise, we will render all layer-shell pop-ups and the top layer on top.
        if mon.render_above_top_layer() {
            under = under
                .or_else(interactive_moved_window_under)
                .or_else(window_under)
                .or_else(|| layer_popup_under(Layer::Top))
                .or_else(|| layer_toplevel_under(Layer::Top))
                .or_else(|| layer_popup_under(Layer::Bottom))
                .or_else(|| layer_popup_under(Layer::Background))
                .or_else(|| layer_toplevel_under(Layer::Bottom))
                .or_else(|| layer_toplevel_under(Layer::Background));
        } else {
            if self.is_inside_hot_corner(output, pos_within_output) {
                rv.hot_corner = true;
                return rv;
            }

            under = under
                .or_else(|| layer_popup_under(Layer::Top))
                .or_else(|| layer_toplevel_under(Layer::Top));

            under = under.or_else(interactive_moved_window_under);

            if !is_overview_open {
                under = under
                    .or_else(|| layer_popup_under(Layer::Bottom))
                    .or_else(|| layer_popup_under(Layer::Background));
            }

            under = under.or_else(window_under);

            if !is_overview_open {
                under = under
                    .or_else(|| layer_toplevel_under(Layer::Bottom))
                    .or_else(|| layer_toplevel_under(Layer::Background));
            }
        }

        let Some((mut surface_and_pos, (window, layer))) = under else {
            return rv;
        };

        if let Some((_, surface_pos)) = &mut surface_and_pos {
            *surface_pos += output_pos_in_global_space.to_f64();
        }

        rv.surface = surface_and_pos;
        rv.window = window;
        rv.layer = layer;
        rv
    }
}
