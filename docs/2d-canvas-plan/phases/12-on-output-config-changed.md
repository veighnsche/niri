# Phase 12: Move `on_output_config_changed` to OutputManager

> **Status**: ⏳ PENDING  
> **LOC**: ~192  
> **Target**: `src/backend/tty/outputs.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 2129-2320

---

## Overview

Move the `on_output_config_changed` method from `Tty` to `OutputManager`. This is the second largest method to move — it handles configuration changes for outputs (mode changes, VRR, enabling/disabling outputs).

---

## Current Signature (in Tty)

```rust
pub fn on_output_config_changed(&mut self, niri: &mut Niri)
```

---

## New Signature (in OutputManager)

```rust
pub fn on_output_config_changed(
    &mut self,
    devices: &mut DeviceManager,
    niri: &mut Niri,
    config: &Rc<RefCell<Config>>,
    session_active: bool,
    should_disable_laptop_panels: bool,
) -> OutputConfigChangedResult
```

Where:
```rust
pub struct OutputConfigChangedResult {
    pub to_disconnect: Vec<(DrmNode, crtc::Handle)>,
    pub to_connect: Vec<(DrmNode, connector::Info, crtc::Handle)>,
}
```

---

## Complexity

This method is complex because it:
1. Iterates all devices and surfaces
2. Checks config for each output
3. Disconnects outputs marked as "off"
4. Changes modes on existing outputs
5. Toggles VRR
6. Connects new outputs
7. Calls connector_connected and connector_disconnected

---

## Implementation Steps

### Step 1: Define result type

```rust
pub struct OutputConfigChangedResult {
    pub to_disconnect: Vec<(DrmNode, crtc::Handle)>,
    pub to_connect: Vec<(DrmNode, connector::Info, crtc::Handle, OutputName)>,
}
```

### Step 2: Move core logic

Move the iteration and decision logic to OutputManager, but return actions instead of executing them:

```rust
impl OutputManager {
    pub fn on_output_config_changed(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
        session_active: bool,
        should_disable_laptop_panels: bool,
    ) -> OutputConfigChangedResult {
        let _span = tracy_client::span!("OutputManager::on_output_config_changed");

        if !session_active {
            self.mark_config_update_on_resume();
            return OutputConfigChangedResult::default();
        }
        self.clear_config_update_on_resume();

        let should_disable = |connector: &str| {
            should_disable_laptop_panels && is_laptop_panel(connector)
        };

        let mut to_disconnect = vec![];
        let mut to_connect = vec![];

        // ... (iteration logic from mod.rs lines 2146-2303)
        // Collect to_disconnect and to_connect instead of calling methods

        OutputConfigChangedResult { to_disconnect, to_connect }
    }
}
```

### Step 3: Create delegation in Tty

```rust
impl Tty {
    pub fn on_output_config_changed(&mut self, niri: &mut Niri) {
        let should_disable = self.should_disable_laptop_panels(niri.outputs.lid_closed());
        
        let result = self.outputs.on_output_config_changed(
            &mut self.devices,
            niri,
            &self.config,
            self.session.is_active(),
            should_disable,
        );
        
        // Handle disconnections
        for (node, crtc) in result.to_disconnect {
            self.devices.connector_disconnected(niri, node, crtc);
        }
        
        // Sort by output name for predictable ordering
        let mut to_connect = result.to_connect;
        to_connect.sort_unstable_by(|a, b| a.3.compare(&b.3));
        
        // Handle connections
        for (node, connector, crtc, _name) in to_connect {
            if let Err(err) = self.connector_connected(niri, node, connector, crtc) {
                warn!("error connecting connector: {err:?}");
            }
        }
        
        self.refresh_ipc_outputs(niri);
    }
}
```

### Step 4: Handle inline mode changes

The method also changes modes inline. This part can stay in OutputManager since it just modifies surface state.

---

## Verification

```bash
cargo check
cargo test
```

Test: Config reload should work, output toggling should work.

---

## Dependencies

- **Requires**: Phase 04 (DeviceManager), Phase 08-11
- **Blocks**: Phase 13 (final cleanup)

---

## Notes

- Largest output management method
- Handles lid close/open (laptop panel disable)
- Mode changes, VRR changes
- Enabling/disabling outputs via config
- Complex state machine
