//! Device management for TTY backend.
//!
//! This module contains `OutputDevice` which represents a single DRM device (GPU).

use std::collections::{HashMap, HashSet};

use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode};
use smithay::reexports::calloop::RegistrationToken;
use smithay::reexports::drm::control::{connector, crtc};
use smithay::wayland::drm_lease::{
    DrmLease, DrmLeaseBuilder, DrmLeaseRequest, DrmLeaseState, LeaseRejected,
};
use smithay_drm_extras::drm_scanner::DrmScanner;
use tracing::warn;

use super::types::{CrtcInfo, Surface};

/// A connected DRM output device (GPU).
///
/// Encapsulates all state for a single DRM device including:
/// - DRM/GBM resources
/// - Connected surfaces (one per active CRTC)
/// - DRM lease state for VR headsets
pub struct OutputDevice {
    pub(super) token: RegistrationToken,
    /// Can be None for display-only devices such as DisplayLink.
    pub(super) render_node: Option<DrmNode>,
    pub(super) drm_scanner: DrmScanner,
    pub(super) surfaces: HashMap<crtc::Handle, Surface>,
    pub(super) known_crtcs: HashMap<crtc::Handle, CrtcInfo>,
    /// SAFETY: drop after all the objects used with them are dropped.
    /// See https://github.com/Smithay/smithay/issues/1102.
    pub(super) drm: DrmDevice,
    pub(super) gbm: GbmDevice<DrmDeviceFd>,
    /// For display-only devices this will be the allocator from the primary device.
    pub(super) allocator: GbmAllocator<DrmDeviceFd>,
    pub(super) drm_lease_state: Option<DrmLeaseState>,
    pub(super) non_desktop_connectors: HashSet<(connector::Handle, crtc::Handle)>,
    pub(super) active_leases: Vec<DrmLease>,
}

impl OutputDevice {
    /// Create a new OutputDevice.
    #[allow(clippy::too_many_arguments)]
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

    // === Core Accessors ===

    /// Get the render node for this device, if any.
    pub fn render_node(&self) -> Option<DrmNode> {
        self.render_node
    }

    /// Get a reference to the DRM device.
    pub fn drm(&self) -> &DrmDevice {
        &self.drm
    }

    /// Get a mutable reference to the DRM device.
    pub fn drm_mut(&mut self) -> &mut DrmDevice {
        &mut self.drm
    }

    /// Get a reference to the GBM device.
    pub fn gbm(&self) -> &GbmDevice<DrmDeviceFd> {
        &self.gbm
    }

    /// Get a reference to the allocator.
    pub fn allocator(&self) -> &GbmAllocator<DrmDeviceFd> {
        &self.allocator
    }

    /// Get the event loop registration token.
    pub fn token(&self) -> RegistrationToken {
        self.token
    }

    // === Surface Management ===

    /// Get a reference to a surface by CRTC handle.
    pub(super) fn surface(&self, crtc: crtc::Handle) -> Option<&Surface> {
        self.surfaces.get(&crtc)
    }

    /// Get a mutable reference to a surface by CRTC handle.
    pub(super) fn surface_mut(&mut self, crtc: crtc::Handle) -> Option<&mut Surface> {
        self.surfaces.get_mut(&crtc)
    }

    /// Check if a surface exists for the given CRTC.
    pub fn has_surface(&self, crtc: crtc::Handle) -> bool {
        self.surfaces.contains_key(&crtc)
    }

    /// Iterate over all surfaces.
    pub(super) fn surfaces(&self) -> impl Iterator<Item = (&crtc::Handle, &Surface)> {
        self.surfaces.iter()
    }

    /// Iterate over all surfaces mutably.
    pub(super) fn surfaces_mut(&mut self) -> impl Iterator<Item = (&crtc::Handle, &mut Surface)> {
        self.surfaces.iter_mut()
    }

