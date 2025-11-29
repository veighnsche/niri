//! Core rendering logic for the Niri compositor.
//!
//! This module contains the main render method and related rendering helpers.

use std::mem;

use niri_config::debug::PreviewRender;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::element::surface::render_elements_from_surface_tree;
use smithay::backend::renderer::element::{
    default_primary_scanout_output_compare, Kind, RenderElementStates,
};
use smithay::desktop::layer_map_for_output;
use smithay::backend::renderer::element::utils::select_dmabuf_feedback;
use smithay::desktop::utils::{
    bbox_from_surface_tree, output_update,
    send_dmabuf_feedback_surface_tree, surface_primary_scanout_output,
    update_surface_primary_scanout_output,
};
use smithay::desktop::{LayerMap, Space};
use smithay::input::pointer::{CursorImageStatus, CursorImageSurfaceData};
use smithay::output::Output;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Scale, Transform};
use smithay::wayland::compositor::{with_states, with_surface_tree_downward, TraversalAction};
use smithay::wayland::shell::wlr_layer::Layer;
use tracing::trace;

use crate::backend::{Backend, RenderResult};
use crate::backend::tty::SurfaceDmabufFeedback;
use crate::cursor::{RenderCursor, XCursor};
use crate::layer::mapped::LayerSurfaceRenderElement;
use crate::render_helpers::debug::draw_opaque_regions;
use niri_config::OutputName;
use crate::render_helpers::solid_color::SolidColorRenderElement;
use crate::render_helpers::{RenderTarget, SplitElements};
use crate::utils::send_scale_transform;

use super::{
    KeyboardFocus, LockRenderState, LockState, Niri, OutputRenderElements, OutputState,
    RedrawState, State,
};
use crate::layout::LayoutElement as _;
use crate::render_helpers::renderer::NiriRenderer;

// =============================================================================
// Pointer Rendering
// =============================================================================

