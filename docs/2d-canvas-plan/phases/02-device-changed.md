# Phase 02: Move `device_changed` to DeviceManager

> **Status**: ⏳ PENDING  
> **LOC**: ~163  
> **Target**: `src/backend/tty/devices.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 737-899

---

## Overview

Move the `device_changed` method from `Tty` to `DeviceManager`. This method handles connector scanning when a DRM device reports changes (monitor hotplug).

---

## Current Signature (in Tty)

```rust
fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool)
```

---

## New Signature (in DeviceManager)

```rust
pub fn device_changed(
    &mut self,
    device_id: dev_t,
    niri: &mut Niri,
    config: &Rc<RefCell<Config>>,
    cleanup: bool,
    should_disable_laptop_panels: bool,
) -> DeviceChangedResult
```

Where:
```rust
pub struct DeviceChangedResult {
    pub needs_device_added: Option<(dev_t, PathBuf)>,
    pub connectors_to_connect: Vec<(DrmNode, connector::Info, crtc::Handle)>,
}
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.ignored_nodes()` | `self.ignored_nodes()` |
| `self.devices.get_mut(&node)` | `self.get_mut(&node)` |
| `self.devices.connector_disconnected(...)` | `self.connector_disconnected(...)` |
| `self.devices.values()` | `self.values()` |
| `self.config.borrow()` | `config.borrow()` |
| `self.should_disable_laptop_panels(...)` | `should_disable_laptop_panels` param |
| `self.device_added(...)` | Return in result |
| `self.on_output_config_changed(...)` | Caller handles |

---

## Implementation Steps

### Step 1: Define result type in devices.rs

```rust
pub struct DeviceChangedResult {
    /// If set, device_added should be called with these args
    pub needs_device_added: Option<(dev_t, PathBuf)>,
    /// Connectors that need to be connected
    pub connectors_to_connect: Vec<(DrmNode, connector::Info, crtc::Handle)>,
}
```

### Step 2: Move method body

Copy lines 737-899 from mod.rs, applying transformations.

### Step 3: Remove calls to Tty methods

Instead of calling:
- `self.device_added(...)` — return in `needs_device_added`
- `self.on_output_config_changed(...)` — caller handles

### Step 4: Create delegation in Tty

```rust
impl Tty {
    fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool) {
        let should_disable = self.should_disable_laptop_panels(niri.outputs.lid_closed());
        
        let result = self.devices.device_changed(
            device_id,
            niri,
            &self.config,
            cleanup,
            should_disable,
        );
        
        // Handle device_added if needed
        if let Some((dev_id, path)) = result.needs_device_added {
            if let Err(err) = self.device_added(dev_id, &path, niri) {
                warn!("error adding device: {err:?}");
            }
        }
        
        // Connect any new connectors
        for (node, connector, crtc) in result.connectors_to_connect {
            if let Err(err) = self.connector_connected(niri, node, connector, crtc) {
                warn!("error connecting connector: {err:?}");
            }
        }
        
        self.on_output_config_changed(niri);
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

---

## Dependencies

- **Requires**: Phase 01 (device_added)
- **Blocks**: Phase 03 (device_removed)

---

## Notes

- Uses DrmScanner to detect connector changes
- Calls connector_disconnected for removed connectors (already in DeviceManager)
- Handles duplicate make/model/serial detection
- Triggers connector cleanup on device resume
