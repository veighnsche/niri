use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::time::Duration;

use calloop::timer::{TimeoutAction, Timer};
use niri_config::{Bind, Config, Key, ModKey, Modifiers, Trigger};
use smithay::backend::input::{
    AbsolutePositionEvent, Event, InputEvent, Keycode, Switch, SwitchState, SwitchToggleEvent,
    TabletToolEvent,
};
use smithay::backend::libinput::LibinputInputBackend;
use smithay::input::keyboard::{FilterResult, Keysym, ModifiersState};
use smithay::input::pointer::GrabStartData as PointerGrabStartData;
use smithay::input::touch::GrabStartData as TouchGrabStartData;
use smithay::input::SeatHandler;
use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Transform};
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor;

use crate::niri::State;
use crate::ui::mru::WindowMruUi;
use crate::ui::screenshot_ui::ScreenshotUi;
use crate::utils::get_monotonic_time;

pub mod backend_ext;
pub mod binds;
pub mod actions;
mod device;
mod gesture;
mod helpers;
mod keyboard;
pub mod move_grab;
mod pointer;
mod tablet;
pub mod pick_color_grab;
pub mod pick_window_grab;
pub mod resize_grab;
pub mod scroll_swipe_gesture;
pub mod scroll_tracker;
pub mod spatial_movement_grab;
pub mod swipe_tracker;
mod touch;
pub mod touch_resize_grab;

// TEAM_086: Import input handler traits for explicit trait dispatch
use device::DeviceInput;
use gesture::GestureInput;
use keyboard::KeyboardInput;
use pointer::PointerInput;
use tablet::TabletInput;
use touch::TouchInput;

// TEAM_085: Re-export bind functions for backwards compatibility
pub use binds::{
    find_bind, find_configured_bind, find_configured_switch_action, mods_with_binds,
    mods_with_finger_scroll_binds, mods_with_mouse_binds, mods_with_wheel_binds,
    modifiers_from_state,
};
// TEAM_085: Re-export device functions
pub use device::apply_libinput_settings;
// TEAM_087: Import helper functions
use helpers::{
    allowed_during_screenshot, allowed_when_locked, hardcoded_overview_bind,
    should_activate_monitors, should_hide_exit_confirm_dialog, should_hide_hotkey_overlay,
    should_notify_activity, should_reset_pointer_inactivity_timer,
};
// TEAM_087: Import action handler trait
use actions::ActionHandler;

use backend_ext::{NiriInputBackend as InputBackend, NiriInputDevice as _};

pub const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabletData {
    pub aspect_ratio: f64,
}

pub enum PointerOrTouchStartData<D: SeatHandler> {
    Pointer(PointerGrabStartData<D>),
    Touch(TouchGrabStartData<D>),
}

impl<D: SeatHandler> PointerOrTouchStartData<D> {
    pub fn location(&self) -> Point<f64, Logical> {
        match self {
            PointerOrTouchStartData::Pointer(x) => x.location,
            PointerOrTouchStartData::Touch(x) => x.location,
        }
    }

    pub fn unwrap_pointer(&self) -> &PointerGrabStartData<D> {
        match self {
            PointerOrTouchStartData::Pointer(x) => x,
            PointerOrTouchStartData::Touch(_) => panic!("start_data is not Pointer"),
        }
    }

    pub fn unwrap_touch(&self) -> &TouchGrabStartData<D> {
        match self {
            PointerOrTouchStartData::Pointer(_) => panic!("start_data is not Touch"),
            PointerOrTouchStartData::Touch(x) => x,
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer(_))
    }

    pub fn is_touch(&self) -> bool {
        matches!(self, Self::Touch(_))
    }
}

