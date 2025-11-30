# Phase T1.2: Enrich OutputDevice

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low  
> **Architectural Benefit**: ⭐⭐⭐ High - better encapsulation for subsystems

---

## Goal

Enrich `OutputDevice` to be a proper self-contained type before the subsystems use it. Move `OutputDevice` into `devices.rs` and give it a richer API that encapsulates its internals.

**Key principle**: `OutputDevice` should hide its internals and expose an intentional API.

---

## Current State

`OutputDevice` exists but has mostly `pub(super)` fields - it's just a data bag:

```rust
// CURRENT: Data bag with public internals
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
```

---

## Target State

`OutputDevice` with private fields and intentional API:

```rust
// TARGET: Encapsulated type with API
pub struct OutputDevice {
    // ALL fields are private
    token: RegistrationToken,
    render_node: Option<DrmNode>,
    drm_scanner: DrmScanner,
    surfaces: HashMap<crtc::Handle, Surface>,
    known_crtcs: HashMap<crtc::Handle, CrtcInfo>,
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,
    allocator: GbmAllocator<DrmDeviceFd>,
    drm_lease_state: Option<DrmLeaseState>,
    non_desktop_connectors: HashSet<(connector::Handle, crtc::Handle)>,
    active_leases: Vec<DrmLease>,
}

impl OutputDevice {
    // Constructor
    pub fn new(...) -> Self
    
    // === Accessors ===
    pub fn render_node(&self) -> Option<DrmNode>
    pub fn drm(&self) -> &DrmDevice
    pub fn drm_mut(&mut self) -> &mut DrmDevice
    pub fn gbm(&self) -> &GbmDevice<DrmDeviceFd>
    pub fn allocator(&self) -> &GbmAllocator<DrmDeviceFd>
    pub fn token(&self) -> RegistrationToken
    
    // === Surface Management ===
    pub fn surface(&self, crtc: crtc::Handle) -> Option<&Surface>
    pub fn surface_mut(&mut self, crtc: crtc::Handle) -> Option<&mut Surface>
    pub fn surfaces(&self) -> impl Iterator<Item = (&crtc::Handle, &Surface)>
    pub fn surfaces_mut(&mut self) -> impl Iterator<Item = (&crtc::Handle, &mut Surface)>
    pub fn insert_surface(&mut self, crtc: crtc::Handle, surface: Surface)
    pub fn remove_surface(&mut self, crtc: crtc::Handle) -> Option<Surface>
    
    // === CRTC Management ===
    pub fn known_crtc(&self, crtc: crtc::Handle) -> Option<&CrtcInfo>
    pub fn known_crtc_name(&self, crtc: crtc::Handle, connector: &connector::Info, ...) -> OutputName
    pub fn insert_known_crtc(&mut self, crtc: crtc::Handle, info: CrtcInfo)
    pub fn remove_known_crtc(&mut self, crtc: crtc::Handle)
    
    // === Scanner ===
    pub fn scanner(&self) -> &DrmScanner
    pub fn scanner_mut(&mut self) -> &mut DrmScanner
    
    // === DRM Leases (VR) ===
    pub fn lease_request(&self, request: DrmLeaseRequest) -> Result<DrmLeaseBuilder, LeaseRejected>
    pub fn new_lease(&mut self, lease: DrmLease)
    pub fn remove_lease(&mut self, lease_id: u32)
    pub fn lease_state(&self) -> Option<&DrmLeaseState>
    pub fn lease_state_mut(&mut self) -> Option<&mut DrmLeaseState>
    
    // === Non-Desktop Connectors ===
    pub fn add_non_desktop(&mut self, connector: connector::Handle, crtc: crtc::Handle)
    pub fn is_non_desktop(&self, connector: connector::Handle) -> bool
    pub fn non_desktop_connectors(&self) -> &HashSet<(connector::Handle, crtc::Handle)>
}
```

---

## Implementation: `src/backend/tty/devices.rs`

