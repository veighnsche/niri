//! Niri compositor state.
//!
//! This module contains the main `Niri` struct and related types.

pub mod config;
mod frame_callbacks;
mod hit_test;
mod init;
mod lock;
mod mru;
mod output;
mod pointer;
mod protocols;
mod render;
mod rules;
mod screencast;
mod screencopy;
mod screenshot;
mod subsystems;
mod types;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, mem, thread};

use anyhow::Context;
use calloop::futures::Scheduler;
pub use config::StateConfigExt;
use niri_config::{
    Config, FloatOrInt, Key, OutputName, TrackLayout, WarpMouseToFocusMode,
    WorkspaceReference, Xkb,
};
pub use protocols::ProtocolStates;
use smithay::backend::allocator::Fourcc;
use smithay::backend::input::Keycode;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::utils::{
    CropRenderElement, Relocate, RelocateRenderElement,
    RescaleRenderElement,
};
use smithay::backend::renderer::element::Element;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::desktop::{
    find_popup_root_surface, layer_map_for_output, LayerSurface,
    PopupUngrabStrategy, Window,
};
use smithay::input::keyboard::Layout as KeyboardLayout;
use smithay::input::pointer::{
    CursorIcon, CursorImageStatus, Focus,
    GrabStartData as PointerGrabStartData, MotionEvent,
};
use smithay::input::Seat;
use smithay::output::{
    Output, Scale as OutputScale,
};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::calloop::{
    LoopHandle, LoopSignal, RegistrationToken,
};
use smithay::reexports::wayland_server::backend::{
    ClientData, ClientId, DisconnectReason, GlobalId,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Display, DisplayHandle};
use smithay::utils::{
    IsAlive as _, Logical, Point, Rectangle, Scale, Size,
    Transform, SERIAL_COUNTER,
};
use smithay::wayland::compositor::{
    with_states, CompositorClientState, CompositorHandler, HookId,
};
use smithay::wayland::input_method::InputMethodSeat;
use smithay::wayland::session_lock::LockSurface;
use smithay::wayland::shell::wlr_layer::{self};
#[cfg(test)]
use smithay::wayland::single_pixel_buffer::SinglePixelBufferState;
pub use subsystems::{
    CursorSubsystem, FocusState, InputTracking, OutputSubsystem, StreamingSubsystem, UiOverlays,
};
use subsystems::{FocusContext, LayerFocusCandidate};
pub use types::*;

#[cfg(feature = "dbus")]
use crate::a11y::A11y;
use crate::animation::Clock;
use crate::backend::{Backend, Headless, Tty, Winit};
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_locale1::Locale1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_login1::Login1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_introspect::{self, IntrospectToNiri, NiriToIntrospect};
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_screenshot::{NiriToScreenshot, ScreenshotToNiri};
#[cfg(feature = "xdp-gnome-screencast")]
use crate::dbus::mutter_screen_cast::ScreenCastToNiri;
use crate::frame_clock::FrameClock;
use crate::input::pick_color_grab::PickColorGrab;
use crate::input::TabletData;
use crate::ipc::server::IpcServer;
use crate::layer::mapped::LayerSurfaceRenderElement;
use crate::layer::MappedLayer;
// TEAM_060: Using RowId directly instead of WorkspaceId alias
use crate::layout::row_types::RowId as WorkspaceId;
use crate::layout::tile::TileRenderElement;
use crate::layout::{Layout, LayoutElement as _, MonitorRenderElement};
use crate::niri_render_elements;
use crate::protocols::ext_workspace::{self};
use crate::protocols::foreign_toplevel::{self};
use crate::pw_utils::PipeWire;
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::{CastSizeChange, PwToNiri};
use crate::render_helpers::primary_gpu_texture::PrimaryGpuTextureRenderElement;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::{
    render_to_texture, RenderTarget,
};
use crate::ui::exit_confirm_dialog::ExitConfirmDialogRenderElement;
use crate::ui::mru::{MruCloseRequest, WindowMruUiRenderElement};
use crate::ui::screen_transition::{self, ScreenTransition};
use crate::ui::screenshot_ui::{ScreenshotUi, ScreenshotUiRenderElement};
use crate::utils::scale::{closest_representable_scale, guess_monitor_scale};
use crate::utils::vblank_throttle::VBlankThrottle;
use crate::utils::watcher::Watcher;
use crate::utils::xwayland::satellite::Satellite;
use crate::utils::{
    center, center_f64, get_monotonic_time, ipc_transform_to_smithay,
    logical_output, output_matches_name, output_size, panel_orientation,
};
use crate::window::mapped::MappedId;
use crate::window::{Mapped, Unmapped};

const CLEAR_COLOR_LOCKED: [f32; 4] = [0.3, 0.1, 0.1, 1.];

// We'll try to send frame callbacks at least once a second. We'll make a timer that fires once a
// second, so with the worst timing the maximum interval between two frame callbacks for a surface
// should be ~1.995 seconds.
const FRAME_CALLBACK_THROTTLE: Option<Duration> = Some(Duration::from_millis(995));

/// Action to take when redrawing a cast.
#[cfg(feature = "xdp-gnome-screencast")]
enum RedrawCastAction {
    Clear,
    QueueRedraw(Output),
    RenderWindow(u64),
}

pub struct Niri {
    pub config: Rc<RefCell<Config>>,

    /// Output config from the config file.
    ///
    /// This does not include transient output config changes done via IPC. It is only used when
    /// reloading the config from disk to determine if the output configuration should be reloaded
    /// (and transient changes dropped).
    pub config_file_output_config: niri_config::Outputs,

    pub config_file_watcher: Option<Watcher>,

    pub event_loop: LoopHandle<'static, State>,
    pub scheduler: Scheduler<()>,
    pub stop_signal: LoopSignal,
    pub display_handle: DisplayHandle,

    /// Whether niri was run with `--session`
    pub is_session_instance: bool,

    /// Name of the Wayland socket.
    ///
    /// This is `None` when creating `Niri` without a Wayland socket.
    pub socket_name: Option<OsString>,

    pub start_time: Instant,

    /// Whether the at-startup=true window rules are active.
    pub is_at_startup: bool,

    /// Clock for driving animations.
    pub clock: Clock,

    // Each row corresponds to a layout area. Each row generally has one Output mapped to it,
    // however it may have none (when there are no outputs connected) or multiple (when mirroring).
    pub layout: Layout<Mapped>,

    // Windows which don't have a buffer attached yet.
    pub unmapped_windows: HashMap<WlSurface, Unmapped>,

    /// Layer surfaces which don't have a buffer attached yet.
    pub unmapped_layer_surfaces: HashSet<WlSurface>,

    /// Extra data for mapped layer surfaces.
    pub mapped_layer_surfaces: HashMap<LayerSurface, MappedLayer>,

    // Cached root surface for every surface, so that we can access it in destroyed() where the
    // normal get_parent() is cleared out.
    pub root_surface: HashMap<WlSurface, WlSurface>,

    // Dmabuf readiness pre-commit hook for a surface.
    pub dmabuf_pre_commit_hook: HashMap<WlSurface, HookId>,

    /// Clients to notify about their blockers being cleared.
    pub blocker_cleared_tx: Sender<Client>,
    pub blocker_cleared_rx: Receiver<Client>,

    /// Output management subsystem.
    pub outputs: OutputSubsystem,

    /// Input tracking subsystem.
    pub input: InputTracking,

    pub devices: HashSet<input::Device>,
    pub tablets: HashMap<input::Device, TabletData>,
    pub touch: HashSet<input::Device>,

    // Smithay protocol states.
    pub protocols: ProtocolStates,

    /// Active popup grab state.
    pub popup_grab: Option<PopupGrabState>,

    pub seat: Seat<State>,
    /// Scancodes of the keys to suppress.
    pub suppressed_keys: HashSet<Keycode>,
    /// Button codes of the mouse buttons to suppress.
    pub suppressed_buttons: HashSet<u32>,
    pub bind_cooldown_timers: HashMap<Key, RegistrationToken>,
    pub bind_repeat_timer: Option<RegistrationToken>,

    /// Keyboard focus management subsystem.
    pub focus: FocusState,

    pub is_fdo_idle_inhibited: Arc<AtomicBool>,

    /// Most recent XKB settings from org.freedesktop.locale1.
    pub xkb_from_locale1: Option<Xkb>,

    /// Cursor management subsystem.
    pub cursor: CursorSubsystem,

    /// Contents under pointer.
    ///
    /// Periodically updated: on motion and other events and in the loop callback. If you require
    /// the real up-to-date contents somewhere, it's better to recompute on the spot.
    ///
    /// This is not pointer focus. I.e. during a click grab, the pointer focus remains on the
    /// client with the grab, but this field will keep updating to the latest contents as if no
    /// grab was active.
    ///
    /// This is primarily useful for emitting pointer motion events for surfaces that move
    /// underneath the cursor on their own (i.e. when the tiling layout moves). In this case, not
    /// taking grabs into account is expected, because we pass the information to pointer.motion()
    /// which passes it down through grabs, which decide what to do with it as they see fit.

    /// Whether the (idle notifier) activity was notified this event loop iteration.
    ///
    /// Used for limiting the notify to once per iteration, so that it's not spammed with high
    /// resolution mice.
    pub notified_activity_this_iteration: bool,

    pub lock_state: LockState,

    // State that we last sent to the logind LockedHint.
    pub locked_hint: Option<bool>,

    /// UI overlays subsystem.
    pub ui: UiOverlays,

    /// Streaming subsystem for screencast and screencopy.
    pub streaming: StreamingSubsystem,

