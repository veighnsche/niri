//! TTY/DRM backend for native display.
//!
//! This module uses the **subsystem ownership pattern**:
//! - `DeviceManager` - owns all DRM device state (future)
//! - `RenderManager` - owns rendering state (future)
//! - `OutputManager` - owns IPC/config state (future)
//!
//! `Tty` is a thin coordinator that dispatches events to subsystems.

mod devices;
mod helpers;
mod types;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::zip;
use std::num::NonZeroU64;
use std::os::fd::{AsFd, OwnedFd};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::mem;

use anyhow::{anyhow, bail, ensure, Context};
use bytemuck::cast_slice_mut;
use libc::dev_t;
use niri_config::{Config, OutputName};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::allocator::format::FormatSet;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmBufferFlags, GbmDevice};
use smithay::backend::drm::compositor::{DrmCompositor, FrameFlags, PrimaryPlaneElement};
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::drm::{
    DrmDevice, DrmDeviceFd, DrmEvent, DrmEventMetadata, DrmEventTime, DrmNode, NodeType, VrrSupport,
};
use smithay::backend::egl::context::ContextPriority;
use smithay::backend::egl::{EGLDevice, EGLDisplay};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::backend::renderer::{DebugFlags, ImportDma, ImportEgl};
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::{Event as SessionEvent, Session};
use smithay::backend::udev::{self, UdevBackend, UdevEvent};
use smithay::output::{Mode, Output, OutputModeSource, PhysicalProperties};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::calloop::{Dispatcher, LoopHandle};
use smithay::reexports::drm::control::atomic::AtomicModeReq;
use smithay::reexports::drm::control::dumbbuffer::DumbBuffer;
use smithay::reexports::drm::control::{
    connector, crtc, plane, property, AtomicCommitFlags, Device, ModeFlags, ModeTypeFlags, PlaneType,
};
use smithay::reexports::gbm::Modifier;
use smithay::reexports::input::Libinput;
use smithay::reexports::rustix::fs::OFlags;
use smithay::reexports::wayland_protocols;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::DeviceFd;
use smithay::wayland::dmabuf::DmabufFeedbackBuilder;
use smithay::wayland::drm_lease::DrmLeaseState;
use smithay::wayland::presentation::Refresh;
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner};
use wayland_protocols::wp::presentation_time::server::wp_presentation_feedback;

pub use devices::{DeviceManager, OutputDevice};
pub use helpers::{calculate_drm_mode_from_modeline, calculate_mode_cvt, set_gamma_for_crtc};
pub use types::{CrtcInfo, SurfaceDmabufFeedback, TtyFrame, TtyRenderer, TtyRendererError};
use devices::format_connector_name;
use helpers::{
    find_drm_property, get_drm_property, get_panel_orientation, ignored_nodes_from_config,
    is_vrr_capable, make_output_name, pick_mode, primary_node_from_config, refresh_interval,
    reset_hdr, surface_dmabuf_feedback,
};
use types::{ConnectorProperties, GammaProps, Surface, TtyOutputState, SUPPORTED_COLOR_FORMATS};

use super::{IpcOutputMap, RenderResult};
use crate::backend::OutputId;
use crate::frame_clock::FrameClock;
use crate::niri::{Niri, RedrawState, State};
use crate::render_helpers::debug::draw_damage;
use crate::render_helpers::renderer::AsGlesRenderer;
use crate::render_helpers::{resources, shaders, RenderTarget};
use crate::utils::{get_monotonic_time, is_laptop_panel, logical_output, PanelOrientation};

pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,
    // Device management subsystem - owns all DRM device state.
    pub(crate) devices: DeviceManager,
    // The output config had changed, but the session is paused, so we need to update it on resume.
    update_output_config_on_resume: bool,
    // The ignored nodes have changed, but the session is paused, so we need to update it on
    // resume.
    update_ignored_nodes_on_resume: bool,
    // Whether the debug tinting is enabled.
    debug_tint: bool,
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
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
            update_output_config_on_resume: false,
            update_ignored_nodes_on_resume: false,
            debug_tint: false,
            ipc_outputs: Arc::new(Mutex::new(HashMap::new())),
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

                if self.update_ignored_nodes_on_resume {
                    self.update_ignored_nodes_on_resume = false;
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

                if self.update_output_config_on_resume {
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
        debug!("adding device: {device_id} {path:?}");

        let node = DrmNode::from_dev_id(device_id)?;

        if node == self.devices.primary_node() {
            debug!("this is the primary node");
        }

        // Only consider primary node on udev event
        // https://gitlab.freedesktop.org/wlroots/wlroots/-/commit/768fbaad54027f8dd027e7e015e8eeb93cb38c52
        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return Ok(());
        }

        if self.devices.ignored_nodes().contains(&node) {
            debug!("node is ignored, skipping");
            return Ok(());
        }

        let _span = tracy_client::span!("Tty::device_added");

        let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = {
            let _span = tracy_client::span!("LibSeatSession::open");
            self.session.open(path, open_flags)
        }?;
        let device_fd = DrmDeviceFd::new(DeviceFd::from(fd));

        let (drm, drm_notifier) = {
            let _span = tracy_client::span!("DrmDevice::new");
            DrmDevice::new(device_fd.clone(), false)
        }?;
        let gbm = {
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
            self.devices.gpu_manager_mut()
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

        if render_node == Some(self.devices.primary_render_node()) && self.devices.dmabuf_global().is_none() {
            let render_node = self.devices.primary_render_node();
            debug!("initializing the primary renderer");

            let mut renderer = self
                .devices
                .gpu_manager_mut()
                .single_renderer(&render_node)
                .context("error creating renderer")?;

            if let Err(err) = renderer.bind_wl_display(&niri.display_handle) {
                warn!("error binding wl-display in EGL: {err:?}");
            }

            let gles_renderer = renderer.as_gles_renderer();
            resources::init(gles_renderer);
            shaders::init(gles_renderer);

            let config = self.config.borrow();
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
            assert!(self.devices.dmabuf_global().is_none());
            self.devices.set_dmabuf_global(Some(dmabuf_global));

            // Update the dmabuf feedbacks for all surfaces.
            let primary_render_node = self.devices.primary_render_node();
            for (node, device) in self.devices.iter_mut() {
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
        } else if let Some(primary_device) = self.devices.get(&self.devices.primary_node()) {
            primary_device.gbm.clone()
        } else {
            bail!("no allocator available for device");
        };
        let gbm_flags = GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT;
        let allocator = GbmAllocator::new(allocator_gbm, gbm_flags);

        let token = niri
            .event_loop
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
        assert!(self.devices.insert(node, device).is_none());

        self.device_changed(device_id, niri, true);

        Ok(())
    }

    fn device_changed(&mut self, device_id: dev_t, niri: &mut Niri, cleanup: bool) {
        debug!("device changed: {device_id}");

        let Ok(node) = DrmNode::from_dev_id(device_id) else {
            warn!("error creating DrmNode");
            return;
        };

        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return;
        }

        if self.devices.ignored_nodes().contains(&node) {
            debug!("node is ignored, skipping");
            return;
        }

        let Some(device) = self.devices.get_mut(&node) else {
            if let Some(path) = node.dev_path() {
                warn!("unknown device; trying to add");

                if let Err(err) = self.device_added(device_id, &path, niri) {
                    warn!("error adding device: {err:?}");
                }
            } else {
                warn!("unknown device");
            }

            return;
        };

        // DrmScanner will preserve any existing connector-CRTC mapping.
        let scan_result = match device.drm_scanner.scan_connectors(&device.drm) {
            Ok(x) => x,
            Err(err) => {
                warn!("error scanning connectors: {err:?}");
                return;
            }
        };

        let mut added = Vec::new();
        let mut removed = Vec::new();
        for event in scan_result {
            match event {
                DrmScanEvent::Connected {
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
                DrmScanEvent::Disconnected {
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

        let Some(device) = self.devices.get_mut(&node) else {
            error!("device disappeared");
            return;
        };

        for crtc in removed {
            if device.known_crtcs.remove(&crtc).is_none() {
                error!("output ID missing for disconnected crtc: {crtc:?}");
            }
        }

        for (crtc, mut info) in added {
            // Make/model/serial can match exactly between different physical monitors. This doesn't
            // happen often, but our Layout does not support such duplicates and will panic.
            //
            // As a workaround, search for duplicates, and unname the new connectors if one is
            // found. Connector names are always unique.
            let name = &mut info.name;
            let formatted = name.format_make_model_serial_or_connector();
            for info in self.devices.values().flat_map(|d| d.known_crtcs.values()) {
                if info.name.matches(&formatted) {
                    let connector = mem::take(&mut name.connector);
                    warn!(
                        "new connector {connector} duplicates make/model/serial \
                         of existing connector {}, unnaming",
                        info.name.connector,
                    );
                    *name = OutputName {
                        connector,
                        make: None,
                        model: None,
                        serial: None,
                    };
                    break;
                }
            }

            // Insert it right away so next added connector will check against this one too.
            let device = self.devices.get_mut(&node).unwrap();
            device.known_crtcs.insert(crtc, info);
        }

        // If the device was just added or resumed, we need to cleanup any disconnected connectors
        // and planes.
        if cleanup {
            let device = self.devices.get(&node).unwrap();

            // Follow the logic in on_output_config_changed().
            let disable_laptop_panels =
                self.should_disable_laptop_panels(niri.outputs.lid_closed());
            let should_disable = |conn: &str| disable_laptop_panels && is_laptop_panel(conn);

            let config = self.config.borrow();
            let disable_monitor_names = config.debug.disable_monitor_names;

            let should_be_off = |crtc, conn: &connector::Info| {
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

            let device = self.devices.get_mut(&node).unwrap();
            for surface in device.surfaces.values_mut() {
                // We aren't force-clearing the CRTCs, so we need to make the surfaces read the
                // updated state after a session resume. This also causes a full damage for the
                // next redraw.
                if let Err(err) = surface.compositor.reset_state() {
                    warn!("error resetting DrmCompositor state: {err:?}");
                }
            }
        }

        // This will connect any new connectors if needed, and apply other changes, such as
        // connecting back the internal laptop monitor once it becomes the only monitor left.
        //
        // It will also call refresh_ipc_outputs(), which will catch the disconnected connectors
        // above.
        self.on_output_config_changed(niri);
    }

    fn device_removed(&mut self, device_id: dev_t, niri: &mut Niri) {
        debug!("removing device: {device_id}");

        let Ok(node) = DrmNode::from_dev_id(device_id) else {
            warn!("error creating DrmNode");
            return;
        };

        if node.ty() != NodeType::Primary {
            debug!("not a primary node, skipping");
            return;
        }

        let Some(device) = self.devices.get_mut(&node) else {
            warn!("unknown device");
            return;
        };

        let crtcs: Vec<_> = device
            .drm_scanner
            .crtcs()
            .map(|(_info, crtc)| crtc)
            .collect();

        for crtc in crtcs {
            self.connector_disconnected(niri, node, crtc);
        }

        let mut device = self.devices.remove(&node).unwrap();
        let device_fd = device.drm.device_fd().device_fd();

        if let Some(lease_state) = &mut device.drm_lease_state {
            lease_state.disable_global::<State>();
        }

        if let Some(render_node) = device.render_node {
            // Sometimes (Asahi DisplayLink), multiple primary nodes will correspond to the same
            // render node. In this case, we want to keep the render node active until the last
            // primary node that uses it is gone.
            let was_last = !self
                .devices
                .values()
                .any(|device| device.render_node == Some(render_node));

            let primary_render_node = self.devices.primary_render_node();
            if was_last && render_node == primary_render_node {
                debug!("destroying the primary renderer");

                match self.devices.gpu_manager_mut().single_renderer(&primary_render_node) {
                    Ok(mut renderer) => renderer.unbind_wl_display(),
                    Err(err) => {
                        warn!("error creating renderer during device removal: {err}");
                    }
                }

                // Disable and destroy the dmabuf global.
                if let Some(global) = self.devices.take_dmabuf_global() {
                    niri.protocols
                        .dmabuf
                        .disable_global::<State>(&niri.display_handle, &global);
                    niri.event_loop
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
                    for device in self.devices.values_mut() {
                        for surface in device.surfaces.values_mut() {
                            surface.dmabuf_feedback = None;
                        }
                    }
                } else {
                    error!("dmabuf global was already missing");
                }
            }

            if was_last {
                self.devices.gpu_manager_mut().as_mut().remove_node(&render_node);
                // Trigger re-enumeration in order to remove the device from gpu_manager.
                let _ = self.devices.gpu_manager_mut().devices();
            }
        }

        niri.event_loop.remove(device.token);

        self.refresh_ipc_outputs(niri);

        let _ = device;

        match TryInto::<OwnedFd>::try_into(device_fd) {
            Ok(fd) => {
                if let Err(err) = self.session.close(fd) {
                    warn!("error closing DRM device fd: {err:?}");
                }
            }
            Err(_) => {
                error!("unable to close DRM device cleanly: fd has unexpected references");
            }
        }
    }

    fn connector_connected(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
    ) -> anyhow::Result<()> {
        let connector_name = format_connector_name(&connector);
        debug!("connecting connector: {connector_name}");

        // Extract values we need before getting mutable device reference.
        let primary_render_node = self.devices.primary_render_node();

        let device = self.devices.get_mut(&node).context("missing device")?;

        let disable_monitor_names = self.config.borrow().debug.disable_monitor_names;
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

        let config = self
            .config
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
            let renderer = self.devices.gpu_manager_mut().single_renderer(&render_node)?;
            let egl_context = renderer.as_ref().egl_context();
            egl_context.dmabuf_render_formats().clone()
        };

        // Re-borrow device.
        let device = self.devices.get_mut(&node).context("missing device")?;

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
            SUPPORTED_COLOR_FORMATS,
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
                    SUPPORTED_COLOR_FORMATS,
                    render_formats,
                    device.drm.cursor_size(),
                    Some(device.gbm.clone()),
                )
                .context("error creating DRM compositor")?
            }
        };

        if self.debug_tint {
            compositor.set_debug_flags(DebugFlags::TINT);
        }

        // Drop device to access gpu_manager_mut for dmabuf feedback.
        let _ = device;

        let mut dmabuf_feedback = None;
        if let Ok(primary_renderer) = self.devices.gpu_manager_mut().single_renderer(&primary_render_node) {
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
        let device = self.devices.get_mut(&node).context("missing device")?;
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

    fn connector_disconnected(&mut self, niri: &mut Niri, node: DrmNode, crtc: crtc::Handle) {
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

    fn on_vblank(
        &mut self,
        niri: &mut Niri,
        node: DrmNode,
        crtc: crtc::Handle,
        meta: DrmEventMetadata,
    ) {
        let span = tracy_client::span!("Tty::on_vblank");

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

        // Finish the Tracy frame, if any.
        drop(surface.vblank_frame.take());

        let name = &surface.name.connector;
        trace!("vblank on {name} {meta:?}");
        span.emit_text(name);

        let presentation_time = match meta.time {
            DrmEventTime::Monotonic(time) => time,
            DrmEventTime::Realtime(_) => {
                // Not supported.

                // This value will be ignored in the frame clock code.
                Duration::ZERO
            }
        };
        let presentation_time = if niri.config.borrow().debug.emulate_zero_presentation_time {
            Duration::ZERO
        } else {
            presentation_time
        };

        let message = if presentation_time.is_zero() {
            format!("vblank on {name}, presentation time unknown")
        } else if presentation_time > now {
            let diff = presentation_time - now;
            tracy_client::Client::running().unwrap().plot(
                surface.time_since_presentation_plot_name,
                -diff.as_secs_f64() * 1000.,
            );
            format!("vblank on {name}, presentation is {diff:?} later")
        } else {
            let diff = now - presentation_time;
            tracy_client::Client::running().unwrap().plot(
                surface.time_since_presentation_plot_name,
                diff.as_secs_f64() * 1000.,
            );
            format!("vblank on {name}, presentation was {diff:?} ago")
        };
        tracy_client::Client::running()
            .unwrap()
            .message(&message, 0);

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

        let time = if presentation_time.is_zero() {
            now
        } else {
            presentation_time
        };

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

        let redraw_needed = match mem::replace(&mut output_state.redraw_state, RedrawState::Idle) {
            RedrawState::WaitingForVBlank { redraw_needed } => redraw_needed,
            state @ (RedrawState::Idle
            | RedrawState::Queued
            | RedrawState::WaitingForEstimatedVBlank(_)
            | RedrawState::WaitingForEstimatedVBlankAndQueued(_)) => {
                // This is an error!() because it shouldn't happen, but on some systems it somehow
                // does. Kernel sending rogue vblank events?
                //
                // https://github.com/YaLTeR/niri/issues/556
                // https://github.com/YaLTeR/niri/issues/615
                error!(
                    "unexpected redraw state for output {name} (should be WaitingForVBlank); \
                     can happen when resuming from sleep or powering on monitors: {state:?}"
                );
                true
            }
        };

        // Mark the last frame as submitted.
        match surface.compositor.frame_submitted() {
            Ok(Some((mut feedback, target_presentation_time))) => {
                let refresh = match refresh_interval {
                    Some(refresh) => {
                        if output_state.frame_clock.vrr() {
                            Refresh::Variable(refresh)
                        } else {
                            Refresh::Fixed(refresh)
                        }
                    }
                    None => Refresh::Unknown,
                };

                // FIXME: ideally should be monotonically increasing for a surface.
                let seq = meta.sequence as u64;
                let mut flags = wp_presentation_feedback::Kind::Vsync
                    | wp_presentation_feedback::Kind::HwCompletion;

                if !presentation_time.is_zero() {
                    flags.insert(wp_presentation_feedback::Kind::HwClock);
                }

                feedback.presented::<_, smithay::utils::Monotonic>(time, refresh, seq, flags);

                if !presentation_time.is_zero() {
                    let misprediction_s =
                        presentation_time.as_secs_f64() - target_presentation_time.as_secs_f64();
                    tracy_client::Client::running().unwrap().plot(
                        surface.presentation_misprediction_plot_name,
                        misprediction_s * 1000.,
                    );
                }
            }
            Ok(None) => (),
            Err(err) => {
                warn!("error marking frame as submitted: {err}");
            }
        }

        if let Some(last_sequence) = output_state.last_drm_sequence {
            let delta = meta.sequence as f64 - last_sequence as f64;
            tracy_client::Client::running()
                .unwrap()
                .plot(surface.sequence_delta_plot_name, delta);
        }
        output_state.last_drm_sequence = Some(meta.sequence);

        output_state.frame_clock.presented(presentation_time);

        if redraw_needed || output_state.unfinished_animations_remain {
            let vblank_frame = tracy_client::Client::running()
                .unwrap()
                .non_continuous_frame(surface.vblank_frame_name);
            surface.vblank_frame = Some(vblank_frame);

            niri.queue_redraw(&output);
        } else {
            niri.send_frame_callbacks(&output);
        }
    }

    fn on_estimated_vblank_timer(&self, niri: &mut Niri, output: Output) {
        let span = tracy_client::span!("Tty::on_estimated_vblank_timer");

        let name = output.name();
        span.emit_text(&name);

        let Some(output_state) = niri.outputs.state_mut(&output) else {
            error!("missing output state for {name}");
            return;
        };

        // We waited for the timer, now we can send frame callbacks again.
        output_state.frame_callback_sequence = output_state.frame_callback_sequence.wrapping_add(1);

        match mem::replace(&mut output_state.redraw_state, RedrawState::Idle) {
            RedrawState::Idle => unreachable!(),
            RedrawState::Queued => unreachable!(),
            RedrawState::WaitingForVBlank { .. } => unreachable!(),
            RedrawState::WaitingForEstimatedVBlank(_) => (),
            // The timer fired just in front of a redraw.
            RedrawState::WaitingForEstimatedVBlankAndQueued(_) => {
                output_state.redraw_state = RedrawState::Queued;
                return;
            }
        }

        if output_state.unfinished_animations_remain {
            niri.queue_redraw(&output);
        } else {
            niri.send_frame_callbacks(&output);
        }
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
        let span = tracy_client::span!("Tty::render");

        let mut rv = RenderResult::Skipped;

        let tty_state: &TtyOutputState = output.user_data().get().unwrap();

        // Extract values we need before getting mutable device reference.
        let primary_render_node = self.devices.primary_render_node();

        let Some(device) = self.devices.get_mut(&tty_state.node) else {
            error!("missing output device");
            return rv;
        };

        let Some(surface) = device.surfaces.get_mut(&tty_state.crtc) else {
            error!("missing surface");
            return rv;
        };

        span.emit_text(&surface.name.connector);

        if !device.drm.is_active() {
            // This branch hits any time we try to render while the user had switched to a
            // different VT, so don't print anything here.
            return rv;
        }

        // Extract values before getting renderer (which borrows gpu_manager).
        let device_render_node = device.render_node;
        let surface_format = surface.compositor.format();

        let mut renderer = match self.devices.gpu_manager.renderer(
            &primary_render_node,
            &device_render_node.unwrap_or(primary_render_node),
            surface_format,
        ) {
            Ok(renderer) => renderer,
            Err(err) => {
                warn!("error creating renderer for primary GPU: {err:?}");
                return rv;
            }
        };

        // Access surface directly through devices.devices to avoid borrow conflict with gpu_manager.
        let device = self.devices.devices.get_mut(&tty_state.node).unwrap();
        let surface = device.surfaces.get_mut(&tty_state.crtc).unwrap();

        // Render the elements.
        let mut elements =
            niri.render::<TtyRenderer>(&mut renderer, output, true, RenderTarget::Output);

        // Visualize the damage, if enabled.
        if niri.debug_draw_damage {
            let output_state = niri.outputs.state_mut(output).unwrap();
            draw_damage(&mut output_state.debug_damage_tracker, &mut elements);
        }

        // Overlay planes are disabled by default as they cause weird performance issues on my
        // system.
        let flags = {
            let debug = &self.config.borrow().debug;

            let primary_scanout_flag = if debug.restrict_primary_scanout_to_matching_format {
                FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT
            } else {
                FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT_ANY
            };
            let mut flags = primary_scanout_flag | FrameFlags::ALLOW_CURSOR_PLANE_SCANOUT;

            if debug.enable_overlay_planes {
                flags.insert(FrameFlags::ALLOW_OVERLAY_PLANE_SCANOUT);
            }
            if debug.disable_direct_scanout {
                flags.remove(primary_scanout_flag);
                flags.remove(FrameFlags::ALLOW_OVERLAY_PLANE_SCANOUT);
            }
            if debug.disable_cursor_plane {
                flags.remove(FrameFlags::ALLOW_CURSOR_PLANE_SCANOUT);
            }
            if debug.skip_cursor_only_updates_during_vrr {
                let output_state = niri.outputs.state(output).unwrap();
                if output_state.frame_clock.vrr() {
                    flags.insert(FrameFlags::SKIP_CURSOR_ONLY_UPDATES);
                }
            }

            flags
        };

        // Hand them over to the DRM.
        let drm_compositor = &mut surface.compositor;
        match drm_compositor.render_frame::<_, _>(&mut renderer, &elements, [0.; 4], flags) {
            Ok(res) => {
                let needs_sync = res.needs_sync()
                    || self
                        .config
                        .borrow()
                        .debug
                        .wait_for_frame_completion_before_queueing;
                if needs_sync {
                    if let PrimaryPlaneElement::Swapchain(element) = res.primary_element {
                        let _span = tracy_client::span!("wait for completion");
                        if let Err(err) = element.sync.wait() {
                            warn!("error waiting for frame completion: {err:?}");
                        }
                    }
                }

                niri.update_primary_scanout_output(output, &res.states);
                if let Some(dmabuf_feedback) = surface.dmabuf_feedback.as_ref() {
                    niri.send_dmabuf_feedbacks(output, dmabuf_feedback, &res.states);
                }

                if !res.is_empty {
                    let presentation_feedbacks =
                        niri.take_presentation_feedbacks(output, &res.states);
                    let data = (presentation_feedbacks, target_presentation_time);

                    match drm_compositor.queue_frame(data) {
                        Ok(()) => {
                            let output_state = niri.outputs.state_mut(output).unwrap();
                            let new_state = RedrawState::WaitingForVBlank {
                                redraw_needed: false,
                            };
                            match mem::replace(&mut output_state.redraw_state, new_state) {
                                RedrawState::Idle => unreachable!(),
                                RedrawState::Queued => (),
                                RedrawState::WaitingForVBlank { .. } => unreachable!(),
                                RedrawState::WaitingForEstimatedVBlank(_) => unreachable!(),
                                RedrawState::WaitingForEstimatedVBlankAndQueued(token) => {
                                    niri.event_loop.remove(token);
                                }
                            };

                            // We queued this frame successfully, so the current client buffers were
                            // latched. We can send frame callbacks now, since a new client commit
                            // will no longer overwrite this frame and will wait for a VBlank.
                            output_state.frame_callback_sequence =
                                output_state.frame_callback_sequence.wrapping_add(1);

                            return RenderResult::Submitted;
                        }
                        Err(err) => {
                            warn!("error queueing frame: {err}");
                        }
                    }
                } else {
                    rv = RenderResult::NoDamage;
                }
            }
            Err(err) => {
                // Can fail if we switched to a different TTY.
                warn!("error rendering frame: {err}");
            }
        }

        // We're not expecting a vblank right after this.
        drop(surface.vblank_frame.take());

        // Queue a timer to fire at the predicted vblank time.
        queue_estimated_vblank_timer(niri, output.clone(), target_presentation_time);

        rv
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
        self.debug_tint = !self.debug_tint;

        for device in self.devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                let compositor = &mut surface.compositor;

                let mut flags = compositor.debug_flags();
                flags.set(DebugFlags::TINT, self.debug_tint);
                compositor.set_debug_flags(flags);
            }
        }
    }

    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        let primary_render_node = self.devices.primary_render_node;
        let mut renderer = match self.devices.gpu_manager.single_renderer(&primary_render_node) {
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
        let primary_render_node = self.devices.primary_render_node;
        if let Err(err) = self.devices.gpu_manager.early_import(
            // We always render on the primary GPU.
            primary_render_node,
            surface,
        ) {
            warn!("error doing early import: {err:?}");
        }
    }

    pub fn get_gamma_size(&self, output: &Output) -> anyhow::Result<u32> {
        let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
        let crtc = tty_state.crtc;

        let device = self
            .devices
            .get(&tty_state.node)
            .context("missing device")?;

        let surface = device.surfaces.get(&crtc).context("missing surface")?;
        if let Some(gamma_props) = &surface.gamma_props {
            gamma_props.gamma_size(&device.drm)
        } else {
            let info = device
                .drm
                .get_crtc(crtc)
                .context("error getting crtc info")?;
            Ok(info.gamma_length())
        }
    }

    pub fn set_gamma(&mut self, output: &Output, ramp: Option<Vec<u16>>) -> anyhow::Result<()> {
        let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
        let crtc = tty_state.crtc;

        let device = self
            .devices
            .get_mut(&tty_state.node)
            .context("missing device")?;
        let surface = device.surfaces.get_mut(&crtc).context("missing surface")?;

        // Cannot change properties while the device is inactive.
        if !self.session.is_active() {
            surface.pending_gamma_change = Some(ramp);
            return Ok(());
        }

        let ramp = ramp.as_deref();
        if let Some(gamma_props) = &mut surface.gamma_props {
            gamma_props.set_gamma(&device.drm, ramp)
        } else {
            set_gamma_for_crtc(&device.drm, crtc, ramp)
        }
    }

    fn refresh_ipc_outputs(&self, niri: &mut Niri) {
        let _span = tracy_client::span!("Tty::refresh_ipc_outputs");

        let mut ipc_outputs = HashMap::new();
        let disable_monitor_names = self.config.borrow().debug.disable_monitor_names;

        for (node, device) in &self.devices {
            for (connector, crtc) in device.drm_scanner.crtcs() {
                let connector_name = format_connector_name(connector);
                let physical_size = connector.size();
                let output_name = device.known_crtc_name(&crtc, connector, disable_monitor_names);

                let surface = device.surfaces.get(&crtc);
                let current_crtc_mode = surface.map(|surface| surface.compositor.pending_mode());
                let mut current_mode = None;
                let mut is_custom_mode = false;

                let mut modes: Vec<niri_ipc::Mode> = connector
                    .modes()
                    .iter()
                    .filter(|m| !m.flags().contains(ModeFlags::INTERLACE))
                    .enumerate()
                    .map(|(idx, m)| {
                        if Some(*m) == current_crtc_mode {
                            current_mode = Some(idx);
                        }

                        niri_ipc::Mode {
                            width: m.size().0,
                            height: m.size().1,
                            refresh_rate: Mode::from(*m).refresh as u32,
                            is_preferred: m.mode_type().contains(ModeTypeFlags::PREFERRED),
                        }
                    })
                    .collect();

                if let Some(crtc_mode) = current_crtc_mode {
                    // Custom mode
                    if crtc_mode.mode_type().contains(ModeTypeFlags::USERDEF) {
                        modes.insert(
                            0,
                            niri_ipc::Mode {
                                width: crtc_mode.size().0,
                                height: crtc_mode.size().1,
                                refresh_rate: Mode::from(crtc_mode).refresh as u32,
                                is_preferred: false,
                            },
                        );
                        current_mode = Some(0);
                        is_custom_mode = true;
                    }

                    if current_mode.is_none() {
                        if crtc_mode.flags().contains(ModeFlags::INTERLACE) {
                            warn!("connector mode list missing current mode (interlaced)");
                        } else {
                            error!("connector mode list missing current mode");
                        }
                    }
                }

                let vrr_supported = surface
                    .map(|surface| {
                        matches!(
                            surface.compositor.vrr_supported(connector.handle()),
                            Ok(VrrSupport::Supported | VrrSupport::RequiresModeset)
                        )
                    })
                    .unwrap_or_else(|| {
                        is_vrr_capable(&device.drm, connector.handle()) == Some(true)
                    });
                let vrr_enabled = surface.is_some_and(|surface| surface.compositor.vrr_enabled());

                let logical = niri
                    .outputs
                    .space()
                    .outputs()
                    .find(|output| {
                        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
                        tty_state.node == *node && tty_state.crtc == crtc
                    })
                    .map(logical_output);

                let id = device.known_crtcs.get(&crtc).map(|info| info.id);
                let id = id.unwrap_or_else(|| {
                    error!("crtc for connector {connector_name} missing from known");
                    OutputId::next()
                });

                let ipc_output = niri_ipc::Output {
                    name: connector_name,
                    make: output_name.make.unwrap_or_else(|| "Unknown".into()),
                    model: output_name.model.unwrap_or_else(|| "Unknown".into()),
                    serial: output_name.serial,
                    physical_size,
                    modes,
                    current_mode,
                    is_custom_mode,
                    vrr_supported,
                    vrr_enabled,
                    logical,
                };

                ipc_outputs.insert(id, ipc_output);
            }
        }

        let mut guard = self.ipc_outputs.lock().unwrap();
        *guard = ipc_outputs;
        niri.ipc_outputs_changed = true;
    }

    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        self.ipc_outputs.clone()
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
        // We only disable the CRTC here, this will also reset the
        // surface state so that the next call to `render_frame` will
        // always produce a new frame and `queue_frame` will change
        // the CRTC to active. This makes sure we always enable a CRTC
        // within an atomic operation.
        if active {
            return;
        }

        for device in self.devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                if let Err(err) = surface.compositor.clear() {
                    warn!("error clearing drm surface: {err:?}");
                }
            }
        }
    }

    pub fn set_output_on_demand_vrr(&mut self, niri: &mut Niri, output: &Output, enable_vrr: bool) {
        let _span = tracy_client::span!("Tty::set_output_on_demand_vrr");

        let output_state = niri.outputs.state_mut(output).unwrap();
        output_state.on_demand_vrr_enabled = enable_vrr;
        if output_state.frame_clock.vrr() == enable_vrr {
            return;
        }
        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
        let target_node = tty_state.node;
        let target_crtc = tty_state.crtc;

        let mut found = false;
        if let Some(device) = self.devices.devices.get_mut(&target_node) {
            if let Some(surface) = device.surfaces.get_mut(&target_crtc) {
                let word = if enable_vrr { "enabling" } else { "disabling" };
                if let Err(err) = surface.compositor.use_vrr(enable_vrr) {
                    warn!(
                        "output {:?}: error {} VRR: {err:?}",
                        surface.name.connector, word
                    );
                }
                output_state
                    .frame_clock
                    .set_vrr(surface.compositor.vrr_enabled());
                found = true;
            }
        }

        if found {
            self.refresh_ipc_outputs(niri);
        }
    }

    pub fn update_ignored_nodes_config(&mut self, niri: &mut Niri) {
        let _span = tracy_client::span!("Tty::update_ignored_nodes_config");

        // If we're inactive, we can't do anything, so just set a flag for later.
        if !self.session.is_active() {
            self.update_ignored_nodes_on_resume = true;
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
        let _span = tracy_client::span!("Tty::on_output_config_changed");

        // If we're inactive, we can't do anything, so just set a flag for later.
        if !self.session.is_active() {
            self.update_output_config_on_resume = true;
            return;
        }
        self.update_output_config_on_resume = false;

        // Figure out if we should disable laptop panels.
        let disable_laptop_panels = self.should_disable_laptop_panels(niri.outputs.lid_closed());
        let should_disable = |connector: &str| disable_laptop_panels && is_laptop_panel(connector);

        let mut to_disconnect = vec![];
        let mut to_connect = vec![];

        for (&node, device) in &mut self.devices {
            for (&crtc, surface) in device.surfaces.iter_mut() {
                let config = self
                    .config
                    .borrow()
                    .outputs
                    .find(&surface.name)
                    .cloned()
                    .unwrap_or_default();
                if config.off || should_disable(&surface.name.connector) {
                    to_disconnect.push((node, crtc));
                    continue;
                }

                // Check if we need to change the mode.
                let Some(connector) = device.drm_scanner.connectors().get(&surface.connector)
                else {
                    error!("missing enabled connector in drm_scanner");
                    continue;
                };

                let mut mode = None;
                if let Some(modeline) = &config.modeline {
                    match calculate_drm_mode_from_modeline(modeline) {
                        Ok(x) => mode = Some(x),
                        Err(err) => {
                            warn!(
                                "output {:?}: invalid custom modeline; \
                                 falling back to advertised modes: {err:?}",
                                surface.name.connector
                            );
                        }
                    }
                }

                let (mode, fallback) = match mode {
                    Some(x) => (x, false),
                    None => match pick_mode(connector, config.mode) {
                        Some(result) => result,
                        None => {
                            warn!("couldn't pick mode for enabled connector");
                            continue;
                        }
                    },
                };

                let change_mode = surface.compositor.pending_mode() != mode;

                let vrr_enabled = surface.compositor.vrr_enabled();
                let change_always_vrr = vrr_enabled != config.is_vrr_always_on();
                let is_on_demand_vrr = config.is_vrr_on_demand();

                if !change_mode && !change_always_vrr && !is_on_demand_vrr {
                    continue;
                }

                let output = niri
                    .outputs
                    .space()
                    .outputs()
                    .find(|output| {
                        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
                        tty_state.node == node && tty_state.crtc == crtc
                    })
                    .cloned();
                let Some(output) = output else {
                    error!("missing output for crtc: {crtc:?}");
                    continue;
                };
                let Some(output_state) = niri.outputs.state_mut(&output) else {
                    error!("missing state for output {:?}", surface.name.connector);
                    continue;
                };

                if (is_on_demand_vrr && vrr_enabled != output_state.on_demand_vrr_enabled)
                    || (!is_on_demand_vrr && change_always_vrr)
                {
                    let vrr = !vrr_enabled;
                    let word = if vrr { "enabling" } else { "disabling" };
                    if let Err(err) = surface.compositor.use_vrr(vrr) {
                        warn!(
                            "output {:?}: error {} VRR: {err:?}",
                            surface.name.connector, word
                        );
                    }
                    output_state
                        .frame_clock
                        .set_vrr(surface.compositor.vrr_enabled());
                }

                if change_mode {
                    if fallback {
                        let target = config.mode.unwrap();
                        warn!(
                            "output {:?}: configured mode {}x{}{} could not be found, \
                             falling back to preferred",
                            surface.name.connector,
                            target.mode.width,
                            target.mode.height,
                            if let Some(refresh) = target.mode.refresh {
                                format!("@{refresh}")
                            } else {
                                String::new()
                            },
                        );
                    }

                    debug!(
                        "output {:?}: picking mode: {mode:?}",
                        surface.name.connector
                    );
                    if let Err(err) = surface.compositor.use_mode(mode) {
                        warn!("error changing mode: {err:?}");
                        continue;
                    }

                    let wl_mode = Mode::from(mode);
                    output.change_current_state(Some(wl_mode), None, None, None);
                    output.set_preferred(wl_mode);
                    output_state.frame_clock = FrameClock::new(
                        Some(refresh_interval(mode)),
                        surface.compositor.vrr_enabled(),
                    );
                    niri.output_resized(&output);
                }
            }

            let config = self.config.borrow();
            let disable_monitor_names = config.debug.disable_monitor_names;

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

                let config = config
                    .outputs
                    .find(&output_name)
                    .cloned()
                    .unwrap_or_default();

                if !(config.off || should_disable(&output_name.connector)) {
                    to_connect.push((node, connector.clone(), crtc, output_name));
                }
            }
        }

        for (node, crtc) in to_disconnect {
            self.connector_disconnected(niri, node, crtc);
        }

        // Sort by output name to get more predictable first focused output at initial compositor
        // startup, when multiple connectors appear at once.
        to_connect.sort_unstable_by(|a, b| a.3.compare(&b.3));

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

fn queue_estimated_vblank_timer(
    niri: &mut Niri,
    output: Output,
    target_presentation_time: Duration,
) {
    let output_state = niri.outputs.state_mut(&output).unwrap();
    match mem::take(&mut output_state.redraw_state) {
        RedrawState::Idle => unreachable!(),
        RedrawState::Queued => (),
        RedrawState::WaitingForVBlank { .. } => unreachable!(),
        RedrawState::WaitingForEstimatedVBlank(token)
        | RedrawState::WaitingForEstimatedVBlankAndQueued(token) => {
            output_state.redraw_state = RedrawState::WaitingForEstimatedVBlank(token);
            return;
        }
    }

    let now = get_monotonic_time();
    let mut duration = target_presentation_time.saturating_sub(now);

    // No use setting a zero timer, since we'll send frame callbacks anyway right after the call to
    // render(). This can happen for example with unknown presentation time from DRM.
    if duration.is_zero() {
        duration += output_state
            .frame_clock
            .refresh_interval()
            // Unknown refresh interval, i.e. winit backend. Would be good to estimate it somehow
            // but it's not that important for this code path.
            .unwrap_or(Duration::from_micros(16_667));
    }

    trace!("queueing estimated vblank timer to fire in {duration:?}");

    let timer = Timer::from_duration(duration);
    let token = niri
        .event_loop
        .insert_source(timer, move |_, _, data| {
            data.backend
                .tty()
                .on_estimated_vblank_timer(&mut data.niri, output.clone());
            TimeoutAction::Drop
        })
        .unwrap();
    output_state.redraw_state = RedrawState::WaitingForEstimatedVBlank(token);
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