```rust
//! Device management subsystem.
//!
//! This module contains:
//! - `OutputDevice` - a single DRM device (GPU)
//! - `CrtcInfo` - information about a connected CRTC
//! - `DeviceManager` - subsystem owning all devices (Phase T1.4)

use std::collections::{HashMap, HashSet};

use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode};
use smithay::reexports::calloop::RegistrationToken;
use smithay::reexports::drm::control::{connector, crtc};
use smithay::wayland::drm_lease::{
    DrmLease, DrmLeaseBuilder, DrmLeaseRequest, DrmLeaseState, LeaseRejected,
};
use smithay_drm_extras::drm_scanner::DrmScanner;

use super::types::Surface;
use crate::backend::OutputId;
use niri_config::OutputName;

/// A connected DRM output device (GPU).
///
/// Encapsulates all state for a single DRM device including:
/// - DRM/GBM resources
/// - Connected surfaces (one per active CRTC)
/// - DRM lease state for VR headsets
pub struct OutputDevice {
    token: RegistrationToken,
    render_node: Option<DrmNode>,
    drm_scanner: DrmScanner,
    surfaces: HashMap<crtc::Handle, Surface>,
    known_crtcs: HashMap<crtc::Handle, CrtcInfo>,
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,
    allocator: GbmAllocator<DrmDeviceFd>,
    drm_lease_state: Option<DrmLeaseState>,
    non_desktop_connectors: HashSet<(connector::Handle, crtc::Handle)>,
    active_leases: Vec<DrmLease>,
}

/// Information about a connected CRTC.
#[derive(Debug, Clone)]
pub struct CrtcInfo {
    pub id: OutputId,
    pub name: OutputName,
}

impl OutputDevice {
    /// Create a new OutputDevice.
    pub fn new(
        token: RegistrationToken,
        render_node: Option<DrmNode>,
        drm: DrmDevice,
        gbm: GbmDevice<DrmDeviceFd>,
        allocator: GbmAllocator<DrmDeviceFd>,
    ) -> Self {
        Self {
            token,
            render_node,
            drm_scanner: DrmScanner::new(),
            surfaces: HashMap::new(),
            known_crtcs: HashMap::new(),
            drm,
            gbm,
            allocator,
            drm_lease_state: None,
            non_desktop_connectors: HashSet::new(),
            active_leases: Vec::new(),
        }
    }

    // === Accessors ===

    pub fn render_node(&self) -> Option<DrmNode> {
        self.render_node
    }

    pub fn drm(&self) -> &DrmDevice {
        &self.drm
    }

    pub fn drm_mut(&mut self) -> &mut DrmDevice {
        &mut self.drm
    }

    pub fn gbm(&self) -> &GbmDevice<DrmDeviceFd> {
        &self.gbm
    }

    pub fn allocator(&self) -> &GbmAllocator<DrmDeviceFd> {
        &self.allocator
    }

    pub fn token(&self) -> RegistrationToken {
        self.token
    }

    // === Surface Management ===

    pub fn surface(&self, crtc: crtc::Handle) -> Option<&Surface> {
        self.surfaces.get(&crtc)
    }

    pub fn surface_mut(&mut self, crtc: crtc::Handle) -> Option<&mut Surface> {
        self.surfaces.get_mut(&crtc)
    }

    pub fn surfaces(&self) -> impl Iterator<Item = (&crtc::Handle, &Surface)> {
        self.surfaces.iter()
    }

    pub fn surfaces_mut(&mut self) -> impl Iterator<Item = (&crtc::Handle, &mut Surface)> {
        self.surfaces.iter_mut()
    }

    pub fn insert_surface(&mut self, crtc: crtc::Handle, surface: Surface) {
        self.surfaces.insert(crtc, surface);
    }

    pub fn remove_surface(&mut self, crtc: crtc::Handle) -> Option<Surface> {
        self.surfaces.remove(&crtc)
    }

    // === CRTC Management ===

    pub fn known_crtc(&self, crtc: crtc::Handle) -> Option<&CrtcInfo> {
        self.known_crtcs.get(&crtc)
    }

    pub fn insert_known_crtc(&mut self, crtc: crtc::Handle, info: CrtcInfo) {
        self.known_crtcs.insert(crtc, info);
    }

    pub fn remove_known_crtc(&mut self, crtc: crtc::Handle) {
        self.known_crtcs.remove(&crtc);
    }

    // === Scanner ===

    pub fn scanner(&self) -> &DrmScanner {
        &self.drm_scanner
    }

    pub fn scanner_mut(&mut self) -> &mut DrmScanner {
        &mut self.drm_scanner
    }

    // === DRM Leases (VR) ===

    pub fn lease_request(
        &self,
        request: DrmLeaseRequest,
    ) -> Result<DrmLeaseBuilder, LeaseRejected> {
        let mut builder = DrmLeaseBuilder::new(&self.drm);
        for connector in request.connectors {
            let (_, crtc) = self
                .non_desktop_connectors
                .iter()
                .find(|(conn, _)| connector == *conn)
                .ok_or_else(|| {
                    tracing::warn!("Attempted to lease connector that is not non-desktop");
                    LeaseRejected::default()
                })?;
            builder.add_connector(connector);
            builder.add_crtc(*crtc);
            let planes = self.drm.planes(crtc).map_err(LeaseRejected::with_cause)?;
            let (primary_plane, primary_plane_claim) = planes
                .primary
                .iter()
                .find_map(|plane| {
                    self.drm
                        .claim_plane(plane.handle, *crtc)
                        .map(|claim| (plane, claim))
                })
                .ok_or_else(LeaseRejected::default)?;
            builder.add_plane(primary_plane.handle, primary_plane_claim);
        }
        Ok(builder)
    }

    pub fn new_lease(&mut self, lease: DrmLease) {
        self.active_leases.push(lease);
    }

    pub fn remove_lease(&mut self, lease_id: u32) {
        self.active_leases.retain(|l| l.id() != lease_id);
    }

    pub fn lease_state(&self) -> Option<&DrmLeaseState> {
        self.drm_lease_state.as_ref()
    }

    pub fn lease_state_mut(&mut self) -> Option<&mut DrmLeaseState> {
        self.drm_lease_state.as_mut()
    }

    pub fn set_lease_state(&mut self, state: Option<DrmLeaseState>) {
        self.drm_lease_state = state;
    }

    // === Non-Desktop Connectors ===

    pub fn add_non_desktop(&mut self, connector: connector::Handle, crtc: crtc::Handle) {
        self.non_desktop_connectors.insert((connector, crtc));
    }

    pub fn remove_non_desktop(&mut self, connector: connector::Handle, crtc: crtc::Handle) {
        self.non_desktop_connectors.remove(&(connector, crtc));
    }

    pub fn non_desktop_connectors(&self) -> &HashSet<(connector::Handle, crtc::Handle)> {
        &self.non_desktop_connectors
    }
}
```

---

## Migration Steps

1. **Create `devices.rs`** with the enriched `OutputDevice`
2. **Update all direct field access** to use the new API
3. **Search for `device.surfaces.`** → use `device.surface()` / `device.surface_mut()`
4. **Search for `device.drm.`** → use `device.drm()`
5. **Search for `device.known_crtcs.`** → use `device.known_crtc()` / etc.

---

## Verification Checklist

- [ ] All `OutputDevice` fields are private
- [ ] All access goes through methods
- [ ] DRM lease functionality works
- [ ] `cargo check` passes
- [ ] Device add/remove still works

---

## Files Changed

| File | Change |
|------|--------|
| `src/backend/tty/devices.rs` | `OutputDevice` with full API (~300 LOC) |
| `src/backend/tty/mod.rs` | Updated to use `devices.rs` |

---

## Next Phase

After completing this phase, proceed to [Phase T1.3: Extract Helpers](phase-T1.3-extract-helpers.md).
