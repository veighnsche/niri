//! Tablet/stylus input event handling.

use std::cmp::min;

use smithay::backend::input::{
    Event, ProximityState, TabletToolButtonEvent, TabletToolEvent, TabletToolProximityEvent,
    TabletToolTipEvent, TabletToolTipState,
};
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::tablet_manager::{TabletDescriptor, TabletSeatTrait};

use super::backend_ext::NiriInputBackend as InputBackend;
use crate::niri::{PointerVisibility, State};

// TEAM_086: Trait-based tablet input handling (replaces pub(super) pattern)

/// Trait for tablet/stylus input event handling.
///
/// This trait defines the interface for processing tablet tool events including
/// axis, tip, proximity, and button events.
pub(crate) trait TabletInput<I: InputBackend>
where
    I::Device: 'static,
{
    /// Handle tablet tool axis event (motion with pressure, tilt, etc.).
    fn on_tablet_tool_axis(&mut self, event: I::TabletToolAxisEvent);
    
    /// Handle tablet tool tip event (pen touch down/up).
    fn on_tablet_tool_tip(&mut self, event: I::TabletToolTipEvent);
    
    /// Handle tablet tool proximity event (pen entering/leaving tablet range).
    fn on_tablet_tool_proximity(&mut self, event: I::TabletToolProximityEvent);
    
    /// Handle tablet tool button event.
    fn on_tablet_tool_button(&mut self, event: I::TabletToolButtonEvent);
}

impl<I: InputBackend> TabletInput<I> for State
where
    I::Device: 'static,
{
    fn on_tablet_tool_axis(&mut self, event: I::TabletToolAxisEvent) {
        let Some(pos) = self.compute_tablet_position(&event) else {
            return;
        };

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

        let tablet_seat = self.niri.seat.tablet_seat();
        let tablet = tablet_seat.get_tablet(&TabletDescriptor::from(&event.device()));
        let tool = tablet_seat.get_tool(&event.tool());
        if let (Some(tablet), Some(tool)) = (tablet, tool) {
            if event.pressure_has_changed() {
                tool.pressure(event.pressure());
            }
            if event.distance_has_changed() {
                tool.distance(event.distance());
            }
            if event.tilt_has_changed() {
                tool.tilt(event.tilt());
            }
            if event.slider_has_changed() {
                tool.slider_position(event.slider_position());
            }
            if event.rotation_has_changed() {
                tool.rotation(event.rotation());
            }
            if event.wheel_has_changed() {
                tool.wheel(event.wheel_delta(), event.wheel_delta_discrete());
            }

            tool.motion(
                pos,
                under.surface,
                &tablet,
                SERIAL_COUNTER.next_serial(),
                event.time_msec(),
            );

            self.niri.cursor.visibility = PointerVisibility::Visible;
            self.niri.cursor.tablet_location = Some(pos);
        }

        // Redraw to update the cursor position.
        // FIXME: redraw only outputs overlapping the cursor.
        self.niri.queue_redraw_all();
    }

    fn on_tablet_tool_tip(&mut self, event: I::TabletToolTipEvent) {
        let tool = self.niri.seat.tablet_seat().get_tool(&event.tool());

        let Some(tool) = tool else {
            return;
        };
        let tip_state = event.tip_state();

        // Overview mode has been removed, this is always false
        let _is_overview_open = false;

        match tip_state {
            TabletToolTipState::Down => {
                let serial = SERIAL_COUNTER.next_serial();
                tool.tip_down(serial, event.time_msec());

                if let Some(pos) = self.niri.cursor.tablet_location {
                    let under = self.niri.contents_under(pos);

                    if self.niri.ui.screenshot.is_open() {
                        if let Some(output) = under.output.clone() {
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
                    } else if let Some(mru_output) = self.niri.ui.mru.output() {
                        if let Some((output, pos_within_output)) = self.niri.output_under(pos) {
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
                        }
                    } else if let Some((window, _)) = under.window {
                        // DEPRECATED(overview): Removed overview-specific window activation
                        self.niri.layout.activate_window(&window);

                        // FIXME: granular.
                        self.niri.queue_redraw_all();
                    // DEPRECATED(overview): Removed overview workspace activation block
                    } else if let Some(output) = under.output {
                        self.niri.layout.focus_output(&output);

                        // FIXME: granular.
                        self.niri.queue_redraw_all();
                    }
                    self.niri.focus_layer_surface_if_on_demand(under.layer);
                }
            }
            TabletToolTipState::Up => {
                if let Some(capture) = self.niri.ui.screenshot.pointer_up(None) {
                    if capture {
                        self.confirm_screenshot(true);
                    } else {
                        self.niri.queue_redraw_all();
                    }
                }

                tool.tip_up(event.time_msec());
            }
        }
    }

    fn on_tablet_tool_proximity(&mut self, event: I::TabletToolProximityEvent) {
        let Some(pos) = self.compute_tablet_position(&event) else {
            return;
        };

        let under = self.niri.contents_under(pos);

        let tablet_seat = self.niri.seat.tablet_seat();
        let display_handle = self.niri.display_handle.clone();
        let tool = tablet_seat.add_tool::<Self>(self, &display_handle, &event.tool());
        let tablet = tablet_seat.get_tablet(&TabletDescriptor::from(&event.device()));
        if let Some(tablet) = tablet {
            match event.state() {
                ProximityState::In => {
                    if let Some(under) = under.surface {
                        tool.proximity_in(
                            pos,
                            under,
                            &tablet,
                            SERIAL_COUNTER.next_serial(),
                            event.time_msec(),
                        );
                    }
                    self.niri.cursor.visibility = PointerVisibility::Visible;
                    self.niri.cursor.tablet_location = Some(pos);
                }
                ProximityState::Out => {
                    tool.proximity_out(event.time_msec());

                    // Move the mouse pointer here to avoid discontinuity.
                    //
                    // Plus, Wayland SDL2 currently warps the pointer into some weird
                    // location on proximity out, so this should help it a little.
                    if let Some(pos) = self.niri.cursor.tablet_location {
                        self.move_cursor(pos);
                    }

                    self.niri.cursor.visibility = PointerVisibility::Visible;
                    self.niri.cursor.tablet_location = None;
                }
            }

            // FIXME: granular.
            self.niri.queue_redraw_all();
        }
    }

    fn on_tablet_tool_button(&mut self, event: I::TabletToolButtonEvent) {
        let tool = self.niri.seat.tablet_seat().get_tool(&event.tool());

        if let Some(tool) = tool {
            tool.button(
                event.button(),
                event.button_state(),
                SERIAL_COUNTER.next_serial(),
                event.time_msec(),
            );
        }
    }
}