impl State {
    pub fn process_input_event<I: InputBackend + 'static>(&mut self, event: InputEvent<I>)
    where
        I::Device: 'static, // Needed for downcasting.
    {
        let _span = tracy_client::span!("process_input_event");

        // Make sure some logic like workspace clean-up has a chance to run before doing actions.
        self.niri.advance_animations();

        if self.niri.outputs.monitors_active() {
            // Notify the idle-notifier of activity.
            if should_notify_activity(&event) {
                self.niri.notify_activity();
            }
        } else {
            // Power on monitors if they were off.
            if should_activate_monitors(&event) {
                self.niri.activate_monitors(&mut self.backend);

                // Notify the idle-notifier of activity only if we're also powering on the
                // monitors.
                self.niri.notify_activity();
            }
        }

        if should_reset_pointer_inactivity_timer(&event) {
            self.niri.reset_pointer_inactivity_timer();
        }

        let hide_hotkey_overlay =
            self.niri.ui.hotkey.is_open() && should_hide_hotkey_overlay(&event);

        let hide_exit_confirm_dialog =
            self.niri.ui.exit_dialog.is_open() && should_hide_exit_confirm_dialog(&event);

        let mut consumed_by_a11y = false;
        use InputEvent::*;
        match event {
            DeviceAdded { device } => DeviceInput::on_device_added(self, device),
            DeviceRemoved { device } => DeviceInput::on_device_removed(self, device),
            Keyboard { event } => {
                KeyboardInput::<I>::on_keyboard(self, event, &mut consumed_by_a11y)
            }
            PointerMotion { event } => PointerInput::<I>::on_pointer_motion(self, event),
            PointerMotionAbsolute { event } => {
                PointerInput::<I>::on_pointer_motion_absolute(self, event)
            }
            PointerButton { event } => PointerInput::<I>::on_pointer_button(self, event),
            PointerAxis { event } => PointerInput::<I>::on_pointer_axis(self, event),
            TabletToolAxis { event } => TabletInput::<I>::on_tablet_tool_axis(self, event),
            TabletToolTip { event } => TabletInput::<I>::on_tablet_tool_tip(self, event),
            TabletToolProximity { event } => TabletInput::<I>::on_tablet_tool_proximity(self, event),
            TabletToolButton { event } => TabletInput::<I>::on_tablet_tool_button(self, event),
            GestureSwipeBegin { event } => GestureInput::<I>::on_gesture_swipe_begin(self, event),
            GestureSwipeUpdate { event } => GestureInput::<I>::on_gesture_swipe_update(self, event),
            GestureSwipeEnd { event } => GestureInput::<I>::on_gesture_swipe_end(self, event),
            GesturePinchBegin { event } => GestureInput::<I>::on_gesture_pinch_begin(self, event),
            GesturePinchUpdate { event } => GestureInput::<I>::on_gesture_pinch_update(self, event),
            GesturePinchEnd { event } => GestureInput::<I>::on_gesture_pinch_end(self, event),
            GestureHoldBegin { event } => GestureInput::<I>::on_gesture_hold_begin(self, event),
            GestureHoldEnd { event } => GestureInput::<I>::on_gesture_hold_end(self, event),
            TouchDown { event } => TouchInput::<I>::on_touch_down(self, event),
            TouchMotion { event } => TouchInput::<I>::on_touch_motion(self, event),
            TouchUp { event } => TouchInput::<I>::on_touch_up(self, event),
            TouchCancel { event } => TouchInput::<I>::on_touch_cancel(self, event),
            TouchFrame { event } => TouchInput::<I>::on_touch_frame(self, event),
            SwitchToggle { event } => self.on_switch_toggle::<I>(event),
            Special(_) => (),
        }

        // Don't hide overlays if consumed by a11y, so that you can use the screen reader
        // navigation keys.
        if consumed_by_a11y {
            return;
        }

        // Do this last so that screenshot still gets it.
        if hide_hotkey_overlay && self.niri.ui.hotkey.hide() {
            self.niri.queue_redraw_all();
        }

        if hide_exit_confirm_dialog && self.niri.ui.exit_dialog.hide() {
            self.niri.queue_redraw_all();
        }
    }

