//! Touch screen event handling.

use std::cmp::min;

use smithay::backend::input::{Event, TouchEvent};
use smithay::input::touch::{
    DownEvent, GrabStartData as TouchGrabStartData, MotionEvent as TouchMotionEvent, UpEvent,
};
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::selection::data_device::DnDGrab;

use super::backend_ext::NiriInputBackend as InputBackend;
use super::move_grab::MoveGrab;
use super::modifiers_from_state;
use super::PointerOrTouchStartData;
use crate::niri::{PointerVisibility, State};

// TEAM_086: Trait-based touch input handling (replaces pub(super) pattern)

/// Trait for touch screen input event handling.
///
/// This trait defines the interface for processing touch events including
/// touch down, up, motion, frame, and cancel events.
pub(crate) trait TouchInput<I: InputBackend> {
    /// Handle touch down event.
    fn on_touch_down(&mut self, evt: I::TouchDownEvent);
    
    /// Handle touch up event.
    fn on_touch_up(&mut self, evt: I::TouchUpEvent);
    
    /// Handle touch motion event.
    fn on_touch_motion(&mut self, evt: I::TouchMotionEvent);
    
    /// Handle touch frame event.
    fn on_touch_frame(&mut self, evt: I::TouchFrameEvent);
    
    /// Handle touch cancel event.
    fn on_touch_cancel(&mut self, evt: I::TouchCancelEvent);
}

impl<I: InputBackend> TouchInput<I> for State {
    fn on_touch_down(&mut self, evt: I::TouchDownEvent) {
        let Some(handle) = self.niri.seat.get_touch() else {
            return;
        };
        let Some(pos) = self.compute_touch_location(&evt) else {
            return;
        };
        let slot = evt.slot();

        let serial = SERIAL_COUNTER.next_serial();

        let under = self.niri.contents_under(pos);

        let mod_key = self.backend.mod_key(&self.niri.config.borrow());

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

                if self
                    .niri
                    .ui
                    .screenshot
                    .pointer_down(output, point, Some(slot))
                {
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
        } else if !handle.is_grabbed() {
            let mods = self.niri.seat.get_keyboard().unwrap().modifier_state();
            let mods = modifiers_from_state(mods);
            let mod_down = mods.contains(mod_key.to_modifiers());

            if false
                // self.niri.layout.is_overview_open() - overview mode removed
                && !mod_down
                && under.layer.is_none()
                && under.output.is_some()
            {
                let (output, _pos_within_output) = self.niri.output_under(pos).unwrap();
                let _output = output.clone();

                let mut _matched_narrow = true;
                let mut ws = self.niri.row_under(false, pos);
                if ws.is_none() {
                    _matched_narrow = false;
                    ws = self.niri.row_under(true, pos);
                }
                let _ws_id = ws.map(|(_, ws)| ws.id());

                let mapped = self.niri.window_under(pos);
                let _window = mapped.map(|mapped| mapped.window.clone());

                // TouchOverviewGrab removed - overview mode is disabled
            } else if let Some((window, _)) = under.window {
                self.niri.layout.activate_window(&window);

                // Check if we need to start a touch move grab.
                if mod_down {
                    let start_data = TouchGrabStartData {
                        focus: None,
                        slot,
                        location: pos,
                    };
                    let start_data = PointerOrTouchStartData::Touch(start_data);
                    if let Some(grab) = MoveGrab::new(self, start_data, window.clone(), true) {
                        handle.set_grab(self, grab, serial);
                    }
                }

                // FIXME: granular.
                self.niri.queue_redraw_all();
            } else if let Some(output) = under.output {
                self.niri.layout.focus_output(&output);

                // FIXME: granular.
                self.niri.queue_redraw_all();
            }
            self.niri.focus_layer_surface_if_on_demand(under.layer);
        };

        handle.down(
            self,
            under.surface,
            &DownEvent {
                slot,
                location: pos,
                serial,
                time: evt.time_msec(),
            },
        );

        // We're using touch, hide the pointer.
        self.niri.cursor.visibility = PointerVisibility::Disabled;
    }

    fn on_touch_up(&mut self, evt: I::TouchUpEvent) {
        let Some(handle) = self.niri.seat.get_touch() else {
            return;
        };
        let slot = evt.slot();

        if let Some(capture) = self.niri.ui.screenshot.pointer_up(Some(slot)) {
            if capture {
                self.confirm_screenshot(true);
            } else {
                self.niri.queue_redraw_all();
            }
        }

        let serial = SERIAL_COUNTER.next_serial();
        handle.up(
            self,
            &UpEvent {
                slot,
                serial,
                time: evt.time_msec(),
            },
        )
    }

    fn on_touch_motion(&mut self, evt: I::TouchMotionEvent) {
        let Some(handle) = self.niri.seat.get_touch() else {
            return;
        };
        let Some(pos) = self.compute_touch_location(&evt) else {
            return;
        };
        let slot = evt.slot();

        if let Some(output) = self.niri.ui.screenshot.selection_output().cloned() {
            let geom = self.niri.outputs.space().output_geometry(&output).unwrap();
            let mut point = (pos - geom.loc.to_f64())
                .to_physical(output.current_scale().fractional_scale())
                .to_i32_round::<i32>();

            let size = output.current_mode().unwrap().size;
            let transform = output.current_transform();
            let size = transform.transform_size(size);
            point.x = point.x.clamp(0, size.w - 1);
            point.y = point.y.clamp(0, size.h - 1);

            self.niri.ui.screenshot.pointer_motion(point, Some(slot));
            self.niri.queue_redraw(&output);
        }

        let under = self.niri.contents_under(pos);
        handle.motion(
            self,
            under.surface,
            &TouchMotionEvent {
                slot,
                location: pos,
                time: evt.time_msec(),
            },
        );

        // Inform the layout of an ongoing DnD operation.
        let mut is_dnd_grab = false;
        handle.with_grab(|_, grab| {
            is_dnd_grab = grab.as_any().is::<DnDGrab<Self>>();
        });
        if is_dnd_grab {
            if let Some((output, pos_within_output)) = self.niri.output_under(pos) {
                let output = output.clone();
                self.niri.layout.dnd_update(output, pos_within_output);
            }
        }
    }

    fn on_touch_frame(&mut self, _evt: I::TouchFrameEvent) {
        let Some(handle) = self.niri.seat.get_touch() else {
            return;
        };
        handle.frame(self);
    }

    fn on_touch_cancel(&mut self, _evt: I::TouchCancelEvent) {
        let Some(handle) = self.niri.seat.get_touch() else {
            return;
        };
        handle.cancel(self);
    }
}
