# Phase T1.2: Extract OutputDevice

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐ Medium - isolates device/lease handling

---

## Goal

Extract `OutputDevice` struct and its implementation into `src/backend/tty/device.rs`.

`OutputDevice` represents a DRM output device (GPU) and handles:
- DRM lease management (for VR headsets, etc.)
- CRTC tracking
- Scanner state

---

## What Moves

### Structs (lines 133-157)

```rust
// src/backend/tty/device.rs

pub struct OutputDevice {
    pub(super) token: RegistrationToken,
    pub(super) render_node: Option<DrmNode>,
    pub(super) drm_scanner: DrmScanner,
    pub(super) surfaces: HashMap<crtc::Handle, Surface>,
    pub(super) known_crtcs: HashMap<crtc::Handle, CrtcInfo>,
    pub(super) drm: DrmDevice,
    pub(super) gbm: GbmDevice<DrmDeviceFd>,
    pub(super) allocator: GbmAllocator<DrmDeviceFd>,
    pub drm_lease_state: Option<DrmLeaseState>,
    pub(super) non_desktop_connectors: HashSet<(connector::Handle, crtc::Handle)>,
    pub(super) active_leases: Vec<DrmLease>,
}

#[derive(Debug, Clone)]
pub struct CrtcInfo {
    pub(super) id: OutputId,
    pub(super) name: OutputName,
}
```

### impl OutputDevice (lines 159-370, ~210 lines)

```rust
impl OutputDevice {
    // Lease management
    pub fn lease_request(&self, request: DrmLeaseRequest) -> Result<DrmLeaseBuilder, LeaseRejected>
    pub fn new_lease(&mut self, lease: DrmLease)
    pub fn remove_lease(&mut self, lease_id: u32)
    
    // CRTC utilities
    pub fn known_crtc_name(&self, crtc: crtc::Handle, connector: connector::Handle) -> Option<OutputName>
    
    // Internal cleanup
    fn cleanup_mismatching_resources(&mut self, niri: &mut Niri)
}
```

---

## Why This is Good Architecture

1. **Self-contained** - OutputDevice is its own logical unit
2. **Clear responsibility** - Manages one DRM device
3. **Lease isolation** - VR lease handling in one place
4. **Testable** - Lease logic could be unit tested

---

## Target: `src/backend/tty/device.rs`

```rust
//! DRM output device management.
//!
//! Handles a single DRM device (GPU), including:
//! - Device state and resources
//! - DRM lease management for VR/external displays
//! - CRTC and connector tracking

use std::collections::{HashMap, HashSet};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode};
use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice};
use smithay::reexports::calloop::RegistrationToken;
use smithay::reexports::drm::control::{connector, crtc};
use smithay::wayland::drm_lease::{DrmLease, DrmLeaseBuilder, DrmLeaseRequest, DrmLeaseState, LeaseRejected};
use smithay_drm_extras::drm_scanner::DrmScanner;

use super::types::Surface;
use crate::backend::OutputId;
use crate::niri::Niri;
use niri_config::OutputName;

/// A connected DRM output device (GPU).
pub struct OutputDevice {
    // ... fields
}

/// Information about a connected CRTC.
#[derive(Debug, Clone)]
pub struct CrtcInfo {
    pub(super) id: OutputId,
    pub(super) name: OutputName,
}

impl OutputDevice {
    /// Handle a DRM lease request (for VR headsets, etc.)
    pub fn lease_request(&self, request: DrmLeaseRequest) -> Result<DrmLeaseBuilder, LeaseRejected> {
        // ...
    }

    /// Register a new active lease.
    pub fn new_lease(&mut self, lease: DrmLease) {
        self.active_leases.push(lease);
    }

    /// Remove a lease by ID.
    pub fn remove_lease(&mut self, lease_id: u32) {
        self.active_leases.retain(|l| l.id() != lease_id);
    }

    /// Get the output name for a known CRTC.
    pub fn known_crtc_name(
        &self,
        crtc: crtc::Handle,
        connector: connector::Handle,
    ) -> Option<OutputName> {
        // ...
    }

    /// Clean up resources that don't match the current state.
    pub(super) fn cleanup_mismatching_resources(&mut self, niri: &mut Niri) {
        // ...
    }
}
```

---

## Verification

- [ ] All lease-related functionality works
- [ ] `cargo check` passes
- [ ] Device add/remove still works

---

## Import Updates

In `mod.rs`:
```rust
mod device;

pub use device::{OutputDevice, CrtcInfo};
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/device.rs` | Created (~250 LOC) |
| `src/backend/tty/mod.rs` | Removed OutputDevice, added `mod device` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.3: Extract Helpers](phase-T1.3-extract-helpers.md).