impl Niri {
    /// Renders the pointer cursor element for the given output.
    pub fn pointer_element<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
    ) -> Vec<OutputRenderElements<R>> {
        if !self.cursor.is_visible() {
            return vec![];
        }

        let _span = tracy_client::span!("Niri::pointer_element");
        let output_scale = output.current_scale();
        let output_pos = self.outputs.space().output_geometry(output).unwrap().loc;

        // Check whether we need to draw the tablet cursor or the regular cursor.
        let pointer_pos = self
            .cursor
            .tablet_location()
            .unwrap_or_else(|| self.seat.get_pointer().unwrap().current_location());
        let pointer_pos = pointer_pos - output_pos.to_f64();

        // Get the render cursor to draw.
        let cursor_scale = output_scale.integer_scale();
        let render_cursor = self.cursor.get_render_cursor(cursor_scale);

        let output_scale = Scale::from(output.current_scale().fractional_scale());

        let mut pointer_elements = match render_cursor {
            RenderCursor::Hidden => vec![],
            RenderCursor::Surface { surface, hotspot } => {
                let pointer_pos =
                    (pointer_pos - hotspot.to_f64()).to_physical_precise_round(output_scale);

                render_elements_from_surface_tree(
                    renderer,
                    &surface,
                    pointer_pos,
                    output_scale,
                    1.,
                    Kind::Cursor,
                )
            }
            RenderCursor::Named {
                icon,
                scale,
                cursor,
            } => {
                let (idx, frame) = cursor.frame(self.start_time.elapsed().as_millis() as u32);
                let hotspot = XCursor::hotspot(frame).to_logical(scale);
                let pointer_pos =
                    (pointer_pos - hotspot.to_f64()).to_physical_precise_round(output_scale);

                let texture = self.cursor.texture_cache.get(icon, scale, &cursor, idx);
                let mut pointer_elements = vec![];
                let pointer_element = match MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    pointer_pos,
                    &texture,
                    None,
                    None,
                    None,
                    Kind::Cursor,
                ) {
                    Ok(element) => Some(element),
                    Err(err) => {
                        warn!("error importing a cursor texture: {err:?}");
                        None
                    }
                };
                if let Some(element) = pointer_element {
                    pointer_elements.push(OutputRenderElements::NamedPointer(element));
                }

                pointer_elements
            }
        };

        if let Some(dnd_icon) = self.cursor.dnd_icon.as_ref() {
            let pointer_pos =
                (pointer_pos + dnd_icon.offset.to_f64()).to_physical_precise_round(output_scale);
            pointer_elements.extend(render_elements_from_surface_tree(
                renderer,
                &dnd_icon.surface,
                pointer_pos,
                output_scale,
                1.,
                Kind::ScanoutCandidate,
            ));
        }

        pointer_elements
    }

    /// Refreshes pointer output associations for scale/transform.
    pub fn refresh_pointer_outputs(&mut self) {
        use smithay::desktop::utils::bbox_from_surface_tree;

        if !self.cursor.is_visible() {
            return;
        }

        let _span = tracy_client::span!("Niri::refresh_pointer_outputs");

        // Check whether we need to draw the tablet cursor or the regular cursor.
        let pointer_pos = self
            .cursor
            .tablet_location()
            .unwrap_or_else(|| self.seat.get_pointer().unwrap().current_location());

        match self.cursor.manager().cursor_image() {
            CursorImageStatus::Surface(ref surface) => {
                let hotspot = with_states(surface, |states| {
                    states
                        .data_map
                        .get::<CursorImageSurfaceData>()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .hotspot
                });

                let surface_pos = pointer_pos.to_i32_round() - hotspot;
                let bbox = bbox_from_surface_tree(surface, surface_pos);

                let dnd = self
                    .cursor.dnd_icon
                    .as_ref()
                    .map(|icon| &icon.surface)
                    .map(|surface| (surface, bbox_from_surface_tree(surface, surface_pos)));

                // FIXME we basically need to pick the largest scale factor across the overlapping
                // outputs, this is how it's usually done in clients as well.
                let mut cursor_scale = 1.;
                let mut cursor_transform = Transform::Normal;
                let mut dnd_scale = 1.;
                let mut dnd_transform = Transform::Normal;
                for output in self.outputs.space().outputs() {
                    let geo = self.outputs.space().output_geometry(output).unwrap();

                    // Compute pointer surface overlap.
                    if let Some(mut overlap) = geo.intersection(bbox) {
                        overlap.loc -= surface_pos;
                        cursor_scale =
                            f64::max(cursor_scale, output.current_scale().fractional_scale());
                        // FIXME: using the largest overlapping or "primary" output transform would
                        // make more sense here.
                        cursor_transform = output.current_transform();
                        output_update(output, Some(overlap), surface);
                    } else {
                        output_update(output, None, surface);
                    }

                    // Compute DnD icon surface overlap.
                    if let Some((surface, bbox)) = dnd {
                        if let Some(mut overlap) = geo.intersection(bbox) {
                            overlap.loc -= surface_pos;
                            dnd_scale =
                                f64::max(dnd_scale, output.current_scale().fractional_scale());
                            // FIXME: using the largest overlapping or "primary" output transform
                            // would make more sense here.
                            dnd_transform = output.current_transform();
                            output_update(output, Some(overlap), surface);
                        } else {
                            output_update(output, None, surface);
                        }
                    }
                }

                with_states(surface, |data| {
                    send_scale_transform(
                        surface,
                        data,
                        smithay::output::Scale::Fractional(cursor_scale),
                        cursor_transform,
                    )
                });
                if let Some((surface, _)) = dnd {
                    with_states(surface, |data| {
                        send_scale_transform(
                            surface,
                            data,
                            smithay::output::Scale::Fractional(dnd_scale),
                            dnd_transform,
                        );
                    });
                }
            }
            cursor_image => {
                // There's no cursor surface, but there might be a DnD icon.
                let Some(surface) = self.cursor.dnd_icon.as_ref().map(|icon| &icon.surface) else {
                    return;
                };

                let icon = if let CursorImageStatus::Named(icon) = cursor_image {
                    *icon
                } else {
                    Default::default()
                };

                let mut dnd_scale = 1.;
                let mut dnd_transform = Transform::Normal;
                for output in self.outputs.space().outputs() {
                    let geo = self.outputs.space().output_geometry(output).unwrap();

                    // The default cursor is rendered at the right scale for each output, which
                    // means that it may have a different hotspot for each output.
                    let output_scale = output.current_scale().integer_scale();
                    let cursor = self
                        .cursor.manager()
                        .get_cursor_with_name(icon, output_scale)
                        .unwrap_or_else(|| self.cursor.manager().get_default_cursor(output_scale));

                    // For simplicity, we always use frame 0 for this computation. Let's hope the
                    // hotspot doesn't change between frames.
                    let hotspot = XCursor::hotspot(&cursor.frames()[0]).to_logical(output_scale);

                    let surface_pos = pointer_pos.to_i32_round() - hotspot;
                    let bbox = bbox_from_surface_tree(surface, surface_pos);

                    if let Some(mut overlap) = geo.intersection(bbox) {
                        overlap.loc -= surface_pos;
                        dnd_scale = f64::max(dnd_scale, output.current_scale().fractional_scale());
                        // FIXME: using the largest overlapping or "primary" output transform would
                        // make more sense here.
                        dnd_transform = output.current_transform();
                        output_update(output, Some(overlap), surface);
                    } else {
                        output_update(output, None, surface);
                    }
                }

                with_states(surface, |data| {
                    send_scale_transform(
                        surface,
                        data,
                        smithay::output::Scale::Fractional(dnd_scale),
                        dnd_transform,
                    );
                });
            }
        }
    }
}