    pub fn process_libinput_event(&mut self, event: &mut InputEvent<LibinputInputBackend>) {
        let _span = tracy_client::span!("process_libinput_event");

        match event {
            InputEvent::DeviceAdded { device } => {
                self.niri.devices.insert(device.clone());

                if device.has_capability(input::DeviceCapability::TabletTool) {
                    match device.size() {
                        Some((w, h)) => {
                            let aspect_ratio = w / h;
                            let data = TabletData { aspect_ratio };
                            self.niri.tablets.insert(device.clone(), data);
                        }
                        None => {
                            warn!("tablet tool device has no size");
                        }
                    }
                }

                if device.has_capability(input::DeviceCapability::Keyboard) {
                    if let Some(led_state) = self
                        .niri
                        .seat
                        .get_keyboard()
                        .map(|keyboard| keyboard.led_state())
                    {
                        device.led_update(led_state.into());
                    }
                }

                if device.has_capability(input::DeviceCapability::Touch) {
                    self.niri.touch.insert(device.clone());
                }

                apply_libinput_settings(&self.niri.config.borrow().input, device);
            }
            InputEvent::DeviceRemoved { device } => {
                self.niri.touch.remove(device);
                self.niri.tablets.remove(device);
                self.niri.devices.remove(device);
            }
            _ => (),
        }
    }

    // Device handlers available via DeviceInput trait (device.rs)

    /// Computes the rectangle that covers all outputs in global space.
    fn global_bounding_rectangle(&self) -> Option<Rectangle<i32, Logical>> {
        self.niri.outputs.space().outputs().fold(
            None,
            |acc: Option<Rectangle<i32, Logical>>, output| {
                self.niri
                    .outputs
                    .space()
                    .output_geometry(output)
                    .map(|geo| acc.map(|acc| acc.merge(geo)).unwrap_or(geo))
            },
        )
    }

    /// Computes the cursor position for the tablet event.
    ///
    /// This function handles the tablet output mapping, as well as coordinate clamping and aspect
    /// ratio correction.
    fn compute_tablet_position<I: InputBackend>(
        &self,
        event: &(impl Event<I> + TabletToolEvent<I>),
    ) -> Option<Point<f64, Logical>>
    where
        I::Device: 'static,
    {
        let device_output = event.device().output(self);
        let device_output = device_output.as_ref();
        let (target_geo, keep_ratio, px, transform) =
            if let Some(output) = device_output.or_else(|| self.niri.output_for_tablet()) {
                (
                    self.niri.outputs.space().output_geometry(output).unwrap(),
                    true,
                    1. / output.current_scale().fractional_scale(),
                    output.current_transform(),
                )
            } else {
                let geo = self.global_bounding_rectangle()?;

                // FIXME: this 1 px size should ideally somehow be computed for the rightmost output
                // corresponding to the position on the right when clamping.
                let output = self.niri.outputs.space().outputs().next().unwrap();
                let scale = output.current_scale().fractional_scale();

                // Do not keep ratio for the unified mode as this is what OpenTabletDriver expects.
                (geo, false, 1. / scale, Transform::Normal)
            };

        let mut pos = {
            let size = transform.invert().transform_size(target_geo.size);
            transform.transform_point_in(event.position_transformed(size), &size.to_f64())
        };

        if keep_ratio {
            pos.x /= target_geo.size.w as f64;
            pos.y /= target_geo.size.h as f64;

            let device = event.device();
            if let Some(device) = (&device as &dyn Any).downcast_ref::<input::Device>() {
                if let Some(data) = self.niri.tablets.get(device) {
                    // This code does the same thing as mutter with "keep aspect ratio" enabled.
                    let size = transform.invert().transform_size(target_geo.size);
                    let output_aspect_ratio = size.w as f64 / size.h as f64;
                    let ratio = data.aspect_ratio / output_aspect_ratio;

                    if ratio > 1. {
                        pos.x *= ratio;
                    } else {
                        pos.y /= ratio;
                    }
                }
            };

            pos.x *= target_geo.size.w as f64;
            pos.y *= target_geo.size.h as f64;
        }

        pos.x = pos.x.clamp(0.0, target_geo.size.w as f64 - px);
        pos.y = pos.y.clamp(0.0, target_geo.size.h as f64 - px);
        Some(pos + target_geo.loc.to_f64())
    }

