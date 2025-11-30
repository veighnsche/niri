//! TTY/DRM backend for native display.
//!
//! This module uses the **subsystem ownership pattern**:
//! - `DeviceManager` (devices.rs) - owns all DRM device state and lifecycle
//! - `RenderManager` (render.rs) - owns rendering and vblank handling
//! - `OutputManager` (outputs.rs) - owns IPC and output configuration
//!
//! `Tty` is a thin coordinator that:
//! - Holds session and udev state
//! - Dispatches events to subsystems
//! - Provides public API delegation

mod devices;
mod helpers;
mod outputs;
mod render;
mod types;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::zip;
use std::num::NonZeroU64;
use std::os::fd::AsFd;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::mem;

use anyhow::{anyhow, ensure, Context};
use bytemuck::cast_slice_mut;
use libc::dev_t;
use niri_config::{Config, OutputName};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::allocator::gbm::GbmDevice;
use smithay::backend::drm::{
    DrmDevice, DrmDeviceFd, DrmEventMetadata, DrmEventTime, DrmNode, NodeType,
};
use smithay::backend::egl::context::ContextPriority;
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::backend::renderer::ImportDma;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::{Event as SessionEvent, Session};
use smithay::backend::udev::{self, UdevBackend, UdevEvent};
use smithay::output::Output;
use smithay::reexports::calloop::{Dispatcher, LoopHandle};
use smithay::reexports::drm::control::atomic::AtomicModeReq;
use smithay::reexports::drm::control::dumbbuffer::DumbBuffer;
use smithay::reexports::drm::control::{
    connector, crtc, plane, property, AtomicCommitFlags, Device, PlaneType,
};
use smithay::reexports::input::Libinput;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

pub use devices::{DeviceManager, OutputDevice};
pub use helpers::{calculate_drm_mode_from_modeline, calculate_mode_cvt, set_gamma_for_crtc};
pub use outputs::OutputManager;
pub use render::RenderManager;
pub use types::{CrtcInfo, SurfaceDmabufFeedback, TtyFrame, TtyRenderer, TtyRendererError};
use devices::format_connector_name;
use helpers::{
    find_drm_property, get_drm_property, ignored_nodes_from_config, primary_node_from_config,
    reset_hdr,
};
use types::{ConnectorProperties, GammaProps, TtyOutputState};

use super::{IpcOutputMap, RenderResult};
use crate::niri::{Niri, State};
use crate::render_helpers::renderer::AsGlesRenderer;
use crate::utils::{get_monotonic_time, is_laptop_panel};

pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,
    // Device management subsystem - owns all DRM device state.
    pub(crate) devices: DeviceManager,
    // Render management subsystem - owns render state.
    render: RenderManager,
    // Output management subsystem - owns IPC and resume state.
    outputs: OutputManager,
}

