//! Pointer (mouse) input event handling.
//!
//! This module handles pointer motion, button clicks, and scroll wheel events.

use std::cmp::min;
use std::time::Duration;

use niri_config::{Action, Bind, Key, Modifiers, Trigger};
use smithay::backend::input::{
    AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, MouseButton, PointerAxisEvent,
    PointerButtonEvent, PointerMotionEvent,
};
use smithay::input::pointer::{
    AxisFrame, ButtonEvent, CursorIcon, CursorImageStatus, Focus,
    GrabStartData as PointerGrabStartData, MotionEvent, RelativeMotionEvent,
};
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::pointer_constraints::{with_pointer_constraint, PointerConstraint};
use smithay::wayland::selection::data_device::DnDGrab;

use super::backend_ext::NiriInputBackend as InputBackend;
use super::move_grab::MoveGrab;
use super::resize_grab::ResizeGrab;
use super::spatial_movement_grab::SpatialMovementGrab;
use super::{find_configured_bind, make_binds_iter, modifiers_from_state, PointerOrTouchStartData};
use super::DOUBLE_CLICK_TIME;
use crate::layout::LayoutElement as _;
use crate::niri::{PointerVisibility, State};
use crate::utils::{center, ResizeEdge};

// TEAM_086: Trait-based pointer input handling (replaces pub(super) pattern)

/// Trait for pointer (mouse) input event handling.
///
/// This trait defines the interface for processing pointer events including
/// motion, button clicks, and scroll wheel input.
pub(crate) trait PointerInput<I: InputBackend> {
    /// Handle relative pointer motion.
    fn on_pointer_motion(&mut self, event: I::PointerMotionEvent);
    
    /// Handle absolute pointer motion.
    fn on_pointer_motion_absolute(&mut self, event: I::PointerMotionAbsoluteEvent);
    
    /// Handle pointer button press/release.
    fn on_pointer_button(&mut self, event: I::PointerButtonEvent);
    
    /// Handle pointer scroll axis events.
    fn on_pointer_axis(&mut self, event: I::PointerAxisEvent);
}