    fn is_inhibiting_shortcuts(&self) -> bool {
        self.niri
            .focus
            .current
            .surface()
            .and_then(|surface| self.niri.focus.shortcut_inhibitors.get(surface))
            .is_some_and(KeyboardShortcutsInhibitor::is_active)
    }

    // Keyboard handlers available via KeyboardInput trait (keyboard.rs)

    pub fn handle_bind(&mut self, bind: Bind) {
        let Some(cooldown) = bind.cooldown else {
            self.do_action(bind.action, bind.allow_when_locked);
            return;
        };

        // Check this first so that it doesn't trigger the cooldown.
        if self.niri.is_locked() && !(bind.allow_when_locked || allowed_when_locked(&bind.action)) {
            return;
        }

        match self.niri.bind_cooldown_timers.entry(bind.key) {
            // The bind is on cooldown.
            Entry::Occupied(_) => (),
            Entry::Vacant(entry) => {
                let timer = Timer::from_duration(cooldown);
                let token = self
                    .niri
                    .event_loop
                    .insert_source(timer, move |_, _, state| {
                        if state.niri.bind_cooldown_timers.remove(&bind.key).is_none() {
                            error!("bind cooldown timer entry disappeared");
                        }
                        TimeoutAction::Drop
                    })
                    .unwrap();
                entry.insert(token);

                self.do_action(bind.action, bind.allow_when_locked);
            }
        }
    }

    // TEAM_087: do_action moved to actions.rs (ActionHandler trait)

    // TEAM_085: on_pointer_motion, on_pointer_motion_absolute, on_pointer_button,
    // on_pointer_axis moved to pointer.rs

    // TEAM_085: on_tablet_tool_* handlers moved to tablet.rs

    // TEAM_085: on_gesture_* handlers moved to gesture.rs

    fn compute_absolute_location<I: InputBackend>(
        &self,
        evt: &impl AbsolutePositionEvent<I>,
        fallback_output: Option<&Output>,
    ) -> Option<Point<f64, Logical>> {
        let output = evt.device().output(self);
        let output = output.as_ref().or(fallback_output)?;
        let output_geo = self.niri.outputs.space().output_geometry(output).unwrap();
        let transform = output.current_transform();
        let size = transform.invert().transform_size(output_geo.size);
        Some(
            transform.transform_point_in(evt.position_transformed(size), &size.to_f64())
                + output_geo.loc.to_f64(),
        )
    }

    /// Computes the cursor position for the touch event.
    ///
    /// This function handles the touch output mapping, as well as coordinate transform
    fn compute_touch_location<I: InputBackend>(
        &self,
        evt: &impl AbsolutePositionEvent<I>,
    ) -> Option<Point<f64, Logical>> {
        self.compute_absolute_location(evt, self.niri.output_for_touch())
    }

    // TEAM_085: on_touch_* handlers moved to touch.rs

    fn on_switch_toggle<I: InputBackend>(&mut self, evt: I::SwitchToggleEvent) {
        let Some(switch) = evt.switch() else {
            return;
        };

        if switch == Switch::Lid {
            let is_closed = evt.state() == SwitchState::On;
            trace!("lid switch {}", if is_closed { "closed" } else { "opened" });
            self.set_lid_closed(is_closed);
        }

        let action = {
            let bindings = &self.niri.config.borrow().switch_events;
            find_configured_switch_action(bindings, switch, evt.state())
        };

        if let Some(action) = action {
            self.do_action(action, true);
        }
    }
}

