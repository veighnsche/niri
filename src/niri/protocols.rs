//! Smithay protocol state container.
//!
//! Groups all Wayland protocol states into a single struct to reduce
//! clutter in the main Niri struct.

use smithay::reexports::wayland_server::DisplayHandle;

use smithay::desktop::PopupManager;
use smithay::input::SeatState;
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
use smithay::wayland::tablet_manager::TabletManagerState;
use smithay::wayland::text_input::TextInputManagerState;
use smithay::wayland::viewporter::ViewporterState;
use smithay::wayland::virtual_keyboard::VirtualKeyboardManagerState;
use smithay::wayland::xdg_activation::XdgActivationState;
use smithay::wayland::xdg_foreign::XdgForeignState;

use crate::protocols::ext_workspace::ExtWorkspaceManagerState;
use crate::protocols::foreign_toplevel::ForeignToplevelManagerState;
use crate::protocols::gamma_control::GammaControlManagerState;
use crate::protocols::mutter_x11_interop::MutterX11InteropManagerState;
use crate::protocols::output_management::OutputManagementManagerState;
use crate::protocols::screencopy::ScreencopyManagerState;
use crate::protocols::virtual_pointer::VirtualPointerManagerState;

use super::State;

/// Container for all Smithay protocol states.
///
/// These are initialized once in `Niri::new` and provide the Wayland
/// protocol implementations. Grouping them here keeps the main `Niri`
/// struct focused on compositor logic rather than protocol plumbing.
pub struct ProtocolStates {
    pub compositor: CompositorState,
    pub xdg_shell: XdgShellState,
    pub xdg_decoration: XdgDecorationState,
    pub kde_decoration: KdeDecorationState,
    pub layer_shell: WlrLayerShellState,
    pub session_lock: SessionLockManagerState,
    pub foreign_toplevel: ForeignToplevelManagerState,
    pub ext_workspace: ExtWorkspaceManagerState,
    pub screencopy: ScreencopyManagerState,
    pub output_management: OutputManagementManagerState,
    pub viewporter: ViewporterState,
    pub xdg_foreign: XdgForeignState,
    pub shm: ShmState,
    pub output_manager: OutputManagerState,
    pub dmabuf: DmabufState,
    pub fractional_scale: FractionalScaleManagerState,
    pub seat: SeatState<State>,
    pub tablet: TabletManagerState,
    pub text_input: TextInputManagerState,
    pub input_method: InputMethodManagerState,
    pub keyboard_shortcuts_inhibit: KeyboardShortcutsInhibitState,
    pub virtual_keyboard: VirtualKeyboardManagerState,
    pub virtual_pointer: VirtualPointerManagerState,
    pub pointer_gestures: PointerGesturesState,
    pub relative_pointer: RelativePointerManagerState,
    pub pointer_constraints: PointerConstraintsState,
    pub idle_notifier: IdleNotifierState<State>,
    pub idle_inhibit: IdleInhibitManagerState,
    pub data_device: DataDeviceState,
    pub primary_selection: PrimarySelectionState,
    pub wlr_data_control: WlrDataControlState,
    pub ext_data_control: ExtDataControlState,
    pub popups: PopupManager,
    pub presentation: PresentationState,
    pub security_context: SecurityContextState,
    pub gamma_control: GammaControlManagerState,
    pub activation: XdgActivationState,
    pub mutter_x11_interop: MutterX11InteropManagerState,
    pub cursor_shape: CursorShapeManagerState,
    
    #[cfg(test)]
    pub single_pixel_buffer: SinglePixelBufferState,
}