// =============================================================================
// Refresh Methods
// =============================================================================

impl Niri {
    /// Refreshes the layout active state based on keyboard focus.
    pub fn refresh_layout(&mut self) {
        let layout_is_active = match &self.focus.current() {
            KeyboardFocus::Layout { .. } => true,
            KeyboardFocus::LayerShell { .. } => false,

            // Draw layout as active in these cases to reduce unnecessary window animations.
            // There's no confusion because these are both fullscreen modes.
            //
            // FIXME: when going into the screenshot UI from a layer-shell focus, and then back to
            // layer-shell, the layout will briefly draw as active, despite never having focus.
            KeyboardFocus::LockScreen { .. } => true,
            KeyboardFocus::ScreenshotUi => true,
            KeyboardFocus::ExitConfirmDialog => true,
            KeyboardFocus::Overview => true,
            KeyboardFocus::Mru => true,
        };

        self.layout.refresh(layout_is_active);
    }

    /// Refreshes idle inhibit state.
    pub fn refresh_idle_inhibit(&mut self) {
        use smithay::desktop::utils::surface_primary_scanout_output;
        use std::sync::atomic::Ordering;

        let _span = tracy_client::span!("Niri::refresh_idle_inhibit");

        self.focus.idle_inhibitors_mut().retain(|s| s.is_alive());

        let is_inhibited = self.is_fdo_idle_inhibited.load(Ordering::SeqCst)
            || self.focus.idle_inhibitors().iter().any(|surface| {
                with_states(surface, |states| {
                    surface_primary_scanout_output(surface, states).is_some()
                })
            });
        self.protocols.idle_notifier.set_is_inhibited(is_inhibited);
    }

    /// Refreshes window tiled states.
    pub fn refresh_window_states(&mut self) {
        let _span = tracy_client::span!("Niri::refresh_window_states");

        let config = self.config.borrow();
        self.layout.with_windows_mut(|mapped, _output| {
            mapped.update_tiled_state(config.prefer_no_csd);
        });
        drop(config);
    }

    /// Refreshes window rules.
    pub fn refresh_window_rules(&mut self) {
        let _span = tracy_client::span!("Niri::refresh_window_rules");

        let config = self.config.borrow();
        let window_rules = &config.window_rules;

        let mut windows = vec![];
        let mut outputs = std::collections::HashSet::new();
        self.layout.with_windows_mut(|mapped, output| {
            if mapped.recompute_window_rules_if_needed(window_rules, self.is_at_startup) {
                windows.push(mapped.window.clone());

                if let Some(output) = output {
                    outputs.insert(output.clone());
                }

                // Since refresh_window_rules() is called after refresh_layout(), we need to update
                // the tiled state right here, so that it's picked up by the following
                // send_pending_configure().
                mapped.update_tiled_state(config.prefer_no_csd);
            }
        });
        drop(config);

        for win in windows {
            self.layout.update_window(&win, None);
            win.toplevel()
                .expect("no X11 support")
                .send_pending_configure();
        }
        for output in outputs {
            self.queue_redraw(&output);
        }
    }

    /// Advances all animations.
    pub fn advance_animations(&mut self) {
        let _span = tracy_client::span!("Niri::advance_animations");

        self.layout.advance_animations();
        self.ui.config_error.advance_animations();
        self.ui.exit_dialog.advance_animations();
        self.ui.screenshot.advance_animations();
        self.ui.mru.advance_animations();

        for state in self.outputs.states_mut() {
            if let Some(transition) = &mut state.screen_transition {
                if transition.is_done() {
                    state.screen_transition = None;
                }
            }
        }
    }

