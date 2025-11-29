//! Input device management and libinput configuration.
//!
//! This module handles device lifecycle (add/remove) and applies
//! libinput settings based on the niri configuration.

use niri_config::input::{Mouse, Tablet, Touch, Touchpad, Trackball, Trackpoint};
use smithay::backend::input::{Device, DeviceCapability};
use smithay::wayland::tablet_manager::{TabletDescriptor, TabletSeatTrait};

use crate::niri::State;

// TEAM_086: Trait-based device input handling (replaces pub(super) pattern)

/// Trait for input device lifecycle handling.
///
/// This trait defines the interface for handling device add/remove events.
pub(crate) trait DeviceInput {
    /// Handle device added event.
    fn on_device_added(&mut self, device: impl Device);
    
    /// Handle device removed event.
    fn on_device_removed(&mut self, device: impl Device);
}

impl DeviceInput for State {
    fn on_device_added(&mut self, device: impl Device) {
        if device.has_capability(DeviceCapability::TabletTool) {
            let tablet_seat = self.niri.seat.tablet_seat();

            let desc = TabletDescriptor::from(&device);
            tablet_seat.add_tablet::<Self>(&self.niri.display_handle, &desc);
        }
        if device.has_capability(DeviceCapability::Touch) && self.niri.seat.get_touch().is_none() {
            self.niri.seat.add_touch();
        }
    }

    fn on_device_removed(&mut self, device: impl Device) {
        if device.has_capability(DeviceCapability::TabletTool) {
            let tablet_seat = self.niri.seat.tablet_seat();

            let desc = TabletDescriptor::from(&device);
            tablet_seat.remove_tablet(&desc);

            // If there are no tablets in seat we can remove all tools
            if tablet_seat.count_tablets() == 0 {
                tablet_seat.clear_tools();
            }
        }
        if device.has_capability(DeviceCapability::Touch) && self.niri.touch.is_empty() {
            self.niri.seat.remove_touch();
        }
    }
}

/// Apply libinput configuration settings to a device.
///
/// This function detects the device type (touchpad, mouse, trackball, trackpoint,
/// tablet, touch) and applies the appropriate settings from the config.
pub fn apply_libinput_settings(
    config: &niri_config::Input,
    device: &mut input::Device,
) {
    // According to Mutter code, this setting is specific to touchpads.
    let is_touchpad = device.config_tap_finger_count() > 0;
    if is_touchpad {
        apply_touchpad_settings(&config.touchpad, device);
    }

    // This is how Mutter tells apart mice.
    let mut is_trackball = false;
    let mut is_trackpoint = false;
    if let Some(udev_device) = unsafe { device.udev_device() } {
        if udev_device.property_value("ID_INPUT_TRACKBALL").is_some() {
            is_trackball = true;
        }
        if udev_device
            .property_value("ID_INPUT_POINTINGSTICK")
            .is_some()
        {
            is_trackpoint = true;
        }
    }

    let is_mouse = device.has_capability(input::DeviceCapability::Pointer)
        && !is_touchpad
        && !is_trackball
        && !is_trackpoint;
    if is_mouse {
        apply_mouse_settings(&config.mouse, device);
    }

    if is_trackball {
        apply_trackball_settings(&config.trackball, device);
    }

    if is_trackpoint {
        apply_trackpoint_settings(&config.trackpoint, device);
    }

    let is_tablet = device.has_capability(input::DeviceCapability::TabletTool);
    if is_tablet {
        apply_tablet_settings(&config.tablet, device);
    }

    let is_touch = device.has_capability(input::DeviceCapability::Touch);
    if is_touch {
        apply_touch_settings(&config.touch, device);
    }
}

fn apply_touchpad_settings(c: &Touchpad, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else if c.disabled_on_external_mouse {
        input::SendEventsMode::DISABLED_ON_EXTERNAL_MOUSE
    } else {
        input::SendEventsMode::ENABLED
    });
    let _ = device.config_tap_set_enabled(c.tap);
    let _ = device.config_dwt_set_enabled(c.dwt);
    let _ = device.config_dwtp_set_enabled(c.dwtp);
    let _ = device.config_tap_set_drag_lock_enabled(c.drag_lock);
    let _ = device.config_scroll_set_natural_scroll_enabled(c.natural_scroll);
    let _ = device.config_accel_set_speed(c.accel_speed.0);
    let _ = device.config_left_handed_set(c.left_handed);
    let _ = device.config_middle_emulation_set_enabled(c.middle_emulation);

    if let Some(drag) = c.drag {
        let _ = device.config_tap_set_drag_enabled(drag);
    } else {
        let default = device.config_tap_default_drag_enabled();
        let _ = device.config_tap_set_drag_enabled(default);
    }

    if let Some(accel_profile) = c.accel_profile {
        let _ = device.config_accel_set_profile(accel_profile.into());
    } else if let Some(default) = device.config_accel_default_profile() {
        let _ = device.config_accel_set_profile(default);
    }

    apply_scroll_method(c.scroll_method, c.scroll_button, c.scroll_button_lock, device);

    if let Some(tap_button_map) = c.tap_button_map {
        let _ = device.config_tap_set_button_map(tap_button_map.into());
    } else if let Some(default) = device.config_tap_default_button_map() {
        let _ = device.config_tap_set_button_map(default);
    }

    if let Some(method) = c.click_method {
        let _ = device.config_click_set_method(method.into());
    } else if let Some(default) = device.config_click_default_method() {
        let _ = device.config_click_set_method(default);
    }
}

