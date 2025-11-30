# Phase 04: Move `connector_connected` to DeviceManager

> **Status**: ✅ COMPLETE  
> **LOC**: ~347  
> **Target**: `src/backend/tty/devices.rs`  
> **Source**: `src/backend/tty/mod.rs` lines 1011-1357

---

## Overview

Move the `connector_connected` method from `Tty` to `DeviceManager`. This is the largest single method to move — it handles creating a DRM surface when a monitor is connected.

Note: `connector_disconnected` is already in DeviceManager.

---

## Current Signature (in Tty)

```rust
fn connector_connected(
    &mut self,
    niri: &mut Niri,
    node: DrmNode,
    connector: connector::Info,
    crtc: crtc::Handle,
) -> anyhow::Result<()>
```

---

## New Signature (in DeviceManager)

```rust
pub fn connector_connected(
    &mut self,
    niri: &mut Niri,
    node: DrmNode,
    connector: connector::Info,
    crtc: crtc::Handle,
    config: &Rc<RefCell<Config>>,
    debug_tint: bool,
) -> anyhow::Result<()>
```

---

## Required Transformations

| Old Code | New Code |
|----------|----------|
| `self.devices.primary_render_node()` | `self.primary_render_node()` |
| `self.devices.get_mut(&node)` | `self.get_mut(&node)` |
| `self.devices.gpu_manager_mut()` | `self.gpu_manager_mut()` |
| `self.config.borrow()` | `config.borrow()` |
| `self.render.debug_tint()` | `debug_tint` param |

---

## Implementation Steps

### Step 1: Add imports to devices.rs

```rust
use smithay::backend::drm::compositor::{DrmCompositor, FrameFlags};
use smithay::backend::drm::VrrSupport;
use smithay::output::{Mode, Output, OutputModeSource, PhysicalProperties};
// ... other imports
```

### Step 2: Move helper imports

The method uses these helpers from mod.rs:
- `format_connector_name` (already in devices.rs)
- `pick_mode` (in helpers.rs)
- `find_drm_property` (in helpers.rs)
- `reset_hdr` (in helpers.rs)
- `get_panel_orientation` (in helpers.rs)
- `set_gamma_for_crtc` (in helpers.rs)
- `surface_dmabuf_feedback` (in helpers.rs)
- `refresh_interval` (in helpers.rs)
- `calculate_drm_mode_from_modeline` (in helpers.rs)

Add imports:
```rust
use super::helpers::{
    pick_mode, find_drm_property, reset_hdr, get_panel_orientation,
    set_gamma_for_crtc, surface_dmabuf_feedback, refresh_interval,
    calculate_drm_mode_from_modeline,
};
```

### Step 3: Move method body

Copy lines 1011-1357 from mod.rs, applying transformations.

### Step 4: Handle split borrows

The method has complex borrow patterns:
```rust
// Extract values before mutable borrow
let device_render_node = device.render_node;
let _ = device; // drop reference

// Get gpu_manager
let renderer = self.gpu_manager_mut().single_renderer(&render_node)?;

// Re-borrow device
let device = self.get_mut(&node)?;
```

This pattern already exists in the code — preserve it.

### Step 5: Create delegation in Tty

```rust
impl Tty {
    fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    ) -> anyhow::Result<()> {
        self.devices.connector_connected(
            niri,
            node,
            connector,
            crtc,
            &self.config,
            self.render.debug_tint(),
        )
    }
}
```

---

## Verification

```bash
cargo check
cargo test
```

Manual test (if possible): plug in a monitor.

---

## Dependencies

- **Requires**: Phase 01-03
- **Blocks**: Phase 05-07 (render), Phase 08-12 (outputs)

---

## Notes

- Largest method (~347 LOC)
- Creates DRM surface, compositor, output
- Handles VRR configuration
- Builds dmabuf feedback
- Sets up gamma properties
- Complex borrow patterns due to gpu_manager access
