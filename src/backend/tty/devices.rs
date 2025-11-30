//! Device management for TTY backend.
//!
//! This module contains:
//! - `OutputDevice` - represents a single DRM device (GPU)
//! - `DeviceManager` - owns all DRM device state (subsystem)

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::reexports::calloop::RegistrationToken;
use smithay::reexports::drm::control::{connector, crtc};
use smithay::wayland::dmabuf::DmabufGlobal;
use smithay::wayland::drm_lease::{
    DrmLease, DrmLeaseBuilder, DrmLeaseRequest, DrmLeaseState, LeaseRejected,
};
use smithay_drm_extras::drm_scanner::DrmScanner;
use tracing::{debug, error, warn};

use super::types::{CrtcInfo, Surface, TtyOutputState};
use crate::niri::Niri;

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

// =============================================================================
// DeviceManager - Subsystem that owns all DRM device state
// =============================================================================

/// Device management subsystem.
///
/// OWNS all DRM device state:
/// - Connected devices (GPUs)
/// - Primary/render node tracking
/// - GPU manager for multi-GPU
/// - DmaBuf global
pub struct DeviceManager {
    /// Devices indexed by DRM node (not necessarily the render node).
    devices: HashMap<DrmNode, OutputDevice>,
    /// DRM node corresponding to the primary GPU. May or may not be the same as
    /// primary_render_node.
    primary_node: DrmNode,
    /// DRM render node corresponding to the primary GPU.
    primary_render_node: DrmNode,
    /// Ignored DRM nodes.
    ignored_nodes: HashSet<DrmNode>,
    /// GPU manager for multi-GPU support.
    gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    /// The dma-buf global corresponds to the output device (the primary GPU).
    /// It is only `Some()` if we have a device corresponding to the primary GPU.
    dmabuf_global: Option<DmabufGlobal>,
}

impl DeviceManager {
    /// Create a new DeviceManager.
    pub fn new(
        primary_node: DrmNode,
        primary_render_node: DrmNode,
        ignored_nodes: HashSet<DrmNode>,
        gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    ) -> Self {
        Self {
            devices: HashMap::new(),
            primary_node,
            primary_render_node,
            ignored_nodes,
            gpu_manager,
            dmabuf_global: None,
        }
    }

    // === Device Access ===

    /// Get a reference to a device by node.
    pub fn get(&self, node: &DrmNode) -> Option<&OutputDevice> {
        self.devices.get(node)
    }

    /// Get a mutable reference to a device by node.
    pub fn get_mut(&mut self, node: &DrmNode) -> Option<&mut OutputDevice> {
        self.devices.get_mut(node)
    }

    /// Iterate over all devices.
    pub fn iter(&self) -> impl Iterator<Item = (&DrmNode, &OutputDevice)> {
        self.devices.iter()
    }