impl<I: InputBackend> PointerInput<I> for State {
    fn on_pointer_motion(&mut self, event: I::PointerMotionEvent) {
        let _was_inside_hot_corner = self.niri.pointer_inside_hot_corner;
        // Any of the early returns here mean that the pointer is not inside the hot corner.
        self.niri.pointer_inside_hot_corner = false;

        // We need an output to be able to move the pointer.
        if self.niri.outputs.space().outputs().next().is_none() {
            return;
        }

        let serial = SERIAL_COUNTER.next_serial();

        let pointer = self.niri.seat.get_pointer().unwrap();

        let pos = pointer.current_location();

        // We have an output, so we can compute the new location and focus.
        let mut new_pos = pos + event.delta();

        // We received an event for the regular pointer, so show it now.
        self.niri.cursor.visibility = PointerVisibility::Visible;
        self.niri.cursor.tablet_location = None;

        // Check if we have an active pointer constraint.
        //
        // FIXME: ideally this should use the pointer focus with up-to-date global location.
        let mut pointer_confined = None;
        if let Some(under) = &self.niri.cursor.contents.surface {
            // No need to check if the pointer focus surface matches, because here we're checking
            // for an already-active constraint, and the constraint is deactivated when the focused
            // surface changes.
            let pos_within_surface = pos - under.1;

            let mut pointer_locked = false;
            with_pointer_constraint(&under.0, &pointer, |constraint| {
                let Some(constraint) = constraint else { return };
                if !constraint.is_active() {
                    return;
                }

                // Constraint does not apply if not within region.
                if let Some(region) = constraint.region() {
                    if !region.contains(pos_within_surface.to_i32_round()) {
                        return;
                    }
                }

                match &*constraint {
                    PointerConstraint::Locked(_locked) => {
                        pointer_locked = true;
                    }
                    PointerConstraint::Confined(confine) => {
                        pointer_confined = Some((under.clone(), confine.region().cloned()));
                    }
                }
            });

            // If the pointer is locked, only send relative motion.
            if pointer_locked {
                pointer.relative_motion(
                    self,
                    Some(under.clone()),
                    &RelativeMotionEvent {
                        delta: event.delta(),
                        delta_unaccel: event.delta_unaccel(),
                        utime: event.time(),
                    },
                );

                pointer.frame(self);

                // I guess a redraw to hide the tablet cursor could be nice? Doesn't matter too
                // much here I think.
                return;
            }
        }

        if self
            .niri
            .outputs
            .space()
            .output_under(new_pos)
            .next()
            .is_none()
        {
            // We ended up outside the outputs and need to clip the movement.
            if let Some(output) = self.niri.outputs.space().output_under(pos).next() {
                // The pointer was previously on some output. Clip the movement against its
                // boundaries.
                let geom = self.niri.outputs.space().output_geometry(output).unwrap();
                new_pos.x = new_pos
                    .x
                    .clamp(geom.loc.x as f64, (geom.loc.x + geom.size.w - 1) as f64);
                new_pos.y = new_pos
                    .y
                    .clamp(geom.loc.y as f64, (geom.loc.y + geom.size.h - 1) as f64);
            } else {
                // The pointer was not on any output in the first place. Find one for it.
                // Let's do the simple thing and just put it on the first output.
                let output = self.niri.outputs.space().outputs().next().unwrap();
                let geom = self.niri.outputs.space().output_geometry(output).unwrap();
                new_pos = center(geom).to_f64();
            }
        }

        if let Some(output) = self.niri.ui.screenshot.selection_output() {
            let geom = self.niri.outputs.space().output_geometry(output).unwrap();
            let mut point = (new_pos - geom.loc.to_f64())
                .to_physical(output.current_scale().fractional_scale())
                .to_i32_round::<i32>();

            let size = output.current_mode().unwrap().size;
            let transform = output.current_transform();
            let size = transform.transform_size(size);
            point.x = point.x.clamp(0, size.w - 1);
            point.y = point.y.clamp(0, size.h - 1);

            self.niri.ui.screenshot.pointer_motion(point, None);
        }

        if let Some(mru_output) = self.niri.ui.mru.output() {
            if let Some((output, pos_within_output)) = self.niri.output_under(new_pos) {
                if mru_output == output {
                    self.niri.ui.mru.pointer_motion(pos_within_output);
                }
            }
        }

        let under = self.niri.contents_under(new_pos);

        // Handle confined pointer.
        if let Some((focus_surface, region)) = pointer_confined {
            let mut prevent = false;

            // Prevent the pointer from leaving the focused surface.
            if Some(&focus_surface.0) != under.surface.as_ref().map(|(s, _)| s) {
                prevent = true;
            }

            // Prevent the pointer from leaving the confine region, if any.
            if let Some(region) = region {
                let new_pos_within_surface = new_pos - focus_surface.1;
                if !region.contains(new_pos_within_surface.to_i32_round()) {
                    prevent = true;
                }
            }

            if prevent {
                pointer.relative_motion(
                    self,
                    Some(focus_surface),
                    &RelativeMotionEvent {
                        delta: event.delta(),
                        delta_unaccel: event.delta_unaccel(),
                        utime: event.time(),
                    },
                );

                pointer.frame(self);

                return;
            }
        }

        self.niri.handle_focus_follows_mouse(&under);

        self.niri.cursor.contents.clone_from(&under);

        pointer.motion(
            self,
            under.surface.clone(),
            &MotionEvent {
                location: new_pos,
                serial,
                time: event.time_msec(),
            },
        );

        pointer.relative_motion(
            self,
            under.surface,
            &RelativeMotionEvent {
                delta: event.delta(),
                delta_unaccel: event.delta_unaccel(),
                utime: event.time(),
            },
        );

        pointer.frame(self);

        // TEAM_014: Removed hot corner overview toggle (Part 3)
        // contents_under() will return no surface when the hot corner should trigger, so
        // pointer.motion() will set the current focus to None.
        if under.hot_corner && pointer.current_focus().is_none() {
            // Hot corner no longer triggers overview (overview removed).
            self.niri.pointer_inside_hot_corner = true;
        }

        // Activate a new confinement if necessary.
        self.niri.maybe_activate_pointer_constraint();

        // Inform the layout of an ongoing DnD operation.
        let mut is_dnd_grab = false;
        pointer.with_grab(|_, grab| {
            is_dnd_grab = grab.as_any().is::<DnDGrab<Self>>();
        });
        if is_dnd_grab {
            if let Some((output, pos_within_output)) = self.niri.output_under(new_pos) {
                let output = output.clone();
                self.niri.layout.dnd_update(output, pos_within_output);
            }
        }

        // Redraw to update the cursor position.
        // FIXME: redraw only outputs overlapping the cursor.
        self.niri.queue_redraw_all();
    }

