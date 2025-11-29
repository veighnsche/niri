//! Niri compositor initialization.
//!
//! Contains the `Niri::new()` constructor.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use calloop::timer::{TimeoutAction, Timer};
use calloop::LoopHandle;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::desktop::{PopupManager, Space};
use smithay::input::SeatState;
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, LoopSignal, Mode, PostAction};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities;
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

use _server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as KdeDecorationsMode;
use smithay::reexports::wayland_protocols_misc::server_decoration as _server_decoration;

#[cfg(feature = "dbus")]
use crate::a11y::A11y;
use crate::animation::Clock;
use crate::backend::Backend;
use crate::cursor::CursorManager;
use crate::handlers::XDG_ACTIVATION_TOKEN_TIMEOUT;
use crate::input::scroll_swipe_gesture::ScrollSwipeGesture;
use crate::input::scroll_tracker::ScrollTracker;
use crate::input::{mods_with_finger_scroll_binds, mods_with_mouse_binds, mods_with_wheel_binds};
use crate::ipc::server::IpcServer;
use crate::layout::Layout;
use crate::protocols::ext_workspace::ExtWorkspaceManagerState;
use crate::protocols::foreign_toplevel::ForeignToplevelManagerState;
use crate::protocols::gamma_control::GammaControlManagerState;
use crate::protocols::mutter_x11_interop::MutterX11InteropManagerState;
use crate::protocols::output_management::OutputManagementManagerState;
use crate::protocols::screencopy::ScreencopyManagerState;
use crate::protocols::virtual_pointer::VirtualPointerManagerState;
use crate::ui::config_error_notification::ConfigErrorNotification;
use crate::ui::exit_confirm_dialog::ExitConfirmDialog;
use crate::ui::hotkey_overlay::HotkeyOverlay;
use crate::ui::mru::WindowMruUi;
use crate::ui::screenshot_ui::ScreenshotUi;
use crate::window::mapped::MappedId;
use super::{State,
    ClientState, KeyboardFocus, LockState, NewClient, Niri, PointContents, PointerVisibility,
};

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

        let compositor_state = CompositorState::new_v6::<State>(&display_handle);
        let xdg_shell_state = XdgShellState::new_with_capabilities::<State>(
            &display_handle,
            [WmCapabilities::Fullscreen, WmCapabilities::Maximize],
        );
        let xdg_decoration_state =
            XdgDecorationState::new_with_filter::<State, _>(&display_handle, |client| {
                client
                    .get_data::<ClientState>()
                    .unwrap()
                    .can_view_decoration_globals
            });
        let kde_decoration_state = KdeDecorationState::new_with_filter::<State, _>(
            &display_handle,
            // If we want CSD we will hide the global.
            KdeDecorationsMode::Server,
            |client| {
                client
                    .get_data::<ClientState>()
                    .unwrap()
                    .can_view_decoration_globals
            },
        );
        let layer_shell_state = WlrLayerShellState::new_with_filter::<State, _>(
            &display_handle,
            client_is_unrestricted,
        );
        let session_lock_state =
            SessionLockManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let shm_state = ShmState::new::<State>(
            &display_handle,
            vec![wl_shm::Format::Xbgr8888, wl_shm::Format::Abgr8888],
        );
        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<State>(&display_handle);
        let dmabuf_state = DmabufState::new();
        let fractional_scale_manager_state =
            FractionalScaleManagerState::new::<State>(&display_handle);
        let mut seat_state = SeatState::new();
        let tablet_state = TabletManagerState::new::<State>(&display_handle);
        let pointer_gestures_state = PointerGesturesState::new::<State>(&display_handle);
        let relative_pointer_state = RelativePointerManagerState::new::<State>(&display_handle);
        let pointer_constraints_state = PointerConstraintsState::new::<State>(&display_handle);
        let idle_notifier_state = IdleNotifierState::new(&display_handle, event_loop.clone());
        let idle_inhibit_manager_state = IdleInhibitManagerState::new::<State>(&display_handle);
        let data_device_state = DataDeviceState::new::<State>(&display_handle);
        let primary_selection_state =
            PrimarySelectionState::new_with_filter::<State, _>(&display_handle, |client| {
                !client
                    .get_data::<ClientState>()
                    .unwrap()
                    .primary_selection_disabled
            });
        let wlr_data_control_state = WlrDataControlState::new::<State, _>(
            &display_handle,
            Some(&primary_selection_state),
            client_is_unrestricted,
        );
        let ext_data_control_state = ExtDataControlState::new::<State, _>(
            &display_handle,
            Some(&primary_selection_state),
            client_is_unrestricted,
        );
        let presentation_state =
            PresentationState::new::<State>(&display_handle, Monotonic::ID as u32);
        let security_context_state =
            SecurityContextState::new::<State, _>(&display_handle, client_is_unrestricted);

        let text_input_state = TextInputManagerState::new::<State>(&display_handle);
        let input_method_state =
            InputMethodManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let keyboard_shortcuts_inhibit_state =
            KeyboardShortcutsInhibitState::new::<State>(&display_handle);
        let virtual_keyboard_state =
            VirtualKeyboardManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let virtual_pointer_state =
            VirtualPointerManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let foreign_toplevel_state =
            ForeignToplevelManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let ext_workspace_state =
            ExtWorkspaceManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let mut output_management_state =
            OutputManagementManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        output_management_state.on_config_changed(config_.outputs.clone());
        let screencopy_state =
            ScreencopyManagerState::new::<State, _>(&display_handle, client_is_unrestricted);
        let viewporter_state = ViewporterState::new::<State>(&display_handle);
        let xdg_foreign_state = XdgForeignState::new::<State>(&display_handle);

        let is_tty = matches!(backend, Backend::Tty(_));
        let gamma_control_manager_state =
            GammaControlManagerState::new::<State, _>(&display_handle, move |client| {
                is_tty && !client.get_data::<ClientState>().unwrap().restricted
            });
        let activation_state = XdgActivationState::new::<State>(&display_handle);
        event_loop
            .insert_source(
                Timer::from_duration(XDG_ACTIVATION_TOKEN_TIMEOUT),
                |_, _, state| {
                    state.niri.activation_state.retain_tokens(|_, token_data| {
                        token_data.timestamp.elapsed() < XDG_ACTIVATION_TOKEN_TIMEOUT
                    });
                    TimeoutAction::ToDuration(XDG_ACTIVATION_TOKEN_TIMEOUT)
                },
            )
            .unwrap();

        let mutter_x11_interop_state =
            MutterX11InteropManagerState::new::<State, _>(&display_handle, move |_| true);

        #[cfg(test)]
        let single_pixel_buffer_state = SinglePixelBufferState::new::<State>(&display_handle);

        let mut seat = seat_state.new_wl_seat(&display_handle, backend.seat_name());
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

        let cursor_shape_manager_state = CursorShapeManagerState::new::<State>(&display_handle);
        let cursor_manager =
            CursorManager::new(&config_.cursor.xcursor_theme, config_.cursor.xcursor_size);

        let mod_key = backend.mod_key(&config.borrow());
        let mods_with_mouse_binds = mods_with_mouse_binds(mod_key, &config_.binds);
        let mods_with_wheel_binds = mods_with_wheel_binds(mod_key, &config_.binds);
        let mods_with_finger_scroll_binds = mods_with_finger_scroll_binds(mod_key, &config_.binds);

        let screenshot_ui = ScreenshotUi::new(animation_clock.clone(), config.clone());
        let window_mru_ui = WindowMruUi::new(config.clone());
        let config_error_notification =
            ConfigErrorNotification::new(animation_clock.clone(), config.clone());

        let mut hotkey_overlay = HotkeyOverlay::new(config.clone(), mod_key);
        if !config_.hotkey_overlay.skip_at_startup {
            hotkey_overlay.show();
        }

        let exit_confirm_dialog = ExitConfirmDialog::new(animation_clock.clone(), config.clone());

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

        #[cfg(feature = "xdp-gnome-screencast")]
        let pw_to_niri = {
            let (pw_to_niri, from_pipewire) = calloop::channel::channel();
            event_loop
                .insert_source(from_pipewire, move |event, _, state| match event {
                    calloop::channel::Event::Msg(msg) => state.on_pw_msg(msg),
                    calloop::channel::Event::Closed => (),
                })
                .unwrap();
            pw_to_niri
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
            global_space: Space::default(),
            sorted_outputs: Vec::default(),
            output_state: HashMap::new(),
            unmapped_windows: HashMap::new(),
            unmapped_layer_surfaces: HashSet::new(),
            mapped_layer_surfaces: HashMap::new(),
            root_surface: HashMap::new(),
            dmabuf_pre_commit_hook: HashMap::new(),
            blocker_cleared_tx,
            blocker_cleared_rx,
            monitors_active: true,
            is_lid_closed: false,

            devices: HashSet::new(),
            tablets: HashMap::new(),
            touch: HashSet::new(),

            compositor_state,
            xdg_shell_state,
            xdg_decoration_state,
            kde_decoration_state,
            layer_shell_state,
            session_lock_state,
            foreign_toplevel_state,
            ext_workspace_state,
            output_management_state,
            screencopy_state,
            viewporter_state,
            xdg_foreign_state,
            text_input_state,
            input_method_state,
            keyboard_shortcuts_inhibit_state,
            virtual_keyboard_state,
            virtual_pointer_state,
            shm_state,
            output_manager_state,
            dmabuf_state,
            fractional_scale_manager_state,
            seat_state,
            tablet_state,
            pointer_gestures_state,
            relative_pointer_state,
            pointer_constraints_state,
            idle_notifier_state,
            idle_inhibit_manager_state,
            data_device_state,
            primary_selection_state,
            wlr_data_control_state,
            ext_data_control_state,
            popups: PopupManager::default(),
            popup_grab: None,
            suppressed_keys: HashSet::new(),
            suppressed_buttons: HashSet::new(),
            bind_cooldown_timers: HashMap::new(),
            bind_repeat_timer: Option::default(),
            presentation_state,
            security_context_state,
            gamma_control_manager_state,
            activation_state,
            mutter_x11_interop_state,
            #[cfg(test)]
            single_pixel_buffer_state,

            seat,
            keyboard_focus: KeyboardFocus::Layout { surface: None },
            layer_shell_on_demand_focus: None,
            idle_inhibiting_surfaces: HashSet::new(),
            is_fdo_idle_inhibited: Arc::new(AtomicBool::new(false)),
            keyboard_shortcuts_inhibiting_surfaces: HashMap::new(),
            xkb_from_locale1: None,
            cursor_manager,
            cursor_texture_cache: Default::default(),
            cursor_shape_manager_state,
            dnd_icon: None,
            pointer_contents: PointContents::default(),
            pointer_visibility: PointerVisibility::Visible,
            pointer_inactivity_timer: None,
            pointer_inactivity_timer_got_reset: false,
            notified_activity_this_iteration: false,
            pointer_inside_hot_corner: false,
            tablet_cursor_location: None,
            gesture_swipe_3f_cumulative: None,
            overview_scroll_swipe_gesture: ScrollSwipeGesture::new(),
            vertical_wheel_tracker: ScrollTracker::new(120),
            horizontal_wheel_tracker: ScrollTracker::new(120),
            mods_with_mouse_binds,
            mods_with_wheel_binds,

            // 10 is copied from Clutter: DISCRETE_SCROLL_STEP.
            vertical_finger_scroll_tracker: ScrollTracker::new(10),
            horizontal_finger_scroll_tracker: ScrollTracker::new(10),
            mods_with_finger_scroll_binds,

            lock_state: LockState::Unlocked,
            locked_hint: None,

            screenshot_ui,
            config_error_notification,
            hotkey_overlay,
            exit_confirm_dialog,

            window_mru_ui,
            pending_mru_commit: None,

            pick_window: None,
            pick_color: None,

            debug_draw_opaque_regions: false,
            debug_draw_damage: false,

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

            pipewire: None,
            casts: vec![],
            #[cfg(feature = "xdp-gnome-screencast")]
            pw_to_niri,

            #[cfg(feature = "xdp-gnome-screencast")]
            mapped_cast_output: HashMap::new(),

            #[cfg(feature = "xdp-gnome-screencast")]
            dynamic_cast_id_for_portal: MappedId::next(),
        };

        niri.reset_pointer_inactivity_timer();

        niri
    }
}
