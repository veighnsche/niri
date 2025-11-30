# Phase I1.2: Extract Device Management

> **Status**: ✅ COMPLETE (TEAM_085)  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐ Medium - isolates libinput configuration

---

## Goal

Extract device management into `src/input/device.rs`. This includes:

1. Device add/remove handlers
2. Libinput settings application
3. Device capability detection

---

## What Moves

From `mod.rs`:

```rust
// Lines 238-265 - Device lifecycle
impl State {
    fn on_device_added(&mut self, device: impl Device) { ... }
    fn on_device_removed(&mut self, device: impl Device) { ... }
}

// Lines 4378-4662 (~285 lines) - Libinput configuration
pub fn apply_libinput_settings(config: &niri_config::Input, device: &mut input::Device) {
    // Touchpad settings
    // Mouse settings
    // Trackball settings
    // Trackpoint settings
    // Tablet settings
    // Touch settings
}
```

---

## Why This is Reasonable

1. **Self-contained** - Device settings don't depend on compositor state
2. **Locality** - All device-related code in one place
3. **Maintainability** - Adding new device types is scoped to one file

**Honest assessment:** This is mostly code organization, not deep architecture. But `apply_libinput_settings` is 285 lines of device configuration that doesn't belong in the main input dispatcher.

---

## Target: `src/input/device.rs`

```rust
//! Input device management and configuration.
//!
//! Handles device lifecycle and libinput settings.

use smithay::backend::input::{Device, DeviceCapability};
use smithay::wayland::tablet_manager::{TabletDescriptor, TabletSeatTrait};

impl State {
    pub(super) fn on_device_added(&mut self, device: impl Device) { ... }
    pub(super) fn on_device_removed(&mut self, device: impl Device) { ... }
}

/// Apply libinput configuration to a device.
pub fn apply_libinput_settings(config: &niri_config::Input, device: &mut input::Device) {
    // Touchpad
    // Mouse
    // Trackball
    // Trackpoint
    // Tablet
    // Touch
}
```

---

## Verification

- [x] Device add/remove works
- [x] Libinput settings apply correctly
- [x] `cargo check` passes

## Implementation Notes (TEAM_085)

**Improvements made:**
- Extracted helper function `apply_scroll_method()` to reduce code duplication
- Split `apply_libinput_settings()` into per-device-type functions
- Better code organization with clear separation

**Files:**
- `src/input/device.rs` - 286 lines (new)
- `src/input/mod.rs` - reduced from 4963 to 4655 lines (-308 lines)