// Additional impl for OutputDevice that uses functions from this module.
// Will be moved to devices.rs in Phase T1.3 when helpers are extracted.
impl OutputDevice {
    fn cleanup_mismatching_resources(
        &self,
        should_be_off: &dyn Fn(crtc::Handle, &connector::Info) -> bool,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("OutputDevice::cleanup_disconnected_resources");

        let res_handles = self
            .drm
            .resource_handles()
            .context("error getting plane handles")?;
        let plane_handles = self
            .drm
            .plane_handles()
            .context("error getting plane handles")?;

        let mut req = AtomicModeReq::new();

        // We want to disable all CRTCs that do not correspond to a connector we're using.
        let mut cleanup = HashSet::<crtc::Handle>::new();
        cleanup.extend(res_handles.crtcs());

        for (conn, info) in self.drm_scanner.connectors() {
            // We only keep the connector if it has a CRTC and the output isn't off in niri.
            if let Some(crtc) = self.drm_scanner.crtc_for_connector(conn) {
                // Verify that the connector's current CRTC matches the CRTC we expect. If not,
                // clear the CRTC and the connector so that all connectors can get the expected
                // CRTCs afterwards. (We do this because we do not handle CRTC rotations across TTY
                // switches.)
                let mut has_different_crtc = false;
                if let Some(enc) = info.current_encoder() {
                    match self.drm.get_encoder(enc) {
                        Ok(enc) => {
                            if let Some(current_crtc) = enc.crtc() {
                                if current_crtc != crtc {
                                    has_different_crtc = true;
                                }
                            }
                        }
                        Err(err) => {
                            debug!("couldn't get encoder: {err:?}");
                            // Err on the safe side.
                            has_different_crtc = true;
                        }
                    }
                }

                if !has_different_crtc && !should_be_off(crtc, info) {
                    // Keep the corresponding CRTC.
                    cleanup.remove(&crtc);
                    continue;
                }
            }

            // Clear the connector.
            let Some((crtc_id, _, _)) = find_drm_property(&self.drm, *conn, "CRTC_ID") else {
                debug!("couldn't find connector CRTC_ID property");
                continue;
            };

            req.add_property(*conn, crtc_id, property::Value::CRTC(None));
        }

        // Legacy fallback.
        if !self.drm.is_atomic() {
            for crtc in res_handles.crtcs() {
                #[allow(deprecated)]
                let _ = self.drm.set_cursor(*crtc, Option::<&DumbBuffer>::None);
            }
            for crtc in cleanup {
                let _ = self.drm.set_crtc(crtc, None, (0, 0), &[], None);
            }
            return Ok(());
        }

        // Disable non-primary planes, and planes belonging to disabled CRTCs.
        let is_primary = |plane: plane::Handle| {
            if let Some((_, info, value)) = find_drm_property(&self.drm, plane, "type") {
                match info.value_type().convert_value(value) {
                    property::Value::Enum(Some(val)) => val.value() == PlaneType::Primary as u64,
                    _ => false,
                }
            } else {
                debug!("couldn't find plane type property");
                false
            }
        };

        for plane in plane_handles {
            let info = match self.drm.get_plane(plane) {
                Ok(x) => x,
                Err(err) => {
                    debug!("error getting plane: {err:?}");
                    continue;
                }
            };

            let Some(crtc) = info.crtc() else {
                continue;
            };

            if !cleanup.contains(&crtc) && is_primary(plane) {
                continue;
            }

            let Some((crtc_id, _, _)) = find_drm_property(&self.drm, plane, "CRTC_ID") else {
                debug!("couldn't find plane CRTC_ID property");
                continue;
            };

            let Some((fb_id, _, _)) = find_drm_property(&self.drm, plane, "FB_ID") else {
                debug!("couldn't find plane FB_ID property");
                continue;
            };

            req.add_property(plane, crtc_id, property::Value::CRTC(None));
            req.add_property(plane, fb_id, property::Value::Framebuffer(None));
        }

        // Disable the CRTCs.
        for crtc in cleanup {
            let Some((mode_id, _, _)) = find_drm_property(&self.drm, crtc, "MODE_ID") else {
                debug!("couldn't find CRTC MODE_ID property");
                continue;
            };

            let Some((active, _, _)) = find_drm_property(&self.drm, crtc, "ACTIVE") else {
                debug!("couldn't find CRTC ACTIVE property");
                continue;
            };

            req.add_property(crtc, mode_id, property::Value::Unknown(0));
            req.add_property(crtc, active, property::Value::Boolean(false));
        }

        self.drm
            .atomic_commit(AtomicCommitFlags::ALLOW_MODESET, req)
            .context("error doing atomic commit")?;

        Ok(())
    }
}

