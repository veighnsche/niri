# Phase T1.4: Extract Device Lifecycle

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐⭐ High - isolates complex device management

---

## Goal

Extract device lifecycle management into `src/backend/tty/lifecycle.rs`. This is the largest extraction but has clear boundaries:
- Device added (udev)
- Device changed (hotplug)
- Device removed (unplug)
- Session events (VT switch)
- Udev events

---

## What Moves

### Event Handlers (lines 542-711, ~170 lines)

```rust
impl Tty {
    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent)
    fn on_session_event(&mut self, niri: &mut Niri, event: SessionEvent)
}
```

### Device Add/Change/Remove (lines 712-1179, ~470 lines)

```rust
impl Tty {
    fn device_added(&mut self, device_id: dev_t, path: PathBuf, niri: &mut Niri)
    fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool)
    fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri)
}
```

---

## Why This is Good Architecture

1. **Clear lifecycle boundary** - All device add/remove logic in one place
2. **Complex but isolated** - These functions are large but don't touch rendering
3. **Event-driven** - Clear input (events) to output (state changes)
4. **Easier debugging** - Device issues are isolated to one file

---

## Method Breakdown

### `on_udev_event` (~60 lines)
Dispatches udev events to `device_added`, `device_changed`, `device_removed`.

### `on_session_event` (~110 lines)
Handles VT switching:
- Pause: Disable DRM, pause devices
- Resume: Re-enable DRM, resume devices, update configs

### `device_added` (~195 lines)
When a new GPU is detected:
1. Open DRM device
2. Initialize GBM
3. Set up GPU manager
4. Create dmabuf global if primary
5. Scan for connectors

### `device_changed` (~165 lines)
When GPU configuration changes (hotplug):
1. Re-scan connectors
2. Handle new/removed connectors
3. Update surfaces

### `device_removed` (~110 lines)
When a GPU is unplugged:
1. Remove surfaces
2. Clean up dmabuf global
3. Remove from GPU manager

---

## Target: `src/backend/tty/lifecycle.rs`

```rust
//! Device lifecycle management for TTY backend.
//!
//! Handles:
//! - Device add/change/remove events from udev
//! - Session events (VT switching)
//! - GPU hotplug scenarios

use std::path::PathBuf;
use libc::dev_t;
use smithay::backend::session::Event as SessionEvent;
use smithay::backend::udev::UdevEvent;

use super::Tty;
use crate::niri::Niri;

impl Tty {
    /// Handle udev events (device add/change/remove).
    pub(super) fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                if !self.session.is_active() {
                    return;
                }
                self.device_added(device_id, path, niri);
            }
            UdevEvent::Changed { device_id } => {
                if !self.session.is_active() {
                    return;
                }
                self.device_changed(device_id, niri, false);
            }
            UdevEvent::Removed { device_id } => {
                if !self.session.is_active() {
                    return;
                }
                self.device_removed(device_id, niri);
            }
        }
    }

    /// Handle session events (VT switch, etc.).
    pub(super) fn on_session_event(&mut self, niri: &mut Niri, event: SessionEvent) {
        match event {
            SessionEvent::PauseSession => {
                // Disable DRM, pause devices...
            }
            SessionEvent::ActivateSession => {
                // Re-enable DRM, resume devices...
            }
        }
    }

    /// Handle a new DRM device being added.
    pub(super) fn device_added(
        &mut self,
        device_id: dev_t,
        path: PathBuf,
        niri: &mut Niri,
    ) {
        // 1. Open DRM device
        // 2. Initialize GBM
        // 3. Set up GPU manager
        // 4. Create dmabuf global if primary
        // 5. Scan for connectors
    }

    /// Handle DRM device configuration change (hotplug).
    pub(super) fn device_changed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        cleanup: bool,
    ) {
        // 1. Re-scan connectors
        // 2. Handle new/removed connectors
        // 3. Update surfaces
    }

    /// Handle DRM device removal.
    pub(super) fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri) {
        // 1. Remove surfaces
        // 2. Clean up dmabuf global
        // 3. Remove from GPU manager
    }
}
```

---

## Dependencies

This module uses:
- `helpers.rs`: `primary_node_from_render_node`, `ignored_nodes_from_config`
- `device.rs`: `OutputDevice`, `CrtcInfo`
- `connectors.rs`: `connector_connected`, `connector_disconnected`

Order matters: Extract helpers and device first, then lifecycle.

---

## Verification

- [ ] Device hotplug works (test with `echo 1 > /sys/class/drm/.../status`)
- [ ] VT switching works (`Ctrl+Alt+F2`, `Ctrl+Alt+F1`)
- [ ] `cargo check` passes
- [ ] GPU add/remove logs appear correctly

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/lifecycle.rs` | Created (~650 LOC) |
| `src/backend/tty/mod.rs` | Removed lifecycle methods, added `mod lifecycle` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.5: Extract Connectors](phase-T1.5-extract-connectors.md).
