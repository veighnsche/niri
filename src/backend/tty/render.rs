//! Rendering subsystem for TTY backend.
//!
//! Handles frame rendering state and debug settings.
//! The actual rendering logic remains in mod.rs due to tight coupling
//! with device management and configuration.

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::time::Duration;

use niri_config::Config;
use smithay::backend::drm::compositor::{FrameFlags, PrimaryPlaneElement};
use smithay::backend::drm::DrmNode;
use smithay::output::Output;
use smithay::reexports::drm::control::crtc;
use smithay::reexports::calloop::timer::{Timer, TimeoutAction};

use smithay::backend::renderer::DebugFlags;
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use smithay::wayland::presentation::Refresh;
use smithay::reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback;
use tracing::{error, warn, trace};
#[cfg(feature = "profile-with-tracy")]
use tracy_client;
use super::devices::DeviceManager;
use super::types::{TtyOutputState, TtyRenderer};
use crate::backend::RenderResult;
use crate::niri::{Niri, RedrawState};
use crate::render_helpers::debug::draw_damage;
use crate::render_helpers::RenderTarget;
use crate::utils::get_monotonic_time;

/// Rendering subsystem.
///
/// Manages render-related state:
/// - Debug tint toggle
/// - Future: render statistics, frame pacing hints
pub struct RenderManager {
    /// Whether debug tinting is enabled.
    debug_tint: bool,
}

impl RenderManager {
    /// Create a new RenderManager.
    pub fn new() -> Self {
        Self { debug_tint: false }
    }

    /// Check if debug tint is enabled.
    pub fn debug_tint(&self) -> bool {
        self.debug_tint
    }

