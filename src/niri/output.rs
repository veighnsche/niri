//! Output management for the Niri compositor.
//!
//! This module contains methods for querying and managing outputs (monitors).

use smithay::desktop::{layer_map_for_output, WindowSurfaceType};
use smithay::input::pointer::CursorImageStatus;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::backend::Backend;
use crate::handlers::configure_lock_surface;
use crate::utils::{center, output_matches_name, output_size, send_scale_transform};

use super::Niri;

// =============================================================================
// Output Query Methods
// =============================================================================

impl Niri {
    /// Returns the output under the given position and the position within that output.
    pub fn output_under(&self, pos: Point<f64, Logical>) -> Option<(&Output, Point<f64, Logical>)> {
        let output = self.outputs.space().output_under(pos).next()?;
        let pos_within_output = pos
            - self
                .outputs
                .space()
                .output_geometry(output)
                .unwrap()
                .loc
                .to_f64();

        Some((output, pos_within_output))
    }

    /// Returns the output under the current cursor position.
    pub fn output_under_cursor(&self) -> Option<Output> {
        let pos = self.seat.get_pointer().unwrap().current_location();
        self.outputs.space().output_under(pos).next().cloned()
    }

    /// Returns the output to the left of the given output.
    pub fn output_left_of(&self, current: &Output) -> Option<Output> {
        let current_geo = self.outputs.space().output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((i32::MIN / 2, current_geo.loc.y)),
            Size::from((i32::MAX, current_geo.size.h)),
        );

        self.outputs.space()
            .outputs()
            .map(|output| (output, self.outputs.space().output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).x < center(current_geo).x && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(current_geo).x - center(*geo).x)
            .map(|(output, _)| output)
            .cloned()
    }

    /// Returns the output to the right of the given output.
    pub fn output_right_of(&self, current: &Output) -> Option<Output> {
        let current_geo = self.outputs.space().output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((i32::MIN / 2, current_geo.loc.y)),
            Size::from((i32::MAX, current_geo.size.h)),
        );

        self.outputs.space()
            .outputs()
            .map(|output| (output, self.outputs.space().output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).x > center(current_geo).x && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(*geo).x - center(current_geo).x)
            .map(|(output, _)| output)
            .cloned()
    }

    /// Returns the output above the given output.
    pub fn output_up_of(&self, current: &Output) -> Option<Output> {
        let current_geo = self.outputs.space().output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((current_geo.loc.x, i32::MIN / 2)),
            Size::from((current_geo.size.w, i32::MAX)),
        );

        self.outputs.space()
            .outputs()
            .map(|output| (output, self.outputs.space().output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).y < center(current_geo).y && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(current_geo).y - center(*geo).y)
            .map(|(output, _)| output)
            .cloned()
    }

    /// Returns the output below the given output.
    pub fn output_down_of(&self, current: &Output) -> Option<Output> {
        let current_geo = self.outputs.space().output_geometry(current)?;
        let extended_geo = Rectangle::new(
            Point::from((current_geo.loc.x, i32::MIN / 2)),
            Size::from((current_geo.size.w, i32::MAX)),
        );

        self.outputs.space()
            .outputs()
            .map(|output| (output, self.outputs.space().output_geometry(output).unwrap()))
            .filter(|(_, geo)| center(*geo).y > center(current_geo).y && geo.overlaps(extended_geo))
            .min_by_key(|(_, geo)| center(*geo).y - center(current_geo).y)
            .map(|(output, _)| output)
            .cloned()
    }

    /// Returns the previous output in the sorted order.
    pub fn output_previous_of(&self, current: &Output) -> Option<Output> {
        self.outputs.iter()
            .rev()
            .skip_while(|&output| output != current)
            .nth(1)
            .or(self.outputs.iter().last())
            .filter(|&output| output != current)
            .cloned()
    }

    /// Returns the next output in the sorted order.
    pub fn output_next_of(&self, current: &Output) -> Option<Output> {
        self.outputs.iter()
            .skip_while(|&output| output != current)
            .nth(1)
            .or(self.outputs.iter().first())
            .filter(|&output| output != current)
            .cloned()
    }

    /// Returns the output to the left of the active output.
    pub fn output_left(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_left_of(active)
    }

    /// Returns the output to the right of the active output.
    pub fn output_right(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_right_of(active)
    }

    /// Returns the output above the active output.
    pub fn output_up(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_up_of(active)
    }

    /// Returns the output below the active output.
    pub fn output_down(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_down_of(active)
    }

    /// Returns the previous output from the active output.
    pub fn output_previous(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_previous_of(active)
    }

    /// Returns the next output from the active output.
    pub fn output_next(&self) -> Option<Output> {
        let active = self.layout.active_output()?;
        self.output_next_of(active)
    }

    /// Returns the output configured for tablet input.
    pub fn output_for_tablet(&self) -> Option<&Output> {
        let config = self.config.borrow();
        let map_to_output = config.input.tablet.map_to_output.as_ref();
        map_to_output.and_then(|name| self.output_by_name_match(name))
    }

    /// Returns the output configured for touch input.
    pub fn output_for_touch(&self) -> Option<&Output> {
        let config = self.config.borrow();
        let map_to_output = config.input.touch.map_to_output.as_ref();
        map_to_output
            .and_then(|name| self.output_by_name_match(name))
            .or_else(|| self.outputs.space().outputs().next())
    }

    /// Returns the output matching the given name.
    pub fn output_by_name_match(&self, target: &str) -> Option<&Output> {
        self.outputs.space()
            .outputs()
            .find(|output| output_matches_name(output, target))
    }

    /// Returns the output containing the given root surface.
    pub fn output_for_root(&self, root: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) -> Option<&Output> {
        // Check the main layout.
        let win_out = self.layout.find_window_and_output(root);
        let layout_output = win_out.map(|(_, output)| output);
        if let Some(output) = layout_output {
            return output;
        }

        // Check layer-shell.
        let has_layer_surface = |o: &&Output| {
            layer_map_for_output(o)
                .layer_for_surface(root, WindowSurfaceType::TOPLEVEL)
                .is_some()
        };
        self.layout.outputs().find(has_layer_surface)
    }
}