    /// Get mutable access to all surface values.
    pub(super) fn surfaces_values_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
        self.surfaces.values_mut()
    }

    /// Insert a surface for a CRTC.
    pub(super) fn insert_surface(&mut self, crtc: crtc::Handle, surface: Surface) -> Option<Surface> {
        self.surfaces.insert(crtc, surface)
    }

    /// Remove a surface for a CRTC.
    pub(super) fn remove_surface(&mut self, crtc: crtc::Handle) -> Option<Surface> {
        self.surfaces.remove(&crtc)
    }

    // === CRTC Management ===

    /// Get info about a known CRTC.
    pub fn known_crtc(&self, crtc: crtc::Handle) -> Option<&CrtcInfo> {
        self.known_crtcs.get(&crtc)
    }

    /// Insert a known CRTC.
    pub fn insert_known_crtc(&mut self, crtc: crtc::Handle, info: CrtcInfo) {
        self.known_crtcs.insert(crtc, info);
    }

    /// Remove a known CRTC.
    pub fn remove_known_crtc(&mut self, crtc: crtc::Handle) -> Option<CrtcInfo> {
        self.known_crtcs.remove(&crtc)
    }

    /// Iterate over known CRTC values.
    pub fn known_crtcs_iter(&self) -> impl Iterator<Item = &CrtcInfo> {
        self.known_crtcs.values()
    }

    // === Scanner ===

    /// Get a reference to the DRM scanner.
    pub fn scanner(&self) -> &DrmScanner {
        &self.drm_scanner
    }

    /// Get a mutable reference to the DRM scanner.
    pub fn scanner_mut(&mut self) -> &mut DrmScanner {
        &mut self.drm_scanner
    }

    /// Scan connectors on this device.
    pub fn scan_connectors(&mut self) -> std::io::Result<smithay_drm_extras::drm_scanner::DrmScanResult> {
        self.drm_scanner.scan_connectors(&self.drm)
    }

    // === DRM Leases (VR) ===

    /// Build a lease request for VR headsets.
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
                    warn!("Attempted to lease connector that is not non-desktop");
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

    /// Add a new active lease.
    pub fn new_lease(&mut self, lease: DrmLease) {
        self.active_leases.push(lease);
    }

    /// Remove a lease by ID.
    pub fn remove_lease(&mut self, lease_id: u32) {
        self.active_leases.retain(|l| l.id() != lease_id);
    }

    /// Get a reference to the lease state.
    pub fn lease_state(&self) -> Option<&DrmLeaseState> {
        self.drm_lease_state.as_ref()
    }

    /// Get a mutable reference to the lease state.
    pub fn lease_state_mut(&mut self) -> Option<&mut DrmLeaseState> {
        self.drm_lease_state.as_mut()
    }

    /// Set the lease state.
    pub fn set_lease_state(&mut self, state: Option<DrmLeaseState>) {
        self.drm_lease_state = state;
    }

    // === Non-Desktop Connectors ===

    /// Add a non-desktop connector (VR headset).
    pub fn add_non_desktop(&mut self, connector: connector::Handle, crtc: crtc::Handle) {
        self.non_desktop_connectors.insert((connector, crtc));
    }

    /// Remove a non-desktop connector.
    pub fn remove_non_desktop(&mut self, connector: connector::Handle, crtc: crtc::Handle) {
        self.non_desktop_connectors.remove(&(connector, crtc));
    }

    /// Get all non-desktop connectors.
    pub fn non_desktop_connectors(&self) -> &HashSet<(connector::Handle, crtc::Handle)> {
        &self.non_desktop_connectors
    }

    // === Output Name ===

    /// Get the output name for a known CRTC.
    pub fn known_crtc_name(
        &self,
        crtc: &crtc::Handle,
        conn: &connector::Info,
        disable_monitor_names: bool,
    ) -> niri_config::OutputName {
        use niri_config::OutputName;

        if disable_monitor_names {
            let conn_name = format_connector_name(conn);
            return OutputName {
                connector: conn_name,
                make: None,
                model: None,
                serial: None,
            };
        }

        let Some(info) = self.known_crtcs.get(crtc) else {
            let conn_name = format_connector_name(conn);
            tracing::error!("crtc for connector {conn_name} missing from known");
            return OutputName {
                connector: conn_name,
                make: None,
                model: None,
                serial: None,
            };
        };
        info.name.clone()
    }
}

pub(super) fn format_connector_name(connector: &connector::Info) -> String {
    format!("{}-{}", connector.interface().as_str(), connector.interface_id())
}