    fn on_pointer_motion_absolute(&mut self, event: I::PointerMotionAbsoluteEvent) {
        let _was_inside_hot_corner = self.niri.pointer_inside_hot_corner;
        // Any of the early returns here mean that the pointer is not inside the hot corner.
        self.niri.pointer_inside_hot_corner = false;

        let Some(pos) = self.compute_absolute_location(&event, None).or_else(|| {
            self.global_bounding_rectangle().map(|output_geo| {
                event.position_transformed(output_geo.size) + output_geo.loc.to_f64()
            })
        }) else {
            return;
        };

        let serial = SERIAL_COUNTER.next_serial();

        let pointer = self.niri.seat.get_pointer().unwrap();

        if let Some(output) = self.niri.ui.screenshot.selection_output() {
            let geom = self.niri.outputs.space().output_geometry(output).unwrap();
            let mut point = (pos - geom.loc.to_f64())
                .to_physical(output.current_scale().fractional_scale())
                .to_i32_round::<i32>();

            let size = output.current_mode().unwrap().size;
            let transform = output.current_transform();
            let size = transform.transform_size(size);
            point.x = point.x.clamp(0, size.w - 1);
            point.y = point.y.clamp(0, size.h - 1);

            self.niri.ui.screenshot.pointer_motion(point, None);
        }

        if let Some(mru_output) = self.niri.ui.mru.output() {
            if let Some((output, pos_within_output)) = self.niri.output_under(pos) {
                if mru_output == output {
                    self.niri.ui.mru.pointer_motion(pos_within_output);
                }
            }
        }

        let under = self.niri.contents_under(pos);

        self.niri.handle_focus_follows_mouse(&under);

        self.niri.cursor.contents.clone_from(&under);

        pointer.motion(
            self,
            under.surface,
            &MotionEvent {
                location: pos,
                serial,
                time: event.time_msec(),
            },
        );

        pointer.frame(self);

        // TEAM_014: Removed hot corner overview toggle (Part 3)
        // contents_under() will return no surface when the hot corner should trigger, so
        // pointer.motion() will set the current focus to None.
        if under.hot_corner && pointer.current_focus().is_none() {
            // Hot corner no longer triggers overview (overview removed).
            self.niri.pointer_inside_hot_corner = true;
        }

        self.niri.maybe_activate_pointer_constraint();

        // We moved the pointer, show it.
        self.niri.cursor.visibility = PointerVisibility::Visible;

        // We moved the regular pointer, so show it now.
        self.niri.cursor.tablet_location = None;

        // Inform the layout of an ongoing DnD operation.
        let mut is_dnd_grab = false;
        pointer.with_grab(|_, grab| {
            is_dnd_grab = grab.as_any().is::<DnDGrab<Self>>();
        });
        if is_dnd_grab {
            if let Some((output, pos_within_output)) = self.niri.output_under(pos) {
                let output = output.clone();
                self.niri.layout.dnd_update(output, pos_within_output);
            }
        }

        // Redraw to update the cursor position.
        // FIXME: redraw only outputs overlapping the cursor.
        self.niri.queue_redraw_all();
    }