    /// Updates render elements for outputs.
    pub fn update_render_elements(&mut self, output: Option<&Output>) {
        self.layout.update_render_elements(output);

        for (out, state) in self.outputs.state_iter_mut() {
            if output.map_or(true, |output| out == output) {
                let scale = Scale::from(out.current_scale().fractional_scale());
                let transform = out.current_transform();

                if let Some(transition) = &mut state.screen_transition {
                    transition.update_render_elements(scale, transform);
                }

                let layer_map = layer_map_for_output(out);
                for surface in layer_map.layers() {
                    let Some(mapped) = self.mapped_layer_surfaces.get_mut(surface) else {
                        continue;
                    };
                    let Some(geo) = layer_map.layer_geometry(surface) else {
                        continue;
                    };

                    mapped.update_render_elements(geo.size.to_f64());
                }
            }
        }
    }

    /// Updates shaders for all render elements.
    pub fn update_shaders(&mut self) {
        self.layout.update_shaders();

        for mapped in self.mapped_layer_surfaces.values_mut() {
            mapped.update_shaders();
        }
    }
}

// =============================================================================
// Core Render Methods (TEAM_083: moved from mod.rs)
// =============================================================================

impl Niri {
    /// Schedules an immediate redraw on all outputs if one is not already scheduled.
    pub fn queue_redraw_all(&mut self) {
        for state in self.outputs.states_mut() {
            state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
        }
    }

    /// Schedules an immediate redraw if one is not already scheduled.
    pub fn queue_redraw(&mut self, output: &Output) {
        let state = self.outputs.state_mut(output).unwrap();
        state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
    }