/// Check whether the key should be intercepted and mark intercepted
/// pressed keys as `suppressed`, thus preventing `releases` corresponding
/// to them from being delivered.
fn should_intercept_key<'a>(
    suppressed_keys: &mut HashSet<Keycode>,
    bindings: impl IntoIterator<Item = &'a Bind>,
    mod_key: ModKey,
    key_code: Keycode,
    modified: Keysym,
    raw: Option<Keysym>,
    pressed: bool,
    mods: ModifiersState,
    screenshot_ui: &ScreenshotUi,
    disable_power_key_handling: bool,
    is_inhibiting_shortcuts: bool,
) -> FilterResult<Option<Bind>> {
    // Actions are only triggered on presses, release of the key
    // shouldn't try to intercept anything unless we have marked
    // the key to suppress.
    if !pressed && !suppressed_keys.contains(&key_code) {
        return FilterResult::Forward;
    }

    let mut final_bind = find_bind(
        bindings,
        mod_key,
        modified,
        raw,
        mods,
        disable_power_key_handling,
    );

    // Allow only a subset of compositor actions while the screenshot UI is open, since the user
    // cannot see the screen.
    if screenshot_ui.is_open() {
        let mut use_screenshot_ui_action = true;

        if let Some(bind) = &final_bind {
            if allowed_during_screenshot(&bind.action) {
                use_screenshot_ui_action = false;
            }
        }

        if use_screenshot_ui_action {
            if let Some(raw) = raw {
                final_bind = screenshot_ui.action(raw, mods).map(|action| Bind {
                    key: Key {
                        trigger: Trigger::Keysym(raw),
                        // Not entirely correct but it doesn't matter in how we currently use it.
                        modifiers: Modifiers::empty(),
                    },
                    action,
                    repeat: true,
                    cooldown: None,
                    allow_when_locked: false,
                    // The screenshot UI owns the focus anyway, so this doesn't really matter.
                    // But logically, nothing can inhibit its actions. Only opening it can be
                    // inhibited.
                    allow_inhibiting: false,
                    hotkey_overlay_title: None,
                });
            }
        }
    }

    match (final_bind, pressed) {
        (Some(bind), true) => {
            if is_inhibiting_shortcuts && bind.allow_inhibiting {
                FilterResult::Forward
            } else {
                suppressed_keys.insert(key_code);
                FilterResult::Intercept(Some(bind))
            }
        }
        (_, false) => {
            // By this point, we know that the key was suppressed on press. Even if we're inhibiting
            // shortcuts, we should still suppress the release.
            // But we don't need to check for shortcuts inhibition here, because
            // if it was inhibited on press (forwarded to the client), it wouldn't be suppressed,
            // so the release would already have been forwarded at the start of this function.
            suppressed_keys.remove(&key_code);
            FilterResult::Intercept(None)
        }
        (None, true) => FilterResult::Forward,
    }
}

// TEAM_085: find_bind, find_configured_bind, find_configured_switch_action,
// and modifiers_from_state moved to binds.rs

// TEAM_087: should_activate_monitors, should_hide_hotkey_overlay,
// should_hide_exit_confirm_dialog, should_notify_activity,
// should_reset_pointer_inactivity_timer, allowed_when_locked,
// allowed_during_screenshot, and hardcoded_overview_bind moved to helpers.rs

// TEAM_085: apply_libinput_settings moved to device.rs

// TEAM_085: mods_with_binds, mods_with_mouse_binds, mods_with_wheel_binds,
// and mods_with_finger_scroll_binds moved to binds.rs