impl Tty {
    pub fn new(
        config: Rc<RefCell<Config>>,
        event_loop: LoopHandle<'static, State>,
    ) -> anyhow::Result<Self> {
        let _span = tracy_client::span!("Tty::new");

        let (session, notifier) = LibSeatSession::new().context(
            "Error creating a session. This might mean that you're trying to run niri on a TTY \
             that is already busy, for example if you're running this inside tmux that had been \
             originally started on a different TTY",
        )?;
        let seat_name = session.seat();

        let udev_backend =
            UdevBackend::new(session.seat()).context("error creating a udev backend")?;
        let udev_dispatcher = Dispatcher::new(udev_backend, move |event, _, state: &mut State| {
            state.backend.tty().on_udev_event(&mut state.niri, event);
        });
        event_loop
            .register_dispatcher(udev_dispatcher.clone())
            .unwrap();

        let mut libinput = Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));
        {
            let _span = tracy_client::span!("Libinput::udev_assign_seat");
            libinput.udev_assign_seat(&seat_name)
        }
        .map_err(|()| anyhow!("error assigning the seat to libinput"))?;

        let input_backend = LibinputInputBackend::new(libinput.clone());
        event_loop
            .insert_source(input_backend, |mut event, _, state| {
                state.process_libinput_event(&mut event);
                state.process_input_event(event);
            })
            .unwrap();

        event_loop
            .insert_source(notifier, move |event, _, state| {
                state.backend.tty().on_session_event(&mut state.niri, event);
            })
            .unwrap();

        let api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let gpu_manager = GpuManager::new(api).context("error creating the GPU manager")?;

        let (primary_node, primary_render_node) = primary_node_from_config(&config.borrow())
            .ok_or(())
            .or_else(|()| {
                let primary_gpu_path = udev::primary_gpu(&seat_name)
                    .context("error getting the primary GPU")?
                    .context("couldn't find a GPU")?;
                let primary_node = DrmNode::from_path(primary_gpu_path)
                    .context("error opening the primary GPU DRM node")?;
                let primary_render_node = primary_node
                    .node_with_type(NodeType::Render)
                    .and_then(Result::ok)
                    .unwrap_or_else(|| {
                        warn!(
                            "error getting the render node for the primary GPU; proceeding anyway"
                        );
                        primary_node
                    });

                Ok::<_, anyhow::Error>((primary_node, primary_render_node))
            })?;

        let mut node_path = String::new();
        if let Some(path) = primary_render_node.dev_path() {
            write!(node_path, "{path:?}").unwrap();
        } else {
            write!(node_path, "{primary_render_node}").unwrap();
        }
        info!("using as the render node: {node_path}");

        let mut ignored_nodes = ignored_nodes_from_config(&config.borrow());
        if ignored_nodes.remove(&primary_node) || ignored_nodes.remove(&primary_render_node) {
            warn!("ignoring the primary node or render node is not allowed");
        }

        // Create the device management subsystem.
        let devices = DeviceManager::new(
            primary_node,
            primary_render_node,
            ignored_nodes,
            gpu_manager,
        );

        Ok(Self {
            config,
            session,
            udev_dispatcher,
            libinput,
            devices,
            render: RenderManager::new(),
            outputs: OutputManager::new(),
        })
    }

    pub fn init(&mut self, niri: &mut Niri) {
        let udev = self.udev_dispatcher.clone();
        let udev = udev.as_source_ref();

        // Initialize the primary node first as later nodes might depend on the primary render node
        // being available.
        if let Some((primary_device_id, primary_device_path)) = udev
            .device_list()
            .find(|&(device_id, _)| device_id == self.devices.primary_node().dev_id())
        {
            if let Err(err) = self.device_added(primary_device_id, primary_device_path, niri) {
                warn!(
                    "error adding primary node device, display-only devices may not work: {err:?}"
                );
            }
        } else {
            warn!("primary node is missing, display-only devices may not work");
        };

        for (device_id, path) in udev.device_list() {
            if device_id == self.devices.primary_node().dev_id() {
                continue;
            }

            if let Err(err) = self.device_added(device_id, path, niri) {
                warn!("error adding device: {err:?}");
            }
        }
    }

    fn on_udev_event(&mut self, niri: &mut Niri, event: UdevEvent) {
        let _span = tracy_client::span!("Tty::on_udev_event");

        match event {
            UdevEvent::Added { device_id, path } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Added as session is inactive");
                    return;
                }

                if let Err(err) = self.device_added(device_id, &path, niri) {
                    warn!("error adding device: {err:?}");
                }
            }
            UdevEvent::Changed { device_id } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Changed as session is inactive");
                    return;
                }

                self.device_changed(device_id, niri, false)
            }
            UdevEvent::Removed { device_id } => {
                if !self.session.is_active() {
                    debug!("skipping UdevEvent::Removed as session is inactive");
                    return;
                }

                self.device_removed(device_id, niri)
            }
        }
    }

    fn on_session_event(&mut self, niri: &mut Niri, event: SessionEvent) {
        let _span = tracy_client::span!("Tty::on_session_event");

        match event {
            SessionEvent::PauseSession => {
                debug!("pausing session");

                self.libinput.suspend();
                self.devices.pause_devices();
            }
            SessionEvent::ActivateSession => {
                debug!("resuming session");

                if self.libinput.resume().is_err() {
                    warn!("error resuming libinput");
                }

                if self.outputs.needs_ignored_nodes_update_on_resume() {
                    self.outputs.clear_ignored_nodes_update_on_resume();
                    let mut ignored_nodes = ignored_nodes_from_config(&self.config.borrow());
                    if ignored_nodes.remove(&self.devices.primary_node())
                        || ignored_nodes.remove(&self.devices.primary_render_node())
                    {
                        warn!("ignoring the primary node or render node is not allowed");
                    }
                    self.devices.set_ignored_nodes(ignored_nodes);
                }

                let mut device_list = self
                    .udev_dispatcher
                    .as_source_ref()
                    .device_list()
                    .map(|(device_id, path)| (device_id, path.to_owned()))
                    .collect::<HashMap<_, _>>();

                let removed_devices = self
                    .devices
                    .keys()
                    .filter(|node| {
                        !device_list.contains_key(&node.dev_id())
                            || self.devices.ignored_nodes().contains(node)
                    })
                    .copied()
                    .collect::<Vec<_>>();

                let remained_devices = self
                    .devices
                    .keys()
                    .filter(|node| {
                        device_list.contains_key(&node.dev_id())
                            && !self.devices.ignored_nodes().contains(node)
                    })
                    .copied()
                    .collect::<Vec<_>>();

                // Remove removed devices.
                for node in removed_devices {
                    device_list.remove(&node.dev_id());
                    self.device_removed(node.dev_id(), niri);
                }

                // Update remained devices.
                for node in remained_devices {
                    device_list.remove(&node.dev_id());

                    // Activate the DRM device and resume lease state.
                    if let Err(err) = self.devices.activate_device(&node) {
                        warn!("error activating DRM device: {err:?}");
                    }

                    // Refresh the connectors.
                    self.device_changed(node.dev_id(), niri, true);

                    // Apply pending gamma changes and restore our existing gamma.
                    let device = self.devices.get_mut(&node).unwrap();
                    for (crtc, surface) in device.surfaces.iter_mut() {
                        if let Ok(props) =
                            ConnectorProperties::try_new(&device.drm, surface.connector)
                        {
                            match reset_hdr(&props) {
                                Ok(()) => (),
                                Err(err) => debug!("couldn't reset HDR properties: {err:?}"),
                            }
                        } else {
                            warn!("failed to get connector properties");
                        };

                        if let Some(ramp) = surface.pending_gamma_change.take() {
                            let ramp = ramp.as_deref();
                            let res = if let Some(gamma_props) = &mut surface.gamma_props {
                                gamma_props.set_gamma(&device.drm, ramp)
                            } else {
                                set_gamma_for_crtc(&device.drm, *crtc, ramp)
                            };
                            if let Err(err) = res {
                                warn!("error applying pending gamma change: {err:?}");
                            }
                        } else if let Some(gamma_props) = &surface.gamma_props {
                            if let Err(err) = gamma_props.restore_gamma(&device.drm) {
                                warn!("error restoring gamma: {err:?}");
                            }
                        }
                    }
                }

                // Add new devices.
                for (device_id, path) in device_list.into_iter() {
                    if let Err(err) = self.device_added(device_id, &path, niri) {
                        warn!("error adding device: {err:?}");
                    }
                }

                if self.outputs.needs_config_update_on_resume() {
                    self.on_output_config_changed(niri);
                }

                self.refresh_ipc_outputs(niri);

                niri.notify_activity();
                niri.outputs.set_monitors_active(true);
                self.set_monitors_active(true);
                niri.queue_redraw_all();
            }
        }
    }

    fn device_added(
        &mut self,
        device_id: dev_t,
        path: &Path,
        niri: &mut Niri,
    ) -> anyhow::Result<()> {
        // TEAM_090: Extract values before borrowing to avoid conflicts
        let event_loop = niri.event_loop.clone();
        let debug_tint = self.render.debug_tint();
        
        // TEAM_090: Delegate to DeviceManager with proper parameters
        self.devices.device_added(
            device_id,
            path,
            &mut self.session,
            &event_loop,
            &self.config,
            niri,
            debug_tint,
        )?;
        
        // device_changed is called by the caller after delegation
        self.device_changed(device_id, niri, true);
        Ok(())
    }

    fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool) {
        // TEAM_091: Extract parameters and delegate to DeviceManager
        let should_disable = self.should_disable_laptop_panels(niri.outputs.lid_closed());
        
        let result = self.devices.device_changed(
            device_id,
            niri,
            &self.config,
            cleanup,
            should_disable,
        );
        
        // Handle device_added if needed
        if let Some((dev_id, path)) = result.needs_device_added {
            if let Err(err) = self.device_added(dev_id, &path, niri) {
                warn!("error adding device: {err:?}");
            }
        }
        
        // Note: connectors_to_connect are handled by on_output_config_changed in the original code
        // The DeviceManager now stores the connector info in known_crtcs, and on_output_config_changed
        // will handle the actual connection
        
        self.on_output_config_changed(niri);
    }

    fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri) {
        // TEAM_092: Delegate to DeviceManager with proper parameters
        let event_loop = niri.event_loop.clone();
        let fd = self.devices.device_removed(
            device_id,
            niri,
            &event_loop,
            &self.session,
        );
        
        if let Some(fd) = fd {
            if let Err(err) = self.session.close(fd) {
                warn!("error closing DRM device fd: {err:?}");
            }
        }
        
        self.refresh_ipc_outputs(niri);
    }

    fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    ) -> anyhow::Result<()> {
        // TEAM_093: Delegate to DeviceManager with proper parameters
        self.devices.connector_connected(
            niri,
            node,
            connector,
            crtc,
            &self.config,
            self.render.debug_tint(),
        )
    }

    fn on_vblank(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        meta: DrmEventMetadata,
    ) {
        // TEAM_095: Handle vblank_throttle callback to avoid circular dependency
        let now = get_monotonic_time();

        let Some(device) = self.devices.get_mut(&node) else {
            // I've seen it happen.
            error!("missing device in vblank callback for crtc {crtc:?}");
            return;
        };

        let Some(surface) = device.surfaces.get_mut(&crtc) else {
            error!("missing surface in vblank callback for crtc {crtc:?}");
            return;
        };

        let name = &surface.name.connector;
        trace!("vblank on {name} {meta:?}");

        let Some(output) = niri
            .outputs
            .space()
            .outputs()
            .find(|output| {
                let tty_state: &TtyOutputState = output.user_data().get().unwrap();
                tty_state.node == node && tty_state.crtc == crtc
            })
            .cloned()
        else {
            error!("missing output in global space for {name}");
            return;
        };

        let Some(output_state) = niri.outputs.state_mut(&output) else {
            error!("missing output state for {name}");
            return;
        };

        let refresh_interval = output_state.frame_clock.refresh_interval();

        let presentation_time = match meta.time {
            DrmEventTime::Monotonic(time) => time,
            DrmEventTime::Realtime(_) => {
                // Not supported.
                Duration::ZERO
            }
        };
        let presentation_time = if self.config.borrow().debug.emulate_zero_presentation_time {
            Duration::ZERO
        } else {
            presentation_time
        };

        let time = if presentation_time.is_zero() {
            now
        } else {
            presentation_time
        };

        // Handle vblank_throttle callback to avoid circular dependency
        if output_state
            .vblank_throttle
            .throttle(refresh_interval, time, move |state| {
                let meta = DrmEventMetadata {
                    sequence: meta.sequence,
                    time: DrmEventTime::Monotonic(Duration::ZERO),
                };

                let tty = state.backend.tty();
                tty.on_vblank(&mut state.niri, node, crtc, meta);
            })
        {
            // Throttled.
            return;
        }

        // Delegate to RenderManager for actual vblank processing
        self.render.on_vblank(
            &mut self.devices,
            niri,
            node,
            crtc,
            meta,
            &self.config,
        )
    }

    pub fn seat_name(&self) -> String {
        self.session.seat()
    }

    pub fn with_primary_renderer<T>(
        &mut self,
        f: impl FnOnce(&mut GlesRenderer) -> T,
    ) -> Option<T> {
        let primary_render_node = self.devices.primary_render_node();
        let mut renderer = self
            .devices
            .gpu_manager_mut()
            .single_renderer(&primary_render_node)
            .ok()?;
        Some(f(renderer.as_gles_renderer()))
    }

    pub fn render(
        &mut self,
        niri: &mut Niri,
        output: &Output,
        target_presentation_time: Duration,
    ) -> RenderResult {
        // TEAM_094: Delegate to RenderManager with proper parameters
        self.render.render(
            &mut self.devices,
            niri,
            output,
            target_presentation_time,
            &self.config,
        )
    }

    pub fn change_vt(&mut self, vt: i32) {
        if let Err(err) = self.session.change_vt(vt) {
            warn!("error changing VT: {err}");
        }
    }

    pub fn suspend(&self) {
        #[cfg(feature = "dbus")]
        if let Err(err) = suspend() {
            warn!("error suspending: {err:?}");
        }
    }

    pub fn toggle_debug_tint(&mut self) {
        self.render.toggle_debug_tint(&mut self.devices);
    }

    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        let primary_render_node = self.devices.primary_render_node();
        let mut renderer = match self.devices.gpu_manager_mut().single_renderer(&primary_render_node) {
            Ok(renderer) => renderer,
            Err(err) => {
                debug!("error creating renderer for primary GPU: {err:?}");
                return false;
            }
        };

        match renderer.import_dmabuf(dmabuf, None) {
            Ok(_texture) => {
                dmabuf.set_node(Some(primary_render_node));
                true
            }
            Err(err) => {
                debug!("error importing dmabuf: {err:?}");
                false
            }
        }
    }

    pub fn early_import(&mut self, surface: &WlSurface) {
        let primary_render_node = self.devices.primary_render_node();
        if let Err(err) = self.devices.gpu_manager_mut().early_import(
            // We always render on the primary GPU.
            primary_render_node,
            surface,
        ) {
            warn!("error doing early import: {err:?}");
        }
    }

    pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32> {
        // TEAM_098: Delegate to OutputManager
        self.outputs.get_gamma_size(&self.devices, output)
    }

    pub fn set_gamma(&mut self, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()> {
        // TEAM_098: Delegate to OutputManager
        self.outputs.set_gamma(&mut self.devices, output, ramp, self.session.is_active())
    }

    fn refresh_ipc_outputs(&self, niri: &mut Niri) {
        // TEAM_097: Delegate to OutputManager
        self.outputs.refresh_ipc_outputs(&self.devices, niri, &self.config)
    }

    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        self.outputs.ipc_outputs()
    }

    #[cfg(feature = "xdp-gnome-screencast")]
    pub fn primary_gbm_device(&self) -> Option<GbmDevice<DrmDeviceFd>> {
        // Try to find a device corresponding to the primary render node.
        let device = self
            .devices
            .values()
            .find(|d| d.render_node == Some(self.devices.primary_render_node()));
        // Otherwise, try to get the device corresponding to the primary node.
        let device = device.or_else(|| self.devices.get(&self.devices.primary_node()));

        Some(device?.gbm.clone())
    }

    pub fn set_monitors_active(&mut self, active: bool) {
        // TEAM_099: Delegate to OutputManager
        self.outputs.set_monitors_active(&mut self.devices, active)
    }

    pub fn set_output_on_demand_vrr(&mut self, niri: &mut Niri, output: &Output, enable_vrr: bool) {
        // TEAM_100: Delegate to OutputManager
        let needs_refresh = self.outputs.set_output_on_demand_vrr(
            &mut self.devices,
            niri,
            output,
            enable_vrr,
        );
        
        if needs_refresh {
            self.refresh_ipc_outputs(niri);
        }
    }

    pub fn update_ignored_nodes_config(&mut self, niri: &mut Niri) {
        let _span = tracy_client::span!("Tty::update_ignored_nodes_config");

        // If we're inactive, we can't do anything, so just set a flag for later.
        if !self.session.is_active() {
            self.outputs.mark_ignored_nodes_update_on_resume();
            return;
        }

        let mut ignored_nodes = ignored_nodes_from_config(&self.config.borrow());
        if ignored_nodes.remove(&self.devices.primary_node())
            || ignored_nodes.remove(&self.devices.primary_render_node())
        {
            warn!("ignoring the primary node or render node is not allowed");
        }

        if &ignored_nodes == self.devices.ignored_nodes() {
            return;
        }
        self.devices.set_ignored_nodes(ignored_nodes);

        let mut device_list = self
            .udev_dispatcher
            .as_source_ref()
            .device_list()
            .map(|(device_id, path)| (device_id, path.to_owned()))
            .collect::<HashMap<_, _>>();

        let removed_devices = self
            .devices
            .keys()
            .filter(|node| {
                self.devices.ignored_nodes().contains(node) || !device_list.contains_key(&node.dev_id())
            })
            .copied()
            .collect::<Vec<_>>();

        for node in removed_devices {
            device_list.remove(&node.dev_id());
            self.device_removed(node.dev_id(), niri);
        }

        for node in self.devices.keys() {
            device_list.remove(&node.dev_id());
        }

        for (device_id, path) in device_list {
            if let Err(err) = self.device_added(device_id, &path, niri) {
                warn!("error adding device {path:?}: {err:?}");
            }
        }
    }

    fn should_disable_laptop_panels(&self, is_lid_closed: bool) -> bool {
        if !is_lid_closed {
            return false;
        }

        let config = self.config.borrow();
        if !config.debug.keep_laptop_panel_on_when_lid_is_closed {
            // Check if any external monitor is connected.
            for device in self.devices.values() {
                for (connector, _crtc) in device.drm_scanner.crtcs() {
                    if !is_laptop_panel(&format_connector_name(connector)) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn on_output_config_changed(&mut self, niri: &mut Niri) {
        // TEAM_101: Delegate to OutputManager
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

    pub fn get_device_from_node(&mut self, node: DrmNode) -> Option<&mut OutputDevice> {
        self.devices.get_mut(&node)
    }

    pub fn disconnected_connector_name_by_name_match(&self, target: &str) -> Option<OutputName> {
        let disable_monitor_names = self.config.borrow().debug.disable_monitor_names;
        for device in self.devices.values() {
            for (connector, crtc) in device.drm_scanner.crtcs() {
                // Check if connected.
                if connector.state() != connector::State::Connected {
                    continue;
                }

                // Check if already enabled.
                if device.surfaces.contains_key(&crtc)
                    || device
                        .non_desktop_connectors
                        .contains(&(connector.handle(), crtc))
                {
                    continue;
                }

                let output_name = device.known_crtc_name(&crtc, connector, disable_monitor_names);
                if output_name.matches(target) {
                    return Some(output_name);
                }
            }
        }

        None
    }
}

impl GammaProps {
    fn new(device: &DrmDevice, crtc: crtc::Handle) -> anyhow::Result<Self> {
        let mut gamma_lut = None;
        let mut gamma_lut_size = None;

        let props = device
            .get_properties(crtc)
            .context("error getting properties")?;
        for (prop, _) in props {
            let Ok(info) = device.get_property(prop) else {
                continue;
            };

            let Ok(name) = info.name().to_str() else {
                continue;
            };

            match name {
                "GAMMA_LUT" => {
                    ensure!(
                        matches!(info.value_type(), property::ValueType::Blob),
                        "wrong GAMMA_LUT value type"
                    );
                    gamma_lut = Some(prop);
                }
                "GAMMA_LUT_SIZE" => {
                    ensure!(
                        matches!(info.value_type(), property::ValueType::UnsignedRange(_, _)),
                        "wrong GAMMA_LUT_SIZE value type"
                    );
                    gamma_lut_size = Some(prop);
                }
                _ => (),
            }
        }

        let gamma_lut = gamma_lut.context("missing GAMMA_LUT property")?;
        let gamma_lut_size = gamma_lut_size.context("missing GAMMA_LUT_SIZE property")?;

        Ok(Self {
            crtc,
            gamma_lut,
            gamma_lut_size,
            previous_blob: None,
        })
    }

    fn gamma_size(&self, device: &DrmDevice) -> anyhow::Result<u32> {
        let value = get_drm_property(device, self.crtc, self.gamma_lut_size)
            .context("missing GAMMA_LUT_SIZE property")?;
        Ok(value as u32)
    }

    fn set_gamma(&mut self, device: &DrmDevice, gamma: Option<&[u16]>) -> anyhow::Result<()> {
        let _span = tracy_client::span!("GammaProps::set_gamma");

        let blob = if let Some(gamma) = gamma {
            let gamma_size = self
                .gamma_size(device)
                .context("error getting gamma size")? as usize;

            ensure!(gamma.len() == gamma_size * 3, "wrong gamma length");

            #[allow(non_camel_case_types)]
            #[repr(C)]
            #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
            pub struct drm_color_lut {
                pub red: u16,
                pub green: u16,
                pub blue: u16,
                pub reserved: u16,
            }

            let (red, rest) = gamma.split_at(gamma_size);
            let (blue, green) = rest.split_at(gamma_size);
            let mut data = zip(zip(red, blue), green)
                .map(|((&red, &green), &blue)| drm_color_lut {
                    red,
                    green,
                    blue,
                    reserved: 0,
                })
                .collect::<Vec<_>>();
            let data = cast_slice_mut(&mut data);

            let blob = drm_ffi::mode::create_property_blob(device.as_fd(), data)
                .context("error creating property blob")?;
            NonZeroU64::new(u64::from(blob.blob_id))
        } else {
            None
        };

        {
            let _span = tracy_client::span!("set_property");

            let blob = blob.map(NonZeroU64::get).unwrap_or(0);
            device
                .set_property(
                    self.crtc,
                    self.gamma_lut,
                    property::Value::Blob(blob).into(),
                )
                .context("error setting GAMMA_LUT")
                .inspect_err(|_| {
                    if blob != 0 {
                        // Destroy the blob we just allocated.
                        if let Err(err) = device.destroy_property_blob(blob) {
                            warn!("error destroying GAMMA_LUT property blob: {err:?}");
                        }
                    }
                })?;
        }

        if let Some(blob) = mem::replace(&mut self.previous_blob, blob) {
            if let Err(err) = device.destroy_property_blob(blob.get()) {
                warn!("error destroying previous GAMMA_LUT blob: {err:?}");
            }
        }

        Ok(())
    }

    fn restore_gamma(&self, device: &DrmDevice) -> anyhow::Result<()> {
        let _span = tracy_client::span!("GammaProps::restore_gamma");

        let blob = self.previous_blob.map(NonZeroU64::get).unwrap_or(0);
        device
            .set_property(
                self.crtc,
                self.gamma_lut,
                property::Value::Blob(blob).into(),
            )
            .context("error setting GAMMA_LUT")?;

        Ok(())
    }
}

// Helper functions moved to helpers.rs

#[cfg(feature = "dbus")]
fn suspend() -> anyhow::Result<()> {
    let conn = zbus::blocking::Connection::system().context("error connecting to system bus")?;

    conn.call_method(
        Some("org.freedesktop.login1"),
        "/org/freedesktop/login1",
        Some("org.freedesktop.login1.Manager"),
        "Suspend",
        &(true),
    )
    .context("error suspending")?;

    Ok(())
}

// Mode calculation helpers moved to helpers.rs

impl<'a> ConnectorProperties<'a> {
    fn try_new(device: &'a DrmDevice, connector: connector::Handle) -> anyhow::Result<Self> {
        let prop_vals = device
            .get_properties(connector)
            .context("error getting properties")?;

        let mut properties = Vec::new();

        for (prop, value) in prop_vals {
            let info = device
                .get_property(prop)
                .context("error getting property")?;

            properties.push((info, value));
        }

        Ok(Self {
            device,
            connector,
            properties,
        })
    }

    fn find(&self, name: &std::ffi::CStr) -> anyhow::Result<&(property::Info, property::RawValue)> {
        for prop in &self.properties {
            if prop.0.name() == name {
                return Ok(prop);
            }
        }

        Err(anyhow!("couldn't find property: {name:?}"))
    }
}

// HDR/VRR/gamma helpers and tests moved to helpers.rs
