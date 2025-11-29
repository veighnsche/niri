//! Keyboard and mouse bind resolution.
//!
//! This module handles matching input events to configured bindings.
//! It is intentionally free of State dependencies for testability.

use std::collections::HashSet;

use niri_config::{Action, Bind, Binds, Key, ModKey, Modifiers, SwitchBinds, Trigger};
use smithay::backend::input::{Switch, SwitchState};
use smithay::input::keyboard::{keysyms, Keysym, ModifiersState};

/// Convert XKB modifier state to our Modifiers type.
pub fn modifiers_from_state(mods: ModifiersState) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    if mods.ctrl {
        modifiers |= Modifiers::CTRL;
    }
    if mods.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if mods.alt {
        modifiers |= Modifiers::ALT;
    }
    if mods.logo {
        modifiers |= Modifiers::SUPER;
    }
    if mods.iso_level3_shift {
        modifiers |= Modifiers::ISO_LEVEL3_SHIFT;
    }
    if mods.iso_level5_shift {
        modifiers |= Modifiers::ISO_LEVEL5_SHIFT;
    }
    modifiers
}

/// Find a bind for the given key input.
///
/// Checks hardcoded binds (VT switching, power key) first, then configured binds.
pub fn find_bind<'a>(
    bindings: impl IntoIterator<Item = &'a Bind>,
    mod_key: ModKey,
    modified: Keysym,
    raw: Option<Keysym>,
    mods: ModifiersState,
    disable_power_key_handling: bool,
) -> Option<Bind> {
    use keysyms::*;

    // Handle hardcoded binds.
    #[allow(non_upper_case_globals)] // wat
    let hardcoded_action = match modified.raw() {
        modified @ KEY_XF86Switch_VT_1..=KEY_XF86Switch_VT_12 => {
            let vt = (modified - KEY_XF86Switch_VT_1 + 1) as i32;
            Some(Action::ChangeVt(vt))
        }
        KEY_XF86PowerOff if !disable_power_key_handling => Some(Action::Suspend),
        _ => None,
    };

    if let Some(action) = hardcoded_action {
        return Some(Bind {
            key: Key {
                // Not entirely correct but it doesn't matter in how we currently use it.
                trigger: Trigger::Keysym(modified),
                modifiers: Modifiers::empty(),
            },
            action,
            repeat: true,
            cooldown: None,
            allow_when_locked: false,
            // In a worst-case scenario, the user has no way to unlock the compositor and a
            // misbehaving client has a keyboard shortcuts inhibitor, "jailing" the user.
            // The user must always be able to change VTs to recover from such a situation.
            // It also makes no sense to inhibit the default power key handling.
            // Hardcoded binds must never be inhibited.
            allow_inhibiting: false,
            hotkey_overlay_title: None,
        });
    }

    let trigger = Trigger::Keysym(raw?);
    find_configured_bind(bindings, mod_key, trigger, mods)
}

/// Find a configured bind matching the trigger and modifiers.
pub fn find_configured_bind<'a>(
    bindings: impl IntoIterator<Item = &'a Bind>,
    mod_key: ModKey,
    trigger: Trigger,
    mods: ModifiersState,
) -> Option<Bind> {
    // Handle configured binds.
    let mut modifiers = modifiers_from_state(mods);

    let mod_down = modifiers_from_state(mods).contains(mod_key.to_modifiers());
    if mod_down {
        modifiers |= Modifiers::COMPOSITOR;
    }

    for bind in bindings {
        if bind.key.trigger != trigger {
            continue;
        }

        let mut bind_modifiers = bind.key.modifiers;
        if bind_modifiers.contains(Modifiers::COMPOSITOR) {
            bind_modifiers |= mod_key.to_modifiers();
        } else if bind_modifiers.contains(mod_key.to_modifiers()) {
            bind_modifiers |= Modifiers::COMPOSITOR;
        }

        if bind_modifiers == modifiers {
            return Some(bind.clone());
        }
    }

    None
}

