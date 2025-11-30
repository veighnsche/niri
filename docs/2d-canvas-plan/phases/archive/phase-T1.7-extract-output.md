# Phase T1.7: Create OutputManager Subsystem

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~1 hour  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - isolates IPC/config state

---

## Goal

Create the `OutputManager` subsystem that handles IPC output reporting, gamma control, VRR, and configuration changes.

---

## What OutputManager Owns

```rust
pub struct OutputManager {
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
    update_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
}
```

---

## OutputManager API

```rust
impl OutputManager {
    pub fn new() -> Self

    // === IPC ===
    pub fn refresh_ipc_outputs(&self, devices: &DeviceManager, niri: &Niri)
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>>

    // === Gamma ===
    pub fn get_gamma_size(&self, devices: &DeviceManager, output: &Output) -> anyhow::Result<u32>
    pub fn set_gamma(&mut self, devices: &mut DeviceManager, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()>

    // === VRR ===
    pub fn set_output_on_demand_vrr(&mut self, devices: &mut DeviceManager, niri: &mut Niri, output: &Output, enable_vrr: bool)

    // === Monitor Control ===
    pub fn set_monitors_active(&mut self, devices: &mut DeviceManager, active: bool)

    // === Configuration ===
    pub fn on_output_config_changed(&mut self, devices: &mut DeviceManager, niri: &mut Niri, config: &Config)
    pub fn mark_config_update_on_resume(&mut self)
    pub fn mark_ignored_nodes_update_on_resume(&mut self)
    pub fn needs_config_update_on_resume(&self) -> bool
    pub fn needs_ignored_nodes_update_on_resume(&self) -> bool
    pub fn clear_resume_flags(&mut self)
}
```

---

## Implementation: `src/backend/tty/outputs.rs`

```rust
//! Output management subsystem for TTY backend.
//!
//! Handles IPC output reporting, gamma control, VRR, and configuration.

use std::sync::{Arc, Mutex};

use smithay::backend::drm::DrmNode;
use smithay::output::Output;

use super::devices::DeviceManager;
use super::helpers;
use crate::backend::IpcOutputMap;
use crate::niri::Niri;
use niri_config::{Config, OutputName};

/// Output management subsystem.
///
/// OWNS:
/// - IPC output map for external queries
/// - Resume update flags
pub struct OutputManager {
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
    update_config_on_resume: bool,
    update_ignored_nodes_on_resume: bool,
}

impl OutputManager {
    pub fn new() -> Self {
        Self {
            ipc_outputs: Arc::new(Mutex::new(HashMap::new())),
            update_config_on_resume: false,
            update_ignored_nodes_on_resume: false,
        }
    }

    // === IPC ===

    pub fn refresh_ipc_outputs(&self, devices: &DeviceManager, niri: &Niri) {
        let mut ipc_outputs = self.ipc_outputs.lock().unwrap();
        ipc_outputs.clear();

        for (node, device) in devices.iter() {
            for (crtc, surface) in device.surfaces() {
                // Build IPC output info
                // ...
            }
        }
    }

    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        Arc::clone(&self.ipc_outputs)
    }

    // === Gamma ===

    pub fn get_gamma_size(
        &self,
        devices: &DeviceManager,
        output: &Output,
    ) -> anyhow::Result<u32> {
        let (node, crtc) = Self::find_output(output)?;
        let device = devices.get(&node).context("missing device")?;
        let surface = device.surface(crtc).context("missing surface")?;

        if let Some(gamma_props) = &surface.gamma_props {
            gamma_props.gamma_size(device.drm())
        } else {
            // Legacy gamma
            let info = device.drm().get_crtc(crtc)?;
            Ok(info.gamma_length() as u32)
        }
    }

    pub fn set_gamma(
        &mut self,
        devices: &mut DeviceManager,
        output: &Output,
        ramp: Option<Vec<u16>>,
    ) -> anyhow::Result<()> {
        let (node, crtc) = Self::find_output(output)?;
        let device = devices.get_mut(&node).context("missing device")?;
        let surface = device.surface_mut(crtc).context("missing surface")?;

        if let Some(gamma_props) = &mut surface.gamma_props {
            gamma_props.set_gamma(device.drm(), ramp.as_deref())
        } else {
            helpers::set_gamma_for_crtc(device.drm(), crtc, ramp.as_deref())
        }
    }

    // === VRR ===

    pub fn set_output_on_demand_vrr(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        enable_vrr: bool,
    ) {
        let Some((node, crtc)) = Self::find_output(output) else {
            return;
        };
        let Some(device) = devices.get_mut(&node) else {
            return;
        };
        let Some(surface) = device.surface_mut(crtc) else {
            return;
        };

        // Toggle VRR
        surface.vrr_enabled = enable_vrr;
        // ...
    }

    // === Monitor Control ===

    pub fn set_monitors_active(&mut self, devices: &mut DeviceManager, active: bool) {
        for device in devices.iter_mut() {
            for surface in device.surfaces_mut() {
                // Enable/disable DPMS
                // ...
            }
        }
    }

    // === Configuration ===

    pub fn on_output_config_changed(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        config: &Config,
    ) {
        // Re-scan and apply config to all outputs
        for (node, device) in devices.iter_mut() {
            // ...
        }
        self.refresh_ipc_outputs(devices, niri);
    }

    pub fn mark_config_update_on_resume(&mut self) {
        self.update_config_on_resume = true;
    }

    pub fn mark_ignored_nodes_update_on_resume(&mut self) {
        self.update_ignored_nodes_on_resume = true;
    }

    pub fn needs_config_update_on_resume(&self) -> bool {
        self.update_config_on_resume
    }

    pub fn needs_ignored_nodes_update_on_resume(&self) -> bool {
        self.update_ignored_nodes_on_resume
    }

    pub fn clear_resume_flags(&mut self) {
        self.update_config_on_resume = false;
        self.update_ignored_nodes_on_resume = false;
    }

    fn find_output(output: &Output) -> Option<(DrmNode, crtc::Handle)> {
        let state = output.user_data().get::<super::types::TtyOutputState>()?;
        Some((state.node, state.crtc))
    }
}
```

