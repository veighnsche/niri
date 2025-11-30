# Phase T1.3: Extract Helper Functions

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - creates testable, pure functions

---

## Goal

Extract all pure helper functions into `src/backend/tty/helpers.rs`. These functions:
- Don't need `&self` or `&mut self`
- Don't access `Tty` or `Niri` state
- Are pure computations on their inputs

This is high-value because these functions become easily unit testable.

---

## What Moves

### DRM Node Helpers (lines 2672-2725)

```rust
// Pure functions for DRM node discovery
fn primary_node_from_render_node(path: &Path) -> Option<(DrmNode, DrmNode)>
fn primary_node_from_config(config: &Config) -> Option<(DrmNode, DrmNode)>
fn ignored_nodes_from_config(config: &Config) -> HashSet<DrmNode>
```

### DRM Property Helpers (lines 2801-2838)

```rust
// Pure functions for DRM property access
fn find_drm_property(drm: &DrmDevice, resource: impl ResourceHandle, name: &str) 
    -> Option<(property::Handle, property::Info, property::RawValue)>
fn get_drm_property(drm: &DrmDevice, resource: impl ResourceHandle, prop: property::Handle) 
    -> Option<property::RawValue>
```

### Mode Calculation (lines 2840-3085)

```rust
// Pure functions for DRM mode calculations
fn refresh_interval(mode: DrmMode) -> Duration
pub fn calculate_drm_mode_from_modeline(modeline: &Modeline) -> anyhow::Result<DrmMode>
pub fn calculate_mode_cvt(width: u16, height: u16, refresh: f64) -> DrmMode
fn modeinfo_name_slice_from_string(mode_name: &str) -> [c_char; 32]
fn pick_mode(connector: &connector::Info, config: &niri_config::Output, ...) -> ...
```

### Connector Helpers (lines 3163-3342)

```rust
// Pure functions for connector handling
fn get_edid_info(edid_blob: &[u8]) -> (Option<String>, Option<String>, Option<String>)
fn reset_hdr(props: &ConnectorProperties) -> anyhow::Result<()>
fn is_vrr_capable(device: &DrmDevice, connector: connector::Handle) -> Option<bool>
fn get_panel_orientation(props: &ConnectorProperties) -> anyhow::Result<Transform>
pub fn set_gamma_for_crtc(device: &DrmDevice, crtc: crtc::Handle, ramp: Option<&[u16]>) -> anyhow::Result<()>
fn format_connector_name(connector: &connector::Info) -> String
fn make_output_name(device_node: &DrmNode, connector: &connector::Info, edid_info: ...) -> OutputName
```

### DmaBuf Feedback (lines 2727-2799)

```rust
// Pure function for building dmabuf feedback
fn surface_dmabuf_feedback(
    compositor: &GbmDrmCompositor,
    primary_formats: FormatSet,
    primary_render_node: DrmNode,
    surface_render_node: Option<DrmNode>,
    surface_scanout_node: DrmNode,
) -> Result<SurfaceDmabufFeedback, io::Error>
```

### Suspend Helper (lines 2865-2878)

```rust
#[cfg(feature = "dbus")]
fn suspend() -> anyhow::Result<()>
```

---

## Why This is Excellent Architecture

1. **Pure functions** - No state mutation, predictable outputs
2. **Easily testable** - Can unit test mode calculations, property lookups
3. **Reusable** - Could be used by other backends
4. **Clear contracts** - Input → Output, no side effects

---

## Target: `src/backend/tty/helpers.rs`

```rust
//! Pure helper functions for TTY backend.
//!
//! These functions have no State or Tty dependencies and can be
//! tested in isolation.

use std::collections::HashSet;
use std::ffi::c_char;
use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, bail, ensure, Context};
use niri_config::{Config, OutputName};
use smithay::backend::drm::{DrmDevice, DrmNode, NodeType};
use smithay::reexports::drm::control::{connector, crtc, property, Mode as DrmMode, ResourceHandle};
use smithay::utils::Transform;

// === DRM Node Discovery ===

pub(super) fn primary_node_from_render_node(path: &Path) -> Option<(DrmNode, DrmNode)> {
    // ...
}

pub(super) fn primary_node_from_config(config: &Config) -> Option<(DrmNode, DrmNode)> {
    // ...
}

pub(super) fn ignored_nodes_from_config(config: &Config) -> HashSet<DrmNode> {
    // ...
}

// === DRM Properties ===

pub(super) fn find_drm_property(
    drm: &DrmDevice,
    resource: impl ResourceHandle,
    name: &str,
) -> Option<(property::Handle, property::Info, property::RawValue)> {
    // ...
}

pub(super) fn get_drm_property(
    drm: &DrmDevice,
    resource: impl ResourceHandle,
    prop: property::Handle,
) -> Option<property::RawValue> {
    // ...
}

// === Mode Calculations ===

pub(super) fn refresh_interval(mode: DrmMode) -> Duration {
    // ...
}

/// Calculate a DRM mode from a modeline configuration.
pub fn calculate_drm_mode_from_modeline(modeline: &Modeline) -> anyhow::Result<DrmMode> {
    // ... (with validation)
}

/// Calculate a DRM mode using CVT formula.
pub fn calculate_mode_cvt(width: u16, height: u16, refresh: f64) -> DrmMode {
    // ...
}

pub(super) fn modeinfo_name_slice_from_string(mode_name: &str) -> [c_char; 32] {
    // ...
}

pub(super) fn pick_mode(...) -> ... {
    // ...
}

// === Connector Utilities ===

pub(super) fn get_edid_info(edid_blob: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    // ...
}

pub(super) fn format_connector_name(connector: &connector::Info) -> String {
    // ...
}

pub(super) fn make_output_name(...) -> OutputName {
    // ...
}

// === HDR/VRR ===

pub(super) fn reset_hdr(props: &ConnectorProperties) -> anyhow::Result<()> {
    // ...
}

pub(super) fn is_vrr_capable(device: &DrmDevice, connector: connector::Handle) -> Option<bool> {
    // ...
}

pub(super) fn get_panel_orientation(props: &ConnectorProperties) -> anyhow::Result<Transform> {
    // ...
}

// === Gamma ===

/// Set gamma LUT for a CRTC.
pub fn set_gamma_for_crtc(
    device: &DrmDevice,
    crtc: crtc::Handle,
    ramp: Option<&[u16]>,
) -> anyhow::Result<()> {
    // ...
}

// === System ===

#[cfg(feature = "dbus")]
pub(super) fn suspend() -> anyhow::Result<()> {
    // ...
}
```

---

## Verification

- [ ] All helper functions work correctly
- [ ] `cargo check` passes
- [ ] Existing tests pass (mode calculation tests)
- [ ] No Tty or State imports in helpers.rs

---

## Tests to Move

The existing tests at lines 3344-3474 should move with the helpers:
- `test_calculate_drmmode_from_modeline`
- `test_calc_cvt`

```rust
// At bottom of helpers.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_drmmode_from_modeline() { ... }

    #[test]
    fn test_calc_cvt() { ... }
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/helpers.rs` | Created (~400 LOC) |
| `src/backend/tty/mod.rs` | Removed helpers, added `mod helpers` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.4: Extract Device Lifecycle](phase-T1.4-extract-lifecycle.md).