fn apply_mouse_settings(c: &Mouse, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else {
        input::SendEventsMode::ENABLED
    });
    let _ = device.config_scroll_set_natural_scroll_enabled(c.natural_scroll);
    let _ = device.config_accel_set_speed(c.accel_speed.0);
    let _ = device.config_left_handed_set(c.left_handed);
    let _ = device.config_middle_emulation_set_enabled(c.middle_emulation);

    if let Some(accel_profile) = c.accel_profile {
        let _ = device.config_accel_set_profile(accel_profile.into());
    } else if let Some(default) = device.config_accel_default_profile() {
        let _ = device.config_accel_set_profile(default);
    }

    apply_scroll_method(c.scroll_method, c.scroll_button, c.scroll_button_lock, device);
}

fn apply_trackball_settings(c: &Trackball, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else {
        input::SendEventsMode::ENABLED
    });
    let _ = device.config_scroll_set_natural_scroll_enabled(c.natural_scroll);
    let _ = device.config_accel_set_speed(c.accel_speed.0);
    let _ = device.config_middle_emulation_set_enabled(c.middle_emulation);
    let _ = device.config_left_handed_set(c.left_handed);

    if let Some(accel_profile) = c.accel_profile {
        let _ = device.config_accel_set_profile(accel_profile.into());
    } else if let Some(default) = device.config_accel_default_profile() {
        let _ = device.config_accel_set_profile(default);
    }

    apply_scroll_method(c.scroll_method, c.scroll_button, c.scroll_button_lock, device);
}

fn apply_trackpoint_settings(c: &Trackpoint, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else {
        input::SendEventsMode::ENABLED
    });
    let _ = device.config_scroll_set_natural_scroll_enabled(c.natural_scroll);
    let _ = device.config_accel_set_speed(c.accel_speed.0);
    let _ = device.config_left_handed_set(c.left_handed);
    let _ = device.config_middle_emulation_set_enabled(c.middle_emulation);

    if let Some(accel_profile) = c.accel_profile {
        let _ = device.config_accel_set_profile(accel_profile.into());
    } else if let Some(default) = device.config_accel_default_profile() {
        let _ = device.config_accel_set_profile(default);
    }

    apply_scroll_method(c.scroll_method, c.scroll_button, c.scroll_button_lock, device);
}

fn apply_tablet_settings(c: &Tablet, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else {
        input::SendEventsMode::ENABLED
    });

    #[rustfmt::skip]
    const IDENTITY_MATRIX: [f32; 6] = [
        1., 0., 0.,
        0., 1., 0.,
    ];

    let _ = device.config_calibration_set_matrix(
        c.calibration_matrix
            .as_deref()
            .and_then(|m| m.try_into().ok())
            .or(device.config_calibration_default_matrix())
            .unwrap_or(IDENTITY_MATRIX),
    );

    let _ = device.config_left_handed_set(c.left_handed);
}

fn apply_touch_settings(c: &Touch, device: &mut input::Device) {
    let _ = device.config_send_events_set_mode(if c.off {
        input::SendEventsMode::DISABLED
    } else {
        input::SendEventsMode::ENABLED
    });

    #[rustfmt::skip]
    const IDENTITY_MATRIX: [f32; 6] = [
        1., 0., 0.,
        0., 1., 0.,
    ];

    let _ = device.config_calibration_set_matrix(
        c.calibration_matrix
            .as_deref()
            .and_then(|m| m.try_into().ok())
            .or(device.config_calibration_default_matrix())
            .unwrap_or(IDENTITY_MATRIX),
    );
}

/// Helper to apply scroll method settings (shared by mouse, trackball, trackpoint, touchpad).
fn apply_scroll_method(
    scroll_method: Option<niri_config::ScrollMethod>,
    scroll_button: Option<u32>,
    scroll_button_lock: bool,
    device: &mut input::Device,
) {
    if let Some(method) = scroll_method {
        let _ = device.config_scroll_set_method(method.into());

        if method == niri_config::ScrollMethod::OnButtonDown {
            if let Some(button) = scroll_button {
                let _ = device.config_scroll_set_button(button);
            }
            let _ = device.config_scroll_set_button_lock(if scroll_button_lock {
                input::ScrollButtonLockState::Enabled
            } else {
                input::ScrollButtonLockState::Disabled
            });
        }
    } else if let Some(default) = device.config_scroll_default_method() {
        let _ = device.config_scroll_set_method(default);

        if default == input::ScrollMethod::OnButtonDown {
            if let Some(button) = scroll_button {
                let _ = device.config_scroll_set_button(button);
            }
            let _ = device.config_scroll_set_button_lock(if scroll_button_lock {
                input::ScrollButtonLockState::Enabled
            } else {
                input::ScrollButtonLockState::Disabled
            });
        }
    }
}