    /// Toggle debug tint on all surfaces.
    pub fn toggle_debug_tint(&mut self, devices: &mut DeviceManager) {
        self.debug_tint = !self.debug_tint;

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                let compositor = &mut surface.compositor;

                let mut flags = compositor.debug_flags();
                flags.set(DebugFlags::TINT, self.debug_tint);
                compositor.set_debug_flags(flags);
            }
        }
    }

    /// Set debug tint to a specific value on all surfaces.
    pub fn set_debug_tint(&mut self, enabled: bool, devices: &mut DeviceManager) {
        if self.debug_tint == enabled {
            return;
        }
        self.debug_tint = enabled;

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                let compositor = &mut surface.compositor;

                let mut flags = compositor.debug_flags();
                flags.set(DebugFlags::TINT, self.debug_tint);
                compositor.set_debug_flags(flags);
            }
        }
    }

    /// Render a frame for the given output.
    ///
    /// This is the core frame rendering logic that handles:
    /// - Renderer creation and management
    /// - Element rendering from niri
    /// - DRM compositor frame submission
    /// - Presentation feedback and frame callbacks
    /// - VBlank timer management
    pub fn render(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
        config: &Rc<RefCell<Config>>,
    ) -> RenderResult {
        let span = tracy_client::span!("RenderManager::render");

        let mut rv = RenderResult::Skipped;

        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
        let node = tty_state.node;
        let crtc = tty_state.crtc;

        // Extract values we need before getting mutable device reference.
        let primary_render_node = devices.primary_render_node();

        // First pass: extract values needed for renderer creation.
        let (device_render_node, surface_format, connector_name, drm_active) = {
            let Some(device) = devices.get(&node) else {
                error!("missing output device");
                return rv;
            };

            let Some(surface) = device.surfaces.get(&crtc) else {
                error!("missing surface");
                return rv;
            };

            (
                device.render_node,
                surface.compositor.format(),
                surface.name.connector.clone(),
                device.drm.is_active(),
            )
        };

        span.emit_text(&connector_name);

        if !drm_active {
            // This branch hits any time we try to render while the user had switched to a
            // different VT, so don't print anything here.
            return rv;
        }

        // Use split borrowing to get both gpu_manager and devices simultaneously.
        let (gpu_manager, devices) = devices.gpu_manager_and_devices_mut();

        let mut renderer = match gpu_manager.renderer(
            &primary_render_node,
            &device_render_node.unwrap_or(primary_render_node),
            surface_format,
        ) {
            Ok(renderer) => renderer,
            Err(err) => {
                warn!("error creating renderer for primary GPU: {err:?}");
                return rv;
            }
        };

        // Now we can access devices through the split borrow.
        let device = devices.get_mut(&node).unwrap();
        let surface = device.surfaces.get_mut(&crtc).unwrap();

        // Render the elements.
        let mut elements =
            niri.render::<TtyRenderer>(&mut renderer, output, true, RenderTarget::Output);

        // Visualize the damage, if enabled.
        if niri.debug_draw_damage {
            let output_state = niri.outputs.state_mut(output).unwrap();
            draw_damage(&mut output_state.debug_damage_tracker, &mut elements);
        }

        // Overlay planes are disabled by default as they cause weird performance issues on my
        // system.
        let flags = {
            let debug = &config.borrow().debug;

            let primary_scanout_flag = if debug.restrict_primary_scanout_to_matching_format {
                FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT
            } else {
                FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT_ANY
            };
            let mut flags = primary_scanout_flag | FrameFlags::ALLOW_CURSOR_PLANE_SCANOUT;

            if debug.enable_overlay_planes {
                flags.insert(FrameFlags::ALLOW_OVERLAY_PLANE_SCANOUT);
            }
            if debug.disable_direct_scanout {
                flags.remove(primary_scanout_flag);
                flags.remove(FrameFlags::ALLOW_OVERLAY_PLANE_SCANOUT);
            }
            if debug.disable_cursor_plane {
                flags.remove(FrameFlags::ALLOW_CURSOR_PLANE_SCANOUT);
            }
            if debug.skip_cursor_only_updates_during_vrr {
                let output_state = niri.outputs.state(output).unwrap();
                if output_state.frame_clock.vrr() {
                    flags.insert(FrameFlags::SKIP_CURSOR_ONLY_UPDATES);
                }
            }

            flags
        };

        // Hand them over to the DRM.
        let drm_compositor = &mut surface.compositor;
        match drm_compositor.render_frame::<_, _>(&mut renderer, &elements, [0.; 4], flags) {
            Ok(res) => {
                let needs_sync = res.needs_sync()
                    || config
                        .borrow()
                        .debug
                        .wait_for_frame_completion_before_queueing;
                if needs_sync {
                    if let PrimaryPlaneElement::Swapchain(element) = res.primary_element {
                        let _span = tracy_client::span!("wait for completion");
                        if let Err(err) = element.sync.wait() {
                            warn!("error waiting for frame completion: {err:?}");
                        }
                    }
                }

                niri.update_primary_scanout_output(output, &res.states);
                if let Some(dmabuf_feedback) = surface.dmabuf_feedback.as_ref() {
                    niri.send_dmabuf_feedbacks(output, dmabuf_feedback, &res.states);
                }

                if !res.is_empty {
                    let presentation_feedbacks =
                        niri.take_presentation_feedbacks(output, &res.states);
                    let data = (presentation_feedbacks, target_presentation_time);

                    match drm_compositor.queue_frame(data) {
                        Ok(()) => {
                            let output_state = niri.outputs.state_mut(output).unwrap();
                            let new_state = RedrawState::WaitingForVBlank {
                                redraw_needed: false,
                            };
                            match mem::replace(&mut output_state.redraw_state, new_state) {
                                RedrawState::Idle => unreachable!(),
                                RedrawState::Queued => (),
                                RedrawState::WaitingForVBlank { .. } => unreachable!(),
                                RedrawState::WaitingForEstimatedVBlank(_) => unreachable!(),
                                RedrawState::WaitingForEstimatedVBlankAndQueued(token) => {
                                    niri.event_loop.remove(token);
                                }
                            };

                            // We queued this frame successfully, so the current client buffers were
                            // latched. We can send frame callbacks now, since a new client commit
                            // will no longer overwrite this frame and will wait for a VBlank.
                            output_state.frame_callback_sequence =
                                output_state.frame_callback_sequence.wrapping_add(1);

                            return RenderResult::Submitted;
                        }
                        Err(err) => {
                            warn!("error queueing frame: {err}");
                        }
                    }
                } else {
                    rv = RenderResult::NoDamage;
                }
            }
            Err(err) => {
                // Can fail if we switched to a different TTY.
                warn!("error rendering frame: {err}");
            }
        }

        // We're not expecting a vblank right after this.
        drop(surface.vblank_frame.take());

        // Queue a timer to fire at the predicted vblank time.
        Self::queue_estimated_vblank_timer(niri, output.clone(), target_presentation_time);

        rv
    }

    /// Queue a timer to fire at the predicted vblank time.
    ///
    /// This is called when frame submission fails or when there's no damage,
    /// to ensure the render loop continues at the appropriate refresh rate.
    fn queue_estimated_vblank_timer(
        niri: &mut Niri,
        output: Output,
        target_presentation_time: Duration,
    ) {
        let output_state = niri.outputs.state_mut(&output).unwrap();
        match mem::take(&mut output_state.redraw_state) {
            RedrawState::Idle => unreachable!(),
            RedrawState::Queued => (),
            RedrawState::WaitingForVBlank { .. } => unreachable!(),
            RedrawState::WaitingForEstimatedVBlank(token)
            | RedrawState::WaitingForEstimatedVBlankAndQueued(token) => {
                output_state.redraw_state = RedrawState::WaitingForEstimatedVBlank(token);
                return;
            }
        }

        let now = get_monotonic_time();
        let mut duration = target_presentation_time.saturating_sub(now);

        // No use setting a zero timer, since we'll send frame callbacks anyway right after the call to
        // render(). This can happen for example with unknown presentation time from DRM.
        if duration.is_zero() {
            duration += output_state
                .frame_clock
                .refresh_interval()
                // Unknown refresh interval, i.e. winit backend. Would be good to estimate it somehow
                // but it's not that important for this code path.
                .unwrap_or(Duration::from_micros(16_667));
        }

        trace!("queueing estimated vblank timer to fire in {duration:?}");

        let timer = Timer::from_duration(duration);
        let token = niri
            .event_loop
            .insert_source(timer, move |_, _, data| {
                data.backend
                    .tty()
                    .render
                    .on_estimated_vblank_timer(&mut data.niri, output.clone());
                TimeoutAction::Drop
            })
            .unwrap();
        output_state.redraw_state = RedrawState::WaitingForEstimatedVBlank(token);
    }

    /// Handle estimated vblank timer events.
    ///
    /// This method is called when the estimated vblank timer fires,
    /// and handles frame callback sequence updates and redraw state transitions.
    pub fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output) {
        let span = tracy_client::span!("RenderManager::on_estimated_vblank_timer");

        let name = output.name();
        span.emit_text(&name);

        let Some(output_state) = niri.outputs.state_mut(&output) else {
            error!("missing output state for {name}");
            return;
        };

        // We waited for the timer, now we can send frame callbacks again.
        output_state.frame_callback_sequence = output_state.frame_callback_sequence.wrapping_add(1);

        match mem::replace(&mut output_state.redraw_state, RedrawState::Idle) {
            RedrawState::Idle => unreachable!(),
            RedrawState::Queued => unreachable!(),
            RedrawState::WaitingForVBlank { .. } => unreachable!(),
            RedrawState::WaitingForEstimatedVBlank(_) => (),
            // The timer fired just in front of a redraw.
            RedrawState::WaitingForEstimatedVBlankAndQueued(_) => {
                output_state.redraw_state = RedrawState::Queued;
                return;
            }
        }

        if output_state.unfinished_animations_remain {
            niri.queue_redraw(&output);
        } else {
            niri.send_frame_callbacks(&output);
        }
    }

    /// Handle DRM vblank events and presentation timing.
    ///
    /// This method processes vblank events from the DRM subsystem and:
    /// - Updates frame clock with presentation time
    /// - Handles presentation feedback for Wayland protocol
    /// - Manages redraw state machine
    /// - Queues next redraw if animations remain
    /// - Handles VBlank throttling for performance
    pub fn on_vblank(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        meta: DrmEventMetadata,
        config: &Rc<RefCell<Config>>,
    ) {
        let span = tracy_client::span!("RenderManager::on_vblank");

        let now = get_monotonic_time();

        let Some(device) = devices.get_mut(&node) else {
            // I've seen it happen.
            error!("missing device in vblank callback for crtc {crtc:?}");
            return;
        };

        let Some(surface) = device.surfaces.get_mut(&crtc) else {
            error!("missing surface in vblank callback for crtc {crtc:?}");
            return;
        };

        // Finish the Tracy frame, if any.
        drop(surface.vblank_frame.take());

        let name = &surface.name.connector;
        trace!("vblank on {name} {meta:?}");
        span.emit_text(name);

        let presentation_time = match meta.time {
            DrmEventTime::Monotonic(time) => time,
            DrmEventTime::Realtime(_) => {
                // Not supported.

                // This value will be ignored in the frame clock code.
                Duration::ZERO
            }
        };
        let presentation_time = if config.borrow().debug.emulate_zero_presentation_time {
            Duration::ZERO
        } else {
            presentation_time
        };

        let message = if presentation_time.is_zero() {
            format!("vblank on {name}, presentation time unknown")
        } else if presentation_time > now {
            let diff = presentation_time - now;
            tracy_client::Client::running().unwrap().plot(
                surface.time_since_presentation_plot_name,
                -diff.as_secs_f64() * 1000.,
            );
            format!("vblank on {name}, presentation is {diff:?} later")
        } else {
            let diff = now - presentation_time;
            tracy_client::Client::running().unwrap().plot(
                surface.time_since_presentation_plot_name,
                diff.as_secs_f64() * 1000.,
            );
            format!("vblank on {name}, presentation was {diff:?} ago")
        };
        tracy_client::Client::running()
            .unwrap()
            .message(&message, 0);

        let Some(output) = niri
            .outputs
            .space()
            .outputs()
            .find(|output| {
                let tty_state: &TtyOutputState = output.user_data().get().unwrap();
                tty_state.node == node && tty_state.crtc == crtc
            })
            .cloned()
        else {
            error!("missing output in global space for {name}");
            return;
        };

        let Some(output_state) = niri.outputs.state_mut(&output) else {
            error!("missing output state for {name}");
            return;
        };

        let refresh_interval = output_state.frame_clock.refresh_interval();

        let time = if presentation_time.is_zero() {
            now
        } else {
            presentation_time
        };

        // Note: The vblank_throttle callback is handled by the caller (Tty) to avoid circular dependency.
        // The caller will check if throttling is needed and call this method again with adjusted metadata.
        // For now, we'll proceed with the vblank handling assuming the caller has handled throttling.
        let redraw_needed = match mem::replace(&mut output_state.redraw_state, RedrawState::Idle) {
            RedrawState::WaitingForVBlank { redraw_needed } => redraw_needed,
            state @ (RedrawState::Idle
            | RedrawState::Queued
            | RedrawState::WaitingForEstimatedVBlank(_)
            | RedrawState::WaitingForEstimatedVBlankAndQueued(_)) => {
                // This is an error!() because it shouldn't happen, but on some systems it somehow
                // does. Kernel sending rogue vblank events?
                //
                // https://github.com/YaLTeR/niri/issues/556
                // https://github.com/YaLTeR/niri/issues/615
                error!(
                    "unexpected redraw state for output {name} (should be WaitingForVBlank); \
                     can happen when resuming from sleep or powering on monitors: {state:?}"
                );
                true
            }
        };

        // Mark the last frame as submitted.
        match surface.compositor.frame_submitted() {
            Ok(Some((mut feedback, target_presentation_time))) => {
                let refresh = match refresh_interval {
                    Some(refresh) => {
                        if output_state.frame_clock.vrr() {
                            Refresh::Variable(refresh)
                        } else {
                            Refresh::Fixed(refresh)
                        }
                    }
                    None => Refresh::Unknown,
                };

                // FIXME: ideally should be monotonically increasing for a surface.
                let seq = meta.sequence as u64;
                let mut flags = wp_presentation_feedback::Kind::Vsync
                    | wp_presentation_feedback::Kind::HwCompletion;

                if !presentation_time.is_zero() {
                    flags.insert(wp_presentation_feedback::Kind::HwClock);
                }

                feedback.presented::<_, smithay::utils::Monotonic>(time, refresh, seq, flags);

                if !presentation_time.is_zero() {
                    let misprediction_s =
                        presentation_time.as_secs_f64() - target_presentation_time.as_secs_f64();
                    tracy_client::Client::running().unwrap().plot(
                        surface.presentation_misprediction_plot_name,
                        misprediction_s * 1000.,
                    );
                }
            }
            Ok(None) => (),
            Err(err) => {
                warn!("error marking frame as submitted: {err}");
            }
        }

        if let Some(last_sequence) = output_state.last_drm_sequence {
            let delta = meta.sequence as f64 - last_sequence as f64;
            tracy_client::Client::running()
                .unwrap()
                .plot(surface.sequence_delta_plot_name, delta);
        }
        output_state.last_drm_sequence = Some(meta.sequence);

        output_state.frame_clock.presented(presentation_time);

        if redraw_needed || output_state.unfinished_animations_remain {
            let vblank_frame = tracy_client::Client::running()
                .unwrap()
                .non_continuous_frame(surface.vblank_frame_name);
            surface.vblank_frame = Some(vblank_frame);

            niri.queue_redraw(&output);
        } else {
            niri.send_frame_callbacks(&output);
        }
    }
}

impl Default for RenderManager {
    fn default() -> Self {
        Self::new()
    }
}
