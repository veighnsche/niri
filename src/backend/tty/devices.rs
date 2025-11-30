//! Device management for TTY backend.
//!
//! This module contains:
//! - `OutputDevice` - represents a single DRM device (GPU)
//! - `DeviceManager` - owns all DRM device state (subsystem)

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use std::mem;
use std::os::fd::OwnedFd;
use std::time::Duration;

use anyhow::{bail, ensure, Context, anyhow};
use libc::dev_t;
use niri_config::Config;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmDevice, GbmBufferFlags};
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode, NodeType, DrmEvent, compositor::DrmCompositor, VrrSupport};
use smithay::backend::renderer::{ImportDma, ImportEgl};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::Session;
use smithay::reexports::calloop::{LoopHandle, RegistrationToken, timer::{Timer, TimeoutAction}};
use smithay::reexports::drm::control::{connector, crtc};
use smithay::reexports::rustix::fs::OFlags;
use smithay::utils::DeviceFd;
use smithay::backend::egl::{EGLDisplay, EGLDevice};
use smithay::backend::renderer::{DebugFlags};
use smithay::backend::allocator::format::FormatSet;
use smithay::reexports::gbm::Modifier;
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::output::{Mode, Output, OutputModeSource, PhysicalProperties};
use smithay::wayland::dmabuf::{DmabufFeedbackBuilder, DmabufGlobal};
use smithay::wayland::drm_lease::{
    DrmLease, DrmLeaseBuilder, DrmLeaseRequest, DrmLeaseState, LeaseRejected,
};
use smithay_drm_extras::drm_scanner::DrmScanner;
use tracing::{debug, error, warn};

#[cfg(feature = "profile-with-tracy")]
use tracy_client;

use super::types::{CrtcInfo, Surface, TtyOutputState, ConnectorProperties, GammaProps};
use super::helpers::{
    surface_dmabuf_feedback, make_output_name,
    pick_mode, find_drm_property, reset_hdr, get_panel_orientation,
    set_gamma_for_crtc, refresh_interval, calculate_drm_mode_from_modeline,
};
use crate::render_helpers::{resources, shaders};
use crate::render_helpers::renderer::AsGlesRenderer;
use crate::backend::OutputId;
use crate::utils::{is_laptop_panel, PanelOrientation};
use crate::niri::{Niri, State};

/// The color formats supported by the DRM compositor.
///
/// These are the formats that we'll advertise as supported for rendering.
const SUPPORTED_COLOR_FORMATS: &'static [smithay::backend::allocator::Fourcc] = &[
    smithay::backend::allocator::Fourcc::Abgr2101010,
    smithay::backend::allocator::Fourcc::Argb2101010,
    smithay::backend::allocator::Fourcc::Xbgr2101010,
    smithay::backend::allocator::Fourcc::Xrgb2101010,
    smithay::backend::allocator::Fourcc::Abgr8888,
    smithay::backend::allocator::Fourcc::Argb8888,
    smithay::backend::allocator::Fourcc::Xbgr8888,
    smithay::backend::allocator::Fourcc::Xrgb8888,
];