// =============================================================================
// Output Mutation Methods
// =============================================================================

impl Niri {
    /// Called when an output is resized.
    pub fn output_resized(&mut self, output: &Output) {
        let output_size = output_size(output);
        let scale = output.current_scale();
        let transform = output.current_transform();

        {
            let mut layer_map = layer_map_for_output(output);
            for layer in layer_map.layers() {
                layer.with_surfaces(|surface, data| {
                    send_scale_transform(surface, data, scale, transform);
                });

                if let Some(mapped) = self.mapped_layer_surfaces.get_mut(layer) {
                    mapped.update_sizes(output_size, scale.fractional_scale());
                }
            }
            layer_map.arrange();
        }

        self.layout.update_output_size(output);

        if let Some(state) = self.outputs.state_mut(output) {
            state.backdrop_buffer.resize(output_size);

            state.lock_color_buffer.resize(output_size);
            if let Some(lock_surface) = &state.lock_surface {
                configure_lock_surface(lock_surface, output);
            }
        }

        // If the output size changed with an open screenshot UI, close the screenshot UI.
        if let Some((old_size, old_scale, old_transform)) = self.ui.screenshot.output_size(output) {
            let output_mode = output.current_mode().unwrap();
            let size = transform.transform_size(output_mode.size);
            let scale = output.current_scale().fractional_scale();
            let transform = output.current_transform();

            // If the output geometry changed while the screenshot UI was open, close it. The UI
            // operates in physical coordinates.
            if old_size != size || old_scale != scale || old_transform != transform {
                self.ui.screenshot.close();
                self.cursor.manager
                    .set_cursor_image(CursorImageStatus::default_named());
                self.queue_redraw_all();
                return;
            }
        }

        self.queue_redraw(output);
    }

    /// Deactivates all monitors.
    pub fn deactivate_monitors(&mut self, backend: &mut Backend) {
        if !self.outputs.monitors_active() {
            return;
        }

        self.outputs.set_monitors_active(false);
        backend.set_monitors_active(false);
    }

    /// Activates all monitors.
    pub fn activate_monitors(&mut self, backend: &mut Backend) {
        if self.outputs.monitors_active() {
            return;
        }

        self.outputs.set_monitors_active(true);
        backend.set_monitors_active(true);

        self.queue_redraw_all();
    }
}