    // Debug visualization flags
    pub debug_draw_damage: bool,
    pub debug_draw_opaque_regions: bool,

    /// Whether the pointer is inside a hot corner.
    pub pointer_inside_hot_corner: bool,

    /// Pending MRU commit data.
    pub pending_mru_commit: Option<PendingMruCommit>,

    #[cfg(feature = "dbus")]
    pub dbus: Option<crate::dbus::DBusServers>,
    #[cfg(feature = "dbus")]
    pub a11y_keyboard_monitor: Option<crate::dbus::freedesktop_a11y::KeyboardMonitor>,
    #[cfg(feature = "dbus")]
    pub a11y: A11y,
    #[cfg(feature = "dbus")]
    pub inhibit_power_key_fd: Option<zbus::zvariant::OwnedFd>,

    pub ipc_server: Option<IpcServer>,
    pub ipc_outputs_changed: bool,

    pub satellite: Option<Satellite>,
}

// PointerVisibility and DndIcon moved to types.rs

pub struct OutputState {
    pub global: GlobalId,
    pub frame_clock: FrameClock,
    pub redraw_state: RedrawState,
    pub on_demand_vrr_enabled: bool,
    // After the last redraw, some ongoing animations still remain.
    pub unfinished_animations_remain: bool,
    /// Last sequence received in a vblank event.
    pub last_drm_sequence: Option<u32>,
    pub vblank_throttle: VBlankThrottle,
    /// Sequence for frame callback throttling.
    ///
    /// We want to send frame callbacks for each surface at most once per monitor refresh cycle.
    ///
    /// Even if a surface commit resulted in empty damage to the monitor, we want to delay the next
    /// frame callback until roughly when a VBlank would occur, had the monitor been damaged. This
    /// is necessary to prevent clients busy-looping with frame callbacks that result in empty
    /// damage.
    ///
    /// This counter wrapping-increments by 1 every time we move into the next refresh cycle, as
    /// far as frame callback throttling is concerned. Specifically, it happens:
    ///
    /// 1. Upon a successful DRM frame submission. Notably, we don't wait for the VBlank here,
    ///    because the client buffers are already "latched" at the point of submission. Even if a
    ///    client submits a new buffer right away, we will wait for a VBlank to draw it, which
    ///    means that busy looping is avoided.
    /// 2. If a frame resulted in empty damage, a timer is queued to fire roughly when a VBlank
    ///    would occur, based on the last presentation time and output refresh interval. Sequence
    ///    is incremented in that timer, before attempting a redraw or sending frame callbacks.
    pub frame_callback_sequence: u32,
    /// Solid color buffer for the backdrop that we use instead of clearing to avoid damage
    /// tracking issues and make screenshots easier.
    pub backdrop_buffer: SolidColorBuffer,
    pub lock_render_state: LockRenderState,
    pub lock_surface: Option<LockSurface>,
    pub lock_color_buffer: SolidColorBuffer,
    screen_transition: Option<ScreenTransition>,
    /// Damage tracker used for the debug damage visualization.
    pub debug_damage_tracker: OutputDamageTracker,
}

// Types moved to types.rs:
// - RedrawState
// - PopupGrabState
// - KeyboardFocus
// - PointContents
// - LockState
// - LockRenderState
// - SurfaceFrameThrottlingState
// - CenterCoords
// - CastTarget
// - PendingMruCommit

pub struct State {
    pub backend: Backend,
    pub niri: Niri,
}

impl State {
    pub fn new(
        config: Config,
        event_loop: LoopHandle<'static, State>,
        stop_signal: LoopSignal,
        display: Display<State>,
        headless: bool,
        create_wayland_socket: bool,
        is_session_instance: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let _span = tracy_client::span!("State::new");

        let config = Rc::new(RefCell::new(config));

        let has_display = env::var_os("WAYLAND_DISPLAY").is_some()
            || env::var_os("WAYLAND_SOCKET").is_some()
            || env::var_os("DISPLAY").is_some();

        let mut backend = if headless {
            let headless = Headless::new();
            Backend::Headless(headless)
        } else if has_display {
            let winit = Winit::new(config.clone(), event_loop.clone())?;
            Backend::Winit(winit)
        } else {
            let tty = Tty::new(config.clone(), event_loop.clone())
                .context("error initializing the TTY backend")?;
            Backend::Tty(tty)
        };

        let mut niri = Niri::new(
            config.clone(),
            event_loop,
            stop_signal,
            display,
            &backend,
            create_wayland_socket,
            is_session_instance,
        );
        backend.init(&mut niri);

        let mut state = Self { backend, niri };

        // Load the xkb_file config option if set by the user.
        state.load_xkb_file();
        // Initialize some IPC server state.
        state.ipc_keyboard_layouts_changed();
        // Focus the default monitor if set by the user.
        state.focus_default_monitor();

        Ok(state)
    }

    pub fn refresh_and_flush_clients(&mut self) {
        let _span = tracy_client::span!("State::refresh_and_flush_clients");

        self.refresh();

        // Advance animations to the current time (not target render time) before rendering outputs
        // in order to clear completed animations and render elements. Even if we're not rendering,
        // it's good to advance every now and then so the row clean-up and animations don't
        // build up (the 1 second frame callback timer will call this line).
        self.niri.advance_animations();

        self.niri.redraw_queued_outputs(&mut self.backend);

        {
            let _span = tracy_client::span!("flush_clients");
            self.niri.display_handle.flush_clients().unwrap();
        }

        #[cfg(feature = "dbus")]
        self.niri.update_locked_hint();

        // Clear the time so it's fetched afresh next iteration.
        self.niri.clock.clear();
        self.niri.cursor.clear_timer_reset_flag();
        self.niri.notified_activity_this_iteration = false;
    }

    // We monitor both libinput and logind: libinput is always there (including without DBus), but
    // it misses some switch events (e.g. after unsuspend) on some systems.
    pub fn set_lid_closed(&mut self, is_closed: bool) {
        if self.niri.outputs.lid_closed() == is_closed {
            return;
        }

        debug!("laptop lid {}", if is_closed { "closed" } else { "opened" });
        self.niri.outputs.set_lid_closed(is_closed);
        self.backend.on_output_config_changed(&mut self.niri);
    }

    fn refresh(&mut self) {
        let _span = tracy_client::span!("State::refresh");

        // Handle commits for surfaces whose blockers cleared this cycle. This should happen before
        // layout.refresh() since this is where these surfaces handle commits.
        self.notify_blocker_cleared();

        // These should be called periodically, before flushing the clients.
        self.niri.protocols.popups.cleanup();
        self.refresh_popup_grab();
        self.update_keyboard_focus();

        // Should be called before refresh_layout() because that one will refresh other window
        // states and then send a pending configure.
        self.niri.refresh_window_states();

        // Needs to be called after updating the keyboard focus.
        self.niri.refresh_layout();

        self.niri.cursor.check_cursor_image_alive();
        self.niri.refresh_pointer_outputs();
        self.niri.outputs.refresh_space();
        self.niri.refresh_idle_inhibit();
        self.refresh_pointer_contents();
        foreign_toplevel::refresh(self);
        ext_workspace::refresh(self);

        #[cfg(feature = "xdp-gnome-screencast")]
        self.niri.refresh_mapped_cast_outputs();
        // Should happen before refresh_window_rules(), but after anything that can start or stop
        // screencasts.
        #[cfg(feature = "xdp-gnome-screencast")]
        self.niri.refresh_mapped_cast_window_rules();

        self.niri.refresh_window_rules();
        self.refresh_ipc_outputs();
        self.ipc_refresh_layout();
        self.ipc_refresh_keyboard_layout_index();

        // Needs to be called after updating the keyboard focus.
        #[cfg(feature = "dbus")]
        self.niri.refresh_a11y();
    }

    fn notify_blocker_cleared(&mut self) {
        let dh = self.niri.display_handle.clone();
        while let Ok(client) = self.niri.blocker_cleared_rx.try_recv() {
            trace!("calling blocker_cleared");
            self.client_compositor_state(&client)
                .blocker_cleared(self, &dh);
        }
    }

    pub fn move_cursor(&mut self, location: Point<f64, Logical>) {
        let mut under = match self.niri.cursor.visibility() {
            PointerVisibility::Disabled => PointContents::default(),
            _ => self.niri.contents_under(location),
        };

        // Disable the hidden pointer if the contents underneath have changed.
        if !self.niri.cursor.is_visible() && self.niri.cursor.contents() != &under {
            self.niri.cursor.set_visibility(PointerVisibility::Disabled);

            // When setting PointerVisibility::Hidden together with pointer contents changing,
            // we can change straight to nothing to avoid one frame of hover. Notably, this can
            // be triggered through warp-mouse-to-focus combined with hide-when-typing.
            under = PointContents::default();
        }

        self.niri.cursor.update_contents(under.clone());

        let pointer = &self.niri.seat.get_pointer().unwrap();
        pointer.motion(
            self,
            under.surface,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time: get_monotonic_time().as_millis() as u32,
            },
        );
        pointer.frame(self);

        self.niri.maybe_activate_pointer_constraint();

        // We do not show the pointer on programmatic or keyboard movement.

