//! Niri compositor initialization.
//!
//! Contains the `Niri::new()` constructor.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use _server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as KdeDecorationsMode;
use calloop::timer::{TimeoutAction, Timer};
use calloop::LoopHandle;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::desktop::{PopupManager, Space};
use smithay::input::SeatState;
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, LoopSignal, Mode, PostAction};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities;
use smithay::reexports::wayland_protocols_misc::server_decoration as _server_decoration;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::reexports::wayland_server::{Client, Display};
use smithay::utils::{ClockSource, Monotonic};
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::cursor_shape::CursorShapeManagerState;
use smithay::wayland::dmabuf::DmabufState;
use smithay::wayland::fractional_scale::FractionalScaleManagerState;
use smithay::wayland::idle_inhibit::IdleInhibitManagerState;
use smithay::wayland::idle_notify::IdleNotifierState;
use smithay::wayland::input_method::InputMethodManagerState;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState;
use smithay::wayland::output::OutputManagerState;
use smithay::wayland::pointer_constraints::PointerConstraintsState;
use smithay::wayland::pointer_gestures::PointerGesturesState;
use smithay::wayland::presentation::PresentationState;
use smithay::wayland::relative_pointer::RelativePointerManagerState;
use smithay::wayland::security_context::SecurityContextState;
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::selection::ext_data_control::DataControlState as ExtDataControlState;
use smithay::wayland::selection::primary_selection::PrimarySelectionState;
use smithay::wayland::selection::wlr_data_control::DataControlState as WlrDataControlState;
use smithay::wayland::session_lock::SessionLockManagerState;
use smithay::wayland::shell::kde::decoration::KdeDecorationState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
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

use super::{
    CursorSubsystem, FocusState, KeyboardFocus, NewClient, OutputSubsystem, PointContents,
    PointerVisibility, State, StreamingSubsystem, UiOverlays,
};
#[cfg(feature = "dbus")]
use crate::a11y::A11y;
use crate::animation::Clock;
use crate::backend::Backend;
use crate::cursor::CursorManager;
use crate::handlers::XDG_ACTIVATION_TOKEN_TIMEOUT;
use crate::input::scroll_swipe_gesture::ScrollSwipeGesture;
use crate::input::scroll_tracker::ScrollTracker;
use crate::ipc::server::IpcServer;
use crate::layout::Layout;
use crate::niri::subsystems::InputTracking;
use crate::niri::{ClientState, LockState, Niri, ProtocolStates};
use crate::protocols::ext_workspace::ExtWorkspaceManagerState;
use crate::protocols::foreign_toplevel::ForeignToplevelManagerState;
use crate::protocols::gamma_control::GammaControlManagerState;
use crate::protocols::mutter_x11_interop::MutterX11InteropManagerState;
use crate::protocols::output_management::OutputManagementManagerState;
use crate::protocols::screencopy::ScreencopyManagerState;
use crate::protocols::virtual_pointer::VirtualPointerManagerState;
use crate::window::mapped::MappedId;

// =============================================================================
// Niri Constructor
// =============================================================================