---

## Final Tty Structure

After all phases, `Tty` becomes a thin coordinator:

```rust
pub struct Tty {
    // Core integration (not subsystem-able)
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,

    // Subsystems (OWN their state)
    pub(crate) devices: DeviceManager,
    pub(crate) render: RenderManager,
    pub(crate) outputs: OutputManager,
}

impl Tty {
    // Thin delegation
    pub fn render(&mut self, niri: &mut Niri, output: &Output, target: Duration) -> RenderResult {
        self.render.render(&mut self.devices, niri, output, target)
    }

    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        self.outputs.ipc_outputs()
    }

    // Event dispatch
    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        match event {
            UdevEvent::Added { device_id, path } => {
                self.devices.device_added(device_id, &path, ...);
            }
            // ...
        }
    }
}
```

---

## Verification Checklist

- [ ] IPC output queries work (`niri msg outputs`)
- [ ] Gamma control works
- [ ] VRR toggle works
- [ ] Config reload works
- [ ] Monitor power control works
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/outputs.rs` | `OutputManager` subsystem (~650 LOC) |
| `src/backend/tty/mod.rs` | Final thin coordinator structure |

---

## Final Architecture Summary

```
src/backend/tty/
├── mod.rs          # Tty thin coordinator (~200 LOC)
├── types.rs        # Type definitions (~150 LOC)
├── helpers.rs      # Pure functions (~400 LOC)
├── devices.rs      # DeviceManager subsystem (~1100 LOC)
│                   #   - OutputDevice
│                   #   - Device lifecycle
│                   #   - Connector handling
├── render.rs       # RenderManager subsystem (~500 LOC)
├── outputs.rs      # OutputManager subsystem (~650 LOC)
└── gamma.rs        # GammaProps (~150 LOC)

Total: ~3150 LOC (down from 3473)
```

### Key Improvements

| Metric | Before | After |
|--------|--------|-------|
| `Tty` fields | 15+ | 7 |
| God object | Yes | No |
| Encapsulation | None | ✅ Private fields |
| Testability | Low | ✅ Each subsystem |
| Cognitive load | High | ⬇️ Per-subsystem |