    fn on_pointer_button(&mut self, event: I::PointerButtonEvent) {
        let pointer = self.niri.seat.get_pointer().unwrap();

        let serial = SERIAL_COUNTER.next_serial();

        let button = event.button();

        let button_code = event.button_code();

        let button_state = event.state();

        let mod_key = self.backend.mod_key(&self.niri.config.borrow());

        // Ignore release events for mouse clicks that triggered a bind.
        if self.niri.suppressed_buttons.remove(&button_code) {
            return;
        }

        if ButtonState::Pressed == button_state {
            let mods = self.niri.seat.get_keyboard().unwrap().modifier_state();
            let modifiers = modifiers_from_state(mods);

            let mut is_mru_open = false;
            if let Some(mru_output) = self.niri.ui.mru.output() {
                is_mru_open = true;
                if let Some(MouseButton::Left) = button {
                    let location = pointer.current_location();
                    let (output, pos_within_output) = self.niri.output_under(location).unwrap();
                    if mru_output == output {
                        let id = self.niri.ui.mru.pointer_motion(pos_within_output);
                        if id.is_some() {
                            self.confirm_mru();
                        } else {
                            self.niri.cancel_mru();
                        }
                    } else {
                        self.niri.cancel_mru();
                    }

                    self.niri.suppressed_buttons.insert(button_code);
                    return;
                }
            }

            if is_mru_open || self.niri.input.mods_with_mouse_binds().contains(&modifiers) {
                if let Some(bind) = match button {
                    Some(MouseButton::Left) => Some(Trigger::MouseLeft),
                    Some(MouseButton::Right) => Some(Trigger::MouseRight),
                    Some(MouseButton::Middle) => Some(Trigger::MouseMiddle),
                    Some(MouseButton::Back) => Some(Trigger::MouseBack),
                    Some(MouseButton::Forward) => Some(Trigger::MouseForward),
                    _ => None,
                }
                .and_then(|trigger| {
                    let config = self.niri.config.borrow();
                    let bindings = make_binds_iter(&config, &mut self.niri.ui.mru, modifiers);
                    find_configured_bind(bindings, mod_key, trigger, mods)
                }) {
                    self.niri.suppressed_buttons.insert(button_code);
                    self.handle_bind(bind.clone());
                    return;
                };
            }

            // We received an event for the regular pointer, so show it now.
            self.niri.cursor.visibility = PointerVisibility::Visible;
            self.niri.cursor.tablet_location = None;

            // Overview mode has been removed, this is always false
            let is_overview_open = false;

            if is_overview_open && !pointer.is_grabbed() && button == Some(MouseButton::Right) {
                if let Some((output, ws)) = self.niri.row_under_cursor(true) {
                    let ws_id = ws.id();
                    let ws_idx = self.niri.layout.find_workspace_by_id(ws_id).unwrap().0;

                    self.niri.layout.focus_output(&output);

                    let location = pointer.current_location();
                    let start_data = PointerGrabStartData {
                        focus: None,
                        button: button_code,
                        location,
                    };
                    self.niri.layout.view_offset_gesture_begin(
                        &output,
                        Some(ws_idx as usize),
                        false,
                    );
                    let grab = SpatialMovementGrab::new(start_data, output, ws_id, true);
                    pointer.set_grab(self, grab, serial, Focus::Clear);
                    self.niri
                        .cursor
                        .manager
                        .set_cursor_image(CursorImageStatus::Named(CursorIcon::AllScroll));

                    // FIXME: granular.
                    self.niri.queue_redraw_all();
                    return;
                }
            }

            if button == Some(MouseButton::Middle) && !pointer.is_grabbed() {
                let mod_down = modifiers_from_state(mods).contains(mod_key.to_modifiers());
                if mod_down {
                    let output_ws = if is_overview_open {
                        self.niri.row_under_cursor(true)
                    } else {
                        // We don't want to accidentally "catch" the wrong workspace during
                        // animations.
                        self.niri.output_under_cursor().and_then(|output| {
                            let mon = self.niri.layout.monitor_for_output(&output)?;
                            let ws = mon.canvas().active_row()?;
                            Some((output, ws))
                        })
                    };

                    if let Some((output, ws)) = output_ws {
                        let ws_id = ws.id();

                        self.niri.layout.focus_output(&output);

                        let location = pointer.current_location();
                        let start_data = PointerGrabStartData {
                            focus: None,
                            button: button_code,
                            location,
                        };
                        let grab = SpatialMovementGrab::new(start_data, output, ws_id, false);
                        pointer.set_grab(self, grab, serial, Focus::Clear);
                        self.niri
                            .cursor
                            .manager
                            .set_cursor_image(CursorImageStatus::Named(CursorIcon::AllScroll));

                        // FIXME: granular.
                        self.niri.queue_redraw_all();

                        // Don't activate the window under the cursor to avoid unnecessary
                        // scrolling when e.g. Mod+MMB clicking on a partially off-screen window.
                        return;
                    }
                }
            }

            if let Some(mapped) = self.niri.window_under_cursor() {
                let window = mapped.window.clone();

                // Check if we need to start an interactive move.
                if button == Some(MouseButton::Left) && !pointer.is_grabbed() {
                    let mod_down = modifiers_from_state(mods).contains(mod_key.to_modifiers());
                    if is_overview_open || mod_down {
                        let location = pointer.current_location();

                        if !is_overview_open {
                            self.niri.layout.activate_window(&window);
                        }

                        let start_data = PointerGrabStartData {
                            focus: None,
                            button: button_code,
                            location,
                        };
                        let start_data = PointerOrTouchStartData::Pointer(start_data);
                        if let Some(grab) = MoveGrab::new(self, start_data, window.clone(), false) {
                            pointer.set_grab(self, grab, serial, Focus::Clear);
                        }
                    }
                }
                // Check if we need to start an interactive resize.
                else if button == Some(MouseButton::Right) && !pointer.is_grabbed() {
                    let mod_down = modifiers_from_state(mods).contains(mod_key.to_modifiers());
                    if mod_down {
                        let location = pointer.current_location();
                        let (output, pos_within_output) = self.niri.output_under(location).unwrap();
                        let edges = self
                            .niri
                            .layout
                            .resize_edges_under(output, pos_within_output)
                            .unwrap_or(ResizeEdge::empty());

                        if !edges.is_empty() {
                            // See if we got a double resize-click gesture.
                            // FIXME: deduplicate with resize_request in xdg-shell somehow.
                            let time = super::get_monotonic_time();
                            let last_cell = mapped.last_interactive_resize_start();
                            let mut last = last_cell.get();
                            last_cell.set(Some((time, edges)));

                            // Floating windows don't have either of the double-resize-click
                            // gestures, so just allow it to resize.
                            if mapped.is_floating() {
                                last = None;
                                last_cell.set(None);
                            }

                            if let Some((last_time, last_edges)) = last {
                                if time.saturating_sub(last_time) <= DOUBLE_CLICK_TIME {
                                    // Allow quick resize after a triple click.
                                    last_cell.set(None);

                                    let intersection = edges.intersection(last_edges);
                                    if intersection.intersects(ResizeEdge::LEFT_RIGHT) {
                                        // FIXME: don't activate once we can pass specific windows
                                        // to actions.
                                        self.niri.layout.activate_window(&window);
                                        self.niri.layout.toggle_full_width();
                                    }
                                    if intersection.intersects(ResizeEdge::TOP_BOTTOM) {
                                        self.niri.layout.activate_window(&window);
                                        self.niri.layout.reset_window_height(Some(&window));
                                    }
                                    // FIXME: granular.
                                    self.niri.queue_redraw_all();
                                    return;
                                }
                            }

                            self.niri.layout.activate_window(&window);

                            if self
                                .niri
                                .layout
                                .interactive_resize_begin(window.clone(), edges)
                            {
                                let start_data = PointerGrabStartData {
                                    focus: None,
                                    button: button_code,
                                    location,
                                };
                                let grab = ResizeGrab::new(start_data, window.clone());
                                pointer.set_grab(self, grab, serial, Focus::Clear);
                                self.niri.cursor.manager.set_cursor_image(
                                    CursorImageStatus::Named(edges.cursor_icon()),
                                );
                            }
                        }
                    }
                }

                if !is_overview_open {
                    self.niri.layout.activate_window(&window);
                }

                // FIXME: granular.
                self.niri.queue_redraw_all();
            // TEAM_014: Removed overview workspace activation (Part 3)
            } else if let Some(output) = self.niri.output_under_cursor() {
                self.niri.layout.focus_output(&output);

                // FIXME: granular.
                self.niri.queue_redraw_all();
            }
        };

        self.update_pointer_contents();

        if ButtonState::Pressed == button_state {
            let layer_under = self.niri.cursor.contents.layer.clone();
            self.niri.focus_layer_surface_if_on_demand(layer_under);
        }

        if button == Some(MouseButton::Left) && self.niri.ui.screenshot.is_open() {
            if button_state == ButtonState::Pressed {
                let pos = pointer.current_location();
                if let Some((output, _)) = self.niri.output_under(pos) {
                    let output = output.clone();
                    let geom = self.niri.outputs.space().output_geometry(&output).unwrap();
                    let mut point = (pos - geom.loc.to_f64())
                        .to_physical(output.current_scale().fractional_scale())
                        .to_i32_round();

                    let size = output.current_mode().unwrap().size;
                    let transform = output.current_transform();
                    let size = transform.transform_size(size);
                    point.x = min(size.w - 1, point.x);
                    point.y = min(size.h - 1, point.y);

                    if self.niri.ui.screenshot.pointer_down(output, point, None) {
                        self.niri.queue_redraw_all();
                    }
                }
            } else if let Some(capture) = self.niri.ui.screenshot.pointer_up(None) {
                if capture {
                    self.confirm_screenshot(true);
                } else {
                    self.niri.queue_redraw_all();
                }
            }
        }

        pointer.button(
            self,
            &ButtonEvent {
                button: button_code,
                state: button_state,
                serial,
                time: event.time_msec(),
            },
        );
        pointer.frame(self);
    }

