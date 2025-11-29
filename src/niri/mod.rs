//! Niri compositor state.
//!
//! This module contains the main `Niri` struct and related types.

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

pub use protocols::ProtocolStates;
pub use subsystems::{CursorSubsystem, FocusState, OutputSubsystem};
pub use types::*;

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, mem, thread};

use _server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as KdeDecorationsMode;
use anyhow::{bail, ensure, Context};
use calloop::futures::Scheduler;
use niri_config::debug::PreviewRender;
use niri_config::{
    Config, FloatOrInt, Key, Modifiers, OutputName, TrackLayout, WarpMouseToFocusMode,
    WorkspaceReference, Xkb,
};
use smithay::backend::allocator::Fourcc;
use smithay::backend::input::Keycode;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::element::surface::{
    render_elements_from_surface_tree, WaylandSurfaceRenderElement,
};
use smithay::backend::renderer::element::utils::{
    select_dmabuf_feedback, CropRenderElement, Relocate, RelocateRenderElement,
    RescaleRenderElement,
};
use smithay::backend::renderer::element::{
    default_primary_scanout_output_compare, Element, Id, Kind, PrimaryScanoutOutput,
    RenderElementStates,
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::sync::SyncPoint;
use smithay::backend::renderer::Color32F;
use smithay::desktop::utils::{
    bbox_from_surface_tree, output_update, send_dmabuf_feedback_surface_tree,
    send_frames_surface_tree, surface_presentation_feedback_flags_from_states,
    surface_primary_scanout_output, take_presentation_feedback_surface_tree,
    under_from_surface_tree, update_surface_primary_scanout_output, OutputPresentationFeedback,
};
use smithay::desktop::{
    find_popup_root_surface, layer_map_for_output, LayerMap, LayerSurface, PopupGrab, PopupManager,
    PopupUngrabStrategy, Space, Window, WindowSurfaceType,
};
use smithay::input::keyboard::{Layout as KeyboardLayout, XkbConfig};
use smithay::input::pointer::{
    CursorIcon, CursorImageStatus, CursorImageSurfaceData, Focus,
    GrabStartData as PointerGrabStartData, MotionEvent,
};
use smithay::input::{Seat, SeatState};
use smithay::output::{self as smithay_output, Output, OutputModeSource, PhysicalProperties, Scale as OutputScale, Subpixel, WeakOutput};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::calloop::{
    Interest, LoopHandle, LoopSignal, Mode, PostAction, RegistrationToken,
};
use smithay::reexports::wayland_protocols::ext::session_lock::v1::server::ext_session_lock_v1::ExtSessionLockV1;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities;
use smithay::reexports::wayland_protocols_misc::server_decoration as _server_decoration;
use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use smithay::reexports::wayland_server::backend::{
    ClientData, ClientId, DisconnectReason, GlobalId,
};
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Display, DisplayHandle, Resource};
use smithay::utils::{
    ClockSource, IsAlive as _, Logical, Monotonic, Physical, Point, Rectangle, Scale, Size,
    Transform, SERIAL_COUNTER,
};
use smithay::wayland::compositor::{
    with_states, with_surface_tree_downward, CompositorClientState, CompositorHandler,
    CompositorState, HookId, SurfaceData, TraversalAction,
};
use smithay::wayland::cursor_shape::CursorShapeManagerState;
use smithay::wayland::dmabuf::DmabufState;
use smithay::wayland::fractional_scale::FractionalScaleManagerState;
use smithay::wayland::idle_inhibit::IdleInhibitManagerState;
use smithay::wayland::idle_notify::IdleNotifierState;
use smithay::wayland::input_method::{InputMethodManagerState, InputMethodSeat};
use smithay::wayland::keyboard_shortcuts_inhibit::{
    KeyboardShortcutsInhibitState, KeyboardShortcutsInhibitor,
};
use smithay::wayland::output::OutputManagerState;
use smithay::wayland::pointer_constraints::{with_pointer_constraint, PointerConstraintsState};
use smithay::wayland::pointer_gestures::PointerGesturesState;
use smithay::wayland::presentation::PresentationState;
use smithay::wayland::relative_pointer::RelativePointerManagerState;
use smithay::wayland::security_context::SecurityContextState;
use smithay::wayland::selection::data_device::{set_data_device_selection, DataDeviceState};
use smithay::wayland::selection::ext_data_control::DataControlState as ExtDataControlState;
use smithay::wayland::selection::primary_selection::PrimarySelectionState;
use smithay::wayland::selection::wlr_data_control::DataControlState as WlrDataControlState;
use smithay::wayland::session_lock::{LockSurface, SessionLockManagerState, SessionLocker};
use smithay::wayland::shell::kde::decoration::KdeDecorationState;
use smithay::wayland::shell::wlr_layer::{self, Layer, WlrLayerShellState};
use smithay::wayland::shell::xdg::decoration::XdgDecorationState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
#[cfg(test)]
use smithay::wayland::single_pixel_buffer::SinglePixelBufferState;
use smithay::wayland::socket::ListeningSocketSource;
use smithay::wayland::tablet_manager::TabletManagerState;
use smithay::wayland::text_input::TextInputManagerState;
use smithay::wayland::viewporter::ViewporterState;
use smithay::wayland::virtual_keyboard::VirtualKeyboardManagerState;
use smithay::wayland::xdg_activation::XdgActivationState;
use smithay::wayland::xdg_foreign::XdgForeignState;

#[cfg(feature = "dbus")]
use crate::a11y::A11y;
use crate::animation::Clock;
use crate::backend::tty::SurfaceDmabufFeedback;
use crate::backend::{Backend, Headless, RenderResult, Tty, Winit};
use crate::cursor::{CursorManager, CursorTextureCache, RenderCursor, XCursor};
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_locale1::Locale1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::freedesktop_login1::Login1ToNiri;
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_introspect::{self, IntrospectToNiri, NiriToIntrospect};
#[cfg(feature = "dbus")]
use crate::dbus::gnome_shell_screenshot::{NiriToScreenshot, ScreenshotToNiri};
#[cfg(feature = "xdp-gnome-screencast")]
use crate::dbus::mutter_screen_cast::{self, ScreenCastToNiri};
use crate::frame_clock::FrameClock;
use crate::handlers::{configure_lock_surface, XDG_ACTIVATION_TOKEN_TIMEOUT};
use crate::input::pick_color_grab::PickColorGrab;
use crate::input::scroll_swipe_gesture::ScrollSwipeGesture;
use crate::input::scroll_tracker::ScrollTracker;
use crate::input::{
    apply_libinput_settings, mods_with_finger_scroll_binds, mods_with_mouse_binds,
    mods_with_wheel_binds, TabletData,
};
use crate::ipc::server::IpcServer;
use crate::layer::mapped::LayerSurfaceRenderElement;
use crate::layer::MappedLayer;
use crate::layout::tile::TileRenderElement;
// TEAM_060: Using RowId directly instead of WorkspaceId alias
use crate::layout::row_types::RowId as WorkspaceId;
use crate::layout::{HitType, Layout, LayoutElement as _, MonitorRenderElement};
use crate::niri_render_elements;
use crate::protocols::ext_workspace::{self, ExtWorkspaceManagerState};
use crate::protocols::foreign_toplevel::{self, ForeignToplevelManagerState};
use crate::protocols::gamma_control::GammaControlManagerState;
use crate::protocols::mutter_x11_interop::MutterX11InteropManagerState;
use crate::protocols::output_management::OutputManagementManagerState;
use crate::protocols::screencopy::{Screencopy, ScreencopyBuffer, ScreencopyManagerState};
use crate::protocols::virtual_pointer::VirtualPointerManagerState;
use crate::pw_utils::{Cast, PipeWire};
#[cfg(feature = "xdp-gnome-screencast")]
use crate::pw_utils::{CastSizeChange, PwToNiri};
use crate::render_helpers::debug::draw_opaque_regions;
use crate::render_helpers::primary_gpu_texture::PrimaryGpuTextureRenderElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::{
    encompassing_geo, render_to_dmabuf, render_to_encompassing_texture, render_to_shm,
    render_to_texture, render_to_vec, shaders, RenderTarget, SplitElements,
};
use crate::ui::config_error_notification::ConfigErrorNotification;
use crate::ui::exit_confirm_dialog::{ExitConfirmDialog, ExitConfirmDialogRenderElement};
use crate::ui::hotkey_overlay::HotkeyOverlay;
use crate::ui::mru::{MruCloseRequest, WindowMruUi, WindowMruUiRenderElement};
use crate::ui::screen_transition::{self, ScreenTransition};
use crate::ui::screenshot_ui::{OutputScreenshot, ScreenshotUi, ScreenshotUiRenderElement};
use crate::utils::scale::{closest_representable_scale, guess_monitor_scale};
use crate::utils::spawning::{CHILD_DISPLAY, CHILD_ENV};
use crate::utils::vblank_throttle::VBlankThrottle;
use crate::utils::watcher::Watcher;
use crate::utils::xwayland::satellite::Satellite;
use crate::utils::{
    center, center_f64, expand_home, get_monotonic_time, ipc_transform_to_smithay, is_mapped,
    logical_output, make_screenshot_path, output_matches_name, output_size, panel_orientation,
    send_scale_transform, write_png_rgba8, xwayland,
};
use crate::window::mapped::MappedId;
use crate::window::{InitialConfigureState, Mapped, ResolvedWindowRules, Unmapped, WindowRef};