/// Result of device_changed operation.
///
/// Contains information about what actions the caller should take after device_changed.
pub struct DeviceChangedResult {
    /// If set, device_added should be called with these args
    pub needs_device_added: Option<(dev_t, std::path::PathBuf)>,
    /// Connectors that need to be connected (node, crtc, crtc_info)
    pub connectors_to_connect: Vec<(DrmNode, smithay::reexports::drm::control::crtc::Handle, CrtcInfo)>,
}

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

    /// Add a new DRM device when it's connected via hotplug.
    ///
    /// This method handles GPU hotplug events by creating a new DRM device from scratch,
    /// initializing EGL display, setting up dmabuf feedback, and registering event loop sources.
    pub fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        session: &mut LibSeatSession,
        event_loop: &LoopHandle<State>,
        config: &Rc<RefCell<Config>>,
        niri: &mut Niri,
        _debug_tint: bool,
    ) -> anyhow::Result<()> {
        debug!("adding device: {device_id} {path:?}");

        let node = DrmNode::from_dev_id(device_id)?;

        if node == self.primary_node() {
            debug!("this is the primary node");
        }

        // Only consider primary node on udev event
        // https://gitlab.freedesktop.org/wlroots/wlroots/-/commit/768fbaad54027f8dd027e7e015e8eeb93cb38c52
        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return Ok(());
        }

        if self.ignored_nodes().contains(&node) {
            debug!("node is ignored, skipping");
            return Ok(());
        }

        #[cfg(feature = "profile-with-tracy")]
        let _span = tracy_client::span!("DeviceManager::device_added");

        let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = {
            #[cfg(feature = "profile-with-tracy")]
            let _span = tracy_client::span!("LibSeatSession::open");
            session.open(path, open_flags)
        }?;
        let device_fd = DrmDeviceFd::new(DeviceFd::from(fd));

        let (drm, drm_notifier) = {
            #[cfg(feature = "profile-with-tracy")]
            let _span = tracy_client::span!("DrmDevice::new");
            DrmDevice::new(device_fd.clone(), false)
        }?;
        let gbm = {
            #[cfg(feature = "profile-with-tracy")]
            let _span = tracy_client::span!("GbmDevice::new");
            GbmDevice::new(device_fd)
        }?;

        let mut try_initialize_gpu = || {
            let display = unsafe { EGLDisplay::new(gbm.clone())? };
            let egl_device = EGLDevice::device_for_display(&display)?;

            // Software EGL devices (e.g., llvmpipe/softpipe) are rejected for now. They have some
            // problems (segfault on importing dmabufs from other renderers) and need to be
            // excluded from some places like DRM leasing.
            ensure!(
                !egl_device.is_software(),
                "software EGL renderers are skipped"
            );

            let render_node = egl_device
                .try_get_render_node()
                .ok()
                .flatten()
                .unwrap_or(node);
            self.gpu_manager_mut()
                .as_mut()
                .add_node(render_node, gbm.clone())
                .context("error adding render node to GPU manager")?;

            Ok(render_node)
        };

        let render_node = match try_initialize_gpu() {
            Ok(render_node) => {
                debug!("got render node: {render_node}");
                Some(render_node)
            }
            Err(err) => {
                debug!("failed to initialize renderer, falling back to primary gpu: {err:?}");
                None
            }
        };

        if render_node == Some(self.primary_render_node()) && self.dmabuf_global().is_none() {
            let render_node = self.primary_render_node();
            debug!("initializing the primary renderer");

            let mut renderer = self
                .gpu_manager_mut()
                .single_renderer(&render_node)
                .context("error creating renderer")?;

            if let Err(err) = renderer.bind_wl_display(&niri.display_handle) {
                warn!("error binding wl-display in EGL: {err:?}");
            }

            let gles_renderer = renderer.as_gles_renderer();
            resources::init(gles_renderer);
            shaders::init(gles_renderer);

            let config = config.borrow();
            if let Some(src) = config.animations.window_resize.custom_shader.as_deref() {
                shaders::set_custom_resize_program(gles_renderer, Some(src));
            }
            if let Some(src) = config.animations.window_close.custom_shader.as_deref() {
                shaders::set_custom_close_program(gles_renderer, Some(src));
            }
            if let Some(src) = config.animations.window_open.custom_shader.as_deref() {
                shaders::set_custom_open_program(gles_renderer, Some(src));
            }
            drop(config);

            niri.update_shaders();

            // Create the dmabuf global.
            let primary_formats = renderer.dmabuf_formats();
            let default_feedback =
                DmabufFeedbackBuilder::new(render_node.dev_id(), primary_formats.clone())
                    .build()
                    .context("error building default dmabuf feedback")?;
            let dmabuf_global = niri
                .protocols
                .dmabuf
                .create_global_with_default_feedback::<State>(
                    &niri.display_handle,
                    &default_feedback,
                );
            assert!(self.dmabuf_global().is_none());
            self.set_dmabuf_global(Some(dmabuf_global));

            // Update the dmabuf feedbacks for all surfaces.
            let primary_render_node = self.primary_render_node();
            for (node, device) in self.iter_mut() {
                for surface in device.surfaces.values_mut() {
                    match surface_dmabuf_feedback(
                        &surface.compositor,
                        primary_formats.clone(),
                        primary_render_node,
                        device.render_node,
                        *node,
                    ) {
                        Ok(feedback) => {
                            surface.dmabuf_feedback = Some(feedback);
                        }
                        Err(err) => {
                            warn!("error building dmabuf feedback: {err:?}");
                        }
                    }
                }
            }
        }

        let allocator_gbm = if render_node.is_some() {
            gbm.clone()
        } else if let Some(primary_device) = self.get(&self.primary_node()) {
            primary_device.gbm.clone()
        } else {
            bail!("no allocator available for device");
        };
        let gbm_flags = GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT;
        let allocator = GbmAllocator::new(allocator_gbm, gbm_flags);

        let token = event_loop
            .insert_source(drm_notifier, move |event, meta, state| {
                let tty = state.backend.tty();
                match event {
                    DrmEvent::VBlank(crtc) => {
                        let meta = meta.expect("VBlank events must have metadata");
                        tty.on_vblank(&mut state.niri, node, crtc, meta);
                    }
                    DrmEvent::Error(error) => warn!("DRM error: {error}"),
                };
            })
            .unwrap();

        let drm_lease_state = DrmLeaseState::new::<State>(&niri.display_handle, &node)
            .map_err(|err| warn!("error initializing DRM leasing for {node}: {err:?}"))
            .ok();

        let device = OutputDevice {
            token,
            render_node,
            drm,
            gbm,
            allocator,
            drm_scanner: DrmScanner::new(),
            surfaces: HashMap::new(),
            known_crtcs: HashMap::new(),
            drm_lease_state,
            active_leases: Vec::new(),
            non_desktop_connectors: HashSet::new(),
        };
        assert!(self.insert(node, device).is_none());

        Ok(())
    }

    /// Handle connector scanning when a DRM device reports changes (monitor hotplug).
    ///
    /// This method scans for connector changes and returns information about what actions
    /// the caller should take (device_added calls, connector connections).
    pub fn device_changed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
        cleanup: bool,
        should_disable_laptop_panels: bool,
    ) -> DeviceChangedResult {
        debug!("device changed: {device_id}");

        let Ok(node) = DrmNode::from_dev_id(device_id) else {
            warn!("error creating DrmNode");
            return DeviceChangedResult {
                needs_device_added: None,
                connectors_to_connect: Vec::new(),
            };
        };

        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return DeviceChangedResult {
                needs_device_added: None,
                connectors_to_connect: Vec::new(),
            };
        }

        if self.ignored_nodes().contains(&node) {
            debug!("node is ignored, skipping");
            return DeviceChangedResult {
                needs_device_added: None,
                connectors_to_connect: Vec::new(),
            };
        }

        let Some(device) = self.get_mut(&node) else {
            if let Some(path) = node.dev_path() {
                warn!("unknown device; trying to add");
                return DeviceChangedResult {
                    needs_device_added: Some((device_id, path.to_path_buf())),
                    connectors_to_connect: Vec::new(),
                };
            } else {
                warn!("unknown device");
                return DeviceChangedResult {
                    needs_device_added: None,
                    connectors_to_connect: Vec::new(),
                };
            }
        };

        // DrmScanner will preserve any existing connector-CRTC mapping.
        let scan_result = match device.drm_scanner.scan_connectors(&device.drm) {
            Ok(x) => x,
            Err(err) => {
                warn!("error scanning connectors: {err:?}");
                return DeviceChangedResult {
                    needs_device_added: None,
                    connectors_to_connect: Vec::new(),
                };
            }
        };

        let mut added = Vec::new();
        let mut removed = Vec::new();
        for event in scan_result {
            match event {
                smithay_drm_extras::drm_scanner::DrmScanEvent::Connected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    let connector_name = format_connector_name(&connector);
                    let name = make_output_name(&device.drm, connector.handle(), connector_name);
                    debug!(
                        "new connector: {} \"{}\"",
                        &name.connector,
                        name.format_make_model_serial(),
                    );

                    // Assign an id to this crtc.
                    let id = OutputId::next();
                    added.push((crtc, CrtcInfo { id, name }));
                }
                smithay_drm_extras::drm_scanner::DrmScanEvent::Disconnected {
                    crtc: Some(crtc), ..
                } => {
                    removed.push(crtc);
                }
                _ => (),
            }
        }

        for crtc in &removed {
            self.connector_disconnected(niri, node, *crtc);
        }

        let Some(device) = self.get_mut(&node) else {
            error!("device disappeared");
            return DeviceChangedResult {
                needs_device_added: None,
                connectors_to_connect: Vec::new(),
            };
        };

        for crtc in removed {
            if device.known_crtcs.remove(&crtc).is_none() {
                error!("output ID missing for disconnected crtc: {crtc:?}");
            }
        }

        let mut connectors_to_connect = Vec::new();
        for (crtc, mut info) in added {
            // Make/model/serial can match exactly between different physical monitors. This doesn't
            // happen often, but our Layout does not support such duplicates and will panic.
            //
            // As a workaround, search for duplicates, and unname the new connectors if one is
            // found. Connector names are always unique.
            let name = &mut info.name;
            let formatted = name.format_make_model_serial_or_connector();
            for info in self.values().flat_map(|d| d.known_crtcs.values()) {
                if info.name.matches(&formatted) {
                    let connector = mem::take(&mut name.connector);
                    warn!(
                        "new connector {connector} duplicates make/model/serial \
                         of existing connector {}, unnaming",
                        info.name.connector,
                    );
                    *name = niri_config::OutputName {
                        connector,
                        make: None,
                        model: None,
                        serial: None,
                    };
                    break;
                }
            }

            // Insert it right away so next added connector will check against this one too.
            let device = self.get_mut(&node).unwrap();
            device.known_crtcs.insert(crtc, info.clone());

            // Store connector info for later connection - we'll need to find the connector info
            // from the device's drm_scanner when connecting
            connectors_to_connect.push((node, crtc, info));
        }

        // If the device was just added or resumed, we need to cleanup any disconnected connectors
        // and planes.
        if cleanup {
            let device = self.get(&node).unwrap();

            // Follow the logic in on_output_config_changed().
            let should_disable = |conn: &str| should_disable_laptop_panels && is_laptop_panel(conn);

            let config = config.borrow();
            let disable_monitor_names = config.debug.disable_monitor_names;

            let should_be_off = |crtc, conn: &smithay::reexports::drm::control::connector::Info| {
                let output_name = device.known_crtc_name(&crtc, conn, disable_monitor_names);

                let config = config
                    .outputs
                    .find(&output_name)
                    .cloned()
                    .unwrap_or_default();

                config.off || should_disable(&output_name.connector)
            };

            if let Err(err) = device.cleanup_mismatching_resources(&should_be_off) {
                warn!("error cleaning up connectors: {err:?}");
            }

            let device = self.get_mut(&node).unwrap();
            for surface in device.surfaces.values_mut() {
                // We aren't force-clearing the CRTCs, so we need to make the surfaces read the
                // updated state after a session resume. This also causes a full damage for the
                // next redraw.
                if let Err(err) = surface.compositor.reset_state() {
                    warn!("error resetting DrmCompositor state: {err:?}");
                }
            }
        }

        DeviceChangedResult {
            needs_device_added: None,
            connectors_to_connect,
        }
    }

    /// Handle GPU unplug cleanup.
    ///
    /// This method removes a DRM device and cleans up all associated resources.
    /// Returns the device FD that needs to be closed via session.
    pub fn device_removed(
        &mut self,
        device_id: dev_t,
        niri: &mut Niri,
        event_loop: &LoopHandle<State>,
        _session: &LibSeatSession,
    ) -> Option<OwnedFd> {
        debug!("removing device: {device_id}");

        let Ok(node) = DrmNode::from_dev_id(device_id) else {
            warn!("error creating DrmNode");
            return None;
        };

        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return None;
        }

        let Some(device) = self.get_mut(&node) else {
            warn!("unknown device");
            return None;
        };

        let crtcs: Vec<_> = device
            .drm_scanner
            .crtcs()
            .map(|(_info, crtc)| crtc)
            .collect();

        for crtc in crtcs {
            self.connector_disconnected(niri, node, crtc);
        }

        let mut device = self.remove(&node).unwrap();
        let device_fd = device.drm.device_fd().device_fd();

        if let Some(lease_state) = &mut device.drm_lease_state {
            lease_state.disable_global::<State>();
        }

        if let Some(render_node) = device.render_node {
            // Sometimes (Asahi DisplayLink), multiple primary nodes will correspond to the same
            // render node. In this case, we want to keep the render node active until the last
            // primary node that uses it is gone.
            let was_last = !self
                .values()
                .any(|device| device.render_node == Some(render_node));

            let primary_render_node = self.primary_render_node();
            if was_last && render_node == primary_render_node {
                debug!("destroying the primary renderer");

                match self.gpu_manager_mut().single_renderer(&primary_render_node) {
                    Ok(mut renderer) => renderer.unbind_wl_display(),
                    Err(err) => {
                        warn!("error creating renderer during device removal: {err}");
                    }
                }

                // Disable and destroy the dmabuf global.
                if let Some(global) = self.take_dmabuf_global() {
                    niri.protocols
                        .dmabuf
                        .disable_global::<State>(&niri.display_handle, &global);
                    event_loop
                        .insert_source(
                            Timer::from_duration(Duration::from_secs(10)),
                            move |_, _, state| {
                                state
                                    .niri
                                    .protocols
                                    .dmabuf
                                    .destroy_global::<State>(&state.niri.display_handle, global);
                                TimeoutAction::Drop
                            },
                        )
                        .unwrap();

                    // Clear the dmabuf feedbacks for all surfaces.
                    for device in self.values_mut() {
                        for surface in device.surfaces.values_mut() {
                            surface.dmabuf_feedback = None;
                        }
                    }
                } else {
                    error!("dmabuf global was already missing");
                }
            }

            if was_last {
                self.gpu_manager_mut().as_mut().remove_node(&render_node);
                // Trigger re-enumeration in order to remove the device from gpu_manager.
                let _ = self.gpu_manager_mut().devices();
            }
        }

        event_loop.remove(device.token);

        let _ = device;

        // Return the device FD for the caller to close via session
        TryInto::<OwnedFd>::try_into(device_fd).ok()
    }

    /// Handle connector connection - creates DRM surface and compositor.
    ///
    /// This is the largest method in the TTY backend (~347 LOC) and handles
    /// creating a DRM surface when a monitor is connected.
    pub fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        config: &Rc<RefCell<Config>>,
        debug_tint: bool,
    ) -> anyhow::Result<()> {
        let connector_name = format_connector_name(&connector);
        debug!("connecting connector: {connector_name}");

        // Extract values we need before getting mutable device reference.
        let primary_render_node = self.primary_render_node();

        let device = self.get_mut(&node).context("missing device")?;

        let disable_monitor_names = config.borrow().debug.disable_monitor_names;
        let output_name = device.known_crtc_name(&crtc, &connector, disable_monitor_names);

        let non_desktop = find_drm_property(&device.drm, connector.handle(), "non-desktop")
            .and_then(|(_, info, value)| info.value_type().convert_value(value).as_boolean())
            .unwrap_or(false);

        if non_desktop {
            debug!("output is non desktop");
            let description = output_name.format_description();
            if let Some(lease_state) = &mut device.drm_lease_state {
                lease_state.add_connector::<State>(connector.handle(), connector_name, description);
            }
            device
                .non_desktop_connectors
                .insert((connector.handle(), crtc));
            return Ok(());
        }

        let config = config
            .borrow()
            .outputs
            .find(&output_name)
            .cloned()
            .unwrap_or_default();

        for m in connector.modes() {
            trace!("{m:?}");
        }

        let mut mode = None;
        if let Some(modeline) = &config.modeline {
            match calculate_drm_mode_from_modeline(modeline) {
                Ok(x) => mode = Some(x),
                Err(err) => {
                    warn!("invalid custom modeline; falling back to advertised modes: {err:?}");
                }
            }
        }

        let (mode, fallback) = match mode {
            Some(x) => (x, false),
            None => pick_mode(&connector, config.mode).ok_or_else(|| anyhow!("no mode"))?,
        };

        if fallback {
            let target = config.mode.unwrap();
            warn!(
                "configured mode {}x{}{} could not be found, falling back to preferred",
                target.mode.width,
                target.mode.height,
                if let Some(refresh) = target.mode.refresh {
                    format!("@{refresh}")
                } else {
                    String::new()
                },
            );
        }

        debug!("picking mode: {mode:?}");

        let mut orientation = None;
        if let Ok(props) = ConnectorProperties::try_new(&device.drm, connector.handle()) {
            match reset_hdr(&props) {
                Ok(()) => (),
                Err(err) => debug!("couldn't reset HDR properties: {err:?}"),
            }

            match get_panel_orientation(&props) {
                Ok(x) => orientation = Some(x),
                Err(err) => {
                    trace!("couldn't get panel orientation: {err:?}");
                }
            }
        } else {
            warn!("failed to get connector properties");
        };

        let mut gamma_props = GammaProps::new(&device.drm, crtc)
            .map_err(|err| debug!("couldn't get gamma properties: {err:?}"))
            .ok();

        // Reset gamma in case it was set before.
        let res = if let Some(gamma_props) = &mut gamma_props {
            gamma_props.set_gamma(&device.drm, None)
        } else {
            set_gamma_for_crtc(&device.drm, crtc, None)
        };
        if let Err(err) = res {
            debug!("couldn't reset gamma: {err:?}");
        }

        let surface = device
            .drm
            .create_surface(crtc, mode, &[connector.handle()])?;

        // Try to enable VRR if requested.
        match surface.vrr_supported(connector.handle()) {
            Ok(VrrSupport::Supported | VrrSupport::RequiresModeset) => {
                // Even if on-demand, we still disable it until later checks.
                let vrr = config.is_vrr_always_on();
                let word = if vrr { "enabling" } else { "disabling" };

                if let Err(err) = surface.use_vrr(vrr) {
                    warn!("error {} VRR: {err:?}", word);
                }
            }
            Ok(VrrSupport::NotSupported) => {
                if !config.is_vrr_always_off() {
                    warn!("cannot enable VRR because connector does not support it");
                }

                // Try to disable it anyway to work around a bug where resetting DRM state causes
                // vrr_capable to be reset to 0, potentially leaving VRR_ENABLED at 1.
                let _ = surface.use_vrr(false);
            }
            Err(err) => {
                warn!("error querying for VRR support: {err:?}");
            }
        }

        // Update the output mode.
        let (physical_width, physical_height) = connector.size().unwrap_or((0, 0));

        let output = Output::new(
            connector_name.clone(),
            PhysicalProperties {
                size: (physical_width as i32, physical_height as i32).into(),
                subpixel: connector.subpixel().into(),
                model: output_name.model.as_deref().unwrap_or("Unknown").to_owned(),
                make: output_name.make.as_deref().unwrap_or("Unknown").to_owned(),
                serial_number: output_name
                    .serial
                    .as_deref()
                    .unwrap_or("Unknown")
                    .to_owned(),
            },
        );

        let wl_mode = Mode::from(mode);
        output.change_current_state(Some(wl_mode), None, None, None);
        output.set_preferred(wl_mode);

        output
            .user_data()
            .insert_if_missing(|| TtyOutputState { node, crtc });
        output.user_data().insert_if_missing(|| output_name.clone());
        if let Some(x) = orientation {
            output.user_data().insert_if_missing(|| PanelOrientation(x));
        }

        // Extract device render_node before we need to re-borrow self.devices.
        let device_render_node = device.render_node;
        let render_node = device_render_node.unwrap_or(primary_render_node);

        // Drop device reference temporarily to access gpu_manager.
        let _ = device;
        let render_formats = {
            let renderer = self.gpu_manager_mut().single_renderer(&render_node)?;
            let egl_context = renderer.as_ref().egl_context();
            egl_context.dmabuf_render_formats().clone()
        };

        // Re-borrow device.
        let device = self.get_mut(&node).context("missing device")?;

        // Filter out the CCS modifiers as they have increased bandwidth, causing some monitor
        // configurations to stop working.
        //
        // For display only devices, restrict to linear buffers for best compatibility.
        //
        // The invalid modifier attempt below should make this unnecessary in some cases, but it
        // would still be a bad idea to remove this until Smithay has some kind of full-device
        // modesetting test that is able to "downgrade" existing connector modifiers to get enough
        // bandwidth for a newly connected one.
        let render_formats = render_formats
            .iter()
            .copied()
            .filter(|format| {
                if device_render_node.is_none() {
                    return format.modifier == Modifier::Linear;
                }

                let is_ccs = matches!(
                    format.modifier,
                    Modifier::I915_y_tiled_ccs
                    // I915_FORMAT_MOD_Yf_TILED_CCS
                    | Modifier::Unrecognized(0x100000000000005)
                    | Modifier::I915_y_tiled_gen12_rc_ccs
                    | Modifier::I915_y_tiled_gen12_mc_ccs
                    // I915_FORMAT_MOD_Y_TILED_GEN12_RC_CCS_CC
                    | Modifier::Unrecognized(0x100000000000008)
                    // I915_FORMAT_MOD_4_TILED_DG2_RC_CCS
                    | Modifier::Unrecognized(0x10000000000000a)
                    // I915_FORMAT_MOD_4_TILED_DG2_MC_CCS
                    | Modifier::Unrecognized(0x10000000000000b)
                    // I915_FORMAT_MOD_4_TILED_DG2_RC_CCS_CC
                    | Modifier::Unrecognized(0x10000000000000c)
                );

                !is_ccs
            })
            .collect::<FormatSet>();

        // Create the compositor.
        let res = DrmCompositor::new(
            OutputModeSource::Auto(output.clone()),
            surface,
            None,
            device.allocator.clone(),
            GbmFramebufferExporter::new(device.gbm.clone(), device.render_node.into()),
            SUPPORTED_COLOR_FORMATS.iter().copied(),
            // This is only used to pick a good internal format, so it can use the surface's render
            // formats, even though we only ever render on the primary GPU.
            render_formats.clone(),
            device.drm.cursor_size(),
            Some(device.gbm.clone()),
        );

        let mut compositor = match res {
            Ok(x) => x,
            Err(err) => {
                warn!("error creating DRM compositor, will try with invalid modifier: {err:?}");

                let render_formats = render_formats
                    .iter()
                    .copied()
                    .filter(|format| format.modifier == Modifier::Invalid)
                    .collect::<FormatSet>();

                // DrmCompositor::new() consumed the surface...
                let surface = device
                    .drm
                    .create_surface(crtc, mode, &[connector.handle()])?;

                DrmCompositor::new(
                    OutputModeSource::Auto(output.clone()),
                    surface,
                    None,
                    device.allocator.clone(),
                    GbmFramebufferExporter::new(device.gbm.clone(), device.render_node.into()),
                    SUPPORTED_COLOR_FORMATS.iter().copied(),
                    render_formats,
                    device.drm.cursor_size(),
                    Some(device.gbm.clone()),
                )
                .context("error creating DRM compositor")?
            }
        };

        if debug_tint {
            compositor.set_debug_flags(DebugFlags::TINT);
        }

        // Drop device to access gpu_manager_mut for dmabuf feedback.
        let _ = device;

        let mut dmabuf_feedback = None;
        if let Ok(primary_renderer) = self.gpu_manager_mut().single_renderer(&primary_render_node) {
            let primary_formats = primary_renderer.dmabuf_formats();

            match surface_dmabuf_feedback(
                &compositor,
                primary_formats,
                primary_render_node,
                device_render_node,
                node,
            ) {
                Ok(feedback) => {
                    dmabuf_feedback = Some(feedback);
                }
                Err(err) => {
                    warn!("error building dmabuf feedback: {err:?}");
                }
            }
        }

        // Some buggy monitors replug upon powering off, so powering on here would prevent such
        // monitors from powering off. Therefore, we avoid unconditionally powering on.
        if !niri.outputs.monitors_active() {
            if let Err(err) = compositor.clear() {
                warn!("error clearing drm surface: {err:?}");
            }
        }

        let vrr_enabled = compositor.vrr_enabled();

        let vblank_frame_name =
            tracy_client::FrameName::new_leak(format!("vblank on {connector_name}"));
        let time_since_presentation_plot_name = tracy_client::PlotName::new_leak(format!(
            "{connector_name} time since presentation, ms"
        ));
        let presentation_misprediction_plot_name = tracy_client::PlotName::new_leak(format!(
            "{connector_name} presentation misprediction, ms"
        ));
        let sequence_delta_plot_name =
            tracy_client::PlotName::new_leak(format!("{connector_name} sequence delta"));

        let surface = Surface {
            name: output_name,
            connector: connector.handle(),
            compositor,
            dmabuf_feedback,
            gamma_props,
            pending_gamma_change: None,
            vblank_frame: None,
            vblank_frame_name,
            time_since_presentation_plot_name,
            presentation_misprediction_plot_name,
            sequence_delta_plot_name,
        };

        // Re-borrow device to insert surface.
        let device = self.get_mut(&node).context("missing device")?;
        let res = device.surfaces.insert(crtc, surface);
        assert!(res.is_none(), "crtc must not have already existed");

        niri.add_output(output.clone(), Some(refresh_interval(mode)), vrr_enabled);

        if niri.outputs.monitors_active() {
            // Redraw the new monitor.
            niri.event_loop.insert_idle(move |state| {
                // Guard against output disconnecting before the idle has a chance to run.
                if state.niri.outputs.state(&output).is_some() {
                    state.niri.queue_redraw(&output);
                }
            });
        }

        Ok(())
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