    fn on_pointer_axis(&mut self, event: I::PointerAxisEvent) {
        let pointer = &self.niri.seat.get_pointer().unwrap();

        let source = event.source();

        let mod_key = self.backend.mod_key(&self.niri.config.borrow());

        // We received an event for the regular pointer, so show it now. This is also needed for
        // update_pointer_contents() below to return the real contents, necessary for the pointer
        // axis event to reach the window.
        self.niri.cursor.visibility = PointerVisibility::Visible;
        self.niri.cursor.tablet_location = None;

        let timestamp = Duration::from_micros(event.time());

        let horizontal_amount_v120 = event.amount_v120(Axis::Horizontal);
        let vertical_amount_v120 = event.amount_v120(Axis::Vertical);

        // Overview mode has been removed, this is always false
        let is_overview_open = false;

        // We should only handle scrolling in the overview if the pointer is not over a (top or
        // overlay) layer surface.
        let should_handle_in_overview = if is_overview_open {
            // FIXME: ideally this should happen after updating the pointer contents, which happens
            // below. However, our pointer actions are supposed to act on the old surface, before
            // updating the pointer contents.
            pointer
                .current_focus()
                .map(|surface| self.niri.find_root_shell_surface(&surface))
                .map_or(true, |root| {
                    !self
                        .niri
                        .mapped_layer_surfaces
                        .keys()
                        .any(|layer| *layer.wl_surface() == root)
                })
        } else {
            false
        };

        let is_mru_open = self.niri.ui.mru.is_open();

        // Handle wheel scroll bindings.
        if source == AxisSource::Wheel {
            // If we have a scroll bind with current modifiers, then accumulate and don't pass to
            // Wayland. If there's no bind, reset the accumulator.
            let mods = self.niri.seat.get_keyboard().unwrap().modifier_state();
            let modifiers = modifiers_from_state(mods);
            let should_handle = should_handle_in_overview
                || is_mru_open
                || self.niri.input.mods_with_wheel_binds().contains(&modifiers);
            if should_handle {
                let horizontal = horizontal_amount_v120.unwrap_or(0.);
                let ticks = self
                    .niri
                    .input
                    .horizontal_wheel_mut()
                    .accumulate(horizontal);
                if ticks != 0 {
                    let (bind_left, bind_right) = if should_handle_in_overview
                        && modifiers.is_empty()
                    {
                        let bind_left = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollLeft,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusColumnLeftUnderMouse,
                            repeat: true,
                            cooldown: None,
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        let bind_right = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollRight,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusColumnRightUnderMouse,
                            repeat: true,
                            cooldown: None,
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        (bind_left, bind_right)
                    } else {
                        let config = self.niri.config.borrow();
                        let bindings = make_binds_iter(&config, &mut self.niri.ui.mru, modifiers);
                        let bind_left = find_configured_bind(
                            bindings.clone(),
                            mod_key,
                            Trigger::WheelScrollLeft,
                            mods,
                        );
                        let bind_right = find_configured_bind(
                            bindings,
                            mod_key,
                            Trigger::WheelScrollRight,
                            mods,
                        );
                        (bind_left, bind_right)
                    };

                    if let Some(right) = bind_right {
                        for _ in 0..ticks {
                            self.handle_bind(right.clone());
                        }
                    }
                    if let Some(left) = bind_left {
                        for _ in ticks..0 {
                            self.handle_bind(left.clone());
                        }
                    }
                }

                let vertical = vertical_amount_v120.unwrap_or(0.);
                let ticks = self.niri.input.vertical_wheel_mut().accumulate(vertical);
                if ticks != 0 {
                    let (bind_up, bind_down) = if should_handle_in_overview && modifiers.is_empty()
                    {
                        let bind_up = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollUp,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusRowUpUnderMouse,
                            repeat: true,
                            cooldown: Some(Duration::from_millis(50)),
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        let bind_down = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollDown,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusRowDownUnderMouse,
                            repeat: true,
                            cooldown: Some(Duration::from_millis(50)),
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        (bind_up, bind_down)
                    } else if should_handle_in_overview && modifiers == Modifiers::SHIFT {
                        let bind_up = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollUp,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusColumnLeftUnderMouse,
                            repeat: true,
                            cooldown: Some(Duration::from_millis(50)),
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        let bind_down = Some(Bind {
                            key: Key {
                                trigger: Trigger::WheelScrollDown,
                                modifiers: Modifiers::empty(),
                            },
                            action: Action::FocusColumnRightUnderMouse,
                            repeat: true,
                            cooldown: Some(Duration::from_millis(50)),
                            allow_when_locked: false,
                            allow_inhibiting: false,
                            hotkey_overlay_title: None,
                        });
                        (bind_up, bind_down)
                    } else {
                        let config = self.niri.config.borrow();
                        let bindings = make_binds_iter(&config, &mut self.niri.ui.mru, modifiers);
                        let bind_up = find_configured_bind(
                            bindings.clone(),
                            mod_key,
                            Trigger::WheelScrollUp,
                            mods,
                        );
                        let bind_down =
                            find_configured_bind(bindings, mod_key, Trigger::WheelScrollDown, mods);
                        (bind_up, bind_down)
                    };

                    if let Some(down) = bind_down {
                        for _ in 0..ticks {
                            self.handle_bind(down.clone());
                        }
                    }
                    if let Some(up) = bind_up {
                        for _ in ticks..0 {
                            self.handle_bind(up.clone());
                        }
                    }
                }

                return;
            } else {
                self.niri.input.horizontal_wheel_mut().reset();
                self.niri.input.vertical_wheel_mut().reset();
            }
        }

        let horizontal_amount = event.amount(Axis::Horizontal);
        let vertical_amount = event.amount(Axis::Vertical);

        // Handle touchpad scroll bindings.
        if source == AxisSource::Finger {
            let mods = self.niri.seat.get_keyboard().unwrap().modifier_state();
            let modifiers = modifiers_from_state(mods);

            let horizontal = horizontal_amount.unwrap_or(0.);
            let vertical = vertical_amount.unwrap_or(0.);

            if should_handle_in_overview && modifiers.is_empty() {
                let mut redraw = false;

                let action = self
                    .niri
                    .input
                    .overview_swipe_mut()
                    .update(horizontal, vertical);
                let is_vertical = self.niri.input.overview_swipe().is_vertical();

                if action.end() {
                    if is_vertical {
                        redraw |= self
                            .niri
                            .layout
                            .row_switch_gesture_end(Some(true))
                            .is_some();
                    } else {
                        redraw |= self
                            .niri
                            .layout
                            .view_offset_gesture_end(Some(true))
                            .is_some();
                    }
                } else {
                    // Maybe begin, then update.
                    if is_vertical {
                        if action.begin() {
                            if let Some(output) = self.niri.output_under_cursor() {
                                self.niri
                                    .layout
                                    .row_switch_gesture_begin(&output, true);
                                redraw = true;
                            }
                        }

                        let res = self
                            .niri
                            .layout
                            .row_switch_gesture_update(vertical, timestamp, true);
                        if let Some(Some(_)) = res {
                            redraw = true;
                        }
                    } else {
                        if action.begin() {
                            if let Some((output, ws)) = self.niri.row_under_cursor(true) {
                                let ws_id = ws.id();
                                let ws_idx =
                                    self.niri.layout.find_workspace_by_id(ws_id).unwrap().0;

                                self.niri.layout.view_offset_gesture_begin(
                                    &output,
                                    Some(ws_idx as usize),
                                    true,
                                );
                                redraw = true;
                            }
                        }

                        let res = self
                            .niri
                            .layout
                            .view_offset_gesture_update(horizontal, timestamp, true);
                        if let Some(Some(_)) = res {
                            redraw = true;
                        }
                    }
                }

                if redraw {
                    self.niri.queue_redraw_all();
                }

                return;
            } else {
                let mut redraw = false;
                if self.niri.input.overview_swipe_mut().reset() {
                    if self.niri.input.overview_swipe().is_vertical() {
                        redraw |= self
                            .niri
                            .layout
                            .row_switch_gesture_end(Some(true))
                            .is_some();
                    } else {
                        redraw |= self
                            .niri
                            .layout
                            .view_offset_gesture_end(Some(true))
                            .is_some();
                    }
                }
                if redraw {
                    self.niri.queue_redraw_all();
                }
            }

            if is_mru_open
                || self
                    .niri
                    .input
                    .mods_with_finger_scroll_binds()
                    .contains(&modifiers)
            {
                let ticks = self
                    .niri
                    .input
                    .horizontal_finger_mut()
                    .accumulate(horizontal);
                if ticks != 0 {
                    let config = self.niri.config.borrow();
                    let bindings = make_binds_iter(&config, &mut self.niri.ui.mru, modifiers);
                    let bind_left = find_configured_bind(
                        bindings.clone(),
                        mod_key,
                        Trigger::TouchpadScrollLeft,
                        mods,
                    );
                    let bind_right =
                        find_configured_bind(bindings, mod_key, Trigger::TouchpadScrollRight, mods);
                    drop(config);

                    if let Some(right) = bind_right {
                        for _ in 0..ticks {
                            self.handle_bind(right.clone());
                        }
                    }
                    if let Some(left) = bind_left {
                        for _ in ticks..0 {
                            self.handle_bind(left.clone());
                        }
                    }
                }

                let ticks = self.niri.input.vertical_finger_mut().accumulate(vertical);
                if ticks != 0 {
                    let config = self.niri.config.borrow();
                    let bindings = make_binds_iter(&config, &mut self.niri.ui.mru, modifiers);
                    let bind_up = find_configured_bind(
                        bindings.clone(),
                        mod_key,
                        Trigger::TouchpadScrollUp,
                        mods,
                    );
                    let bind_down =
                        find_configured_bind(bindings, mod_key, Trigger::TouchpadScrollDown, mods);
                    drop(config);

                    if let Some(down) = bind_down {
                        for _ in 0..ticks {
                            self.handle_bind(down.clone());
                        }
                    }
                    if let Some(up) = bind_up {
                        for _ in ticks..0 {
                            self.handle_bind(up.clone());
                        }
                    }
                }

                return;
            } else {
                self.niri.input.horizontal_finger_mut().reset();
                self.niri.input.vertical_finger_mut().reset();
            }
        }

        self.update_pointer_contents();

        let device_scroll_factor = {
            let config = self.niri.config.borrow();
            match source {
                AxisSource::Wheel => config.input.mouse.scroll_factor,
                AxisSource::Finger => config.input.touchpad.scroll_factor,
                _ => None,
            }
        };

        // Get window-specific scroll factor
        let window_scroll_factor = pointer
            .current_focus()
            .map(|focused| self.niri.find_root_shell_surface(&focused))
            .and_then(|root| self.niri.layout.find_window_and_output(&root).unzip().0)
            .and_then(|window| window.rules().scroll_factor)
            .unwrap_or(1.);

        // Determine final scroll factors based on configuration
        let (horizontal_factor, vertical_factor) = device_scroll_factor
            .map(|x| x.h_v_factors())
            .unwrap_or((1.0, 1.0));
        let (horizontal_factor, vertical_factor) = (
            horizontal_factor * window_scroll_factor,
            vertical_factor * window_scroll_factor,
        );

        let horizontal_amount = horizontal_amount.unwrap_or_else(|| {
            // Winit backend, discrete scrolling.
            horizontal_amount_v120.unwrap_or(0.0) / 120. * 15.
        }) * horizontal_factor;

        let vertical_amount = vertical_amount.unwrap_or_else(|| {
            // Winit backend, discrete scrolling.
            vertical_amount_v120.unwrap_or(0.0) / 120. * 15.
        }) * vertical_factor;

        let horizontal_amount_v120 = horizontal_amount_v120.map(|x| x * horizontal_factor);
        let vertical_amount_v120 = vertical_amount_v120.map(|x| x * vertical_factor);

        let mut frame = AxisFrame::new(event.time_msec()).source(source);
        if horizontal_amount != 0.0 {
            frame = frame
                .relative_direction(Axis::Horizontal, event.relative_direction(Axis::Horizontal));
            frame = frame.value(Axis::Horizontal, horizontal_amount);
            if let Some(v120) = horizontal_amount_v120 {
                frame = frame.v120(Axis::Horizontal, v120 as i32);
            }
        }
        if vertical_amount != 0.0 {
            frame =
                frame.relative_direction(Axis::Vertical, event.relative_direction(Axis::Vertical));
            frame = frame.value(Axis::Vertical, vertical_amount);
            if let Some(v120) = vertical_amount_v120 {
                frame = frame.v120(Axis::Vertical, v120 as i32);
            }
        }

        if source == AxisSource::Finger {
            if event.amount(Axis::Horizontal) == Some(0.0) {
                frame = frame.stop(Axis::Horizontal);
            }
            if event.amount(Axis::Vertical) == Some(0.0) {
                frame = frame.stop(Axis::Vertical);
            }
        }

        pointer.axis(self, frame);
        pointer.frame(self);
    }
}