impl ProtocolStates {
    /// Creates all protocol states for the given display.
    pub fn new(
        display: &DisplayHandle,
        event_loop: &calloop::LoopHandle<'static, State>,
        config: &niri_config::Config,
        backend: &crate::backend::Backend,
    ) -> Self {
        use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities;
        use smithay::reexports::wayland_server::protocol::wl_shm;
        use smithay::reexports::wayland_protocols_misc::server_decoration as _server_decoration;
        use _server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as KdeDecorationsMode;
        use smithay::utils::{Monotonic, ClockSource};
        use std::time::Duration;

        fn client_is_unrestricted(client: &smithay::reexports::wayland_server::Client) -> bool {
            !client.get_data::<crate::niri::ClientState>().unwrap().restricted
        }

        let compositor = CompositorState::new_v6::<State>(display);
        let xdg_shell = XdgShellState::new_with_capabilities::<State>(
            display,
            [WmCapabilities::Fullscreen, WmCapabilities::Maximize],
        );
        let xdg_decoration =
            XdgDecorationState::new_with_filter::<State, _>(display, |client| {
                client
                    .get_data::<crate::niri::ClientState>()
                    .unwrap()
                    .can_view_decoration_globals
            });
        let kde_decoration = KdeDecorationState::new_with_filter::<State, _>(
            display,
            // If we want CSD we will hide the global.
            KdeDecorationsMode::Server,
            |client| {
                client
                    .get_data::<crate::niri::ClientState>()
                    .unwrap()
                    .can_view_decoration_globals
            },
        );
        let layer_shell = WlrLayerShellState::new_with_filter::<State, _>(
            display,
            client_is_unrestricted,
        );
        let session_lock =
            SessionLockManagerState::new::<State, _>(display, client_is_unrestricted);
        let shm = ShmState::new::<State>(
            display,
            vec![wl_shm::Format::Xbgr8888, wl_shm::Format::Abgr8888],
        );
        let output_manager = OutputManagerState::new_with_xdg_output::<State>(display);
        let dmabuf = DmabufState::new();
        let fractional_scale = FractionalScaleManagerState::new::<State>(display);
        let mut seat = SeatState::new();
        let tablet = TabletManagerState::new::<State>(display);
        let pointer_gestures = PointerGesturesState::new::<State>(display);
        let relative_pointer = RelativePointerManagerState::new::<State>(display);
        let pointer_constraints = PointerConstraintsState::new::<State>(display);
        let idle_notifier = IdleNotifierState::new(display, event_loop.clone());
        let idle_inhibit = IdleInhibitManagerState::new::<State>(display);
        let data_device = DataDeviceState::new::<State>(display);
        let primary_selection =
            PrimarySelectionState::new_with_filter::<State, _>(display, |client| {
                !client
                    .get_data::<crate::niri::ClientState>()
                    .unwrap()
                    .primary_selection_disabled
            });
        let wlr_data_control = WlrDataControlState::new::<State, _>(
            display,
            Some(&primary_selection),
            client_is_unrestricted,
        );
        let ext_data_control = ExtDataControlState::new::<State, _>(
            display,
            Some(&primary_selection),
            client_is_unrestricted,
        );
        let presentation =
            PresentationState::new::<State>(display, Monotonic::ID as u32);
        let security_context =
            SecurityContextState::new::<State, _>(display, client_is_unrestricted);

        let text_input = TextInputManagerState::new::<State>(display);
        let input_method =
            InputMethodManagerState::new::<State, _>(display, client_is_unrestricted);
        let keyboard_shortcuts_inhibit =
            KeyboardShortcutsInhibitState::new::<State>(display);

        let virtual_keyboard =
            VirtualKeyboardManagerState::new::<State, _>(display, client_is_unrestricted);
        let virtual_pointer =
            VirtualPointerManagerState::new::<State, _>(display, client_is_unrestricted);
        let foreign_toplevel =
            ForeignToplevelManagerState::new::<State, _>(display, client_is_unrestricted);
        let ext_workspace =
            ExtWorkspaceManagerState::new::<State, _>(display, client_is_unrestricted);
        let mut output_management =
            OutputManagementManagerState::new::<State, _>(display, client_is_unrestricted);
        output_management.on_config_changed(config.outputs.clone());
        let screencopy =
            ScreencopyManagerState::new::<State, _>(display, client_is_unrestricted);
        let viewporter = ViewporterState::new::<State>(display);
        let xdg_foreign = XdgForeignState::new::<State>(display);

        let is_tty = matches!(backend, crate::backend::Backend::Tty(_));
        let gamma_control =
            GammaControlManagerState::new::<State, _>(display, move |client| {
                is_tty && !client.get_data::<crate::niri::ClientState>().unwrap().restricted
            });
        let activation = XdgActivationState::new::<State>(display);

        let mutter_x11_interop =
            MutterX11InteropManagerState::new::<State, _>(display, move |_| true);

        #[cfg(test)]
        let single_pixel_buffer = SinglePixelBufferState::new::<State>(display);

        let cursor_shape = CursorShapeManagerState::new::<State>(display);

        Self {
            compositor,
            xdg_shell,
            xdg_decoration,
            kde_decoration,
            layer_shell,
            session_lock,
            foreign_toplevel,
            ext_workspace,
            screencopy,
            output_management,
            viewporter,
            xdg_foreign,
            shm,
            output_manager,
            dmabuf,
            fractional_scale,
            seat,
            tablet,
            text_input,
            input_method,
            keyboard_shortcuts_inhibit,
            virtual_keyboard,
            virtual_pointer,
            pointer_gestures,
            relative_pointer,
            pointer_constraints,
            idle_notifier,
            idle_inhibit,
            data_device,
            primary_selection,
            wlr_data_control,
            ext_data_control,
            popups: PopupManager::default(),
            presentation,
            security_context,
            gamma_control,
            activation,
            mutter_x11_interop,
            cursor_shape,
            #[cfg(test)]
            single_pixel_buffer,
        }
    }
}