    /// Iterate over all devices mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&DrmNode, &mut OutputDevice)> {
        self.devices.iter_mut()
    }

    /// Check if a device exists for the given node.
    pub fn contains(&self, node: &DrmNode) -> bool {
        self.devices.contains_key(node)
    }

    /// Insert a device. Returns the previous device if one existed.
    pub fn insert(&mut self, node: DrmNode, device: OutputDevice) -> Option<OutputDevice> {
        self.devices.insert(node, device)
    }

    /// Remove a device.
    pub fn remove(&mut self, node: &DrmNode) -> Option<OutputDevice> {
        self.devices.remove(node)
    }

    /// Get the number of devices.
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Check if there are no devices.
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    /// Iterate over device keys.
    pub fn keys(&self) -> impl Iterator<Item = &DrmNode> {
        self.devices.keys()
    }

    /// Iterate over device values mutably.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut OutputDevice> {
        self.devices.values_mut()
    }

    /// Iterate over device values.
    pub fn values(&self) -> impl Iterator<Item = &OutputDevice> {
        self.devices.values()
    }

    // === Node Info ===

    /// Get the primary DRM node.
    pub fn primary_node(&self) -> DrmNode {
        self.primary_node
    }

    /// Get the primary render node.
    pub fn primary_render_node(&self) -> DrmNode {
        self.primary_render_node
    }

    /// Check if a node is ignored.
    pub fn is_ignored(&self, node: &DrmNode) -> bool {
        self.ignored_nodes.contains(node)
    }

    /// Get all ignored nodes.
    pub fn ignored_nodes(&self) -> &HashSet<DrmNode> {
        &self.ignored_nodes
    }

    /// Set the ignored nodes.
    pub fn set_ignored_nodes(&mut self, ignored: HashSet<DrmNode>) {
        self.ignored_nodes = ignored;
    }

    // === GPU Manager ===

    /// Get a reference to the GPU manager.
    pub fn gpu_manager(&self) -> &GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>> {
        &self.gpu_manager
    }

    /// Get a mutable reference to the GPU manager.
    pub fn gpu_manager_mut(&mut self) -> &mut GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>> {
        &mut self.gpu_manager
    }

    /// Get mutable references to both the GPU manager and devices map.
    /// This enables split borrowing for cases where you need to create a renderer
    /// and then access devices while the renderer is still alive.
    pub fn gpu_manager_and_devices_mut(
        &mut self,
    ) -> (
        &mut GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
        &mut HashMap<DrmNode, OutputDevice>,
    ) {
        (&mut self.gpu_manager, &mut self.devices)
    }

    // === DmaBuf ===

    /// Get a reference to the dmabuf global.
    pub fn dmabuf_global(&self) -> Option<&DmabufGlobal> {
        self.dmabuf_global.as_ref()
    }

    /// Set the dmabuf global.
    pub fn set_dmabuf_global(&mut self, global: Option<DmabufGlobal>) {
        self.dmabuf_global = global;
    }

    /// Take the dmabuf global.
    pub fn take_dmabuf_global(&mut self) -> Option<DmabufGlobal> {
        self.dmabuf_global.take()
    }

    // === Session Events ===

    /// Pause all DRM devices (called on session pause).
    pub fn pause_devices(&mut self) {
        for device in self.devices.values_mut() {
            device.drm.pause();
            if let Some(lease_state) = &mut device.drm_lease_state {
                lease_state.suspend();
            }
        }
    }

    /// Activate a specific DRM device (called on session resume).
    /// Returns Ok(()) if successful, Err if activation failed.
    pub fn activate_device(&mut self, node: &DrmNode) -> anyhow::Result<()> {
        let device = self.devices.get_mut(node).context("device not found")?;
        device.drm.activate(false).context("error activating DRM device")?;
        if let Some(lease_state) = &mut device.drm_lease_state {
            lease_state.resume::<crate::niri::State>();
        }
        Ok(())
    }

    // === Connector Lifecycle ===

    /// Handle a connector becoming disconnected (monitor unplugged).
    pub fn connector_disconnected(&mut self, niri: &mut Niri, node: DrmNode, crtc: crtc::Handle) {
        let Some(device) = self.devices.get_mut(&node) else {
            debug!("disconnecting connector for crtc: {crtc:?}");
            error!("missing device");
            return;
        };

        let Some(surface) = device.surfaces.remove(&crtc) else {
            debug!("disconnecting connector for crtc: {crtc:?}");

            if let Some((conn, _)) = device
                .non_desktop_connectors
                .iter()
                .find(|(_, crtc_)| *crtc_ == crtc)
            {
                debug!("withdrawing non-desktop connector from DRM leasing");

                let conn = *conn;
                device.non_desktop_connectors.remove(&(conn, crtc));

                if let Some(lease_state) = &mut device.drm_lease_state {
                    lease_state.withdraw_connector(conn);
                }
            } else {
                debug!("crtc wasn't enabled");
            }

            return;
        };

        debug!("disconnecting connector: {:?}", surface.name.connector);

        let output = niri
            .outputs
            .space()
            .outputs()
            .find(|output| {
                let tty_state: &TtyOutputState = output.user_data().get().unwrap();
                tty_state.node == node && tty_state.crtc == crtc
            })
            .cloned();
        if let Some(output) = output {
            niri.remove_output(&output);
        } else {
            error!("missing output for crtc {crtc:?}");
        };
    }
}

// Implement IntoIterator for convenient iteration
impl<'a> IntoIterator for &'a DeviceManager {
    type Item = (&'a DrmNode, &'a OutputDevice);
    type IntoIter = std::collections::hash_map::Iter<'a, DrmNode, OutputDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter()
    }
}

impl<'a> IntoIterator for &'a mut DeviceManager {
    type Item = (&'a DrmNode, &'a mut OutputDevice);
    type IntoIter = std::collections::hash_map::IterMut<'a, DrmNode, OutputDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter_mut()
    }
}
