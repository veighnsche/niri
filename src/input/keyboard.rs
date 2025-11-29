//! Keyboard input event handling.

use std::time::Duration;

use smithay::backend::input::{Event, KeyState, KeyboardKeyEvent};
use smithay::input::keyboard::{FilterResult, Keysym};
use smithay::utils::SERIAL_COUNTER;

use super::actions::ActionHandler;
use super::backend_ext::NiriInputBackend as InputBackend;
use super::{
    hardcoded_overview_bind, make_binds_iter, modifiers_from_state, should_intercept_key,
};
use crate::niri::{PointerVisibility, State};
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_a11y::KbMonBlock;
use niri_config::{Action, Bind};

// TEAM_086: Trait-based keyboard input handling (replaces pub(super) pattern)

/// Trait for keyboard input event handling.
///
/// This trait defines the interface for processing keyboard events.
/// Implemented by `State` to handle keyboard input in a modular way.
pub(crate) trait KeyboardInput<I: InputBackend> {
    /// Handle a keyboard key event.
    fn on_keyboard(&mut self, event: I::KeyboardKeyEvent, consumed_by_a11y: &mut bool);
}

impl<I: InputBackend> KeyboardInput<I> for State {
    fn on_keyboard(
        &mut self,
        event: I::KeyboardKeyEvent,
        consumed_by_a11y: &mut bool,
    ) {
        let mod_key = self.backend.mod_key(&self.niri.config.borrow());

        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(&event);
        let pressed = event.state() == KeyState::Pressed;

        // Stop bind key repeat on any release. This won't work 100% correctly in cases like:
        // 1. Press Mod
        // 2. Press Left (repeat starts)
        // 3. Press PgDown (new repeat starts)
        // 4. Release Left (PgDown repeat stops)
        // But it's good enough for now.
        // FIXME: handle this properly.
        if !pressed {
            if let Some(token) = self.niri.bind_repeat_timer.take() {
                self.niri.event_loop.remove(token);
            }
        }

        if pressed {
            self.hide_cursor_if_needed();
        }

        let is_inhibiting_shortcuts = self.is_inhibiting_shortcuts();

        // Accessibility modifier grabs should override XKB state changes (e.g. Caps Lock), so we
        // need to process them before keyboard.input() below.
        //
        // Other accessibility-grabbed keys should still update our XKB state, but not cause any
        // other changes.
        #[cfg(feature = "dbus")]
        let block = {
            let block = self.a11y_process_key(
                Duration::from_millis(u64::from(time)),
                event.key_code(),
                event.state(),
            );
            if block != KbMonBlock::Pass {
                *consumed_by_a11y = true;
            }
            // The accessibility modifier first press must not change XKB state, so we return
            // early here.
            if block == KbMonBlock::ModifierFirstPress {
                return;
            }
            block
        };
        #[cfg(not(feature = "dbus"))]
        let _ = consumed_by_a11y;

        let Some(Some(bind)) = self.niri.seat.get_keyboard().unwrap().input(
            self,
            event.key_code(),
            event.state(),
            serial,
            time,
            |this, mods, keysym| {
                let key_code = event.key_code();
                let modified = keysym.modified_sym();
                let raw = keysym.raw_latin_sym_or_raw_current_sym();
                let modifiers = modifiers_from_state(*mods);

                // After updating XKB state from accessibility-grabbed keys, return right away and
                // don't handle them.
                #[cfg(feature = "dbus")]
                if block != KbMonBlock::Pass {
                    // HACK: there's a slight problem with this code. Here we filter out keys
                    // consumed by accessibility from getting sent to the Wayland client. However,
                    // the Wayland client can still receive these keys from the wl_keyboard
                    // enter/modifiers events. In particular, this can easily happen when opening
                    // the Orca actions menu with Orca + Shift + A: in most cases, when this menu
                    // opens, Shift is still held down, so the menu receives it in
                    // wl_keyboard.enter/modifiers. Then the menu won't react to Enter presses
                    // until the user taps Shift again to "release" it (since the initial Shift
                    // release will be intercepted here).
                    //
                    // I don't think there's any good way of dealing with this apart from keeping a
                    // separate xkb state for accessibility, so that we can track the pressed
                    // modifiers without accidentally leaking them to wl_keyboard.enter. So for now
                    // let's forward modifier releases to the clients here to deal with the most
                    // common case.
                    if !pressed
                        && matches!(
                            modified,
                            Keysym::Shift_L
                                | Keysym::Shift_R
                                | Keysym::Control_L
                                | Keysym::Control_R
                                | Keysym::Super_L
                                | Keysym::Super_R
                                | Keysym::Alt_L
                                | Keysym::Alt_R
                        )
                    {
                        return FilterResult::Forward;
                    } else {
                        return FilterResult::Intercept(None);
                    }
                }

                if this.niri.ui.exit_dialog.is_open() && pressed {
                    if raw == Some(Keysym::Return) {
                        info!("quitting after confirming exit dialog");
                        this.niri.stop_signal.stop();
                    }

                    // Don't send this press to any clients.
                    this.niri.suppressed_keys.insert(key_code);
                    return FilterResult::Intercept(None);
                }

                // Check if all modifiers were released while the MRU UI was open. If so, close the
                // UI (which will also transfer the focus to the current MRU UI selection).
                if this.niri.ui.mru.is_open() && !pressed && modifiers.is_empty() {
                    this.do_action(Action::MruConfirm, false);

                    if this.niri.suppressed_keys.remove(&key_code) {
                        return FilterResult::Intercept(None);
                    } else {
                        return FilterResult::Forward;
                    }
                }

                if pressed
                    && raw == Some(Keysym::Escape)
                    && (this.niri.ui.pick_window.is_some() || this.niri.ui.pick_color.is_some())
                {
                    // We window picking state so the pick window grab must be active.
                    // Unsetting it cancels window picking.
                    this.niri
                        .seat
                        .get_pointer()
                        .unwrap()
                        .unset_grab(this, serial, time);
                    this.niri.suppressed_keys.insert(key_code);
                    return FilterResult::Intercept(None);
                }

                if let Some(Keysym::space) = raw {
                    this.niri.ui.screenshot.set_space_down(pressed);
                }

                let res = {
                    let config = this.niri.config.borrow();
                    let bindings = make_binds_iter(&config, &mut this.niri.ui.mru, modifiers);

                    should_intercept_key(
                        &mut this.niri.suppressed_keys,
                        bindings,
                        mod_key,
                        key_code,
                        modified,
                        raw,
                        pressed,
                        *mods,
                        &this.niri.ui.screenshot,
                        this.niri.config.borrow().input.disable_power_key_handling,
                        is_inhibiting_shortcuts,
                    )
                };

                if matches!(res, FilterResult::Forward) {
                    // If we didn't find any bind, try other hardcoded keys.
                    if this.niri.focus.current.is_overview() && pressed {
                        if let Some(bind) = raw.and_then(|raw| hardcoded_overview_bind(raw, *mods))
                        {
                            this.niri.suppressed_keys.insert(key_code);
                            return FilterResult::Intercept(Some(bind));
                        }
                    }

                    // Interaction with the active window, immediately update the active window's
                    // focus timestamp without waiting for a possible pending MRU lock-in delay.
                    this.niri.mru_apply_keyboard_commit();
                }

                res
            },
        ) else {
            return;
        };

        if !pressed {
            return;
        }

        self.handle_bind(bind.clone());

        self.start_key_repeat(bind);
    }
}