        // FIXME: granular
        self.niri.queue_redraw_all();
    }

    /// Moves cursor within the specified rectangle, only adjusting coordinates if needed.
    fn move_cursor_to_rect(&mut self, rect: Rectangle<f64, Logical>, mode: CenterCoords) -> bool {
        let pointer = &self.niri.seat.get_pointer().unwrap();
        let cur_loc = pointer.current_location();
        let x_in_bound = cur_loc.x >= rect.loc.x && cur_loc.x <= rect.loc.x + rect.size.w;
        let y_in_bound = cur_loc.y >= rect.loc.y && cur_loc.y <= rect.loc.y + rect.size.h;

        let p = match mode {
            CenterCoords::Separately => {
                if x_in_bound && y_in_bound {
                    return false;
                } else if y_in_bound {
                    // adjust x
                    Point::from((rect.loc.x + rect.size.w / 2.0, cur_loc.y))
                } else if x_in_bound {
                    // adjust y
                    Point::from((cur_loc.x, rect.loc.y + rect.size.h / 2.0))
                } else {
                    // adjust x and y
                    center_f64(rect)
                }
            }
            CenterCoords::Both => {
                if x_in_bound && y_in_bound {
                    return false;
                } else {
                    // adjust x and y
                    center_f64(rect)
                }
            }
            CenterCoords::BothAlways => center_f64(rect),
        };

        self.move_cursor(p);
        true
    }

    pub fn move_cursor_to_focused_tile(&mut self, mode: CenterCoords) -> bool {
        if !self.niri.focus.current().is_layout() {
            return false;
        }

        if self.niri.cursor.tablet_location.is_some() {
            return false;
        }

        let Some(output) = self.niri.layout.active_output() else {
            return false;
        };
        let monitor = self.niri.layout.monitor_for_output(output).unwrap();

        let mut rv = false;
        let rect = monitor.active_tile_visual_rectangle();

        if let Some(rect) = rect {
            let output_geo = self.niri.outputs.space().output_geometry(output).unwrap();
            let mut rect = rect;
            rect.loc += output_geo.loc.to_f64();
            rv = self.move_cursor_to_rect(rect, mode);
        }

        rv
    }

    pub fn focus_default_monitor(&mut self) {
        // Our default target is the first output in sorted order.
        let Some(mut target) = self.niri.outputs.first().cloned() else {
            // No outputs are connected.
            return;
        };

        let config = self.niri.config.borrow();
        for config in &config.outputs.0 {
            if !config.focus_at_startup {
                continue;
            }
            if let Some(output) = self.niri.output_by_name_match(&config.name) {
                target = output.clone();
                break;
            }
        }
        drop(config);

        self.niri.layout.focus_output(&target);
        self.move_cursor_to_output(&target);
    }

    /// Focus a specific window, taking care of a potential active output change and cursor
    /// warp.
    pub fn focus_window(&mut self, window: &Window) {
        let active_output = self.niri.layout.active_output().cloned();

        self.niri.layout.activate_window(window);

        let new_active = self.niri.layout.active_output().cloned();
        #[allow(clippy::collapsible_if)]
        if new_active != active_output {
            if !self.maybe_warp_cursor_to_focus_centered() {
                self.move_cursor_to_output(&new_active.unwrap());
            }
        } else {
            self.maybe_warp_cursor_to_focus();
        }

        // FIXME: granular
        self.niri.queue_redraw_all();
    }

    pub fn confirm_mru(&mut self) {
        if let Some(window) = self.niri.close_mru(MruCloseRequest::Confirm) {
            self.focus_window(&window);
        }
    }

    pub fn maybe_warp_cursor_to_focus(&mut self) -> bool {
        let focused = match self.niri.config.borrow().input.warp_mouse_to_focus {
            None => return false,
            Some(inner) => match inner.mode {
                None => CenterCoords::Separately,
                Some(WarpMouseToFocusMode::CenterXy) => CenterCoords::Both,
                Some(WarpMouseToFocusMode::CenterXyAlways) => CenterCoords::BothAlways,
            },
        };
        self.move_cursor_to_focused_tile(focused)
    }

    pub fn maybe_warp_cursor_to_focus_centered(&mut self) -> bool {
        let focused = match self.niri.config.borrow().input.warp_mouse_to_focus {
            None => return false,
            Some(inner) => match inner.mode {
                None => CenterCoords::Both,
                Some(WarpMouseToFocusMode::CenterXy) => CenterCoords::Both,
                Some(WarpMouseToFocusMode::CenterXyAlways) => CenterCoords::BothAlways,
            },
        };
        self.move_cursor_to_focused_tile(focused)
    }

    pub fn refresh_pointer_contents(&mut self) {
        let _span = tracy_client::span!("Niri::refresh_pointer_contents");

        let pointer = &self.niri.seat.get_pointer().unwrap();
        let location = pointer.current_location();

        if !self.niri.ui.exit_dialog.is_open()
            && !self.niri.is_locked()
            && !self.niri.ui.screenshot.is_open()
        {
            // Don't refresh cursor focus during transitions.
            if let Some((output, _)) = self.niri.output_under(location) {
                let monitor = self.niri.layout.monitor_for_output(output).unwrap();
                if monitor.are_transitions_ongoing() {
                    return;
                }
            }
        }

        if !self.update_pointer_contents() {
            return;
        }

        pointer.frame(self);

        // Pointer motion from a surface to nothing triggers a cursor change to default, which
        // means we may need to redraw.

        // FIXME: granular
        self.niri.queue_redraw_all();
    }

    pub fn update_pointer_contents(&mut self) -> bool {
        let _span = tracy_client::span!("Niri::update_pointer_contents");

        let pointer = &self.niri.seat.get_pointer().unwrap();
        let location = pointer.current_location();
        let mut under = match self.niri.cursor.visibility() {
            PointerVisibility::Disabled => PointContents::default(),
            _ => self.niri.contents_under(location),
        };

        // We're not changing the global cursor location here, so if the contents did not change,
        // then nothing changed.
        if self.niri.cursor.contents() == &under {
            return false;
        }

        // Disable the hidden pointer if the contents underneath have changed.
        if !self.niri.cursor.is_visible() {
            self.niri.cursor.set_visibility(PointerVisibility::Disabled);

            // When setting PointerVisibility::Hidden together with pointer contents changing,
            // we can change straight to nothing to avoid one frame of hover. Notably, this can
            // be triggered through warp-mouse-to-focus combined with hide-when-typing.
            under = PointContents::default();
            if self.niri.cursor.contents() == &under {
                return false;
            }
        }

        self.niri.cursor.update_contents(under.clone());

        pointer.motion(
            self,
            under.surface,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time: get_monotonic_time().as_millis() as u32,
            },
        );

        self.niri.maybe_activate_pointer_constraint();

        true
    }

    pub fn move_cursor_to_output(&mut self, output: &Output) {
        let geo = self.niri.outputs.space().output_geometry(output).unwrap();
        self.move_cursor(center(geo).to_f64());
    }

    pub fn refresh_popup_grab(&mut self) {
        let keyboard_grabbed = self.niri.seat.input_method().keyboard_grabbed();

        if let Some(grab) = &mut self.niri.popup_grab {
            if grab.grab.has_ended() {
                self.niri.popup_grab = None;
            } else if keyboard_grabbed {
                // HACK: remove popup grab if IME grabbed the keyboard, because we can't yet do
                // popup grabs together with an IME grab.
                // FIXME: do this properly.
                grab.grab.ungrab(PopupUngrabStrategy::All);
                self.niri.seat.get_pointer().unwrap().unset_grab(
                    self,
                    SERIAL_COUNTER.next_serial(),
                    get_monotonic_time().as_millis() as u32,
                );
                self.niri.popup_grab = None;
            }
        }
    }

    /// Builds the focus context for focus computation.
    fn build_focus_context(&self) -> FocusContext<'_> {
        // Collect all the state needed for focus computation
        FocusContext {
            exit_dialog_open: self.niri.ui.exit_dialog.is_open(),
            is_locked: self.niri.is_locked(),
            lock_surface: self.niri.lock_surface_focus(),
            screenshot_ui_open: self.niri.ui.screenshot.is_open(),
            mru_ui_open: self.niri.ui.mru.is_open(),
            popup_grab: self.build_popup_grab_info(),
            layer_surfaces: self.collect_layer_focus_candidates(),
            layout_above_top: self.layout_renders_above_top(),
            layout_focus: self
                .niri
                .layout
                .focus()
                .map(|win| win.toplevel().wl_surface().clone()),
        }
    }

    /// Cleans up stale on-demand layer focus.
    fn cleanup_layer_on_demand_focus(&mut self) {
        self.niri.focus.cleanup_layer_on_demand(|surface| {
            // Must be alive
            if !surface.alive() {
                return false;
            }

            // Must have on-demand interactivity
            if surface.cached_state().keyboard_interactivity
                != wlr_layer::KeyboardInteractivity::OnDemand
            {
                return false;
            }

            // Must be mapped and not in backdrop
            match self.niri.mapped_layer_surfaces.get(surface) {
                Some(mapped) => !mapped.place_within_backdrop(),
                None => false, // Unmapped
            }
        });
    }

    /// Builds popup grab information for focus context.
    fn build_popup_grab_info(
        &self,
    ) -> Option<(WlSurface, smithay::wayland::shell::wlr_layer::Layer)> {
        self.niri.popup_grab.as_ref().map(|grab| {
            // Get the layer from the popup grab's root surface
            // Default to Top layer if we can't determine it
            let layer = self
                .niri
                .mapped_layer_surfaces
                .keys()
                .find(|ls| ls.wl_surface() == &grab.root)
                .map(|ls| ls.layer())
                .unwrap_or(smithay::wayland::shell::wlr_layer::Layer::Top);
            (grab.root.clone(), layer)
        })
    }

    /// Collects layer surface focus candidates.
    fn collect_layer_focus_candidates(&self) -> Vec<LayerFocusCandidate<'_>> {
        let mut candidates = Vec::new();

        for (surface, mapped) in &self.niri.mapped_layer_surfaces {
            let is_on_demand_focused = self.niri.focus.layer_on_demand() == Some(surface);

            candidates.push(LayerFocusCandidate {
                surface,
                layer: surface.layer(),
                is_exclusive: surface.cached_state().keyboard_interactivity
                    == smithay::wayland::shell::wlr_layer::KeyboardInteractivity::Exclusive,
                is_on_demand_focused,
                is_in_backdrop: mapped.place_within_backdrop(),
            });
        }

        candidates
    }

    /// Checks if layout renders above top layer (fullscreen mode).
    fn layout_renders_above_top(&self) -> bool {
        // Check if any window on the active output is fullscreen
        if let Some(output) = self.niri.layout.active_output() {
            for mapped in self.niri.layout.windows_for_output(&output) {
                if mapped.sizing_mode().is_fullscreen() {
                    return true;
                }
            }
        }
        false
    }

    /// Updates MRU timestamp with debounce.
    #[allow(dead_code)]
    fn update_mru_timestamp(&mut self, mapped: &mut Mapped) {
        let stamp = get_monotonic_time();
        let debounce = self.niri.config.borrow().recent_windows.debounce_ms;
        let debounce = Duration::from_millis(u64::from(debounce));

        if mapped.get_focus_timestamp().is_none() || debounce.is_zero() {
            mapped.set_focus_timestamp(stamp);
        } else {
            self.schedule_mru_commit(mapped.id(), stamp, debounce);
        }
    }

    /// Schedules a delayed MRU commit.
    fn schedule_mru_commit(&mut self, id: MappedId, stamp: Duration, debounce: Duration) {
        let timer = Timer::from_duration(debounce);
        let token = self
            .niri
            .event_loop
            .insert_source(timer, move |_, _, state| {
                state.niri.mru_apply_keyboard_commit();
                TimeoutAction::Drop
            })
            .unwrap();

        if let Some(PendingMruCommit {
            token: old_token, ..
        }) = self
            .niri
            .pending_mru_commit
            .replace(PendingMruCommit { id, token, stamp })
        {
            self.niri.event_loop.remove(old_token);
        }
    }

    /// Handles all side effects of a focus change.
    fn handle_focus_change(&mut self, old_focus: &KeyboardFocus, new_focus: &KeyboardFocus) {
        self.update_window_focus_states(old_focus, new_focus);
        self.handle_popup_grab_on_focus_change(new_focus);
        self.handle_keyboard_layout_tracking(old_focus, new_focus);
        self.apply_keyboard_focus(new_focus);
    }

    /// Applies the keyboard focus change.
    fn apply_keyboard_focus(&mut self, new_focus: &KeyboardFocus) {
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        self.niri.focus.set_current(new_focus.clone());
        keyboard.set_focus(
            self,
            new_focus.clone().into_surface(),
            SERIAL_COUNTER.next_serial(),
        );

        // FIXME: can be more granular.
        self.niri.queue_redraw_all();
    }

    /// Handles per-window keyboard layout tracking.
    fn handle_keyboard_layout_tracking(&mut self, old: &KeyboardFocus, new: &KeyboardFocus) {
        if self.niri.config.borrow().input.keyboard.track_layout != TrackLayout::Window {
            return;
        }

        let keyboard = self.niri.seat.get_keyboard().unwrap();
        let current_layout = keyboard.with_xkb_state(self, |context| {
            let xkb = context.xkb().lock().unwrap();
            xkb.active_layout()
        });

        // Store current layout for old focus surface
        if let Some(surface) = old.surface() {
            with_states(surface, |data| {
                let cell = data
                    .data_map
                    .get_or_insert::<Cell<KeyboardLayout>, _>(Cell::default);
                cell.set(current_layout);
            });
        }

        // Restore layout for new focus surface
        let new_layout = new.surface().map_or(current_layout, |surface| {
            with_states(surface, |data| {
                data.data_map
                    .get_or_insert::<Cell<KeyboardLayout>, _>(|| {
                        // The default layout is effectively the first layout in the
                        // keymap, so use it for new windows.
                        Cell::new(KeyboardLayout::default())
                    })
                    .get()
            })
        });

        if new_layout != current_layout && new.surface().is_some() {
            keyboard.set_focus(self, None, SERIAL_COUNTER.next_serial());
            keyboard.with_xkb_state(self, |mut context| {
                context.set_layout(new_layout);
            });
        }
    }

    /// Handles popup grab when focus changes.
    fn handle_popup_grab_on_focus_change(&mut self, new_focus: &KeyboardFocus) {
        let Some(grab) = self.niri.popup_grab.as_mut() else {
            return;
        };

        if grab.has_keyboard_grab && Some(&grab.root) != new_focus.surface() {
            trace!(
                "grab root {:?} is not the new focus {:?}, ungrabbing",
                grab.root,
                new_focus
            );

            let keyboard = self.niri.seat.get_keyboard().unwrap();
            grab.grab.ungrab(PopupUngrabStrategy::All);
            keyboard.unset_grab(self);
            self.niri.seat.get_pointer().unwrap().unset_grab(
                self,
                SERIAL_COUNTER.next_serial(),
                get_monotonic_time().as_millis() as u32,
            );
            self.niri.popup_grab = None;
        }
    }

    /// Updates window focus states and MRU timestamps.
    fn update_window_focus_states(&mut self, old: &KeyboardFocus, new: &KeyboardFocus) {
        // Unfocus old window
        if let KeyboardFocus::Layout {
            surface: Some(surface),
        } = old
        {
            if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                mapped.set_is_focused(false);
            }
        }

        // Focus new window
        if let KeyboardFocus::Layout {
            surface: Some(surface),
        } = new
        {
            // Extract data needed for MRU scheduling before releasing the borrow.
            let mru_schedule_data =
                if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                    mapped.set_is_focused(true);

                    let stamp = get_monotonic_time();
                    let debounce = self.niri.config.borrow().recent_windows.debounce_ms;
                    let debounce = Duration::from_millis(u64::from(debounce));

                    if mapped.get_focus_timestamp().is_none() || debounce.is_zero() {
                        mapped.set_focus_timestamp(stamp);
                        None
                    } else {
                        Some((mapped.id(), stamp, debounce))
                    }
                } else {
                    None
                };

            // Schedule MRU commit after releasing the layout borrow.
            if let Some((id, stamp, debounce)) = mru_schedule_data {
                self.schedule_mru_commit(id, stamp, debounce);
            }
        }
    }

    pub fn update_keyboard_focus(&mut self) {
        // Clean up stale on-demand focus
        self.cleanup_layer_on_demand_focus();

        // Compute new focus
        let ctx = self.build_focus_context();
        let new_focus = self.niri.focus.compute_focus(&ctx);

        // Handle focus change if different
        let old_focus = self.niri.focus.current().clone();
        if old_focus != new_focus {
            trace!(
                "keyboard focus changed from {:?} to {:?}",
                old_focus,
                new_focus
            );
            self.handle_focus_change(&old_focus, &new_focus);
        }
    }

    pub fn modify_output_config<F>(&mut self, name: &str, fun: F)
    where
        F: FnOnce(&mut niri_config::Output),
    {
        // Try hard to find the output config section corresponding to the output set by the
        // user. Since if we add a new section and some existing section also matches the
        // output, then our new section won't do anything.
        let temp;
        let match_name = if let Some(output) = self.niri.output_by_name_match(name) {
            output.user_data().get::<OutputName>().unwrap()
        } else if let Some(output_name) = self
            .backend
            .tty_checked()
            .and_then(|tty| tty.disconnected_connector_name_by_name_match(name))
        {
            temp = output_name;
            &temp
        } else {
            // Even if name is "make model serial", matching will work fine this way.
            temp = OutputName {
                connector: name.to_owned(),
                make: None,
                model: None,
                serial: None,
            };
            &temp
        };

        let mut config = self.niri.config.borrow_mut();
        let config = if let Some(config) = config.outputs.find_mut(match_name) {
            config
        } else {
            config.outputs.0.push(niri_config::Output {
                // Save name as set by the user.
                name: String::from(name),
                ..Default::default()
            });
            config.outputs.0.last_mut().unwrap()
        };

        fun(config);
    }

    pub fn apply_transient_output_config(&mut self, name: &str, action: niri_ipc::OutputAction) {
        self.modify_output_config(name, move |config| match action {
            niri_ipc::OutputAction::Off => config.off = true,
            niri_ipc::OutputAction::On => config.off = false,
            niri_ipc::OutputAction::Mode { mode } => {
                config.mode = match mode {
                    niri_ipc::ModeToSet::Automatic => None,
                    niri_ipc::ModeToSet::Specific(mode) => Some(niri_config::output::Mode {
                        custom: false,
                        mode,
                    }),
                };
                config.modeline = None;
            }
            niri_ipc::OutputAction::CustomMode { mode } => {
                config.mode = Some(niri_config::output::Mode { custom: true, mode });
                config.modeline = None;
            }
            niri_ipc::OutputAction::Modeline {
                clock,
                hdisplay,
                hsync_start,
                hsync_end,
                htotal,
                vdisplay,
                vsync_start,
                vsync_end,
                vtotal,
                hsync_polarity,
                vsync_polarity,
            } => {
                // Do not reset config.mode to None since it's used as a fallback.
                config.modeline = Some(niri_config::output::Modeline {
                    clock,
                    hdisplay,
                    hsync_start,
                    hsync_end,
                    htotal,
                    vdisplay,
                    vsync_start,
                    vsync_end,
                    vtotal,
                    hsync_polarity,
                    vsync_polarity,
                })
            }
            niri_ipc::OutputAction::Scale { scale } => {
                config.scale = match scale {
                    niri_ipc::ScaleToSet::Automatic => None,
                    niri_ipc::ScaleToSet::Specific(scale) => Some(FloatOrInt(scale)),
                }
            }
            niri_ipc::OutputAction::Transform { transform } => config.transform = transform,
            niri_ipc::OutputAction::Position { position } => {
                config.position = match position {
                    niri_ipc::PositionToSet::Automatic => None,
                    niri_ipc::PositionToSet::Specific(position) => Some(niri_config::Position {
                        x: position.x,
                        y: position.y,
                    }),
                }
            }
            niri_ipc::OutputAction::Vrr { vrr } => {
                config.variable_refresh_rate = if vrr.vrr {
                    Some(niri_config::Vrr {
                        on_demand: vrr.on_demand,
                    })
                } else {
                    None
                }
            }
        });

        self.reload_output_config();
    }

    pub fn refresh_ipc_outputs(&mut self) {
        if !self.niri.ipc_outputs_changed {
            return;
        }
        self.niri.ipc_outputs_changed = false;

        let _span = tracy_client::span!("State::refresh_ipc_outputs");

        for ipc_output in self.backend.ipc_outputs().lock().unwrap().values_mut() {
            let logical = self
                .niri
                .outputs
                .space()
                .outputs()
                .find(|output| output.name() == ipc_output.name)
                .map(logical_output);
            ipc_output.logical = logical;
        }

        #[cfg(feature = "dbus")]
        self.niri.on_ipc_outputs_changed();

        let new_config = self.backend.ipc_outputs().lock().unwrap().clone();
        self.niri
            .protocols
            .output_management
            .notify_changes(new_config);
    }

    pub fn open_screenshot_ui(&mut self, show_pointer: bool, path: Option<String>) {
        if self.niri.is_locked() || self.niri.ui.screenshot.is_open() {
            return;
        }

        // Redraw the pointer if hidden through cursor{} options
        if self.niri.cursor.visibility() == PointerVisibility::Hidden {
            self.niri.cursor.set_visibility(PointerVisibility::Visible);
            self.niri.queue_redraw_all();
        }

        let default_output = self
            .niri
            .output_under_cursor()
            .or_else(|| self.niri.layout.active_output().cloned());
        let Some(default_output) = default_output else {
            return;
        };

        self.niri.update_render_elements(None);

        let Some(screenshots) = self
            .backend
            .with_primary_renderer(|renderer| self.niri.capture_screenshots(renderer).collect())
        else {
            return;
        };

        // Now that we captured the screenshots, clear grabs like drag-and-drop, etc.
        self.niri.seat.get_pointer().unwrap().unset_grab(
            self,
            SERIAL_COUNTER.next_serial(),
            get_monotonic_time().as_millis() as u32,
        );
        if let Some(touch) = self.niri.seat.get_touch() {
            touch.unset_grab(self);
        }

        self.backend.with_primary_renderer(|renderer| {
            self.niri
                .ui
                .screenshot
                .open(renderer, screenshots, default_output, show_pointer, path)
        });

        self.niri
            .cursor
            .manager
            .set_cursor_image(CursorImageStatus::Named(CursorIcon::Crosshair));
        self.niri.queue_redraw_all();
    }

    pub fn handle_pick_color(&mut self, tx: async_channel::Sender<Option<niri_ipc::PickedColor>>) {
        let pointer = self.niri.seat.get_pointer().unwrap();
        let start_data = PointerGrabStartData {
            focus: None,
            button: 0,
            location: pointer.current_location(),
        };
        let grab = PickColorGrab::new(start_data);
        pointer.set_grab(self, grab, SERIAL_COUNTER.next_serial(), Focus::Clear);
        self.niri.ui.pick_color = Some(tx);
        self.niri
            .cursor
            .manager
            .set_cursor_image(CursorImageStatus::Named(CursorIcon::Crosshair));
        self.niri.queue_redraw_all();
    }

    pub fn confirm_screenshot(&mut self, write_to_disk: bool) {
        let ScreenshotUi::Open { path, .. } = &mut self.niri.ui.screenshot else {
            return;
        };
        let path = path.take();

        self.backend.with_primary_renderer(|renderer| {
            match self.niri.ui.screenshot.capture(renderer) {
                Ok((size, pixels)) => {
                    if let Err(err) = self.niri.save_screenshot(size, pixels, write_to_disk, path) {
                        warn!("error saving screenshot: {err:?}");
                    }
                }
                Err(err) => {
                    warn!("error capturing screenshot: {err:?}");
                }
            }
        });

        self.niri.ui.screenshot.close();
        self.niri
            .cursor
            .manager
            .set_cursor_image(CursorImageStatus::default_named());
        self.niri.queue_redraw_all();
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_pw_msg(&mut self, msg: PwToNiri) {
        match msg {
            PwToNiri::StopCast { session_id } => self.niri.stop_cast(session_id),
            PwToNiri::Redraw { stream_id } => self.redraw_cast(stream_id),
            PwToNiri::FatalError => {
                warn!("stopping PipeWire due to fatal error");
                // Collect session IDs first to avoid borrow conflicts.
                let ids = self.niri.streaming.collect_session_ids();
                if let Some(pw) = self.niri.streaming.take_pipewire() {
                    for id in ids {
                        self.niri.stop_cast(id);
                    }
                    self.niri.event_loop.remove(pw.token);
                }
            }
        }
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    fn redraw_cast(&mut self, stream_id: usize) {
        let _span = tracy_client::span!("State::redraw_cast");

        // First, determine what action to take based on cast target.
        let action = {
            let Some(cast) = self.niri.streaming.find_cast(stream_id) else {
                warn!("cast to redraw is missing");
                return;
            };

            match &cast.target {
                CastTarget::Nothing => RedrawCastAction::Clear,
                CastTarget::Output(weak) => {
                    if let Some(output) = weak.upgrade() {
                        RedrawCastAction::QueueRedraw(output)
                    } else {
                        return;
                    }
                }
                CastTarget::Window { id } => RedrawCastAction::RenderWindow(*id),
            }
        };

        match action {
            RedrawCastAction::Clear => {
                self.backend.with_primary_renderer(|renderer| {
                    if let Some(cast) = self.niri.streaming.find_cast_mut(stream_id) {
                        if cast.dequeue_buffer_and_clear(renderer) {
                            cast.last_frame_time = get_monotonic_time();
                        }
                    }
                });
            }
            RedrawCastAction::QueueRedraw(output) => {
                self.niri.queue_redraw(&output);
            }
            RedrawCastAction::RenderWindow(id) => {
                self.redraw_cast_window(stream_id, id);
            }
        }
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    fn redraw_cast_window(&mut self, stream_id: usize, window_id: u64) {
        // Find window and get needed data.
        let window_data = {
            let mut windows = self.niri.layout.windows();
            let Some((_, mapped)) = windows.find(|(_, mapped)| mapped.id().get() == window_id)
            else {
                return;
            };

            let Some(output) = self.niri.streaming.mapped_cast_output(&mapped.window) else {
                return;
            };

            let scale = Scale::from(output.current_scale().fractional_scale());
            let bbox = mapped
                .window
                .bbox_with_popups()
                .to_physical_precise_up(scale);

            (mapped.window.clone(), scale, bbox)
        };

        let (window, scale, bbox) = window_data;

        // Check size and render.
        let should_stop = {
            let Some(cast) = self.niri.streaming.find_cast_mut(stream_id) else {
                return;
            };

            match cast.ensure_size(bbox.size) {
                Ok(CastSizeChange::Ready) => None,
                Ok(CastSizeChange::Pending) => return,
                Err(err) => {
                    warn!("error updating stream size, stopping screencast: {err:?}");
                    Some(cast.session_id)
                }
            }
        };

        if let Some(session_id) = should_stop {
            self.niri.stop_cast(session_id);
            return;
        }

        // Find mapped again for rendering (window clone lets us release borrow).
        let mut windows = self.niri.layout.windows();
        let Some((_, mapped)) = windows.find(|(_, m)| m.window == window) else {
            return;
        };

        self.backend.with_primary_renderer(|renderer| {
            let elements = mapped
                .render_for_screen_cast(renderer, scale)
                .rev()
                .collect::<Vec<_>>();

            if let Some(cast) = self.niri.streaming.find_cast_mut(stream_id) {
                if cast.dequeue_buffer_and_render(renderer, &elements, bbox.size, scale) {
                    cast.last_frame_time = get_monotonic_time();
                }
            }
        });
    }

    #[cfg(not(feature = "xdp-gnome-screencast"))]
    pub fn set_dynamic_cast_target(&mut self, _target: CastTarget) {}

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn set_dynamic_cast_target(&mut self, target: CastTarget) {
        let _span = tracy_client::span!("State::set_dynamic_cast_target");

        let mut refresh = None;
        match &target {
            // Leave refresh as is when clearing. Chances are, the next refresh will match it,
            // then we'll avoid reconfiguring.
            CastTarget::Nothing => (),
            CastTarget::Output(output) => {
                if let Some(output) = output.upgrade() {
                    refresh = Some(output.current_mode().unwrap().refresh as u32);
                }
            }
            CastTarget::Window { id } => {
                let mut windows = self.niri.layout.windows();
                if let Some((_, mapped)) = windows.find(|(_, mapped)| mapped.id().get() == *id) {
                    if let Some(output) = self.niri.streaming.mapped_cast_output(&mapped.window) {
                        refresh = Some(output.current_mode().unwrap().refresh as u32);
                    }
                }
            }
        }

        let mut to_redraw = Vec::new();
        let mut to_stop = Vec::new();
        for cast in self.niri.streaming.casts_mut() {
            if !cast.dynamic_target {
                continue;
            }

            if let Some(refresh) = refresh {
                if let Err(err) = cast.set_refresh(refresh) {
                    warn!("error changing cast FPS: {err:?}");
                    to_stop.push(cast.session_id);
                    continue;
                }
            }

            cast.target = target.clone();
            to_redraw.push(cast.stream_id);
        }

        for id in to_redraw {
            self.redraw_cast(id);
        }
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn on_screen_cast_msg(&mut self, msg: ScreenCastToNiri) {
        use smithay::reexports::gbm::Modifier;

        use crate::dbus::mutter_screen_cast::StreamTargetId;

        match msg {
            ScreenCastToNiri::StartCast {
                session_id,
                stream_id,
                target,
                cursor_mode,
                signal_ctx,
            } => {
                let _span = tracy_client::span!("StartCast");

                debug!(session_id, stream_id, "StartCast");

                let Some(gbm) = self.backend.gbm_device() else {
                    warn!("error starting screencast: no GBM device available");
                    self.niri.stop_cast(session_id);
                    return;
                };

                if self.niri.streaming.pipewire().is_none() {
                    match PipeWire::new(
                        self.niri.event_loop.clone(),
                        self.niri.streaming.pw_sender().unwrap().clone(),
                    ) {
                        Ok(pipewire) => self.niri.streaming.set_pipewire(Some(pipewire)),
                        Err(err) => {
                            warn!(
                                "error starting screencast: PipeWire failed to initialize: {err:?}"
                            );
                            self.niri.stop_cast(session_id);
                            return;
                        }
                    }
                }
                let Some(pw) = self.niri.streaming.pipewire() else {
                    warn!("error starting screencast: PipeWire not available");
                    self.niri.stop_cast(session_id);
                    return;
                };

                let mut dynamic_target = false;
                let (target, size, refresh, alpha) = match target {
                    StreamTargetId::Output { name } => {
                        let global_space = &self.niri.outputs.space();
                        let output = global_space.outputs().find(|out| out.name() == name);
                        let Some(output) = output else {
                            warn!("error starting screencast: requested output is missing");
                            self.niri.stop_cast(session_id);
                            return;
                        };

                        let mode = output.current_mode().unwrap();
                        let transform = output.current_transform();
                        let size = transform.transform_size(mode.size);
                        let refresh = mode.refresh as u32;
                        (CastTarget::Output(output.downgrade()), size, refresh, false)
                    }
                    StreamTargetId::Window { id }
                        if id == self.niri.streaming.dynamic_cast_id().unwrap().get() =>
                    {
                        dynamic_target = true;

                        // All dynamic casts start as Nothing to avoid surprises and exposing
                        // sensitive info.
                        (CastTarget::Nothing, Size::from((1, 1)), 1000, true)
                    }
                    StreamTargetId::Window { id } => {
                        let Some(window) = self.niri.layout.windows().find_map(|(_, mapped)| {
                            (mapped.id().get() == id).then_some(&mapped.window)
                        }) else {
                            warn!("error starting screencast: requested window is missing");
                            self.niri.stop_cast(session_id);
                            return;
                        };

                        // Use the cached output since it will be present even if the output was
                        // currently disconnected.
                        let Some(output) = self.niri.streaming.mapped_cast_output(window) else {
                            warn!("error starting screencast: requested window is missing");
                            self.niri.stop_cast(session_id);
                            return;
                        };

                        let scale = Scale::from(output.current_scale().fractional_scale());
                        let bbox = window.bbox_with_popups().to_physical_precise_up(scale);
                        let refresh = output.current_mode().unwrap().refresh as u32;

                        (CastTarget::Window { id }, bbox.size, refresh, true)
                    }
                };

                let mut render_formats = self
                    .backend
                    .with_primary_renderer(|renderer| {
                        renderer.egl_context().dmabuf_render_formats().clone()
                    })
                    .unwrap_or_default();

                {
                    let config = self.niri.config.borrow();
                    if config.debug.force_pipewire_invalid_modifier {
                        render_formats = render_formats
                            .into_iter()
                            .filter(|f| f.modifier == Modifier::Invalid)
                            .collect();
                    }
                }

                let res = pw.start_cast(
                    gbm,
                    render_formats,
                    session_id,
                    stream_id,
                    target,
                    dynamic_target,
                    size,
                    refresh,
                    alpha,
                    cursor_mode,
                    signal_ctx,
                );
                match res {
                    Ok(cast) => {
                        self.niri.streaming.casts_mut().push(cast);
                    }
                    Err(err) => {
                        warn!("error starting screencast: {err:?}");
                        self.niri.stop_cast(session_id);
                    }
                }
            }
            ScreenCastToNiri::StopCast { session_id } => self.niri.stop_cast(session_id),
        }
    }

    #[cfg(feature = "dbus")]
    pub fn on_screen_shot_msg(
        &mut self,
        to_screenshot: &async_channel::Sender<NiriToScreenshot>,
        msg: ScreenshotToNiri,
    ) {
        match msg {
            ScreenshotToNiri::TakeScreenshot { include_cursor } => {
                self.handle_take_screenshot(to_screenshot, include_cursor);
            }
            ScreenshotToNiri::PickColor(tx) => {
                self.handle_pick_color(tx);
            }
        }
    }

    #[cfg(feature = "dbus")]
    fn handle_take_screenshot(
        &mut self,
        to_screenshot: &async_channel::Sender<NiriToScreenshot>,
        include_cursor: bool,
    ) {
        let _span = tracy_client::span!("TakeScreenshot");

        let rv = self.backend.with_primary_renderer(|renderer| {
            let on_done = {
                let to_screenshot = to_screenshot.clone();
                move |path| {
                    let msg = NiriToScreenshot::ScreenshotResult(Some(path));
                    if let Err(err) = to_screenshot.send_blocking(msg) {
                        warn!("error sending path to screenshot: {err:?}");
                    }
                }
            };

            let res = self
                .niri
                .screenshot_all_outputs(renderer, include_cursor, on_done);

            if let Err(err) = res {
                warn!("error taking a screenshot: {err:?}");

                let msg = NiriToScreenshot::ScreenshotResult(None);
                if let Err(err) = to_screenshot.send_blocking(msg) {
                    warn!("error sending None to screenshot: {err:?}");
                }
            }
        });

        if rv.is_none() {
            let msg = NiriToScreenshot::ScreenshotResult(None);
            if let Err(err) = to_screenshot.send_blocking(msg) {
                warn!("error sending None to screenshot: {err:?}");
            }
        }
    }

    #[cfg(feature = "dbus")]
    pub fn on_introspect_msg(
        &mut self,
        to_introspect: &async_channel::Sender<NiriToIntrospect>,
        msg: IntrospectToNiri,
    ) {
        use crate::utils::with_toplevel_role;

        let IntrospectToNiri::GetWindows = msg;
        let _span = tracy_client::span!("GetWindows");

        let mut windows = HashMap::new();

        #[cfg(feature = "xdp-gnome-screencast")]
        windows.insert(
            self.niri.streaming.dynamic_cast_id().unwrap().get(),
            gnome_shell_introspect::WindowProperties {
                title: String::from("niri Dynamic Cast Target"),
                app_id: String::from("rs.bxt.niri.desktop"),
            },
        );

        self.niri.layout.with_windows(|mapped, _, _, _| {
            let id = mapped.id().get();
            let props = with_toplevel_role(mapped.toplevel(), |role| {
                gnome_shell_introspect::WindowProperties {
                    title: role.title.clone().unwrap_or_default(),
                    app_id: role
                        .app_id
                        .as_ref()
                        // We don't do proper .desktop file tracking (it's quite involved), and
                        // Wayland windows can set any app id they want. However, this seems to
                        // work well enough in practice.
                        .map(|app_id| format!("{app_id}.desktop"))
                        .unwrap_or_default(),
                }
            });

            windows.insert(id, props);
        });

        let msg = NiriToIntrospect::Windows(windows);
        if let Err(err) = to_introspect.send_blocking(msg) {
            warn!("error sending windows to introspect: {err:?}");
        }
    }

    #[cfg(feature = "dbus")]
    pub fn on_login1_msg(&mut self, msg: Login1ToNiri) {
        let Login1ToNiri::LidClosedChanged(is_closed) = msg;

        trace!("login1 lid {}", if is_closed { "closed" } else { "opened" });
        self.set_lid_closed(is_closed);
    }

    #[cfg(feature = "dbus")]
    pub fn on_locale1_msg(&mut self, msg: Locale1ToNiri) {
        let Locale1ToNiri::XkbChanged(xkb) = msg;

        trace!("locale1 xkb settings changed: {xkb:?}");
        let xkb = self.niri.xkb_from_locale1.insert(xkb);

        {
            let config = self.niri.config.borrow();
            if config.input.keyboard.xkb != Xkb::default() {
                trace!("ignoring locale1 xkb change because niri config has xkb settings");
                return;
            }
        }

        let xkb = xkb.clone();
        self.set_xkb_config(xkb.to_xkb_config());
        self.ipc_keyboard_layouts_changed();
    }
}

// Niri::new() moved to init.rs

// TEAM_083: Removed compatibility accessors per Rule 5.
// Access subsystem fields directly: niri.outputs.space(), niri.cursor.manager, niri.focus.current,
// etc.

impl Niri {
    pub fn insert_client(&mut self, client: NewClient) {
        let NewClient {
            client,
            restricted,
            credentials_unknown,
        } = client;

        let config = self.config.borrow();
        let data = Arc::new(ClientState {
            compositor_state: Default::default(),
            can_view_decoration_globals: config.prefer_no_csd,
            primary_selection_disabled: config.clipboard.disable_primary,
            restricted,
            credentials_unknown,
        });

        if let Err(err) = self.display_handle.insert_client(client, data) {
            warn!("error inserting client: {err}");
        }
    }

    #[cfg(feature = "dbus")]
    pub fn inhibit_power_key(&mut self) -> anyhow::Result<()> {
        use smithay::reexports::rustix::io::{fcntl_setfd, FdFlags};

        let conn = zbus::blocking::Connection::system()?;

        let message = conn.call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "Inhibit",
            &("handle-power-key", "niri", "Power key handling", "block"),
        )?;

        let fd: zbus::zvariant::OwnedFd = message.body().deserialize()?;

        // Don't leak the fd to child processes.
        if let Err(err) = fcntl_setfd(&fd, FdFlags::CLOEXEC) {
            warn!("error setting CLOEXEC on inhibit fd: {err:?}");
        };

        self.inhibit_power_key_fd = Some(fd);

        Ok(())
    }

    /// Repositions all outputs, optionally adding a new output.
    pub fn reposition_outputs(&mut self, new_output: Option<&Output>) {
        let _span = tracy_client::span!("Niri::reposition_outputs");

        #[derive(Debug)]
        struct Data {
            output: Output,
            name: OutputName,
            position: Option<Point<i32, Logical>>,
            config: Option<niri_config::Position>,
        }

        let config = self.config.borrow();
        let mut outputs = vec![];
        for output in self.outputs.space().outputs().chain(new_output) {
            let name = output.user_data().get::<OutputName>().unwrap();
            let position = self
                .outputs
                .space()
                .output_geometry(output)
                .map(|geo| geo.loc);
            let config = config.outputs.find(name).and_then(|c| c.position);

            outputs.push(Data {
                output: output.clone(),
                name: name.clone(),
                position,
                config,
            });
        }
        drop(config);

        for Data { output, .. } in &outputs {
            self.outputs.unmap_output(output);
        }

        // Connectors can appear in udev in any order. If we sort by name then we get output
        // positioning that does not depend on the order they appeared.
        //
        // This sorting first compares by make/model/serial so that it is stable regardless of the
        // connector name. However, if make/model/serial is equal or unknown, then it does fall
        // back to comparing the connector name, which should always be unique.
        outputs.sort_unstable_by(|a, b| a.name.compare(&b.name));

        // Place all outputs with explicitly configured position first, then the unconfigured ones.
        outputs.sort_by_key(|d| d.config.is_none());

        trace!(
            "placing outputs in order: {:?}",
            outputs.iter().map(|d| &d.name.connector)
        );

        self.outputs.set_sorted(
            outputs
                .iter()
                .map(|Data { output, .. }| output.clone())
                .collect(),
        );

        for data in outputs.into_iter() {
            let Data {
                output,
                name,
                position,
                config,
            } = data;

            let size = output_size(&output).to_i32_round();

            let new_position = config
                .map(|pos| Point::from((pos.x, pos.y)))
                .filter(|pos| {
                    // Ensure that the requested position does not overlap any existing output.
                    let target_geom = Rectangle::new(*pos, size);

                    let overlap = self
                        .outputs
                        .space()
                        .outputs()
                        .map(|output| self.outputs.space().output_geometry(output).unwrap())
                        .find(|geom| geom.overlaps(target_geom));

                    if let Some(overlap) = overlap {
                        warn!(
                            "output {} at x={} y={} sized {}x{} \
                             overlaps an existing output at x={} y={} sized {}x{}, \
                             falling back to automatic placement",
                            name.connector,
                            pos.x,
                            pos.y,
                            size.w,
                            size.h,
                            overlap.loc.x,
                            overlap.loc.y,
                            overlap.size.w,
                            overlap.size.h,
                        );

                        false
                    } else {
                        true
                    }
                })
                .unwrap_or_else(|| {
                    let x = self
                        .outputs
                        .space()
                        .outputs()
                        .map(|output| self.outputs.space().output_geometry(output).unwrap())
                        .map(|geom| geom.loc.x + geom.size.w)
                        .max()
                        .unwrap_or(0);

                    Point::from((x, 0))
                });

            self.outputs.map_output_at(&output, new_position);

            // By passing new_output as an Option, rather than mapping it into a bogus location
            // in global_space, we ensure that this branch always runs for it.
            if Some(new_position) != position {
                debug!(
                    "putting output {} at x={} y={}",
                    name.connector, new_position.x, new_position.y
                );
                output.change_current_state(None, None, None, Some(new_position));
                self.ipc_outputs_changed = true;
                self.queue_redraw(&output);
            }
        }
    }

    pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool) {
        let global = output.create_global::<State>(&self.display_handle);

        let name = output.user_data().get::<OutputName>().unwrap();

        let config = self.config.borrow();
        let c = config.outputs.find(name);
        let scale = c.and_then(|c| c.scale).map(|s| s.0).unwrap_or_else(|| {
            let size_mm = output.physical_properties().size;
            let resolution = output.current_mode().unwrap().size;
            guess_monitor_scale(size_mm, resolution)
        });
        let scale = closest_representable_scale(scale.clamp(0.1, 10.));

        let mut transform = panel_orientation(&output)
            + c.map(|c| ipc_transform_to_smithay(c.transform))
                .unwrap_or(Transform::Normal);

        let mut backdrop_color = c
            .and_then(|c| c.backdrop_color)
            .unwrap_or(niri_config::appearance::DEFAULT_BACKDROP_COLOR)
            .to_array_unpremul();
        backdrop_color[3] = 1.;

        // FIXME: fix winit damage on other transforms.
        if name.connector == "winit" {
            transform = Transform::Flipped180;
        }

        let mut layout_config = c.and_then(|c| c.layout.clone());
        // Support the deprecated non-layout background-color key.
        if let Some(layout) = &mut layout_config {
            if layout.background_color.is_none() {
                layout.background_color = c.and_then(|c| c.background_color);
            }
        }
        drop(config);

        // Set scale and transform before adding to the layout since that will read the output size.
        output.change_current_state(
            None,
            Some(transform),
            Some(OutputScale::Fractional(scale)),
            None,
        );

        self.layout.add_output(output.clone(), layout_config);

        // TEAM_042: Ensure named rows from config are created on the correct output.
        // TEAM_055: Renamed from workspaces to rows
        // This is needed because Layout::new doesn't create named rows, and they need
        // to be created after outputs are added so they can be placed on the correct output.
        // Only create rows that target this specific output or have no target output.
        let config = self.config.borrow();
        for row_config in &config.rows {
            let targets_this_output = row_config
                .open_on_output
                .as_deref()
                .is_none_or(|target_name| output_matches_name(&output, target_name));
            if targets_this_output {
                self.layout.ensure_named_row(row_config);
            }
        }
        drop(config);

        let lock_render_state = if self.is_locked() {
            // We haven't rendered anything yet so it's as good as locked.
            LockRenderState::Locked
        } else {
            LockRenderState::Unlocked
        };

        let size = output_size(&output);
        let state = OutputState {
            global,
            redraw_state: RedrawState::Idle,
            on_demand_vrr_enabled: false,
            unfinished_animations_remain: false,
            frame_clock: FrameClock::new(refresh_interval, vrr),
            last_drm_sequence: None,
            vblank_throttle: VBlankThrottle::new(self.event_loop.clone(), name.connector.clone()),
            frame_callback_sequence: 0,
            backdrop_buffer: SolidColorBuffer::new(size, backdrop_color),
            lock_render_state,
            lock_surface: None,
            lock_color_buffer: SolidColorBuffer::new(size, CLEAR_COLOR_LOCKED),
            screen_transition: None,
            debug_damage_tracker: OutputDamageTracker::from_output(&output),
        };
        let rv = self.outputs.insert_state(output.clone(), state);
        assert!(rv.is_none(), "output was already tracked");

        // Must be last since it will call queue_redraw(output) which needs things to be filled-in.
        self.reposition_outputs(Some(&output));
    }

    pub fn remove_output(&mut self, output: &Output) {
        for layer in layer_map_for_output(output).layers() {
            layer.layer_surface().send_close();
        }

        self.layout.remove_output(output);
        self.outputs.space_mut().unmap_output(output);
        self.reposition_outputs(None);
        self.protocols.gamma_control.output_removed(output);

        let state = self.outputs.remove_state(output).unwrap();

        match state.redraw_state {
            RedrawState::Idle => (),
            RedrawState::Queued => (),
            RedrawState::WaitingForVBlank { .. } => (),
            RedrawState::WaitingForEstimatedVBlank(token) => self.event_loop.remove(token),
            RedrawState::WaitingForEstimatedVBlankAndQueued(token) => self.event_loop.remove(token),
        }

        #[cfg(feature = "xdp-gnome-screencast")]
        self.stop_casts_for_target(CastTarget::Output(output.downgrade()));

        self.remove_screencopy_output(output);

        // Disable the output global and remove some time later to give the clients some time to
        // process it.
        let global = state.global;
        self.display_handle.disable_global::<State>(global.clone());
        self.event_loop
            .insert_source(
                Timer::from_duration(Duration::from_secs(10)),
                move |_, _, state| {
                    state
                        .niri
                        .display_handle
                        .remove_global::<State>(global.clone());
                    TimeoutAction::Drop
                },
            )
            .unwrap();

        match mem::take(&mut self.lock_state) {
            LockState::Locking(confirmation) => {
                // We're locking and an output was removed, check if the requirements are now met.
                let all_locked = self
                    .outputs
                    .states()
                    .all(|state| state.lock_render_state == LockRenderState::Locked);

                if all_locked {
                    let lock = confirmation.ext_session_lock().clone();
                    confirmation.lock();
                    self.lock_state = LockState::Locked(lock);
                } else {
                    // Still waiting.
                    self.lock_state = LockState::Locking(confirmation);
                }
            }
            lock_state => {
                self.lock_state = lock_state;
                self.maybe_continue_to_locking();
            }
        }

        if self.ui.screenshot.close() {
            self.cursor
                .manager_mut()
                .set_cursor_image(CursorImageStatus::default_named());
            self.queue_redraw_all();
        }

        if self.ui.mru.output() == Some(output) {
            self.cancel_mru();
        }
    }

    // output_resized, deactivate_monitors, activate_monitors, output_under moved to output.rs
    // is_inside_hot_corner, is_sticky_obscured_under, is_layout_obscured_under,
    // row_under, row_under_cursor, window_under, window_under_cursor, contents_under
    // moved to hit_test.rs

    // output_under_cursor, output_left_of, output_right_of, output_up_of, output_down_of,
    // output_previous_of, output_next_of, output_left, output_right, output_up, output_down,
    // output_previous, output_next moved to output.rs

    pub fn find_output_and_workspace_index(
        &self,
        workspace_reference: WorkspaceReference,
    ) -> Option<(Option<Output>, usize)> {
        let (target_row_index, _target_row) = match workspace_reference {
            WorkspaceReference::Index(index) => {
                return Some((None, index.saturating_sub(1) as usize));
            }
            WorkspaceReference::Name(name) => self.layout.find_workspace_by_name(&name)?,
            WorkspaceReference::Id(id) => {
                let id = WorkspaceId::specific(id);
                self.layout.find_workspace_by_id(id)?
            }
        };

        let target_output = None; // TODO: TEAM_023: Get output from monitor when workspace methods return monitor info
        Some((target_output, target_row_index as usize))
    }

    pub fn find_window_by_id(&self, id: MappedId) -> Option<Window> {
        self.layout
            .windows()
            .find(|(_, m)| m.id() == id)
            .map(|(_, m)| m.window.clone())
    }

    // output_for_tablet, output_for_touch, output_by_name_match, output_for_root moved to output.rs
    // lock_surface_focus moved to lock.rs

    // TEAM_083: queue_redraw_all, queue_redraw, redraw_queued_outputs moved to render.rs
    // pointer_element, refresh_pointer_outputs, refresh_layout, refresh_idle_inhibit,
    // refresh_window_states, refresh_window_rules moved to render.rs
    // refresh_mapped_cast_window_rules, refresh_mapped_cast_outputs moved to screencast.rs
    // advance_animations, update_render_elements, update_shaders moved to render.rs
    // TEAM_083: render, render_layer, redraw, refresh_on_demand_vrr moved to render.rs
    // TEAM_083: update_primary_scanout_output, send_dmabuf_feedbacks, debug_toggle_damage moved to
    // render.rs

    // NOTE: The render methods were previously here but are now in render.rs
    // This is actual code movement, not just comments claiming it happened

    // send_frame_callbacks, send_frame_callbacks_on_fallback_timer,
    // take_presentation_feedbacks moved to frame_callbacks.rs

    // render_for_screen_cast, render_windows_for_screen_cast,
    // stop_cast, stop_casts_for_target moved to screencast.rs

    // render_for_screencopy_with_damage, render_for_screencopy_without_damage,
    // render_for_screencopy_internal, remove_screencopy_output moved to screencopy.rs

    /// Tries to find and return the root shell surface for a given surface.
    ///
    /// I.e. for popups, this function will try to find the parent toplevel or layer surface. For
    /// regular subsurfaces, it will find the root surface.
    pub fn find_root_shell_surface(&self, surface: &WlSurface) -> WlSurface {
        let Some(root) = self.root_surface.get(surface) else {
            return surface.clone();
        };

        if let Some(popup) = self.protocols.popups.find_popup(root) {
            return find_popup_root_surface(&popup).unwrap_or_else(|_| root.clone());
        }

        root.clone()
    }

    #[cfg(feature = "dbus")]
    pub fn on_ipc_outputs_changed(&self) {
        let _span = tracy_client::span!("Niri::on_ipc_outputs_changed");

        let Some(dbus) = &self.dbus else { return };
        let Some(conn_display_config) = dbus.conn_display_config.clone() else {
            return;
        };

        let res = thread::Builder::new()
            .name("DisplayConfig MonitorsChanged Emitter".to_owned())
            .spawn(move || {
                use crate::dbus::mutter_display_config::DisplayConfig;
                let _span = tracy_client::span!("MonitorsChanged");
                let iface = match conn_display_config
                    .object_server()
                    .interface::<_, DisplayConfig>("/org/gnome/Mutter/DisplayConfig")
                {
                    Ok(iface) => iface,
                    Err(err) => {
                        warn!("error getting DisplayConfig interface: {err:?}");
                        return;
                    }
                };

                async_io::block_on(async move {
                    if let Err(err) = DisplayConfig::monitors_changed(iface.signal_emitter()).await
                    {
                        warn!("error emitting MonitorsChanged: {err:?}");
                    }
                });
            });

        if let Err(err) = res {
            warn!("error spawning a thread to send MonitorsChanged: {err:?}");
        }
    }

    pub fn do_screen_transition(&mut self, renderer: &mut GlesRenderer, delay_ms: Option<u16>) {
        let _span = tracy_client::span!("Niri::do_screen_transition");

        self.update_render_elements(None);

        let textures: Vec<_> = self
            .outputs
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|output| {
                let size = output.current_mode().unwrap().size;
                let transform = output.current_transform();

                let scale = Scale::from(output.current_scale().fractional_scale());
                let targets = [
                    RenderTarget::Output,
                    RenderTarget::Screencast,
                    RenderTarget::ScreenCapture,
                ];
                let textures = targets.map(|target| {
                    let elements = self.render::<GlesRenderer>(renderer, &output, false, target);
                    let elements = elements.iter().rev();

                    let res = render_to_texture(
                        renderer,
                        size,
                        scale,
                        transform,
                        Fourcc::Abgr8888,
                        elements,
                    );

                    if let Err(err) = &res {
                        warn!("error rendering output {}: {err:?}", output.name());
                    }

                    res
                });

                if textures.iter().any(|res| res.is_err()) {
                    return None;
                }

                let textures = textures.map(|res| {
                    let texture = res.unwrap().0;
                    TextureBuffer::from_texture(
                        renderer,
                        texture,
                        scale,
                        transform,
                        Vec::new(), // We want windows below to get frame callbacks.
                    )
                });

                Some((output, textures))
            })
            .collect();

        let delay = delay_ms.map_or(screen_transition::DELAY, |d| {
            Duration::from_millis(u64::from(d))
        });

        for (output, from_texture) in textures {
            let state = self.outputs.state_mut(&output).unwrap();
            state.screen_transition = Some(ScreenTransition::new(
                from_texture,
                delay,
                self.clock.clone(),
            ));
        }

        // We don't actually need to queue a redraw because the point is to freeze the screen for a
        // bit, and even if the delay was zero, we're drawing the same contents anyway.
    }

    // TEAM_083: Removed duplicate render code - all render methods are now in render.rs
    // recompute_window_rules, recompute_layer_rules moved to rules.rs
    // close_mru, cancel_mru, mru_apply_keyboard_commit, queue_redraw_mru_output moved to mru.rs
}