    pub fn redraw_queued_outputs(&mut self, backend: &mut Backend) {
        let _span = tracy_client::span!("Niri::redraw_queued_outputs");

        // Use behavior method to find output needing redraw without holding borrow.
        while let Some(output) = self.outputs.find_output_needing_redraw(|_, state| {
            matches!(
                state.redraw_state,
                RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)
            )
        }) {
            trace!("redrawing output");
            self.redraw(backend, &output);
        }
    }

    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        include_pointer: bool,
        mut target: RenderTarget,
    ) -> Vec<OutputRenderElements<R>> {
        let _span = tracy_client::span!("Niri::render");

        if target == RenderTarget::Output {
            if let Some(preview) = self.config.borrow().debug.preview_render {
                target = match preview {
                    PreviewRender::Screencast => RenderTarget::Screencast,
                    PreviewRender::ScreenCapture => RenderTarget::ScreenCapture,
                };
            }
        }

        let output_scale = Scale::from(output.current_scale().fractional_scale());

        // The pointer goes on the top.
        let mut elements = vec![];
        if include_pointer {
            elements = self.pointer_element(renderer, output);
        }

        // Next, the screen transition texture.
        {
            let state = self.outputs.state(output).unwrap();
            if let Some(transition) = &state.screen_transition {
                elements.push(transition.render(target).into());
            }
        }

        // Next, the exit confirm dialog.
        elements.extend(
            self.ui.exit_dialog
                .render(renderer, output)
                .into_iter()
                .map(OutputRenderElements::from),
        );

        // Next, the config error notification too.
        if let Some(element) = self.ui.config_error.render(renderer, output) {
            elements.push(element.into());
        }

        // If the session is locked, draw the lock surface.
        if self.is_locked() {
            let state = self.outputs.state(output).unwrap();
            if let Some(surface) = state.lock_surface.as_ref() {
                elements.extend(render_elements_from_surface_tree(
                    renderer,
                    surface.wl_surface(),
                    (0, 0),
                    output_scale,
                    1.,
                    Kind::ScanoutCandidate,
                ));
            }

            // Draw the solid color background.
            elements.push(
                SolidColorRenderElement::from_buffer(
                    &state.lock_color_buffer,
                    (0., 0.),
                    1.,
                    Kind::Unspecified,
                )
                .into(),
            );

            if self.debug_draw_opaque_regions {
                draw_opaque_regions(&mut elements, output_scale);
            }
            return elements;
        }

        // Prepare the background elements.
        let state = self.outputs.state(output).unwrap();
        let backdrop = SolidColorRenderElement::from_buffer(
            &state.backdrop_buffer,
            (0., 0.),
            1.,
            Kind::Unspecified,
        )
        .into();

        // If the screenshot UI is open, draw it.
        if self.ui.screenshot.is_open() {
            elements.extend(
                self.ui.screenshot
                    .render_output(output, target)
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add the backdrop for outputs that were connected while the screenshot UI was open.
            elements.push(backdrop);

            if self.debug_draw_opaque_regions {
                draw_opaque_regions(&mut elements, output_scale);
            }
            return elements;
        }

        // Draw the hotkey overlay on top.
        if let Some(element) = self.ui.hotkey.render(renderer, output) {
            elements.push(element.into());
        }

        // Then, the Alt-Tab switcher.
        let mru_elements = self
            .ui.mru
            .render_output(self, output, renderer, target)
            .into_iter()
            .flatten()
            .map(OutputRenderElements::from);
        elements.extend(mru_elements);

        // Don't draw the focus ring on the workspaces while interactively moving above those
        // workspaces, since the interactively-moved window already has a focus ring.
        let focus_ring = !self.layout.interactive_move_is_moving_above_output(output);

        // Get monitor elements.
        let mon = self.layout.monitor_for_output(output).unwrap();
        // Overview mode has been removed, zoom is always 1.0
        let zoom = 1.0;
        let monitor_elements = mon.render_elements(renderer, target, focus_ring);
        // render_workspace_shadows removed - workspace shadows no longer exist
        let insert_hint_elements = mon.render_insert_hint_between_workspaces(renderer);
        let int_move_elements: Vec<_> = self
            .layout
            .render_interactive_move_for_output(renderer, output, target)
            .collect();

        // Get layer-shell elements.
        let layer_map = layer_map_for_output(output);
        let mut extend_from_layer =
            |elements: &mut SplitElements<LayerSurfaceRenderElement<R>>, layer, for_backdrop| {
                self.render_layer(renderer, target, &layer_map, layer, elements, for_backdrop);
            };

        // The overlay layer elements go next.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Overlay, false);
        elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

        // Collect the top layer elements.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Top, false);
        let top_layer = layer_elems;

        // When rendering above the top layer, we put the regular monitor elements first.
        // Otherwise, we will render all layer-shell pop-ups and the top layer on top.
        if mon.render_above_top_layer() {
            // Collect all other layer-shell elements.
            let mut layer_elems = SplitElements::default();
            extend_from_layer(&mut layer_elems, Layer::Bottom, false);
            extend_from_layer(&mut layer_elems, Layer::Background, false);

            elements.extend(
                int_move_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );
            elements.extend(
                insert_hint_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            let mut ws_background: Option<SolidColorRenderElement> = None;
            // TODO: TEAM_023: Update render elements handling for Canvas2D
            // The old workspace-based render elements need to be adapted
            elements.extend(
                monitor_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            elements.extend(top_layer.into_iter().map(OutputRenderElements::from));
            elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

            if let Some(ws_background) = ws_background {
                elements.push(OutputRenderElements::from(ws_background));
            }

            // workspace_shadow_elements removed - no longer exist
        } else {
            elements.extend(top_layer.into_iter().map(OutputRenderElements::from));

            elements.extend(
                int_move_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            elements.extend(
                insert_hint_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // TEAM_048: Fixed Canvas2D rendering - monitor_elements must be added to output
            // Collect layer-shell elements that go below windows
            let mut layer_elems = SplitElements::default();
            extend_from_layer(&mut layer_elems, Layer::Bottom, false);
            extend_from_layer(&mut layer_elems, Layer::Background, false);

            // Add layer popups first (they go on top of windows)
            elements.extend(
                layer_elems
                    .popups
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add the monitor/canvas elements (contains window tiles)
            elements.extend(
                monitor_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add normal layer-shell elements (background layers)
            elements.extend(
                layer_elems
                    .normal
                    .into_iter()
                    .map(OutputRenderElements::from),
            );
        }

        // Then the backdrop.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Background, true);
        elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

        elements.push(backdrop);

        if self.debug_draw_opaque_regions {
            draw_opaque_regions(&mut elements, output_scale);
        }

        elements
    }

    #[allow(clippy::too_many_arguments)]
    fn render_layer<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        layer_map: &LayerMap,
        layer: Layer,
        elements: &mut SplitElements<LayerSurfaceRenderElement<R>>,
        for_backdrop: bool,
    ) {
        // LayerMap returns layers in reverse stacking order.
        let iter = layer_map.layers_on(layer).rev().filter_map(|surface| {
            let mapped = self.mapped_layer_surfaces.get(surface)?;

            if for_backdrop != mapped.place_within_backdrop() {
                return None;
            }

            let geo = layer_map.layer_geometry(surface)?;
            Some((mapped, geo))
        });
        for (mapped, geo) in iter {
            elements.extend(mapped.render(renderer, geo.loc.to_f64(), target));
        }
    }

    fn redraw(&mut self, backend: &mut Backend, output: &Output) {
        let _span = tracy_client::span!("Niri::redraw");

        // Verify our invariant.
        let state = self.outputs.state_mut(output).unwrap();
        assert!(matches!(
            state.redraw_state,
            RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)
        ));

        let target_presentation_time = state.frame_clock.next_presentation_time();

        // Freeze the clock at the target time.
        self.clock.set_unadjusted(target_presentation_time);

        self.update_render_elements(Some(output));

        let mut res = RenderResult::Skipped;
        if self.outputs.monitors_active() {
            let state = self.outputs.state_mut(output).unwrap();
            state.unfinished_animations_remain = self.layout.are_animations_ongoing(Some(output));
            state.unfinished_animations_remain |=
                self.ui.config_error.are_animations_ongoing();
            state.unfinished_animations_remain |= self.ui.exit_dialog.are_animations_ongoing();
            state.unfinished_animations_remain |= self.ui.screenshot.are_animations_ongoing();
            state.unfinished_animations_remain |= self.ui.mru.are_animations_ongoing();
            state.unfinished_animations_remain |= state.screen_transition.is_some();

            // Also keep redrawing if the current cursor is animated.
            state.unfinished_animations_remain |= self
                .cursor.manager
                .is_current_cursor_animated(output.current_scale().integer_scale());

            // Also check layer surfaces.
            if !state.unfinished_animations_remain {
                state.unfinished_animations_remain |= layer_map_for_output(output)
                    .layers()
                    .filter_map(|surface| self.mapped_layer_surfaces.get(surface))
                    .any(|mapped| mapped.are_animations_ongoing());
            }

            // Render.
            res = backend.render(self, output, target_presentation_time);
        }

        let is_locked = self.is_locked();
        let monitors_active = self.outputs.monitors_active();
        let state = self.outputs.state_mut(output).unwrap();

        if res == RenderResult::Skipped {
            // Update the redraw state on failed render.
            state.redraw_state = if let RedrawState::WaitingForEstimatedVBlank(token)
            | RedrawState::WaitingForEstimatedVBlankAndQueued(token) =
                state.redraw_state
            {
                RedrawState::WaitingForEstimatedVBlank(token)
            } else {
                RedrawState::Idle
            };
        }

        // Update the lock render state on successful render, or if monitors are inactive. When
        // monitors are inactive on a TTY, they have no framebuffer attached, so no sensitive data
        // from a last render will be visible.
        if res != RenderResult::Skipped || !monitors_active {
            state.lock_render_state = if is_locked {
                LockRenderState::Locked
            } else {
                LockRenderState::Unlocked
            };
        }

        // If we're in process of locking the session, check if the requirements were met.
        match mem::take(&mut self.lock_state) {
            LockState::Locking(confirmation) => {
                if state.lock_render_state == LockRenderState::Unlocked {
                    // We needed to render a locked frame on this output but failed.
                    self.unlock();
                } else {
                    // Check if all outputs are now locked.
                    let all_locked = self
                        .outputs
                        .states()
                        .all(|state| state.lock_render_state == LockRenderState::Locked);

                    if all_locked {
                        // All outputs are locked, report success.
                        let lock = confirmation.ext_session_lock().clone();
                        confirmation.lock();
                        self.lock_state = LockState::Locked(lock);
                    } else {
                        // Still waiting for other outputs.
                        self.lock_state = LockState::Locking(confirmation);
                    }
                }
            }
            lock_state => self.lock_state = lock_state,
        }

        self.refresh_on_demand_vrr(backend, output);

        // Send the frame callbacks.
        //
        // FIXME: The logic here could be a bit smarter. Currently, during an animation, the
        // surfaces that are visible for the very last frame (e.g. because the camera is moving
        // away) will receive frame callbacks, and the surfaces that are invisible but will become
        // visible next frame will not receive frame callbacks (so they will show stale contents for
        // one frame). We could advance the animations for the next frame and send frame callbacks
        // according to the expected new positions.
        //
        // However, this should probably be restricted to sending frame callbacks to more surfaces,
        // to err on the safe side.
        self.send_frame_callbacks(output);
        backend.with_primary_renderer(|renderer| {
            #[cfg(feature = "xdp-gnome-screencast")]
            {
                // Render and send to PipeWire screencast streams.
                self.render_for_screen_cast(renderer, output, target_presentation_time);

                // FIXME: when a window is hidden, it should probably still receive frame callbacks
                // and get rendered for screen cast. This is currently
                // unimplemented, but happens to work by chance, since output
                // redrawing is more eager than it should be.
                self.render_windows_for_screen_cast(renderer, output, target_presentation_time);
            }

            self.render_for_screencopy_with_damage(renderer, output);
        });
    }

    pub fn refresh_on_demand_vrr(&mut self, backend: &mut Backend, output: &Output) {
        let _span = tracy_client::span!("Niri::refresh_on_demand_vrr");

        let name = output.user_data().get::<OutputName>().unwrap();
        let on_demand = self
            .config
            .borrow()
            .outputs
            .find(name)
            .is_some_and(|output| output.is_vrr_on_demand());
        if !on_demand {
            return;
        }

        let current = self.layout.windows_for_output(output).any(|mapped| {
            mapped.rules().variable_refresh_rate == Some(true) && {
                let mut visible = false;
                mapped.window.with_surfaces(|surface, states| {
                    if !visible
                        && surface_primary_scanout_output(surface, states).as_ref() == Some(output)
                    {
                        visible = true;
                    }
                });
                visible
            }
        });

        backend.set_output_on_demand_vrr(self, output, current);
    }

    pub fn update_primary_scanout_output(
        &self,
        output: &Output,
        render_element_states: &RenderElementStates,
    ) {
        // FIXME: potentially tweak the compare function. The default one currently always prefers a
        // higher refresh-rate output, which is not always desirable (i.e. with a very small
        // overlap).
        //
        // While we only have cursors and DnD icons crossing output boundaries though, it doesn't
        // matter all that much.
        if let CursorImageStatus::Surface(surface) = &self.cursor.manager().cursor_image() {
            with_surface_tree_downward(
                surface,
                (),
                |_, _, _| TraversalAction::DoChildren(()),
                |surface, states, _| {
                    update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        render_element_states,
                        default_primary_scanout_output_compare,
                    );
                },
                |_, _, _| true,
            );
        }

        if let Some(surface) = self.cursor.dnd_icon.as_ref().map(|icon| &icon.surface) {
            with_surface_tree_downward(
                surface,
                (),
                |_, _, _| TraversalAction::DoChildren(()),
                |surface, states, _| {
                    update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        render_element_states,
                        default_primary_scanout_output_compare,
                    );
                },
                |_, _, _| true,
            );
        }
    }

    pub fn send_dmabuf_feedbacks(
        &self,
        output: &Output,
        feedback: &SurfaceDmabufFeedback,
        render_element_states: &RenderElementStates,
    ) {
        let _span = tracy_client::span!("Niri::send_dmabuf_feedbacks");

        // We can unconditionally send the current output's feedback to regular and layer-shell
        // surfaces, as they can only be displayed on a single output at a time. Even if a surface
        // is currently invisible, this is the DMABUF feedback that it should know about.
        for mapped in self.layout.windows_for_output(output) {
            mapped.window.send_dmabuf_feedback(
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        for surface in layer_map_for_output(output).layers() {
            surface.send_dmabuf_feedback(
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let Some(surface) = self.outputs.lock_surface(output) {
            send_dmabuf_feedback_surface_tree(
                surface.wl_surface(),
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let Some(surface) = self.cursor.dnd_icon.as_ref().map(|icon| &icon.surface) {
            send_dmabuf_feedback_surface_tree(
                surface,
                output,
                surface_primary_scanout_output,
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let CursorImageStatus::Surface(surface) = &self.cursor.manager().cursor_image() {
            send_dmabuf_feedback_surface_tree(
                surface,
                output,
                surface_primary_scanout_output,
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }
    }

    pub fn debug_toggle_damage(&mut self) {
        self.debug_draw_damage = !self.debug_draw_damage;

        if self.debug_draw_damage {
            for (output, state) in self.outputs.state_iter_mut() {
                state.debug_damage_tracker = OutputDamageTracker::from_output(output);
            }
        }

        self.queue_redraw_all();
    }
}
