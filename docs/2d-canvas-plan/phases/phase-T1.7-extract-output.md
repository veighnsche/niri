# Phase T1.7: Extract Output Management

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐ Medium - organizes remaining output code

---

## Goal

Extract output management functions into `src/backend/tty/output.rs`. These are the remaining functions that deal with output configuration:
- IPC output reporting
- Gamma control
- VRR control
- Monitor enable/disable
- Config changes

---

## What Moves

### IPC Output Management (lines 2057-2173, ~120 lines)

```rust
impl Tty {
    fn refresh_ipc_outputs(&self, niri: &mut Niri)
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>>
}
```

### Gamma Control (lines 2012-2056, ~45 lines)

```rust
impl Tty {
    pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32>
    pub fn set_gamma(&mut self, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()>
}
```

### VRR Control (lines 2205-2234, ~30 lines)

```rust
impl Tty {
    pub fn set_output_on_demand_vrr(&mut self, niri: &mut Niri, output: &Output, enable_vrr: bool)
}
```

### Monitor Control (lines 2186-2204, ~20 lines)

```rust
impl Tty {
    pub fn set_monitors_active(&mut self, active: bool)
}
```

### Config Changes (lines 2235-2500, ~265 lines)

```rust
impl Tty {
    pub fn update_ignored_nodes_config(&mut self, niri: &mut Niri)
    fn should_disable_laptop_panels(&self, is_lid_closed: bool) -> bool
    pub fn on_output_config_changed(&mut self, niri: &mut Niri)
}
```

### Device Access (lines 2501-2530, ~30 lines)

```rust
impl Tty {
    pub fn get_device_from_node(&mut self, node: DrmNode) -> Option<&mut OutputDevice>
    pub fn disconnected_connector_name_by_name_match(&self, target: &str) -> Option<OutputName>
}
```

---

## Why This is Good Architecture

1. **Cleanup phase** - Organizes remaining code after larger extractions
2. **Related functionality** - All output control in one place
3. **Config handling isolated** - Config changes don't pollute other code
4. **IPC separate** - Output reporting is distinct from output management

---

## Target: `src/backend/tty/output.rs`

```rust
//! Output management for TTY backend.
//!
//! Handles:
//! - IPC output reporting
//! - Gamma control
//! - VRR (Variable Refresh Rate) control
//! - Monitor enable/disable
//! - Configuration changes

use std::sync::{Arc, Mutex};
use smithay::backend::drm::DrmNode;
use smithay::output::Output;

use super::Tty;
use super::device::OutputDevice;
use crate::backend::IpcOutputMap;
use crate::niri::Niri;
use niri_config::OutputName;

impl Tty {
    // === IPC ===

    /// Refresh the IPC output map with current state.
    pub(super) fn refresh_ipc_outputs(&self, niri: &mut Niri) {
        // ...
    }

    /// Get the IPC output map for external queries.
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        Arc::clone(&self.ipc_outputs)
    }

    // === Gamma ===

    /// Get the gamma LUT size for an output.
    pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32> {
        // ...
    }

    /// Set the gamma LUT for an output.
    pub fn set_gamma(
        &mut self,
        output: &Output,
        ramp: Option<Vec<u16>>,
    ) -> anyhow::Result<()> {
        // ...
    }

    // === VRR ===

    /// Enable or disable on-demand VRR for an output.
    pub fn set_output_on_demand_vrr(
        &mut self,
        niri: &mut Niri,
        output: &Output,
        enable_vrr: bool,
    ) {
        // ...
    }

    // === Monitor Control ===

    /// Enable or disable all monitors (for screen lock, etc.).
    pub fn set_monitors_active(&mut self, active: bool) {
        // ...
    }

    // === Configuration ===

    /// Update ignored DRM nodes based on config.
    pub fn update_ignored_nodes_config(&mut self, niri: &mut Niri) {
        // ...
    }

    /// Check if laptop panels should be disabled (lid closed).
    pub(super) fn should_disable_laptop_panels(&self, is_lid_closed: bool) -> bool {
        // ...
    }

    /// Handle output configuration changes.
    pub fn on_output_config_changed(&mut self, niri: &mut Niri) {
        // ...
    }

    // === Device Access ===

    /// Get a mutable reference to a device by DRM node.
    pub fn get_device_from_node(&mut self, node: DrmNode) -> Option<&mut OutputDevice> {
        self.devices.get_mut(&node)
    }

    /// Find a disconnected connector name by matching against target.
    pub fn disconnected_connector_name_by_name_match(
        &self,
        target: &str,
    ) -> Option<OutputName> {
        // ...
    }
}
```

---

## Also Extract: GammaProps

The `impl GammaProps` block (lines 2534-2670, ~140 lines) should go to `gamma.rs`:

```rust
// src/backend/tty/gamma.rs

//! Gamma LUT management for TTY backend.

use smithay::backend::drm::DrmDevice;
use smithay::reexports::drm::control::{crtc, property};

pub(super) struct GammaProps {
    pub crtc: crtc::Handle,
    pub gamma_lut: property::Handle,
    pub gamma_lut_size: u64,
    pub previous_blob: Option<NonZeroU64>,
}

impl GammaProps {
    pub fn new(device: &DrmDevice, crtc: crtc::Handle) -> anyhow::Result<Self> {
        // ...
    }

    pub fn gamma_size(&self, device: &DrmDevice) -> anyhow::Result<u32> {
        // ...
    }

    pub fn set_gamma(&mut self, device: &DrmDevice, gamma: Option<&[u16]>) -> anyhow::Result<()> {
        // ...
    }

    pub fn restore_gamma(&self, device: &DrmDevice) -> anyhow::Result<()> {
        // ...
    }
}
```

---

## Verification

- [ ] IPC output queries work (`niri msg outputs`)
- [ ] Gamma control works
- [ ] VRR toggle works
- [ ] Config reload works
- [ ] `cargo check` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/output.rs` | Created (~530 LOC) |
| `src/backend/tty/gamma.rs` | Created (~150 LOC) |
| `src/backend/tty/mod.rs` | Removed output methods, added `mod output; mod gamma` |

---

## Final State

After this phase, `mod.rs` should contain only:
- `Tty` struct definition
- `new()` constructor
- `init()` initialization
- Public API methods that delegate to other modules
- Module declarations and re-exports

Target: ~300 LOC in `mod.rs`

---

## Verification Checklist (Full Refactor)

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] Monitor hotplug works
- [ ] VT switching works
- [ ] Gamma/VRR control works
- [ ] IPC queries work
- [ ] No functionality regression

---

## Summary

This completes the TTY refactor. The 3473-line monolith is now split into:

| File | LOC | Responsibility |
|------|-----|----------------|
| `mod.rs` | ~300 | Struct, init, coordination |
| `types.rs` | ~150 | Type definitions |
| `device.rs` | ~250 | OutputDevice management |
| `helpers.rs` | ~400 | Pure helper functions |
| `lifecycle.rs` | ~650 | Device add/change/remove |
| `connectors.rs` | ~400 | Connect/disconnect |
| `render.rs` | ~420 | Rendering pipeline |
| `output.rs` | ~530 | Output control |
| `gamma.rs` | ~150 | Gamma LUT |
| **Total** | ~3250 | (similar to original) |