pub struct NewClient {
    pub client: UnixStream,
    pub restricted: bool,
    pub credentials_unknown: bool,
}

pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub can_view_decoration_globals: bool,
    pub primary_selection_disabled: bool,
    /// Whether this client is denied from the restricted protocols such as security-context.
    pub restricted: bool,
    /// We cannot retrieve this client's socket credentials.
    pub credentials_unknown: bool,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

#[allow(dead_code)]
fn scale_relocate_crop<E: Element>(
    elem: E,
    output_scale: Scale<f64>,
    zoom: f64,
    ws_geo: Rectangle<f64, Logical>,
) -> Option<CropRenderElement<RelocateRenderElement<RescaleRenderElement<E>>>> {
    let ws_geo = ws_geo.to_physical_precise_round(output_scale);
    let elem = RescaleRenderElement::from_element(elem, Point::from((0, 0)), zoom);
    let elem = RelocateRenderElement::from_element(elem, ws_geo.loc, Relocate::Relative);
    CropRenderElement::from_element(elem, output_scale, ws_geo)
}

niri_render_elements! {
    OutputRenderElements<R> => {
        Monitor = MonitorRenderElement<R>,
        RescaledTile = RescaleRenderElement<TileRenderElement<R>>,
        LayerSurface = LayerSurfaceRenderElement<R>,
        RelocatedLayerSurface = CropRenderElement<RelocateRenderElement<RescaleRenderElement<
            LayerSurfaceRenderElement<R>
        >>>,
        Wayland = WaylandSurfaceRenderElement<R>,
        NamedPointer = MemoryRenderBufferRenderElement<R>,
        SolidColor = SolidColorRenderElement,
        ScreenshotUi = ScreenshotUiRenderElement,
        WindowMruUi = WindowMruUiRenderElement<R>,
        ExitConfirmDialog = ExitConfirmDialogRenderElement,
        Texture = PrimaryGpuTextureRenderElement,
        // Used for the CPU-rendered panels.
        RelocatedMemoryBuffer = RelocateRenderElement<MemoryRenderBufferRenderElement<R>>,
    }
}
