//! Touchpad gesture event handling.

use std::any::Any;
use std::time::Duration;

use input::event::gesture::GestureEventCoordinates as _;
use smithay::backend::input::{Event, GestureBeginEvent, GestureEndEvent};
use smithay::backend::input::{GesturePinchUpdateEvent as _, GestureSwipeUpdateEvent as _};
use smithay::input::pointer::{
    GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
    GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
};
use smithay::utils::SERIAL_COUNTER;

use super::backend_ext::NiriInputBackend as InputBackend;
use crate::niri::State;

// TEAM_086: Trait-based gesture input handling (replaces pub(super) pattern)

/// Trait for touchpad gesture event handling.
///
/// This trait defines the interface for processing touchpad gestures including
/// swipe, pinch, and hold gestures.
pub(crate) trait GestureInput<I: InputBackend + 'static>
where
    I::Device: 'static,
{
    /// Handle swipe gesture begin.
    fn on_gesture_swipe_begin(&mut self, event: I::GestureSwipeBeginEvent);
    
    /// Handle swipe gesture update.
    fn on_gesture_swipe_update(&mut self, event: I::GestureSwipeUpdateEvent);
    
    /// Handle swipe gesture end.
    fn on_gesture_swipe_end(&mut self, event: I::GestureSwipeEndEvent);
    
    /// Handle pinch gesture begin.
    fn on_gesture_pinch_begin(&mut self, event: I::GesturePinchBeginEvent);
    
    /// Handle pinch gesture update.
    fn on_gesture_pinch_update(&mut self, event: I::GesturePinchUpdateEvent);
    
    /// Handle pinch gesture end.
    fn on_gesture_pinch_end(&mut self, event: I::GesturePinchEndEvent);
    
    /// Handle hold gesture begin.
    fn on_gesture_hold_begin(&mut self, event: I::GestureHoldBeginEvent);
    
    /// Handle hold gesture end.
    fn on_gesture_hold_end(&mut self, event: I::GestureHoldEndEvent);
}