const CLEAR_COLOR_LOCKED: [f32; 4] = [0.3, 0.1, 0.1, 1.];

// We'll try to send frame callbacks at least once a second. We'll make a timer that fires once a
// second, so with the worst timing the maximum interval between two frame callbacks for a surface
// should be ~1.995 seconds.
const FRAME_CALLBACK_THROTTLE: Option<Duration> = Some(Duration::from_millis(995));

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
    
    pub gesture_swipe_3f_cumulative: Option<(f64, f64)>,
    pub overview_scroll_swipe_gesture: ScrollSwipeGesture,
    pub vertical_wheel_tracker: ScrollTracker,
    pub horizontal_wheel_tracker: ScrollTracker,
    pub mods_with_mouse_binds: HashSet<Modifiers>,
    pub mods_with_wheel_binds: HashSet<Modifiers>,
    pub vertical_finger_scroll_tracker: ScrollTracker,
    pub horizontal_finger_scroll_tracker: ScrollTracker,
    pub mods_with_finger_scroll_binds: HashSet<Modifiers>,

    pub lock_state: LockState,

    // State that we last sent to the logind LockedHint.
    pub locked_hint: Option<bool>,

    pub screenshot_ui: ScreenshotUi,
    pub config_error_notification: ConfigErrorNotification,
    pub hotkey_overlay: HotkeyOverlay,
    pub exit_confirm_dialog: ExitConfirmDialog,

    pub window_mru_ui: WindowMruUi,
    pub pending_mru_commit: Option<PendingMruCommit>,

    pub pick_window: Option<async_channel::Sender<Option<MappedId>>>,
    pub pick_color: Option<async_channel::Sender<Option<niri_ipc::PickedColor>>>,

    pub debug_draw_opaque_regions: bool,
    pub debug_draw_damage: bool,

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

    // Casts are dropped before PipeWire to prevent a double-free (yay).
    pub casts: Vec<Cast>,
    pub pipewire: Option<PipeWire>,
    #[cfg(feature = "xdp-gnome-screencast")]
    pub pw_to_niri: calloop::channel::Sender<PwToNiri>,

    // Screencast output for each mapped window.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub mapped_cast_output: HashMap<Window, Output>,

    /// Window ID for the "dynamic cast" special window for the xdp-gnome picker.
    #[cfg(feature = "xdp-gnome-screencast")]
    pub dynamic_cast_id_for_portal: MappedId,
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
        if self.niri.is_lid_closed == is_closed {
            return;
        }

        debug!("laptop lid {}", if is_closed { "closed" } else { "opened" });
        self.niri.is_lid_closed = is_closed;
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

        self.niri.cursor.manager().check_cursor_image_surface_alive();
        self.niri.refresh_pointer_outputs();
        self.niri.global_space.refresh();
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

        if self.niri.tablet_cursor_location.is_some() {
            return false;
        }

        let Some(output) = self.niri.layout.active_output() else {
            return false;
        };
        let monitor = self.niri.layout.monitor_for_output(output).unwrap();

        let mut rv = false;
        let rect = monitor.active_tile_visual_rectangle();

        if let Some(rect) = rect {
            let output_geo = self.niri.global_space.output_geometry(output).unwrap();
            let mut rect = rect;
            rect.loc += output_geo.loc.to_f64();
            rv = self.move_cursor_to_rect(rect, mode);
        }

        rv
    }

    pub fn focus_default_monitor(&mut self) {
        // Our default target is the first output in sorted order.
        let Some(mut target) = self.niri.sorted_outputs.first().cloned() else {
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

        if !self.niri.exit_confirm_dialog.is_open()
            && !self.niri.is_locked()
            && !self.niri.screenshot_ui.is_open()
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
        let geo = self.niri.global_space.output_geometry(output).unwrap();
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

    pub fn update_keyboard_focus(&mut self) {
        // Clean up on-demand layer surface focus if necessary.
        if let Some(surface) = &self.niri.focus.layer_on_demand() {
            // Still alive and has on-demand interactivity.
            let mut good = surface.alive()
                && surface.cached_state().keyboard_interactivity
                    == wlr_layer::KeyboardInteractivity::OnDemand;

            if let Some(mapped) = self.niri.mapped_layer_surfaces.get(surface) {
                // Check if it moved to the overview backdrop.
                if mapped.place_within_backdrop() {
                    good = false;
                }
            } else {
                // The layer surface is alive but it got unmapped.
                good = false;
            }

            if !good {
                self.niri.focus.set_layer_on_demand(None);
            }
        }

        // Compute the current focus.
        let focus = if self.niri.exit_confirm_dialog.is_open() {
            KeyboardFocus::ExitConfirmDialog
        } else if self.niri.is_locked() {
            KeyboardFocus::LockScreen {
                surface: self.niri.lock_surface_focus(),
            }
        } else if self.niri.screenshot_ui.is_open() {
            KeyboardFocus::ScreenshotUi
        } else if self.niri.window_mru_ui.is_open() {
            KeyboardFocus::Mru
        } else if let Some(output) = self.niri.layout.active_output() {
            let mon = self.niri.layout.monitor_for_output(output).unwrap();
            let layers = layer_map_for_output(output);

            // Explicitly check for layer-shell popup grabs here, our keyboard focus will stay on
            // the root layer surface while it has grabs.
            let layer_grab = self.niri.popup_grab.as_ref().and_then(|g| {
                layers
                    .layer_for_surface(&g.root, WindowSurfaceType::TOPLEVEL)
                    .and_then(|l| l.can_receive_keyboard_focus().then(|| (&g.root, l.layer())))
            });
            let grab_on_layer = |layer: Layer| {
                layer_grab
                    .and_then(move |(s, l)| if l == layer { Some(s.clone()) } else { None })
                    .map(|surface| KeyboardFocus::LayerShell { surface })
            };

            let layout_focus = || {
                self.niri
                    .layout
                    .focus()
                    .map(|win| win.toplevel().wl_surface().clone())
                    .map(|surface| KeyboardFocus::Layout {
                        surface: Some(surface),
                    })
            };

            let excl_focus_on_layer = |layer| {
                layers.layers_on(layer).find_map(|surface| {
                    if surface.cached_state().keyboard_interactivity
                        != wlr_layer::KeyboardInteractivity::Exclusive
                    {
                        return None;
                    }

                    let mapped = self.niri.mapped_layer_surfaces.get(surface)?;
                    if mapped.place_within_backdrop() {
                        return None;
                    }

                    let surface = surface.wl_surface().clone();
                    Some(KeyboardFocus::LayerShell { surface })
                })
            };

            let on_d_focus_on_layer = |layer| {
                layers.layers_on(layer).find_map(|surface| {
                    let is_on_demand_surface =
                        Some(surface) == self.niri.focus.layer_on_demand();
                    is_on_demand_surface
                        .then(|| surface.wl_surface().clone())
                        .map(|surface| KeyboardFocus::LayerShell { surface })
                })
            };

            // Prefer exclusive focus on a layer, then check on-demand focus.
            let focus_on_layer =
                |layer| excl_focus_on_layer(layer).or_else(|| on_d_focus_on_layer(layer));

            // Overview mode has been removed, this is always false
            let is_overview_open = false;

            let mut surface = grab_on_layer(Layer::Overlay);
            // FIXME: we shouldn't prioritize the top layer grabs over regular overlay input or a
            // fullscreen layout window. This will need tracking in grab() to avoid handing it out
            // in the first place. Or a better way to structure this code.
            surface = surface.or_else(|| grab_on_layer(Layer::Top));

            if !is_overview_open {
                surface = surface.or_else(|| grab_on_layer(Layer::Bottom));
                surface = surface.or_else(|| grab_on_layer(Layer::Background));
            }

            surface = surface.or_else(|| focus_on_layer(Layer::Overlay));

            if mon.render_above_top_layer() {
                surface = surface.or_else(layout_focus);
                surface = surface.or_else(|| focus_on_layer(Layer::Top));
                surface = surface.or_else(|| focus_on_layer(Layer::Bottom));
                surface = surface.or_else(|| focus_on_layer(Layer::Background));
            } else {
                surface = surface.or_else(|| focus_on_layer(Layer::Top));

                if is_overview_open {
                    surface = Some(surface.unwrap_or(KeyboardFocus::Overview));
                }

                surface = surface.or_else(|| on_d_focus_on_layer(Layer::Bottom));
                surface = surface.or_else(|| on_d_focus_on_layer(Layer::Background));
                surface = surface.or_else(layout_focus);

                // Bottom and background layers can only receive exclusive focus when there are no
                // layout windows.
                surface = surface.or_else(|| excl_focus_on_layer(Layer::Bottom));
                surface = surface.or_else(|| excl_focus_on_layer(Layer::Background));
            }

            surface.unwrap_or(KeyboardFocus::Layout { surface: None })
        } else {
            KeyboardFocus::Layout { surface: None }
        };

        let keyboard = self.niri.seat.get_keyboard().unwrap();
        if *self.niri.focus.current() != focus {
            trace!(
                "keyboard focus changed from {:?} to {:?}",
                self.niri.focus.current(),
                focus
            );

            // Tell the windows their new focus state for window rule purposes.
            if let KeyboardFocus::Layout {
                surface: Some(surface),
            } = &self.niri.focus.current()
            {
                if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                    mapped.set_is_focused(false);
                }
            }
            if let KeyboardFocus::Layout {
                surface: Some(surface),
            } = &focus
            {
                if let Some((mapped, _)) = self.niri.layout.find_window_and_output_mut(surface) {
                    mapped.set_is_focused(true);

                    // If `mapped` does not have a focus timestamp, then the window is newly
                    // created/mapped and a timestamp is unconditionally created.
                    //
                    // If `mapped` already has a timestamp only update it after the focus lock-in
                    // period has gone by without the focus having elsewhere.
                    let stamp = get_monotonic_time();

                    let debounce = self.niri.config.borrow().recent_windows.debounce_ms;
                    let debounce = Duration::from_millis(u64::from(debounce));

                    if mapped.get_focus_timestamp().is_none() || debounce.is_zero() {
                        mapped.set_focus_timestamp(stamp);
                    } else {
                        let timer = Timer::from_duration(debounce);

                        let focus_token = self
                            .niri
                            .event_loop
                            .insert_source(timer, move |_, _, state| {
                                state.niri.mru_apply_keyboard_commit();
                                TimeoutAction::Drop
                            })
                            .unwrap();
                        if let Some(PendingMruCommit { token, .. }) =
                            self.niri.pending_mru_commit.replace(PendingMruCommit {
                                id: mapped.id(),
                                token: focus_token,
                                stamp,
                            })
                        {
                            self.niri.event_loop.remove(token);
                        }
                    }
                }
            }

            if let Some(grab) = self.niri.popup_grab.as_mut() {
                if grab.has_keyboard_grab && Some(&grab.root) != focus.surface() {
                    trace!(
                        "grab root {:?} is not the new focus {:?}, ungrabbing",
                        grab.root,
                        focus
                    );

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

            if self.niri.config.borrow().input.keyboard.track_layout == TrackLayout::Window {
                let current_layout = keyboard.with_xkb_state(self, |context| {
                    let xkb = context.xkb().lock().unwrap();
                    xkb.active_layout()
                });

                let mut new_layout = current_layout;
                // Store the currently active layout for the surface.
                if let Some(current_focus) = self.niri.keyboard_focus.surface() {
                    with_states(current_focus, |data| {
                        let cell = data
                            .data_map
                            .get_or_insert::<Cell<KeyboardLayout>, _>(Cell::default);
                        cell.set(current_layout);
                    });
                }

                if let Some(focus) = focus.surface() {
                    new_layout = with_states(focus, |data| {
                        let cell = data.data_map.get_or_insert::<Cell<KeyboardLayout>, _>(|| {
                            // The default layout is effectively the first layout in the
                            // keymap, so use it for new windows.
                            Cell::new(KeyboardLayout::default())
                        });
                        cell.get()
                    });
                }
                if new_layout != current_layout && focus.surface().is_some() {
                    keyboard.set_focus(self, None, SERIAL_COUNTER.next_serial());
                    keyboard.with_xkb_state(self, |mut context| {
                        context.set_layout(new_layout);
                    });
                }
            }

            self.niri.focus.set_current(focus.clone());
            keyboard.set_focus(self, focus.into_surface(), SERIAL_COUNTER.next_serial());

            // FIXME: can be more granular.
            self.niri.queue_redraw_all();
        }
    }

    /// Loads the xkb keymap from a file config setting.
    fn set_xkb_file(&mut self, xkb_file: String) -> anyhow::Result<()> {
        let xkb_file = PathBuf::from(xkb_file);
        let xkb_file = expand_home(&xkb_file)
            .context("failed to expand ~")?
            .unwrap_or(xkb_file);

        let keymap = std::fs::read_to_string(xkb_file).context("failed to read xkb_file")?;

        let xkb = self.niri.seat.get_keyboard().unwrap();
        xkb.set_keymap_from_string(self, keymap)
            .context("failed to set keymap")?;

        Ok(())
    }

    fn load_xkb_file(&mut self) {
        let xkb_file = self.niri.config.borrow().input.keyboard.xkb.file.clone();
        if let Some(xkb_file) = xkb_file {
            if let Err(err) = self.set_xkb_file(xkb_file) {
                warn!("error loading xkb_file: {err:?}");
            }
        }
    }

    fn set_xkb_config(&mut self, xkb: XkbConfig) {
        let keyboard = self.niri.seat.get_keyboard().unwrap();
        let num_lock = keyboard.modifier_state().num_lock;
        if let Err(err) = keyboard.set_xkb_config(self, xkb) {
            warn!("error updating xkb config: {err:?}");
            return;
        }

        // Restore num lock to its previous value.
        let mut mods_state = keyboard.modifier_state();
        if mods_state.num_lock != num_lock {
            mods_state.num_lock = num_lock;
            keyboard.set_modifier_state(mods_state);
        }
    }

    pub fn reload_config(&mut self, config: Result<Config, ()>) {
        let _span = tracy_client::span!("State::reload_config");

        let mut config = match config {
            Ok(config) => config,
            Err(()) => {
                self.niri.config_error_notification.show();
                self.niri.queue_redraw_all();

                #[cfg(feature = "dbus")]
                self.niri.a11y_announce_config_error();

                return;
            }
        };

        self.niri.config_error_notification.hide();

        // TEAM_055: Renamed from workspaces to rows
        // Find & orphan removed named rows.
        let mut removed_rows: Vec<String> = vec![];
        for row in &self.niri.config.borrow().rows {
            if !config.rows.iter().any(|r| r.name == row.name) {
                removed_rows.push(row.name.0.clone());
            }
        }
        for name in removed_rows {
            self.niri.layout.unname_workspace(&name);
        }

        self.niri.layout.update_config(&config);
        for mapped in self.niri.mapped_layer_surfaces.values_mut() {
            mapped.update_config(&config);
        }

        // TEAM_055: Renamed from workspaces to rows
        // Create new named rows.
        for row_config in &config.rows {
            self.niri.layout.ensure_named_row(row_config);
        }

        let rate = 1.0 / config.animations.slowdown.max(0.001);
        self.niri.clock.set_rate(rate);
        self.niri
            .clock
            .set_complete_instantly(config.animations.off);

        *CHILD_ENV.write().unwrap() = mem::take(&mut config.environment);

        let mut reload_xkb = None;
        let mut libinput_config_changed = false;
        let mut output_config_changed = false;
        let mut preserved_output_config = None;
        let mut window_rules_changed = false;
        let mut layer_rules_changed = false;
        let mut shaders_changed = false;
        let mut cursor_inactivity_timeout_changed = false;
        let mut recent_windows_changed = false;
        let mut xwls_changed = false;
        let mut old_config = self.niri.config.borrow_mut();

        // Reload the cursor.
        if config.cursor != old_config.cursor {
            self.niri
                .cursor_manager
                .reload(&config.cursor.xcursor_theme, config.cursor.xcursor_size);
            self.niri.cursor_texture_cache.clear();
        }

        // We need &mut self to reload the xkb config, so just store it here.
        if config.input.keyboard.xkb != old_config.input.keyboard.xkb {
            reload_xkb = Some(config.input.keyboard.xkb.clone());
        }

        // Reload the repeat info.
        if config.input.keyboard.repeat_rate != old_config.input.keyboard.repeat_rate
            || config.input.keyboard.repeat_delay != old_config.input.keyboard.repeat_delay
        {
            let keyboard = self.niri.seat.get_keyboard().unwrap();
            keyboard.change_repeat_info(
                config.input.keyboard.repeat_rate.into(),
                config.input.keyboard.repeat_delay.into(),
            );
        }

        if config.input.touchpad != old_config.input.touchpad
            || config.input.mouse != old_config.input.mouse
            || config.input.trackball != old_config.input.trackball
            || config.input.trackpoint != old_config.input.trackpoint
            || config.input.tablet != old_config.input.tablet
            || config.input.touch != old_config.input.touch
        {
            libinput_config_changed = true;
        }

        let ignored_nodes_changed =
            config.debug.ignored_drm_devices != old_config.debug.ignored_drm_devices;

        if config.outputs != self.niri.config_file_output_config {
            output_config_changed = true;
            self.niri
                .config_file_output_config
                .clone_from(&config.outputs);
        } else {
            // Output config did not change from the last disk load, so we need to preserve the
            // transient changes.
            preserved_output_config = Some(mem::take(&mut old_config.outputs));
        }

        let binds_changed = config.binds != old_config.binds;
        let new_mod_key = self.backend.mod_key(&config);
        if new_mod_key != self.backend.mod_key(&old_config) || binds_changed {
            self.niri
                .hotkey_overlay
                .on_hotkey_config_updated(new_mod_key);
            self.niri.mods_with_mouse_binds = mods_with_mouse_binds(new_mod_key, &config.binds);
            self.niri.mods_with_wheel_binds = mods_with_wheel_binds(new_mod_key, &config.binds);
            self.niri.mods_with_finger_scroll_binds =
                mods_with_finger_scroll_binds(new_mod_key, &config.binds);
        }

        if config.window_rules != old_config.window_rules {
            window_rules_changed = true;
        }

        if config.layer_rules != old_config.layer_rules {
            layer_rules_changed = true;
        }

        if config.animations.window_resize.custom_shader
            != old_config.animations.window_resize.custom_shader
        {
            let src = config.animations.window_resize.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_resize_program(renderer, src);
            });
            shaders_changed = true;
        }

        if config.animations.window_close.custom_shader
            != old_config.animations.window_close.custom_shader
        {
            let src = config.animations.window_close.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_close_program(renderer, src);
            });
            shaders_changed = true;
        }

        if config.animations.window_open.custom_shader
            != old_config.animations.window_open.custom_shader
        {
            let src = config.animations.window_open.custom_shader.as_deref();
            self.backend.with_primary_renderer(|renderer| {
                shaders::set_custom_open_program(renderer, src);
            });
            shaders_changed = true;
        }

        if config.cursor.hide_after_inactive_ms != old_config.cursor.hide_after_inactive_ms {
            cursor_inactivity_timeout_changed = true;
        }

        if config.debug.keep_laptop_panel_on_when_lid_is_closed
            != old_config.debug.keep_laptop_panel_on_when_lid_is_closed
        {
            output_config_changed = true;
        }

        if config.debug.ignored_drm_devices != old_config.debug.ignored_drm_devices {
            output_config_changed = true;
        }

        // FIXME: move backdrop rendering into layout::Monitor, then this will become unnecessary.
        // Overview mode has been removed, no backdrop color to check.
        if config.layout.background_color != old_config.layout.background_color {
            output_config_changed = true;
        }

        if config.recent_windows != old_config.recent_windows {
            recent_windows_changed = true;
        }

        if config.xwayland_satellite != old_config.xwayland_satellite {
            xwls_changed = true;
        }

        *old_config = config;

        if let Some(outputs) = preserved_output_config {
            old_config.outputs = outputs;
        }

        // Release the borrow.
        drop(old_config);

        // Now with a &mut self we can reload the xkb config.
        if let Some(mut xkb) = reload_xkb {
            let mut set_xkb_config = true;

            // It's fine to .take() the xkb file, as this is a
            // clone and the file field is not used in the XkbConfig.
            if let Some(xkb_file) = xkb.file.take() {
                if let Err(err) = self.set_xkb_file(xkb_file) {
                    warn!("error reloading xkb_file: {err:?}");
                } else {
                    // We successfully set xkb file so we don't need to fallback to XkbConfig.
                    set_xkb_config = false;
                }
            }

            if set_xkb_config {
                // If xkb is unset in the niri config, use settings from locale1.
                if xkb == Xkb::default() {
                    trace!("using xkb from locale1");
                    xkb = self.niri.xkb_from_locale1.clone().unwrap_or_default();
                }

                self.set_xkb_config(xkb.to_xkb_config());
            }

            self.ipc_keyboard_layouts_changed();
        }

        if libinput_config_changed {
            let config = self.niri.config.borrow();
            for mut device in self.niri.devices.iter().cloned() {
                apply_libinput_settings(&config.input, &mut device);
            }
        }

        if ignored_nodes_changed {
            self.backend.update_ignored_nodes_config(&mut self.niri);
        }

        if output_config_changed {
            self.reload_output_config();
        }

        if window_rules_changed {
            self.niri.recompute_window_rules();
        }

        if layer_rules_changed {
            self.niri.recompute_layer_rules();
        }

        if shaders_changed {
            self.niri.update_shaders();
        }

        if cursor_inactivity_timeout_changed {
            // Force reset due to timeout change.
            self.niri.cursor.clear_timer_reset_flag();
            self.niri.reset_pointer_inactivity_timer();
        }

        if binds_changed {
            self.niri.window_mru_ui.update_binds();
        }

        if recent_windows_changed {
            self.niri.window_mru_ui.update_config();
        }

        if xwls_changed {
            // If xwl-s was previously working and is now off, we don't try to kill it or stop
            // watching the sockets, for simplicity's sake.
            let was_working = self.niri.satellite.is_some();

            // Try to start, or restart in case the user corrected the path or something.
            xwayland::satellite::setup(self);

            let config = self.niri.config.borrow();
            let display_name = (!config.xwayland_satellite.off)
                .then_some(self.niri.satellite.as_ref())
                .flatten()
                .map(|satellite| satellite.display_name().to_owned());

            if let Some(name) = &display_name {
                if !was_working {
                    info!("listening on X11 socket: {name}");
                }
            }

            // This won't change the systemd environment, but oh well.
            *CHILD_DISPLAY.write().unwrap() = display_name;
        }

        // Can't really update xdg-decoration settings since we have to hide the globals for CSD
        // due to the SDL2 bug... I don't imagine clients are prepared for the xdg-decoration
        // global suddenly appearing? Either way, right now it's live-reloaded in a sense that new
        // clients will use the new xdg-decoration setting.

        self.niri.queue_redraw_all();
    }

    pub fn reload_output_config(&mut self) {
        let mut resized_outputs = vec![];
        let mut recolored_outputs = vec![];

        for output in self.niri.global_space.outputs() {
            let name = output.user_data().get::<OutputName>().unwrap();
            let full_config = self.niri.config.borrow_mut();
            let config = full_config.outputs.find(name);

            let scale = config
                .and_then(|c| c.scale)
                .map(|s| s.0)
                .unwrap_or_else(|| {
                    let size_mm = output.physical_properties().size;
                    let resolution = output.current_mode().unwrap().size;
                    guess_monitor_scale(size_mm, resolution)
                });
            let scale = closest_representable_scale(scale.clamp(0.1, 10.));

            let mut transform = panel_orientation(output)
                + config
                    .map(|c| ipc_transform_to_smithay(c.transform))
                    .unwrap_or(Transform::Normal);
            // FIXME: fix winit damage on other transforms.
            if name.connector == "winit" {
                transform = Transform::Flipped180;
            }

            if output.current_scale().fractional_scale() != scale
                || output.current_transform() != transform
            {
                output.change_current_state(
                    None,
                    Some(transform),
                    Some(OutputScale::Fractional(scale)),
                    None,
                );
                self.niri.ipc_outputs_changed = true;
                resized_outputs.push(output.clone());
            }

            let mut backdrop_color = config
                .and_then(|c| c.backdrop_color)
                .unwrap_or(niri_config::appearance::DEFAULT_BACKDROP_COLOR)
                .to_array_unpremul();
            backdrop_color[3] = 1.;
            let backdrop_color = Color32F::from(backdrop_color);

            if let Some(state) = self.niri.outputs.state_mut(output) {
                if state.backdrop_buffer.color() != backdrop_color {
                    state.backdrop_buffer.set_color(backdrop_color);
                    recolored_outputs.push(output.clone());
                }
            }

            for mon in self.niri.layout.monitors_mut() {
                if mon.output() != output {
                    continue;
                }

                let mut layout_config = config.and_then(|c| c.layout.clone());
                // Support the deprecated non-layout background-color key.
                if let Some(layout) = &mut layout_config {
                    if layout.background_color.is_none() {
                        layout.background_color = config.and_then(|c| c.background_color);
                    }
                }

                if mon.update_layout_config(layout_config) {
                    // Also redraw these; if anything, the background color could've changed.
                    recolored_outputs.push(output.clone());
                }
                break;
            }
        }

        for output in resized_outputs {
            self.niri.output_resized(&output);
        }

        for output in recolored_outputs {
            self.niri.queue_redraw(&output);
        }

        self.backend.on_output_config_changed(&mut self.niri);

        self.niri.reposition_outputs(None);

        if let Some(touch) = self.niri.seat.get_touch() {
            touch.cancel(self);
        }

        let config = self.niri.config.borrow().outputs.clone();
        self.niri.protocols.output_management.on_config_changed(config);
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
                .global_space
                .outputs()
                .find(|output| output.name() == ipc_output.name)
                .map(logical_output);
            ipc_output.logical = logical;
        }

        #[cfg(feature = "dbus")]
        self.niri.on_ipc_outputs_changed();

        let new_config = self.backend.ipc_outputs().lock().unwrap().clone();
        self.niri.protocols.output_management.notify_changes(new_config);
    }

    pub fn open_screenshot_ui(&mut self, show_pointer: bool, path: Option<String>) {
        if self.niri.is_locked() || self.niri.screenshot_ui.is_open() {
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
                .screenshot_ui
                .open(renderer, screenshots, default_output, show_pointer, path)
        });

        self.niri
            .cursor_manager
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
        self.niri.pick_color = Some(tx);
        self.niri
            .cursor_manager
            .set_cursor_image(CursorImageStatus::Named(CursorIcon::Crosshair));
        self.niri.queue_redraw_all();
    }

    pub fn confirm_screenshot(&mut self, write_to_disk: bool) {
        let ScreenshotUi::Open { path, .. } = &mut self.niri.screenshot_ui else {
            return;
        };
        let path = path.take();

        self.backend.with_primary_renderer(|renderer| {
            match self.niri.screenshot_ui.capture(renderer) {
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

        self.niri.screenshot_ui.close();
        self.niri
            .cursor_manager
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
                if let Some(pw) = self.niri.pipewire.take() {
                    let ids: Vec<_> = self.niri.casts.iter().map(|cast| cast.session_id).collect();
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

        let casts = &mut self.niri.casts;
        let Some(cast) = casts.iter_mut().find(|cast| cast.stream_id == stream_id) else {
            warn!("cast to redraw is missing");
            return;
        };

        match &cast.target {
            CastTarget::Nothing => {
                self.backend.with_primary_renderer(|renderer| {
                    if cast.dequeue_buffer_and_clear(renderer) {
                        cast.last_frame_time = get_monotonic_time();
                    }
                });
            }
            CastTarget::Output(weak) => {
                if let Some(output) = weak.upgrade() {
                    self.niri.queue_redraw(&output);
                }
            }
            CastTarget::Window { id } => {
                let mut windows = self.niri.layout.windows();
                let Some((_, mapped)) = windows.find(|(_, mapped)| mapped.id().get() == *id) else {
                    return;
                };

                // Use the cached output since it will be present even if the output was
                // currently disconnected.
                let Some(output) = self.niri.mapped_cast_output.get(&mapped.window) else {
                    return;
                };

                let scale = Scale::from(output.current_scale().fractional_scale());
                let bbox = mapped
                    .window
                    .bbox_with_popups()
                    .to_physical_precise_up(scale);

                match cast.ensure_size(bbox.size) {
                    Ok(CastSizeChange::Ready) => (),
                    Ok(CastSizeChange::Pending) => return,
                    Err(err) => {
                        warn!("error updating stream size, stopping screencast: {err:?}");
                        drop(windows);
                        let session_id = cast.session_id;
                        self.niri.stop_cast(session_id);
                        return;
                    }
                }

                self.backend.with_primary_renderer(|renderer| {
                    // FIXME: pointer.
                    let elements = mapped
                        .render_for_screen_cast(renderer, scale)
                        .rev()
                        .collect::<Vec<_>>();

                    if cast.dequeue_buffer_and_render(renderer, &elements, bbox.size, scale) {
                        cast.last_frame_time = get_monotonic_time();
                    }
                });
            }
        }
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
                    if let Some(output) = self.niri.mapped_cast_output.get(&mapped.window) {
                        refresh = Some(output.current_mode().unwrap().refresh as u32);
                    }
                }
            }
        }

        let mut to_redraw = Vec::new();
        let mut to_stop = Vec::new();
        for cast in &mut self.niri.casts {
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

                let pw = if let Some(pw) = &self.niri.pipewire {
                    pw
                } else {
                    match PipeWire::new(self.niri.event_loop.clone(), self.niri.pw_to_niri.clone())
                    {
                        Ok(pipewire) => self.niri.pipewire.insert(pipewire),
                        Err(err) => {
                            warn!(
                                "error starting screencast: PipeWire failed to initialize: {err:?}"
                            );
                            self.niri.stop_cast(session_id);
                            return;
                        }
                    }
                };

                let mut dynamic_target = false;
                let (target, size, refresh, alpha) = match target {
                    StreamTargetId::Output { name } => {
                        let global_space = &self.niri.global_space;
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
                        if id == self.niri.dynamic_cast_id_for_portal.get() =>
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
                        let Some(output) = self.niri.mapped_cast_output.get(window) else {
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
                        self.niri.casts.push(cast);
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
            self.niri.dynamic_cast_id_for_portal.get(),
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
        for output in self.global_space.outputs().chain(new_output) {
            let name = output.user_data().get::<OutputName>().unwrap();
            let position = self.global_space.output_geometry(output).map(|geo| geo.loc);
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
            self.global_space.unmap_output(output);
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

        self.sorted_outputs = outputs
            .iter()
            .map(|Data { output, .. }| output.clone())
            .collect();

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
                        .global_space
                        .outputs()
                        .map(|output| self.global_space.output_geometry(output).unwrap())
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
                        .global_space
                        .outputs()
                        .map(|output| self.global_space.output_geometry(output).unwrap())
                        .map(|geom| geom.loc.x + geom.size.w)
                        .max()
                        .unwrap_or(0);

                    Point::from((x, 0))
                });

            self.global_space.map_output(&output, new_position);

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
        let rv = self.outputs.state.insert(output.clone(), state);
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

        let state = self.outputs.state.remove(output).unwrap();

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
                    .outputs.state
                    .values()
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

        if self.screenshot_ui.close() {
            self.cursor.manager()
                .set_cursor_image(CursorImageStatus::default_named());
            self.queue_redraw_all();
        }

        if self.window_mru_ui.output() == Some(output) {
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
        let (target_row_index, target_row) = match workspace_reference {
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

    /// Schedules an immediate redraw on all outputs if one is not already scheduled.
    pub fn queue_redraw_all(&mut self) {
        for state in self.outputs.state.values_mut() {
            state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
        }
    }

    /// Schedules an immediate redraw if one is not already scheduled.
    pub fn queue_redraw(&mut self, output: &Output) {
        let state = self.outputs.state.get_mut(output).unwrap();
        state.redraw_state = mem::take(&mut state.redraw_state).queue_redraw();
    }

    pub fn redraw_queued_outputs(&mut self, backend: &mut Backend) {
        let _span = tracy_client::span!("Niri::redraw_queued_outputs");

        while let Some((output, _)) = self.outputs.state.iter().find(|(_, state)| {
            matches!(
                state.redraw_state,
                RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)
            )
        }) {
            trace!("redrawing output");
            let output = output.clone();
            self.redraw(backend, &output);
        }
    }

    // pointer_element, refresh_pointer_outputs, refresh_layout, refresh_idle_inhibit,
    // refresh_window_states, refresh_window_rules moved to render.rs
    // refresh_mapped_cast_window_rules, refresh_mapped_cast_outputs moved to screencast.rs
    // advance_animations, update_render_elements, update_shaders moved to render.rs

    pub fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        output: &Output,
        include_pointer: bool,
        mut target: RenderTarget,
    ) -> Vec<OutputRenderElements<R>> {
        let _span = tracy_client::span!("Niri::render");

        if target == RenderTarget::Output {
            if let Some(preview) = self.config.borrow().debug.preview_render {
                target = match preview {
                    PreviewRender::Screencast => RenderTarget::Screencast,
                    PreviewRender::ScreenCapture => RenderTarget::ScreenCapture,
                };
            }
        }

        let output_scale = Scale::from(output.current_scale().fractional_scale());

        // The pointer goes on the top.
        let mut elements = vec![];
        if include_pointer {
            elements = self.pointer_element(renderer, output);
        }

        // Next, the screen transition texture.
        {
            let state = self.outputs.state.get(output).unwrap();
            if let Some(transition) = &state.screen_transition {
                elements.push(transition.render(target).into());
            }
        }

        // Next, the exit confirm dialog.
        elements.extend(
            self.exit_confirm_dialog
                .render(renderer, output)
                .into_iter()
                .map(OutputRenderElements::from),
        );

        // Next, the config error notification too.
        if let Some(element) = self.config_error_notification.render(renderer, output) {
            elements.push(element.into());
        }

        // If the session is locked, draw the lock surface.
        if self.is_locked() {
            let state = self.outputs.state.get(output).unwrap();
            if let Some(surface) = state.lock_surface.as_ref() {
                elements.extend(render_elements_from_surface_tree(
                    renderer,
                    surface.wl_surface(),
                    (0, 0),
                    output_scale,
                    1.,
                    Kind::ScanoutCandidate,
                ));
            }

            // Draw the solid color background.
            elements.push(
                SolidColorRenderElement::from_buffer(
                    &state.lock_color_buffer,
                    (0., 0.),
                    1.,
                    Kind::Unspecified,
                )
                .into(),
            );

            if self.debug_draw_opaque_regions {
                draw_opaque_regions(&mut elements, output_scale);
            }
            return elements;
        }

        // Prepare the background elements.
        let state = self.outputs.state.get(output).unwrap();
        let backdrop = SolidColorRenderElement::from_buffer(
            &state.backdrop_buffer,
            (0., 0.),
            1.,
            Kind::Unspecified,
        )
        .into();

        // If the screenshot UI is open, draw it.
        if self.screenshot_ui.is_open() {
            elements.extend(
                self.screenshot_ui
                    .render_output(output, target)
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add the backdrop for outputs that were connected while the screenshot UI was open.
            elements.push(backdrop);

            if self.debug_draw_opaque_regions {
                draw_opaque_regions(&mut elements, output_scale);
            }
            return elements;
        }

        // Draw the hotkey overlay on top.
        if let Some(element) = self.hotkey_overlay.render(renderer, output) {
            elements.push(element.into());
        }

        // Then, the Alt-Tab switcher.
        let mru_elements = self
            .window_mru_ui
            .render_output(self, output, renderer, target)
            .into_iter()
            .flatten()
            .map(OutputRenderElements::from);
        elements.extend(mru_elements);

        // Don't draw the focus ring on the workspaces while interactively moving above those
        // workspaces, since the interactively-moved window already has a focus ring.
        let focus_ring = !self.layout.interactive_move_is_moving_above_output(output);

        // Get monitor elements.
        let mon = self.layout.monitor_for_output(output).unwrap();
        // Overview mode has been removed, zoom is always 1.0
        let zoom = 1.0;
        let monitor_elements = mon.render_elements(renderer, target, focus_ring);
        // render_workspace_shadows removed - workspace shadows no longer exist
        let insert_hint_elements = mon.render_insert_hint_between_workspaces(renderer);
        let int_move_elements: Vec<_> = self
            .layout
            .render_interactive_move_for_output(renderer, output, target)
            .collect();

        // Get layer-shell elements.
        let layer_map = layer_map_for_output(output);
        let mut extend_from_layer =
            |elements: &mut SplitElements<LayerSurfaceRenderElement<R>>, layer, for_backdrop| {
                self.render_layer(renderer, target, &layer_map, layer, elements, for_backdrop);
            };

        // The overlay layer elements go next.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Overlay, false);
        elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

        // Collect the top layer elements.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Top, false);
        let top_layer = layer_elems;

        // When rendering above the top layer, we put the regular monitor elements first.
        // Otherwise, we will render all layer-shell pop-ups and the top layer on top.
        if mon.render_above_top_layer() {
            // Collect all other layer-shell elements.
            let mut layer_elems = SplitElements::default();
            extend_from_layer(&mut layer_elems, Layer::Bottom, false);
            extend_from_layer(&mut layer_elems, Layer::Background, false);

            elements.extend(
                int_move_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );
            elements.extend(
                insert_hint_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            let mut ws_background: Option<SolidColorRenderElement> = None;
            // TODO: TEAM_023: Update render elements handling for Canvas2D
            // The old workspace-based render elements need to be adapted
            elements.extend(
                monitor_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            elements.extend(top_layer.into_iter().map(OutputRenderElements::from));
            elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

            if let Some(ws_background) = ws_background {
                elements.push(OutputRenderElements::from(ws_background));
            }

            // workspace_shadow_elements removed - no longer exist
        } else {
            elements.extend(top_layer.into_iter().map(OutputRenderElements::from));

            elements.extend(
                int_move_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            elements.extend(
                insert_hint_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // TEAM_048: Fixed Canvas2D rendering - monitor_elements must be added to output
            // Collect layer-shell elements that go below windows
            let mut layer_elems = SplitElements::default();
            extend_from_layer(&mut layer_elems, Layer::Bottom, false);
            extend_from_layer(&mut layer_elems, Layer::Background, false);

            // Add layer popups first (they go on top of windows)
            elements.extend(
                layer_elems
                    .popups
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add the monitor/canvas elements (contains window tiles)
            elements.extend(
                monitor_elements
                    .into_iter()
                    .map(OutputRenderElements::from),
            );

            // Add normal layer-shell elements (background layers)
            elements.extend(
                layer_elems
                    .normal
                    .into_iter()
                    .map(OutputRenderElements::from),
            );
        }

        // Then the backdrop.
        let mut layer_elems = SplitElements::default();
        extend_from_layer(&mut layer_elems, Layer::Background, true);
        elements.extend(layer_elems.into_iter().map(OutputRenderElements::from));

        elements.push(backdrop);

        if self.debug_draw_opaque_regions {
            draw_opaque_regions(&mut elements, output_scale);
        }

        elements
    }

    #[allow(clippy::too_many_arguments)]
    fn render_layer<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        target: RenderTarget,
        layer_map: &LayerMap,
        layer: Layer,
        elements: &mut SplitElements<LayerSurfaceRenderElement<R>>,
        for_backdrop: bool,
    ) {
        // LayerMap returns layers in reverse stacking order.
        let iter = layer_map.layers_on(layer).rev().filter_map(|surface| {
            let mapped = self.mapped_layer_surfaces.get(surface)?;

            if for_backdrop != mapped.place_within_backdrop() {
                return None;
            }

            let geo = layer_map.layer_geometry(surface)?;
            Some((mapped, geo))
        });
        for (mapped, geo) in iter {
            elements.extend(mapped.render(renderer, geo.loc.to_f64(), target));
        }
    }

    fn redraw(&mut self, backend: &mut Backend, output: &Output) {
        let _span = tracy_client::span!("Niri::redraw");

        // Verify our invariant.
        let state = self.outputs.state.get_mut(output).unwrap();
        assert!(matches!(
            state.redraw_state,
            RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)
        ));

        let target_presentation_time = state.frame_clock.next_presentation_time();

        // Freeze the clock at the target time.
        self.clock.set_unadjusted(target_presentation_time);

        self.update_render_elements(Some(output));

        let mut res = RenderResult::Skipped;
        if self.outputs.monitors_active() {
            let state = self.outputs.state.get_mut(output).unwrap();
            state.unfinished_animations_remain = self.layout.are_animations_ongoing(Some(output));
            state.unfinished_animations_remain |=
                self.config_error_notification.are_animations_ongoing();
            state.unfinished_animations_remain |= self.exit_confirm_dialog.are_animations_ongoing();
            state.unfinished_animations_remain |= self.screenshot_ui.are_animations_ongoing();
            state.unfinished_animations_remain |= self.window_mru_ui.are_animations_ongoing();
            state.unfinished_animations_remain |= state.screen_transition.is_some();

            // Also keep redrawing if the current cursor is animated.
            state.unfinished_animations_remain |= self
                .cursor_manager
                .is_current_cursor_animated(output.current_scale().integer_scale());

            // Also check layer surfaces.
            if !state.unfinished_animations_remain {
                state.unfinished_animations_remain |= layer_map_for_output(output)
                    .layers()
                    .filter_map(|surface| self.mapped_layer_surfaces.get(surface))
                    .any(|mapped| mapped.are_animations_ongoing());
            }

            // Render.
            res = backend.render(self, output, target_presentation_time);
        }

        let is_locked = self.is_locked();
        let state = self.outputs.state.get_mut(output).unwrap();

        if res == RenderResult::Skipped {
            // Update the redraw state on failed render.
            state.redraw_state = if let RedrawState::WaitingForEstimatedVBlank(token)
            | RedrawState::WaitingForEstimatedVBlankAndQueued(token) =
                state.redraw_state
            {
                RedrawState::WaitingForEstimatedVBlank(token)
            } else {
                RedrawState::Idle
            };
        }

        // Update the lock render state on successful render, or if monitors are inactive. When
        // monitors are inactive on a TTY, they have no framebuffer attached, so no sensitive data
        // from a last render will be visible.
        if res != RenderResult::Skipped || !self.monitors_active {
            state.lock_render_state = if is_locked {
                LockRenderState::Locked
            } else {
                LockRenderState::Unlocked
            };
        }

        // If we're in process of locking the session, check if the requirements were met.
        match mem::take(&mut self.lock_state) {
            LockState::Locking(confirmation) => {
                if state.lock_render_state == LockRenderState::Unlocked {
                    // We needed to render a locked frame on this output but failed.
                    self.unlock();
                } else {
                    // Check if all outputs are now locked.
                    let all_locked = self
                        .outputs.state
                        .values()
                        .all(|state| state.lock_render_state == LockRenderState::Locked);

                    if all_locked {
                        // All outputs are locked, report success.
                        let lock = confirmation.ext_session_lock().clone();
                        confirmation.lock();
                        self.lock_state = LockState::Locked(lock);
                    } else {
                        // Still waiting for other outputs.
                        self.lock_state = LockState::Locking(confirmation);
                    }
                }
            }
            lock_state => self.lock_state = lock_state,
        }

        self.refresh_on_demand_vrr(backend, output);

        // Send the frame callbacks.
        //
        // FIXME: The logic here could be a bit smarter. Currently, during an animation, the
        // surfaces that are visible for the very last frame (e.g. because the camera is moving
        // away) will receive frame callbacks, and the surfaces that are invisible but will become
        // visible next frame will not receive frame callbacks (so they will show stale contents for
        // one frame). We could advance the animations for the next frame and send frame callbacks
        // according to the expected new positions.
        //
        // However, this should probably be restricted to sending frame callbacks to more surfaces,
        // to err on the safe side.
        self.send_frame_callbacks(output);
        backend.with_primary_renderer(|renderer| {
            #[cfg(feature = "xdp-gnome-screencast")]
            {
                // Render and send to PipeWire screencast streams.
                self.render_for_screen_cast(renderer, output, target_presentation_time);

                // FIXME: when a window is hidden, it should probably still receive frame callbacks
                // and get rendered for screen cast. This is currently
                // unimplemented, but happens to work by chance, since output
                // redrawing is more eager than it should be.
                self.render_windows_for_screen_cast(renderer, output, target_presentation_time);
            }

            self.render_for_screencopy_with_damage(renderer, output);
        });
    }

    pub fn refresh_on_demand_vrr(&mut self, backend: &mut Backend, output: &Output) {
        let _span = tracy_client::span!("Niri::refresh_on_demand_vrr");

        let name = output.user_data().get::<OutputName>().unwrap();
        let on_demand = self
            .config
            .borrow()
            .outputs
            .find(name)
            .is_some_and(|output| output.is_vrr_on_demand());
        if !on_demand {
            return;
        }

        let current = self.layout.windows_for_output(output).any(|mapped| {
            mapped.rules().variable_refresh_rate == Some(true) && {
                let mut visible = false;
                mapped.window.with_surfaces(|surface, states| {
                    if !visible
                        && surface_primary_scanout_output(surface, states).as_ref() == Some(output)
                    {
                        visible = true;
                    }
                });
                visible
            }
        });

        backend.set_output_on_demand_vrr(self, output, current);
    }

    pub fn update_primary_scanout_output(
        &self,
        output: &Output,
        render_element_states: &RenderElementStates,
    ) {
        // FIXME: potentially tweak the compare function. The default one currently always prefers a
        // higher refresh-rate output, which is not always desirable (i.e. with a very small
        // overlap).
        //
        // While we only have cursors and DnD icons crossing output boundaries though, it doesn't
        // matter all that much.
        if let CursorImageStatus::Surface(surface) = &self.cursor.manager().cursor_image() {
            with_surface_tree_downward(
                surface,
                (),
                |_, _, _| TraversalAction::DoChildren(()),
                |surface, states, _| {
                    update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        render_element_states,
                        default_primary_scanout_output_compare,
                    );
                },
                |_, _, _| true,
            );
        }

        if let Some(surface) = self.dnd_icon.as_ref().map(|icon| &icon.surface) {
            with_surface_tree_downward(
                surface,
                (),
                |_, _, _| TraversalAction::DoChildren(()),
                |surface, states, _| {
                    update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        render_element_states,
                        default_primary_scanout_output_compare,
                    );
                },
                |_, _, _| true,
            );
        }

        // We're only updating the current output's windows and layer surfaces. This should be fine
        // as in niri they can only be rendered on a single output at a time.
        //
        // The reason to do this at all is that it keeps track of whether the surface is visible or
        // not in a unified way with the pointer surfaces, which makes the logic elsewhere simpler.

        for mapped in self.layout.windows_for_output(output) {
            let win = &mapped.window;
            let offscreen_data = mapped.offscreen_data();
            let offscreen_data = offscreen_data.as_ref();

            win.with_surfaces(|surface, states| {
                let primary_scanout_output = states
                    .data_map
                    .get_or_insert_threadsafe(Mutex::<PrimaryScanoutOutput>::default);
                let mut primary_scanout_output = primary_scanout_output.lock().unwrap();

                let mut id = Id::from_wayland_resource(surface);

                if let Some(data) = offscreen_data {
                    // We have offscreen data; it's likely that all surfaces are on it.
                    if data.states.element_was_presented(id.clone()) {
                        // If the surface was presented to the offscreen, use the offscreen's id.
                        id = data.id.clone();
                    }

                    // If we the surface wasn't presented to the offscreen it can mean:
                    //
                    // - The surface was invisible. For example, it's obscured by another surface on
                    //   the offscreen, or simply isn't mapped.
                    // - The surface is rendered separately from the offscreen, for example: popups
                    //   during the window resize animation.
                    //
                    // In both of these cases, using the original surface element id and the
                    // original states is the correct thing to do. We may find the surface in the
                    // original states (in the second case). Either way, we definitely know it is
                    // *not* in the offscreen, and we won't miss it.
                    //
                    // There's one edge case: if the surface is both in the offscreen and separate,
                    // and the offscreen itself is invisible, while the separate surface is
                    // visible. In this case we'll currently mark the surface as invisible. We
                    // don't really use offscreens like that however, and if we start, it's easy
                    // enough to fix (need an extra check).
                }

                primary_scanout_output.update_from_render_element_states(
                    id,
                    output,
                    render_element_states,
                    |_, _, output, _| output,
                );
            });
        }

        for surface in layer_map_for_output(output).layers() {
            surface.with_surfaces(|surface, states| {
                update_surface_primary_scanout_output(
                    surface,
                    output,
                    states,
                    render_element_states,
                    // Layer surfaces are shown only on one output at a time.
                    |_, _, output, _| output,
                );
            });
        }

        if let Some(surface) = &self.outputs.state[output].lock_surface {
            with_surface_tree_downward(
                surface.wl_surface(),
                (),
                |_, _, _| TraversalAction::DoChildren(()),
                |surface, states, _| {
                    update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        render_element_states,
                        default_primary_scanout_output_compare,
                    );
                },
                |_, _, _| true,
            );
        }
    }

    pub fn send_dmabuf_feedbacks(
        &self,
        output: &Output,
        feedback: &SurfaceDmabufFeedback,
        render_element_states: &RenderElementStates,
    ) {
        let _span = tracy_client::span!("Niri::send_dmabuf_feedbacks");

        // We can unconditionally send the current output's feedback to regular and layer-shell
        // surfaces, as they can only be displayed on a single output at a time. Even if a surface
        // is currently invisible, this is the DMABUF feedback that it should know about.
        for mapped in self.layout.windows_for_output(output) {
            mapped.window.send_dmabuf_feedback(
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        for surface in layer_map_for_output(output).layers() {
            surface.send_dmabuf_feedback(
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let Some(surface) = &self.outputs.state[output].lock_surface {
            send_dmabuf_feedback_surface_tree(
                surface.wl_surface(),
                output,
                |_, _| Some(output.clone()),
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let Some(surface) = self.dnd_icon.as_ref().map(|icon| &icon.surface) {
            send_dmabuf_feedback_surface_tree(
                surface,
                output,
                surface_primary_scanout_output,
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }

        if let CursorImageStatus::Surface(surface) = &self.cursor.manager().cursor_image() {
            send_dmabuf_feedback_surface_tree(
                surface,
                output,
                surface_primary_scanout_output,
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        &feedback.render,
                        &feedback.scanout,
                    )
                },
            );
        }
    }

    // send_frame_callbacks, send_frame_callbacks_on_fallback_timer,
    // take_presentation_feedbacks moved to frame_callbacks.rs

    // render_for_screen_cast, render_windows_for_screen_cast,
    // stop_cast, stop_casts_for_target moved to screencast.rs

    // render_for_screencopy_with_damage, render_for_screencopy_without_damage,
    // render_for_screencopy_internal, remove_screencopy_output moved to screencopy.rs

    pub fn debug_toggle_damage(&mut self) {
        self.debug_draw_damage = !self.debug_draw_damage;

        if self.debug_draw_damage {
            for (output, state) in &mut self.outputs.state {
                state.debug_damage_tracker = OutputDamageTracker::from_output(output);
            }
        }

        self.queue_redraw_all();
    }

    // capture_screenshots, screenshot, save_screenshot, screenshot_all_outputs,
    // screenshot_window moved to screenshot.rs

    // is_locked, lock, maybe_continue_to_locking, continue_to_locking, unlock,
    // update_locked_hint, new_lock_surface moved to lock.rs

    // maybe_activate_pointer_constraint, focus_layer_surface_if_on_demand,
    // handle_focus_follows_mouse, reset_pointer_inactivity_timer,
    // notify_activity moved to pointer.rs

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
            .outputs.state
            .keys()
            .cloned()
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
