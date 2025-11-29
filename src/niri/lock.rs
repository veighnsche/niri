//! Session lock management for the Niri compositor.
//!
//! This module handles the ext-session-lock protocol for screen locking.

use std::mem;
use std::time::Duration;

use calloop::timer::{TimeoutAction, Timer};
use smithay::input::pointer::CursorImageStatus;
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Resource;
use smithay::wayland::session_lock::{LockSurface, SessionLocker};

use crate::utils::is_mapped;

use super::{LockState, Niri};

// =============================================================================
// Lock Query Methods
// =============================================================================

impl Niri {
    /// Returns whether the session is locked.
    pub fn is_locked(&self) -> bool {
        match self.lock_state {
            LockState::Unlocked | LockState::WaitingForSurfaces { .. } => false,
            LockState::Locking(_) | LockState::Locked(_) => true,
        }
    }

    /// Returns the lock surface to focus.
    pub fn lock_surface_focus(&self) -> Option<WlSurface> {
        let output_under_cursor = self.output_under_cursor();
        let output = output_under_cursor
            .as_ref()
            .or_else(|| self.layout.active_output())
            .or_else(|| self.outputs.space().outputs().next())?;

        let state = self.outputs.state(output)?;
        state.lock_surface.as_ref().map(|s| s.wl_surface()).cloned()
    }
}

// =============================================================================
// Lock Mutation Methods
// =============================================================================

impl Niri {
    /// Locks the session.
    pub fn lock(&mut self, confirmation: SessionLocker) {
        // Check if another client is in the process of locking.
        if matches!(
            self.lock_state,
            LockState::WaitingForSurfaces { .. } | LockState::Locking(_)
        ) {
            info!("refusing lock as another client is currently locking");
            return;
        }

        // Check if we're already locked with an active client.
        if let LockState::Locked(lock) = &self.lock_state {
            if lock.is_alive() {
                info!("refusing lock as already locked with an active client");
                return;
            }

            // If the client had died, continue with the new lock.
            info!("locking session (replacing existing dead lock)");

            // Since the session was already locked, we know that the outputs are blanked, and
            // can lock right away.
            let lock = confirmation.ext_session_lock().clone();
            confirmation.lock();
            self.lock_state = LockState::Locked(lock);

            return;
        }

        info!("locking session");

        if self.outputs.state_count() == 0 {
            // There are no outputs, lock the session right away.
            self.ui.screenshot.close();
            self.cursor.manager
                .set_cursor_image(CursorImageStatus::default_named());

            let lock = confirmation.ext_session_lock().clone();
            confirmation.lock();
            self.lock_state = LockState::Locked(lock);
        } else {
            // There are outputs which we need to redraw before locking. But before we do that,
            // let's wait for the lock surfaces.
            //
            // Give them a second; swaylock can take its time to paint a big enough image.
            let timer = Timer::from_duration(Duration::from_millis(1000));
            let deadline_token = self
                .event_loop
                .insert_source(timer, |_, _, state| {
                    trace!("lock deadline expired, continuing");
                    state.niri.continue_to_locking();
                    TimeoutAction::Drop
                })
                .unwrap();

            self.lock_state = LockState::WaitingForSurfaces {
                confirmation,
                deadline_token,
            };
        }
    }

    /// Checks if we should continue to locking (all lock surfaces are ready).
    pub fn maybe_continue_to_locking(&mut self) {
        if !matches!(self.lock_state, LockState::WaitingForSurfaces { .. }) {
            // Not waiting.
            return;
        }

        // Check if there are any outputs whose lock surfaces had not had a commit yet.
        for state in self.outputs.states() {
            let Some(surface) = &state.lock_surface else {
                // Surface not created yet.
                return;
            };

            if !is_mapped(surface.wl_surface()) {
                return;
            }
        }

        // All good.
        trace!("lock surfaces are ready, continuing");
        self.continue_to_locking();
    }