impl Niri {
    /// Creates a new Niri compositor instance.
    pub fn new(
        config: Rc<RefCell<niri_config::Config>>,
        event_loop: LoopHandle<'static, State>,
        stop_signal: LoopSignal,
        display: Display<State>,
        backend: &Backend,
        create_wayland_socket: bool,
        is_session_instance: bool,
    ) -> Self {
        let _span = tracy_client::span!("Niri::new");

        let (executor, scheduler) = calloop::futures::executor().unwrap();
        event_loop.insert_source(executor, |_, _, _| ()).unwrap();

        let display_handle = display.handle();
        let config_ = config.borrow();
        let config_file_output_config = config_.outputs.clone();

        let mut animation_clock = Clock::default();

        let rate = 1.0 / config_.animations.slowdown.max(0.001);
        animation_clock.set_rate(rate);
        animation_clock.set_complete_instantly(config_.animations.off);

        let layout = Layout::new(animation_clock.clone(), &config_);

        let (blocker_cleared_tx, blocker_cleared_rx) = mpsc::channel();

        fn client_is_unrestricted(client: &Client) -> bool {
            !client.get_data::<ClientState>().unwrap().restricted
        }

        let cursor_manager =
            CursorManager::new(&config_.cursor.xcursor_theme, config_.cursor.xcursor_size);

        let mut protocols = ProtocolStates::new(&display_handle, &event_loop, &config_, backend);

        event_loop
            .insert_source(
                Timer::from_duration(XDG_ACTIVATION_TOKEN_TIMEOUT),
                |_, _, state| {
                    state
                        .niri
                        .protocols
                        .activation
                        .retain_tokens(|_, token_data| {
                            token_data.timestamp.elapsed() < XDG_ACTIVATION_TOKEN_TIMEOUT
                        });
                    TimeoutAction::ToDuration(XDG_ACTIVATION_TOKEN_TIMEOUT)
                },
            )
            .unwrap();

        let mut seat = protocols
            .seat
            .new_wl_seat(&display_handle, backend.seat_name());
        let mod_key = backend.mod_key(&config_);
        let keyboard = match seat.add_keyboard(
            config_.input.keyboard.xkb.to_xkb_config(),
            config_.input.keyboard.repeat_delay.into(),
            config_.input.keyboard.repeat_rate.into(),
        ) {
            Err(err) => {
                if let smithay::input::keyboard::Error::BadKeymap = err {
                    warn!("error loading the configured xkb keymap, trying default");
                } else {
                    warn!("error adding keyboard: {err:?}");
                }
                seat.add_keyboard(
                    Default::default(),
                    config_.input.keyboard.repeat_delay.into(),
                    config_.input.keyboard.repeat_rate.into(),
                )
                .unwrap()
            }
            Ok(keyboard) => keyboard,
        };
        if config_.input.keyboard.numlock {
            let mut modifier_state = keyboard.modifier_state();
            modifier_state.num_lock = true;
            keyboard.set_modifier_state(modifier_state);
        }
        seat.add_pointer();

        let ui = UiOverlays::new(&config_, &animation_clock, &config);

        #[cfg(feature = "dbus")]
        let a11y = A11y::new(event_loop.clone());

        event_loop
            .insert_source(
                Timer::from_duration(Duration::from_secs(1)),
                |_, _, state| {
                    state.niri.send_frame_callbacks_on_fallback_timer();
                    TimeoutAction::ToDuration(Duration::from_secs(1))
                },
            )
            .unwrap();

        let socket_name = create_wayland_socket.then(|| {
            let socket_source = ListeningSocketSource::new_auto().unwrap();
            let socket_name = socket_source.socket_name().to_os_string();
            event_loop
                .insert_source(socket_source, move |client, _, state| {
                    state.niri.insert_client(NewClient {
                        client,
                        restricted: false,
                        credentials_unknown: false,
                    });
                })
                .unwrap();
            socket_name
        });

        let ipc_server = match IpcServer::start(&event_loop, socket_name.as_deref()) {
            Ok(server) => Some(server),
            Err(err) => {
                warn!("error starting IPC server: {err:?}");
                None
            }
        };

        let display_source = Generic::new(display, Interest::READ, Mode::Level);
        event_loop
            .insert_source(display_source, |_, display, state| {
                // SAFETY: we don't drop the display.
                unsafe {
                    display.get_mut().dispatch_clients(state).unwrap();
                }
                Ok(PostAction::Continue)
            })
            .unwrap();

        event_loop
            .insert_source(
                Timer::from_duration(Duration::from_secs(60)),
                |_, _, state| {
                    let _span = tracy_client::span!("startup timeout");
                    state.niri.is_at_startup = false;
                    state.niri.recompute_window_rules();
                    state.niri.recompute_layer_rules();
                    TimeoutAction::Drop
                },
            )
            .unwrap();

        let input = InputTracking::new(&config_);
        drop(config_);
        let mut niri = Self {
            config,
            config_file_output_config,
            config_file_watcher: None,

            event_loop,
            scheduler,
            stop_signal,
            socket_name,
            display_handle,
            is_session_instance,
            start_time: Instant::now(),
            is_at_startup: true,
            clock: animation_clock,

            layout,
            unmapped_windows: HashMap::new(),
            unmapped_layer_surfaces: HashSet::new(),
            mapped_layer_surfaces: HashMap::new(),
            root_surface: HashMap::new(),
            dmabuf_pre_commit_hook: HashMap::new(),
            blocker_cleared_tx,
            blocker_cleared_rx,
            outputs: OutputSubsystem::new(),
            input,

            devices: HashSet::new(),
            tablets: HashMap::new(),
            touch: HashSet::new(),

            protocols,
            popup_grab: None,
            suppressed_keys: HashSet::new(),
            suppressed_buttons: HashSet::new(),
            bind_cooldown_timers: HashMap::new(),
            bind_repeat_timer: Option::default(),

            seat,
            focus: FocusState::new(),
            is_fdo_idle_inhibited: Arc::new(AtomicBool::new(false)),
            xkb_from_locale1: None,
            cursor: CursorSubsystem::new(cursor_manager),
            notified_activity_this_iteration: false,

            lock_state: LockState::Unlocked,
            locked_hint: None,

            ui,

            streaming: StreamingSubsystem::new(),
            debug_draw_opaque_regions: false,
            debug_draw_damage: false,
            pointer_inside_hot_corner: false,
            pending_mru_commit: None,

            #[cfg(feature = "dbus")]
            dbus: None,
            #[cfg(feature = "dbus")]
            a11y_keyboard_monitor: None,
            #[cfg(feature = "dbus")]
            a11y,
            #[cfg(feature = "dbus")]
            inhibit_power_key_fd: None,

            ipc_server,
            ipc_outputs_changed: false,

            satellite: None,
        };

        // Initialize streaming subsystem components.
        #[cfg(feature = "xdp-gnome-screencast")]
        niri.streaming.init_pipewire_channel(&niri.event_loop);

        niri.reset_pointer_inactivity_timer();

        niri
    }
}
