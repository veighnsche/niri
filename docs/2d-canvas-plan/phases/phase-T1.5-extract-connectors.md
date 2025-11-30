# Phase T1.5: Extract Connector Handling

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟡 Medium  
> **Architectural Benefit**: ⭐⭐ Medium - isolates output connection logic

---

## Goal

Extract connector connect/disconnect handling into `src/backend/tty/connectors.rs`.

These are called when:
- A monitor is plugged in
- A monitor is unplugged
- Device scanning finds new/removed connectors

---

## What Moves

### connector_connected (lines 1180-1509, ~330 lines)

```rust
impl Tty {
    fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    )
}
```

This function:
1. Reads EDID for monitor info
2. Creates output name
3. Picks display mode
4. Sets up DRM compositor surface
5. Creates smithay Output
6. Configures VRR if available
7. Sets up dmabuf feedback
8. Registers with niri

### connector_disconnected (lines 1510-1557, ~50 lines)

```rust
impl Tty {
    fn connector_disconnected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
    )
}
```

This function:
1. Removes surface
2. Cleans up output state
3. Notifies niri

---

## Why This is Good Architecture

1. **Clear boundary** - Connect/disconnect is distinct from device lifecycle
2. **Complex but focused** - `connector_connected` is 330 lines but does one thing
3. **Hot path isolation** - Monitor plug/unplug is performance-sensitive
4. **Easier debugging** - Monitor issues are isolated to one file

---

## Target: `src/backend/tty/connectors.rs`

```rust
//! Connector connect/disconnect handling for TTY backend.
//!
//! Handles:
//! - Monitor connection (EDID, mode selection, surface creation)
//! - Monitor disconnection (cleanup)

use smithay::backend::drm::DrmNode;
use smithay::reexports::drm::control::{connector, crtc};

use super::Tty;
use super::helpers::{
    get_edid_info, make_output_name, pick_mode, format_connector_name,
    reset_hdr, is_vrr_capable, get_panel_orientation,
};
use super::types::ConnectorProperties;
use crate::niri::Niri;

impl Tty {
    /// Handle a connector becoming connected (monitor plugged in).
    ///
    /// This sets up:
    /// 1. EDID parsing for monitor identification
    /// 2. Mode selection based on config
    /// 3. DRM compositor surface
    /// 4. VRR configuration
    /// 5. Dmabuf feedback
    pub(super) fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    ) {
        let _span = tracy_client::span!("Tty::connector_connected");

        // 1. Get connector properties
        let props = match ConnectorProperties::try_new(&device.drm, connector.handle()) {
            Ok(props) => props,
            Err(err) => {
                warn!("error getting connector properties: {err:?}");
                return;
            }
        };

        // 2. Read EDID
        let edid_info = props.edid_blob
            .as_ref()
            .map(|blob| get_edid_info(blob));

        // 3. Create output name
        let output_name = make_output_name(&node, &connector, edid_info, ...);

        // 4. Pick mode
        let mode = match pick_mode(&connector, &output_config, ...) {
            Ok(mode) => mode,
            Err(err) => {
                warn!("error picking mode: {err:?}");
                return;
            }
        };

        // 5. Create DRM compositor surface
        // ...

        // 6. Create smithay Output
        // ...

        // 7. Configure VRR
        // ...

        // 8. Set up dmabuf feedback
        // ...

        // 9. Register with niri
        niri.add_output(output, ...);
    }

    /// Handle a connector becoming disconnected (monitor unplugged).
    pub(super) fn connector_disconnected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
    ) {
        let _span = tracy_client::span!("Tty::connector_disconnected");

        let device = match self.devices.get_mut(&node) {
            Some(device) => device,
            None => return,
        };

        // 1. Get surface info
        let Some(surface) = device.surfaces.remove(&crtc) else {
            return;
        };

        // 2. Remove known CRTC
        device.known_crtcs.remove(&crtc);

        // 3. Notify niri
        niri.remove_output(&surface.output);
    }
}
```

---

## Dependencies

This module uses:
- `helpers.rs`: EDID parsing, mode selection, connector utilities
- `types.rs`: `ConnectorProperties`, `Surface`, `SurfaceDmabufFeedback`
- `device.rs`: `OutputDevice` access

---

## Verification

- [ ] Monitor hotplug works (unplug/replug monitor)
- [ ] Correct mode is selected
- [ ] VRR is configured when available
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/connectors.rs` | Created (~400 LOC) |
| `src/backend/tty/mod.rs` | Removed connector methods, added `mod connectors` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.6: Extract Render](phase-T1.6-extract-render.md).