    /// Continues the locking process after surfaces are ready or timeout.
    pub(super) fn continue_to_locking(&mut self) {
        match mem::take(&mut self.lock_state) {
            LockState::WaitingForSurfaces {
                confirmation,
                deadline_token,
            } => {
                self.event_loop.remove(deadline_token);

                self.ui.screenshot.close();
                self.cursor.manager
                    .set_cursor_image(CursorImageStatus::default_named());
                self.cancel_mru();

                if self.outputs.state_count() == 0 {
                    // There are no outputs, lock the session right away.
                    let lock = confirmation.ext_session_lock().clone();
                    confirmation.lock();
                    self.lock_state = LockState::Locked(lock);
                } else {
                    // There are outputs which we need to redraw before locking.
                    self.lock_state = LockState::Locking(confirmation);
                    self.queue_redraw_all();
                }
            }
            other => {
                error!("continue_to_locking() called with wrong lock state: {other:?}",);
                self.lock_state = other;
            }
        }
    }

    /// Unlocks the session.
    pub fn unlock(&mut self) {
        info!("unlocking session");

        let prev = mem::take(&mut self.lock_state);
        if let LockState::WaitingForSurfaces { deadline_token, .. } = prev {
            self.event_loop.remove(deadline_token);
        }

        for output_state in self.outputs.states_mut() {
            output_state.lock_surface = None;
        }
        self.queue_redraw_all();
    }

    /// Updates the logind LockedHint.
    #[cfg(feature = "dbus")]
    pub(super) fn update_locked_hint(&mut self) {
        use std::sync::LazyLock;
        use std::thread;

        if !self.is_session_instance {
            return;
        }

        static XDG_SESSION_ID: LazyLock<Option<String>> = LazyLock::new(|| {
            let id = std::env::var("XDG_SESSION_ID").ok();
            if id.is_none() {
                warn!(
                    "env var 'XDG_SESSION_ID' is unset or invalid; logind LockedHint won't be set"
                );
            }
            id
        });

        let Some(session_id) = &*XDG_SESSION_ID else {
            return;
        };

        fn call(session_id: &str, locked: bool) -> anyhow::Result<()> {
            use anyhow::Context;

            let conn = zbus::blocking::Connection::system()
                .context("error connecting to the system bus")?;

            let message = conn
                .call_method(
                    Some("org.freedesktop.login1"),
                    "/org/freedesktop/login1",
                    Some("org.freedesktop.login1.Manager"),
                    "GetSession",
                    &(session_id),
                )
                .context("failed to call GetSession")?;

            let message_body = message.body();
            let session_path: zbus::zvariant::ObjectPath = message_body
                .deserialize()
                .context("failed to deserialize GetSession reply")?;

            conn.call_method(
                Some("org.freedesktop.login1"),
                session_path,
                Some("org.freedesktop.login1.Session"),
                "SetLockedHint",
                &(locked),
            )
            .context("failed to call SetLockedHint")?;

            Ok(())
        }

        // Consider only the fully locked state here. When using the locked hint with sleep
        // inhibitor tools, we want to allow sleep only after the screens are fully cleared with
        // the lock screen, which corresponds to the Locked state.
        let locked = matches!(self.lock_state, LockState::Locked(_));

        if self.locked_hint.is_some_and(|h| h == locked) {
            return;
        }

        self.locked_hint = Some(locked);

        let res = thread::Builder::new()
            .name("Logind LockedHint Updater".to_owned())
            .spawn(move || {
                let _span = tracy_client::span!("LockedHint");

                if let Err(err) = call(session_id, locked) {
                    warn!("failed to set logind LockedHint: {err:?}");
                }
            });

        if let Err(err) = res {
            warn!("error spawning a thread to set logind LockedHint: {err:?}");
        }
    }

    /// Adds a new lock surface for an output.
    pub fn new_lock_surface(&mut self, surface: LockSurface, output: &Output) {
        let lock = match &self.lock_state {
            LockState::Unlocked => {
                error!("tried to add a lock surface on an unlocked session");
                return;
            }
            LockState::WaitingForSurfaces { confirmation, .. } => confirmation.ext_session_lock(),
            LockState::Locking(confirmation) => confirmation.ext_session_lock(),
            LockState::Locked(lock) => lock,
        };

        if lock.client() != surface.wl_surface().client() {
            debug!("ignoring lock surface from an unrelated client");
            return;
        }

        let Some(output_state) = self.outputs.state_mut(output) else {
            error!("missing output state");
            return;
        };

        output_state.lock_surface = Some(surface);
    }
}