// Private helper methods that don't depend on the InputBackend type parameter
impl State {
    fn start_key_repeat(&mut self, bind: Bind) {
        use calloop::timer::{TimeoutAction, Timer};

        if !bind.repeat {
            return;
        }

        // Stop the previous key repeat if any.
        if let Some(token) = self.niri.bind_repeat_timer.take() {
            self.niri.event_loop.remove(token);
        }

        let config = self.niri.config.borrow();
        let config = &config.input.keyboard;

        let repeat_rate = config.repeat_rate;
        if repeat_rate == 0 {
            return;
        }
        let repeat_duration = Duration::from_secs_f64(1. / f64::from(repeat_rate));

        let repeat_timer =
            Timer::from_duration(Duration::from_millis(u64::from(config.repeat_delay)));

        let token = self
            .niri
            .event_loop
            .insert_source(repeat_timer, move |_, _, state| {
                state.handle_bind(bind.clone());
                TimeoutAction::ToDuration(repeat_duration)
            })
            .unwrap();

        self.niri.bind_repeat_timer = Some(token);
    }

    fn hide_cursor_if_needed(&mut self) {
        // If the pointer is already invisible, don't reset it back to Hidden causing one frame
        // of hover.
        if !self.niri.cursor.visibility.is_visible() {
            return;
        }

        if !self.niri.config.borrow().cursor.hide_when_typing {
            return;
        }

        // niri keeps this set only while actively using a tablet, which means the cursor position
        // is likely to change almost immediately, causing pointer_visibility to just flicker back
        // and forth.
        if self.niri.cursor.tablet_location.is_some() {
            return;
        }

        self.niri.cursor.visibility = PointerVisibility::Hidden;
        self.niri.queue_redraw_all();
    }
}