/// Returns an iterator over bindings.
///
/// Includes dynamically populated bindings like the MRU UI.
fn make_binds_iter<'a>(
    config: &'a Config,
    mru: &'a mut WindowMruUi,
    mods: Modifiers,
) -> impl Iterator<Item = &'a Bind> + Clone {
    // Figure out the binds to use depending on whether the MRU is enabled and/or open.
    let general_binds = (!mru.is_open()).then_some(config.binds.0.iter());
    let general_binds = general_binds.into_iter().flatten();

    let mru_binds =
        (config.recent_windows.on || mru.is_open()).then_some(config.recent_windows.binds.iter());
    let mru_binds = mru_binds.into_iter().flatten();

    let mru_open_binds = mru.is_open().then(|| mru.opened_bindings(mods));
    let mru_open_binds = mru_open_binds.into_iter().flatten();

    // General binds take precedence over the MRU binds.
    general_binds.chain(mru_binds).chain(mru_open_binds)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use niri_config::{Action, Binds};

    use super::*;
    use crate::animation::Clock;
    use crate::ui::screenshot_ui::ScreenshotUi;

    #[test]
    fn bindings_suppress_keys() {
        let close_keysym = Keysym::q;
        let bindings = Binds(vec![Bind {
            key: Key {
                trigger: Trigger::Keysym(close_keysym),
                modifiers: Modifiers::COMPOSITOR | Modifiers::CTRL,
            },
            action: Action::CloseWindow,
            repeat: true,
            cooldown: None,
            allow_when_locked: false,
            allow_inhibiting: true,
            hotkey_overlay_title: None,
        }]);

        let comp_mod = ModKey::Super;
        let mut suppressed_keys = HashSet::new();

        let clock = Clock::with_time(Duration::ZERO);
        let config = std::rc::Rc::new(std::cell::RefCell::new(niri_config::Config::default()));
        let screenshot_ui = ScreenshotUi::new(clock, config);

        let disable_power_key_handling = false;
        let is_inhibiting_shortcuts = Cell::new(false);

        // The key_code we pick is arbitrary, the only thing
        // that matters is that they are different between cases.

        let close_key_code = Keycode::from(close_keysym.raw() + 8u32);
        let close_key_event = |suppr: &mut HashSet<Keycode>, mods: ModifiersState, pressed| {
            should_intercept_key(
                suppr,
                &bindings.0,
                comp_mod,
                close_key_code,
                close_keysym,
                Some(close_keysym),
                pressed,
                mods,
                &screenshot_ui,
                disable_power_key_handling,
                is_inhibiting_shortcuts.get(),
            )
        };

        // Key event with the code which can't trigger any action.
        let none_key_event = |suppr: &mut HashSet<Keycode>, mods: ModifiersState, pressed| {
            should_intercept_key(
                suppr,
                &bindings.0,
                comp_mod,
                Keycode::from(Keysym::l.raw() + 8),
                Keysym::l,
                Some(Keysym::l),
                pressed,
                mods,
                &screenshot_ui,
                disable_power_key_handling,
                is_inhibiting_shortcuts.get(),
            )
        };

        let mut mods = ModifiersState {
            logo: true,
            ctrl: true,
            ..Default::default()
        };

        // Action press/release.

        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(
            filter,
            FilterResult::Intercept(Some(Bind {
                action: Action::CloseWindow,
                ..
            }))
        ));
        assert!(suppressed_keys.contains(&close_key_code));

        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Intercept(None)));
        assert!(suppressed_keys.is_empty());

        // Remove mod to make it for a binding.

        mods.shift = true;
        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(filter, FilterResult::Forward));

        mods.shift = false;
        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Forward));

        // Just none press/release.

        let filter = none_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(filter, FilterResult::Forward));

        let filter = none_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Forward));

        // Press action, press arbitrary, release action, release arbitrary.

        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(
            filter,
            FilterResult::Intercept(Some(Bind {
                action: Action::CloseWindow,
                ..
            }))
        ));

        let filter = none_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(filter, FilterResult::Forward));

        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Intercept(None)));

        let filter = none_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Forward));

        // Trigger and remove all mods.

        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(
            filter,
            FilterResult::Intercept(Some(Bind {
                action: Action::CloseWindow,
                ..
            }))
        ));

        mods = Default::default();
        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Intercept(None)));

        // Ensure that no keys are being suppressed.
        assert!(suppressed_keys.is_empty());

        // Now test shortcut inhibiting.

        // With inhibited shortcuts, we don't intercept our shortcut.
        is_inhibiting_shortcuts.set(true);

        mods = ModifiersState {
            logo: true,
            ctrl: true,
            ..Default::default()
        };

        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(filter, FilterResult::Forward));
        assert!(suppressed_keys.is_empty());

        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Forward));
        assert!(suppressed_keys.is_empty());

        // Toggle it off after pressing the shortcut.
        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(filter, FilterResult::Forward));
        assert!(suppressed_keys.is_empty());

        is_inhibiting_shortcuts.set(false);

        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Forward));
        assert!(suppressed_keys.is_empty());

        // Toggle it on after pressing the shortcut.
        let filter = close_key_event(&mut suppressed_keys, mods, true);
        assert!(matches!(
            filter,
            FilterResult::Intercept(Some(Bind {
                action: Action::CloseWindow,
                ..
            }))
        ));
        assert!(suppressed_keys.contains(&close_key_code));

        is_inhibiting_shortcuts.set(true);

        let filter = close_key_event(&mut suppressed_keys, mods, false);
        assert!(matches!(filter, FilterResult::Intercept(None)));
        assert!(suppressed_keys.is_empty());
    }

    #[test]
    fn comp_mod_handling() {
        let bindings = Binds(vec![
            Bind {
                key: Key {
                    trigger: Trigger::Keysym(Keysym::q),
                    modifiers: Modifiers::COMPOSITOR,
                },
                action: Action::CloseWindow,
                repeat: true,
                cooldown: None,
                allow_when_locked: false,
                allow_inhibiting: true,
                hotkey_overlay_title: None,
            },
            Bind {
                key: Key {
                    trigger: Trigger::Keysym(Keysym::h),
                    modifiers: Modifiers::SUPER,
                },
                action: Action::FocusColumnLeft,
                repeat: true,
                cooldown: None,
                allow_when_locked: false,
                allow_inhibiting: true,
                hotkey_overlay_title: None,
            },
            Bind {
                key: Key {
                    trigger: Trigger::Keysym(Keysym::j),
                    modifiers: Modifiers::empty(),
                },
                action: Action::FocusWindowDown,
                repeat: true,
                cooldown: None,
                allow_when_locked: false,
                allow_inhibiting: true,
                hotkey_overlay_title: None,
            },
            Bind {
                key: Key {
                    trigger: Trigger::Keysym(Keysym::k),
                    modifiers: Modifiers::COMPOSITOR | Modifiers::SUPER,
                },
                action: Action::FocusWindowUp,
                repeat: true,
                cooldown: None,
                allow_when_locked: false,
                allow_inhibiting: true,
                hotkey_overlay_title: None,
            },
            Bind {
                key: Key {
                    trigger: Trigger::Keysym(Keysym::l),
                    modifiers: Modifiers::SUPER | Modifiers::ALT,
                },
                action: Action::FocusColumnRight,
                repeat: true,
                cooldown: None,
                allow_when_locked: false,
                allow_inhibiting: true,
                hotkey_overlay_title: None,
            },
        ]);

        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::q),
                ModifiersState {
                    logo: true,
                    ..Default::default()
                }
            )
            .as_ref(),
            Some(&bindings.0[0])
        );
        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::q),
                ModifiersState::default(),
            ),
            None,
        );

        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::h),
                ModifiersState {
                    logo: true,
                    ..Default::default()
                }
            )
            .as_ref(),
            Some(&bindings.0[1])
        );
        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::h),
                ModifiersState::default(),
            ),
            None,
        );

        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::j),
                ModifiersState {
                    logo: true,
                    ..Default::default()
                }
            ),
            None,
        );
        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::j),
                ModifiersState::default(),
            )
            .as_ref(),
            Some(&bindings.0[2])
        );

        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::k),
                ModifiersState {
                    logo: true,
                    ..Default::default()
                }
            )
            .as_ref(),
            Some(&bindings.0[3])
        );
        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::k),
                ModifiersState::default(),
            ),
            None,
        );

        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::l),
                ModifiersState {
                    logo: true,
                    alt: true,
                    ..Default::default()
                }
            )
            .as_ref(),
            Some(&bindings.0[4])
        );
        assert_eq!(
            find_configured_bind(
                &bindings.0,
                ModKey::Super,
                Trigger::Keysym(Keysym::l),
                ModifiersState {
                    logo: true,
                    ..Default::default()
                },
            ),
            None,
        );
    }
}