impl<I: InputBackend + 'static> GestureInput<I> for State
where
    I::Device: 'static,
{
    fn on_gesture_swipe_begin(
        &mut self,
        event: I::GestureSwipeBeginEvent,
    ) {
        if self.niri.ui.mru.is_open() {
            // Don't start swipe gestures while in the MRU.
            return;
        }

        if event.fingers() == 3 {
            self.niri.input.set_swipe_3f(Some((0., 0.)));

            // We handled this event.
            return;
        } else if event.fingers() == 4 {
            // DEPRECATED(overview): 4-finger gesture used to open overview, now does nothing
            return;
        }

        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_swipe_begin(
            self,
            &GestureSwipeBeginEvent {
                serial,
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }

    fn on_gesture_swipe_update(&mut self, event: I::GestureSwipeUpdateEvent) {
        let mut delta_x = event.delta_x();
        let mut delta_y = event.delta_y();

        if let Some(libinput_event) =
            (&event as &dyn Any).downcast_ref::<input::event::gesture::GestureSwipeUpdateEvent>()
        {
            delta_x = libinput_event.dx_unaccelerated();
            delta_y = libinput_event.dy_unaccelerated();
        }

        let _uninverted_delta_y = delta_y;

        let device = event.device();
        if let Some(device) = (&device as &dyn Any).downcast_ref::<input::Device>() {
            if device.config_scroll_natural_scroll_enabled() {
                delta_x = -delta_x;
                delta_y = -delta_y;
            }
        }

        // Overview mode has been removed, this is always false
        let is_overview_open = false;

        if let Some((cx, cy)) = self.niri.input.swipe_3f() {
            let cx = cx;
            let cy = cy;

            // Update cumulative values
            self.niri.input.add_swipe_3f(delta_x, delta_y);

            // Check if the gesture moved far enough to decide. Threshold copied from GNOME Shell.
            if cx * cx + cy * cy >= 16. * 16. {
                self.niri.input.set_swipe_3f(None);

                if let Some(output) = self.niri.output_under_cursor() {
                    if cx.abs() > cy.abs() {
                        let output_ws = if is_overview_open {
                            self.niri.row_under_cursor(true)
                        } else {
                            // We don't want to accidentally "catch" the wrong workspace during
                            // animations.
                            // TEAM_033: active_workspace_ref() returns Option, so flatten it
                            self.niri.output_under_cursor().and_then(|output| {
                                let mon = self.niri.layout.monitor_for_output(&output)?;
                                let ws = mon.active_workspace_ref()?;
                                Some((output, ws))
                            })
                        };

                        if let Some((output, ws)) = output_ws {
                            let ws_idx = self.niri.layout.find_workspace_by_id(ws.id()).unwrap().0;
                            self.niri.layout.view_offset_gesture_begin(
                                &output,
                                Some(ws_idx as usize),
                                true,
                            );
                        }
                    } else {
                        self.niri
                            .layout
                            .workspace_switch_gesture_begin(&output, true);
                    }
                }
            }
        }

        let timestamp = Duration::from_micros(event.time());

        let mut handled = false;
        let res = self
            .niri
            .layout
            .workspace_switch_gesture_update(delta_y, timestamp, true);
        if let Some(output) = res {
            if let Some(output) = output {
                self.niri.queue_redraw(&output);
            }
            handled = true;
        }

        let res = self
            .niri
            .layout
            .view_offset_gesture_update(delta_x, timestamp, true);
        if let Some(output) = res {
            if let Some(output) = output {
                self.niri.queue_redraw(&output);
            }
            handled = true;
        }

        // DEPRECATED(overview): Removed overview_gesture_update call

        if handled {
            // We handled this event.
            return;
        }

        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_swipe_update(
            self,
            &GestureSwipeUpdateEvent {
                time: event.time_msec(),
                delta: event.delta(),
            },
        );
    }

    fn on_gesture_swipe_end(&mut self, event: I::GestureSwipeEndEvent) {
        self.niri.input.set_swipe_3f(None);

        let mut handled = false;
        let res = self.niri.layout.workspace_switch_gesture_end(Some(true));
        if let Some(output) = res {
            self.niri.queue_redraw(&output);
            handled = true;
        }

        let res = self.niri.layout.view_offset_gesture_end(Some(true));
        if let Some(output) = res {
            self.niri.queue_redraw(&output);
            handled = true;
        }

        // DEPRECATED(overview): Removed overview_gesture_end call

        if handled {
            // We handled this event.
            return;
        }

        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_swipe_end(
            self,
            &GestureSwipeEndEvent {
                serial,
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }

    fn on_gesture_pinch_begin(&mut self, event: I::GesturePinchBeginEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_pinch_begin(
            self,
            &GesturePinchBeginEvent {
                serial,
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }

    fn on_gesture_pinch_update(&mut self, event: I::GesturePinchUpdateEvent) {
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_pinch_update(
            self,
            &GesturePinchUpdateEvent {
                time: event.time_msec(),
                delta: event.delta(),
                scale: event.scale(),
                rotation: event.rotation(),
            },
        );
    }

    fn on_gesture_pinch_end(&mut self, event: I::GesturePinchEndEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_pinch_end(
            self,
            &GesturePinchEndEvent {
                serial,
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }

    fn on_gesture_hold_begin(&mut self, event: I::GestureHoldBeginEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_hold_begin(
            self,
            &GestureHoldBeginEvent {
                serial,
                time: event.time_msec(),
                fingers: event.fingers(),
            },
        );
    }

    fn on_gesture_hold_end(&mut self, event: I::GestureHoldEndEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let pointer = self.niri.seat.get_pointer().unwrap();

        if self.update_pointer_contents() {
            pointer.frame(self);
        }

        pointer.gesture_hold_end(
            self,
            &GestureHoldEndEvent {
                serial,
                time: event.time_msec(),
                cancelled: event.cancelled(),
            },
        );
    }
}
