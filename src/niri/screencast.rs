//! PipeWire screencasting for the Niri compositor.
//!
//! This module handles PipeWire-based screen casting functionality.
//! Most code is gated behind `#[cfg(feature = "xdp-gnome-screencast")]`.

#[cfg(feature = "xdp-gnome-screencast")]
use std::collections::hash_map::Entry;
#[cfg(feature = "xdp-gnome-screencast")]
use std::collections::HashSet;
#[cfg(feature = "xdp-gnome-screencast")]
use std::mem;
#[cfg(feature = "xdp-gnome-screencast")]
use std::time::Duration;

#[cfg(feature = "xdp-gnome-screencast")]
use smithay::backend::renderer::gles::GlesRenderer;
#[cfg(feature = "xdp-gnome-screencast")]
use smithay::output::Output;
#[cfg(feature = "xdp-gnome-screencast")]
use smithay::utils::Scale;

#[cfg(feature = "xdp-gnome-screencast")]
use crate::dbus::mutter_screen_cast;
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::CastSizeChange;
#[cfg(feature = "xdp-gnome-screencast")]
use crate::render_helpers::RenderTarget;

use super::{CastTarget, Niri};

// =============================================================================
// Screencast Methods
// =============================================================================

impl Niri {
    /// Refreshes window rules related to cast targets.
    ///
    /// This marks windows as cast targets so they can apply appropriate window rules.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn refresh_mapped_cast_window_rules(&mut self) {
        // O(N^2) but should be fine since there aren't many casts usually.
        self.layout.with_windows_mut(|mapped, _| {
            let id = mapped.id().get();
            // Find regardless of cast.is_active.
            let value = self
                .casts
                .iter()
                .any(|cast| cast.target == (CastTarget::Window { id }));
            mapped.set_is_window_cast_target(value);
        });
    }

    /// Refreshes the output tracking for window casts.
    ///
    /// This updates which output each window is on for proper cast rendering.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn refresh_mapped_cast_outputs(&mut self) {
        let mut seen = HashSet::new();
        let mut output_changed = vec![];

        self.layout.with_windows(|mapped, output, _, _| {
            seen.insert(mapped.window.clone());

            let Some(output) = output else {
                return;
            };

            match self.mapped_cast_output.entry(mapped.window.clone()) {
                Entry::Occupied(mut entry) => {
                    if entry.get() != output {
                        entry.insert(output.clone());
                        output_changed.push((mapped.id(), output.clone()));
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(output.clone());
                }
            }
        });

        self.mapped_cast_output.retain(|win, _| seen.contains(win));

        let mut to_stop = vec![];
        for (id, out) in output_changed {
            let refresh = out.current_mode().unwrap().refresh as u32;
            let target = CastTarget::Window { id: id.get() };
            for cast in self.casts.iter_mut().filter(|cast| cast.target == target) {
                if let Err(err) = cast.set_refresh(refresh) {
                    warn!("error changing cast FPS: {err:?}");
                    to_stop.push(cast.session_id);
                };
            }
        }

        for session_id in to_stop {
            self.stop_cast(session_id);
        }
    }

    /// Stops all casts for a given target (no-op when feature disabled).
    #[cfg(not(feature = "xdp-gnome-screencast"))]
    pub fn stop_casts_for_target(&mut self, _target: CastTarget) {}

    /// Stops all casts for a given target.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn stop_casts_for_target(&mut self, target: CastTarget) {
        let _span = tracy_client::span!("Niri::stop_casts_for_target");

        // This is O(N^2) but it shouldn't be a problem I think.
        let mut saw_dynamic = false;
        let mut ids = Vec::new();
        for cast in &self.casts {
            if cast.target != target {
                continue;
            }

            if cast.dynamic_target {
                saw_dynamic = true;
                continue;
            }

            ids.push(cast.session_id);
        }

        for id in ids {
            self.stop_cast(id);
        }

        // We don't stop dynamic casts, instead we switch them to Nothing.
        if saw_dynamic {
            self.event_loop
                .insert_idle(|state| state.set_dynamic_cast_target(CastTarget::Nothing));
        }
    }

    /// Renders output casts for PipeWire screen casting.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub(super) fn render_for_screen_cast(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        target_presentation_time: Duration,
    ) {
        let _span = tracy_client::span!("Niri::render_for_screen_cast");

        let target = CastTarget::Output(output.downgrade());

        let size = output.current_mode().unwrap().size;
        let transform = output.current_transform();
        let size = transform.transform_size(size);

        let scale = Scale::from(output.current_scale().fractional_scale());

        let mut elements = None;
        let mut casts_to_stop = vec![];

        let mut casts = mem::take(&mut self.casts);
        for cast in &mut casts {
            if !cast.is_active() {
                continue;
            }

            if cast.target != target {
                continue;
            }

            match cast.ensure_size(size) {
                Ok(CastSizeChange::Ready) => (),
                Ok(CastSizeChange::Pending) => continue,
                Err(err) => {
                    warn!("error updating stream size, stopping screencast: {err:?}");
                    casts_to_stop.push(cast.session_id);
                }
            }

            if cast.check_time_and_schedule(output, target_presentation_time) {
                continue;
            }

            // FIXME: Hidden / embedded / metadata cursor
            let elements = elements.get_or_insert_with(|| {
                self.render(renderer, output, true, RenderTarget::Screencast)
            });

            if cast.dequeue_buffer_and_render(renderer, elements, size, scale) {
                cast.last_frame_time = target_presentation_time;
            }
        }
        self.casts = casts;

        for id in casts_to_stop {
            self.stop_cast(id);
        }
    }

    /// Renders window casts for PipeWire screen casting.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub(super) fn render_windows_for_screen_cast(
        &mut self,
        renderer: &mut GlesRenderer,
        output: &Output,
        target_presentation_time: Duration,
    ) {
        let _span = tracy_client::span!("Niri::render_windows_for_screen_cast");

        let scale = Scale::from(output.current_scale().fractional_scale());

        let mut casts_to_stop = vec![];

        let mut casts = mem::take(&mut self.casts);
        for cast in &mut casts {
            if !cast.is_active() {
                continue;
            }

            let CastTarget::Window { id } = cast.target else {
                continue;
            };

            let mut windows = self.layout.windows_for_output(output);
            let Some(mapped) = windows.find(|win| win.id().get() == id) else {
                continue;
            };

            let bbox = mapped
                .window
                .bbox_with_popups()
                .to_physical_precise_up(scale);

            match cast.ensure_size(bbox.size) {
                Ok(CastSizeChange::Ready) => (),
                Ok(CastSizeChange::Pending) => continue,
                Err(err) => {
                    warn!("error updating stream size, stopping screencast: {err:?}");
                    casts_to_stop.push(cast.session_id);
                }
            }

            if cast.check_time_and_schedule(output, target_presentation_time) {
                continue;
            }

            // FIXME: pointer.
            let elements: Vec<_> = mapped.render_for_screen_cast(renderer, scale).collect();

            if cast.dequeue_buffer_and_render(renderer, &elements, bbox.size, scale) {
                cast.last_frame_time = target_presentation_time;
            }
        }
        self.casts = casts;

        for id in casts_to_stop {
            self.stop_cast(id);
        }
    }

    /// Stops a specific screen cast session.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub(super) fn stop_cast(&mut self, session_id: usize) {
        let _span = tracy_client::span!("Niri::stop_cast");

        debug!(session_id, "StopCast");

        for i in (0..self.casts.len()).rev() {
            let cast = &self.casts[i];
            if cast.session_id != session_id {
                continue;
            }

            let cast = self.casts.swap_remove(i);
            if let Err(err) = cast.stream.disconnect() {
                warn!("error disconnecting stream: {err:?}");
            }
        }

        let dbus = &self.dbus.as_ref().unwrap();
        let server = dbus.conn_screen_cast.as_ref().unwrap().object_server();
        let path = format!("/org/gnome/Mutter/ScreenCast/Session/u{session_id}");
        if let Ok(iface) = server.interface::<_, mutter_screen_cast::Session>(path) {
            let _span = tracy_client::span!("invoking Session::stop");

            async_io::block_on(async move {
                iface
                    .get()
                    .stop(server.inner(), iface.signal_emitter().clone())
                    .await
            });
        }
    }
}