/// Find switch action (lid open/close, tablet mode on/off).
pub fn find_configured_switch_action(
    bindings: &SwitchBinds,
    switch: Switch,
    state: SwitchState,
) -> Option<Action> {
    let switch_action = match (switch, state) {
        (Switch::Lid, SwitchState::Off) => &bindings.lid_open,
        (Switch::Lid, SwitchState::On) => &bindings.lid_close,
        (Switch::TabletMode, SwitchState::Off) => &bindings.tablet_mode_off,
        (Switch::TabletMode, SwitchState::On) => &bindings.tablet_mode_on,
        _ => unreachable!(),
    };
    switch_action
        .as_ref()
        .map(|switch_action| Action::Spawn(switch_action.spawn.clone()))
}

/// Get modifiers that have bindings for given triggers.
pub fn mods_with_binds(mod_key: ModKey, binds: &Binds, triggers: &[Trigger]) -> HashSet<Modifiers> {
    let mut rv = HashSet::new();
    for bind in &binds.0 {
        if !triggers.contains(&bind.key.trigger) {
            continue;
        }

        let mut mods = bind.key.modifiers;
        if mods.contains(Modifiers::COMPOSITOR) {
            mods.remove(Modifiers::COMPOSITOR);
            mods.insert(mod_key.to_modifiers());
        }

        rv.insert(mods);
    }

    rv
}

/// Get modifiers that have mouse button bindings.
pub fn mods_with_mouse_binds(mod_key: ModKey, binds: &Binds) -> HashSet<Modifiers> {
    mods_with_binds(
        mod_key,
        binds,
        &[
            Trigger::MouseLeft,
            Trigger::MouseRight,
            Trigger::MouseMiddle,
            Trigger::MouseBack,
            Trigger::MouseForward,
        ],
    )
}

/// Get modifiers that have mouse wheel bindings.
pub fn mods_with_wheel_binds(mod_key: ModKey, binds: &Binds) -> HashSet<Modifiers> {
    mods_with_binds(
        mod_key,
        binds,
        &[
            Trigger::WheelScrollUp,
            Trigger::WheelScrollDown,
            Trigger::WheelScrollLeft,
            Trigger::WheelScrollRight,
        ],
    )
}

/// Get modifiers that have touchpad scroll bindings.
pub fn mods_with_finger_scroll_binds(mod_key: ModKey, binds: &Binds) -> HashSet<Modifiers> {
    mods_with_binds(
        mod_key,
        binds,
        &[
            Trigger::TouchpadScrollUp,
            Trigger::TouchpadScrollDown,
            Trigger::TouchpadScrollLeft,
            Trigger::TouchpadScrollRight,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_from_state() {
        let mods = ModifiersState {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        let result = modifiers_from_state(mods);
        assert!(result.contains(Modifiers::CTRL));
        assert!(result.contains(Modifiers::SHIFT));
        assert!(!result.contains(Modifiers::ALT));
        assert!(!result.contains(Modifiers::SUPER));
    }

    #[test]
    fn test_modifiers_from_state_empty() {
        let mods = ModifiersState::default();
        let result = modifiers_from_state(mods);
        assert!(result.is_empty());
    }

    #[test]
    fn test_modifiers_from_state_all() {
        let mods = ModifiersState {
            ctrl: true,
            shift: true,
            alt: true,
            logo: true,
            iso_level3_shift: true,
            iso_level5_shift: true,
            ..Default::default()
        };
        let result = modifiers_from_state(mods);
        assert!(result.contains(Modifiers::CTRL));
        assert!(result.contains(Modifiers::SHIFT));
        assert!(result.contains(Modifiers::ALT));
        assert!(result.contains(Modifiers::SUPER));
        assert!(result.contains(Modifiers::ISO_LEVEL3_SHIFT));
        assert!(result.contains(Modifiers::ISO_LEVEL5_SHIFT));
    }
}
