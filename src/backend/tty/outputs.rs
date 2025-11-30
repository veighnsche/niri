//! Output management subsystem for TTY backend.
//!
//! Handles IPC output reporting and resume state flags.
//! The actual output configuration logic remains in mod.rs due to
//! tight coupling with device management and niri state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use niri_config::Config;
use smithay::backend::drm::{DrmNode, VrrSupport};
use smithay::reexports::drm::control::{ModeFlags, ModeTypeFlags};
use smithay::output::Mode;
use tracing::{debug, error, warn};
#[cfg(feature = "profile-with-tracy")]
use tracy_client;

use super::devices::{format_connector_name, DeviceManager};
use super::helpers::{is_vrr_capable, set_gamma_for_crtc, calculate_drm_mode_from_modeline, pick_mode, refresh_interval};
use super::types::TtyOutputState;
use crate::backend::{IpcOutputMap, OutputId};
use crate::niri::Niri;
use crate::frame_clock::FrameClock;
use crate::utils::{logical_output, is_laptop_panel};

// === Output Configuration ===

use smithay::reexports::drm::control::connector;

/// Result of output configuration change processing.
pub struct OutputConfigChangedResult {
    pub to_disconnect: Vec<(DrmNode, crtc::Handle)>,
    pub to_connect: Vec<(DrmNode, connector::Info, crtc::Handle, niri_config::OutputName)>,
}

impl Default for OutputConfigChangedResult {
    fn default() -> Self {
        Self {
            to_disconnect: Vec::new(),
            to_connect: Vec::new(),
        }
    }
}

// === Gamma Control ===

use anyhow::Context;
use smithay::output::Output;
use smithay::reexports::drm::control::{crtc, Device};

/// Output management subsystem.
///
/// OWNS:
/// - IPC output map for external queries
/// - Resume update flags
pub struct OutputManager {
    /// IPC output map shared with external queries.
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
    /// The output config had changed, but the session is paused.
    update_config_on_resume: bool,
    /// The ignored nodes have changed, but the session is paused.
    update_ignored_nodes_on_resume: bool,
}

impl OutputManager {
    /// Create a new OutputManager.
    pub fn new() -> Self {
        Self {
            ipc_outputs: Arc::new(Mutex::new(HashMap::new())),
            update_config_on_resume: false,
            update_ignored_nodes_on_resume: false,
        }
    }

    // === IPC ===

    /// Get a clone of the IPC outputs map.
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        Arc::clone(&self.ipc_outputs)
    }

    /// Update the IPC outputs map with new data.
    pub fn set_ipc_outputs(&self, outputs: IpcOutputMap) {
        let mut guard = self.ipc_outputs.lock().unwrap();
        *guard = outputs;
    }

    /// Refresh the IPC outputs map with current device and output state.
    ///
    /// This method iterates over all devices and connectors to build
    /// the IPC output information that external clients can query.
    pub fn refresh_ipc_outputs(
        &self,
        devices: &DeviceManager,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
    ) {
        let _span = tracy_client::span!("OutputManager::refresh_ipc_outputs");

        let mut ipc_outputs = HashMap::new();
        let disable_monitor_names = config.borrow().debug.disable_monitor_names;

        for (node, device) in devices {
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

        self.set_ipc_outputs(ipc_outputs);
        niri.ipc_outputs_changed = true;
    }

    // === Gamma Control ===

    /// Get the gamma size for the given output.
    ///
    /// Returns the size of the gamma ramp that can be set for this output.
    pub fn get_gamma_size(
        &self,
        devices: &DeviceManager,
        output: &Output,
    ) -> anyhow::Result<u32> {
        let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
        let crtc = tty_state.crtc;

        let device = devices
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

    /// Set the gamma ramp for the given output.
    ///
    /// Supports both atomic GAMMA_LUT and legacy gamma. Defers gamma changes
    /// when the session is inactive.
    pub fn set_gamma(
        &self,
        devices: &mut DeviceManager,
        output: &Output,
        ramp: Option<Vec<u16>>,
        session_active: bool,
    ) -> anyhow::Result<()> {
        let tty_state = output.user_data().get::<TtyOutputState>().unwrap();
        let crtc = tty_state.crtc;

        let device = devices
            .get_mut(&tty_state.node)
            .context("missing device")?;
        let surface = device.surfaces.get_mut(&crtc).context("missing surface")?;

        // Cannot change properties while the device is inactive.
        if !session_active {
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

    // === Monitor Control ===

    /// Set monitors active/inactive state.
    ///
    /// Controls DPMS (monitor power state). Only handles deactivation by clearing
    /// surfaces; activation happens automatically on next render.
    pub fn set_monitors_active(&self, devices: &mut DeviceManager, active: bool) {
        // We only disable the CRTC here, this will also reset the
        // surface state so that the next call to `render_frame` will
        // always produce a new frame and `queue_frame` will change
        // the CRTC to active. This makes sure we always enable a CRTC
        // within an atomic operation.
        if active {
            return;
        }

        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                if let Err(err) = surface.compositor.clear() {
                    warn!("error clearing drm surface: {err:?}");
                }
            }
        }
    }

    // === VRR Control ===

    /// Set on-demand Variable Refresh Rate for the given output.
    ///
    /// Enables/disables VRR based on content. Updates frame clock VRR state
    /// and returns whether IPC refresh should be triggered.
    pub fn set_output_on_demand_vrr(
        &self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        output: &Output,
        enable_vrr: bool,
    ) -> bool {
        let _span = tracy_client::span!("OutputManager::set_output_on_demand_vrr");

        let output_state = niri.outputs.state_mut(output).unwrap();
        output_state.on_demand_vrr_enabled = enable_vrr;
        if output_state.frame_clock.vrr() == enable_vrr {
            return false;
        }
        
        let tty_state: &TtyOutputState = output.user_data().get().unwrap();
        let target_node = tty_state.node;
        let target_crtc = tty_state.crtc;

        let mut found = false;
        if let Some(device) = devices.get_mut(&target_node) {
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

        found
    }

    // === Output Configuration ===

    /// Handle output configuration changes.
    ///
    /// This is the main output management method that handles:
    /// - Mode changes
    /// - VRR changes  
    /// - Enabling/disabling outputs via config
    /// - Laptop panel disable on lid close
    /// 
    /// Returns actions to be performed by the caller instead of executing them directly.
    pub fn on_output_config_changed(
        &mut self,
        devices: &mut DeviceManager,
        niri: &mut Niri,
        config: &Rc<RefCell<Config>>,
        session_active: bool,
        should_disable_laptop_panels: bool,
    ) -> OutputConfigChangedResult {
        let _span = tracy_client::span!("OutputManager::on_output_config_changed");

        if !session_active {
            self.mark_config_update_on_resume();
            return OutputConfigChangedResult::default();
        }
        self.clear_config_update_on_resume();

        let should_disable = |connector: &str| {
            should_disable_laptop_panels && is_laptop_panel(connector)
        };

        let mut to_disconnect = vec![];
        let mut to_connect = vec![];

        for (&node, device) in devices.iter_mut() {
            for (&crtc, surface) in device.surfaces.iter_mut() {
                let config = config
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

            let config = config.borrow();
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

        OutputConfigChangedResult { to_disconnect, to_connect }
    }

    // === Resume Flags ===

    /// Mark that output config needs to be updated on resume.
    pub fn mark_config_update_on_resume(&mut self) {
        self.update_config_on_resume = true;
    }

    /// Mark that ignored nodes need to be updated on resume.
    pub fn mark_ignored_nodes_update_on_resume(&mut self) {
        self.update_ignored_nodes_on_resume = true;
    }

    /// Check if output config needs to be updated on resume.
    pub fn needs_config_update_on_resume(&self) -> bool {
        self.update_config_on_resume
    }

    /// Check if ignored nodes need to be updated on resume.
    pub fn needs_ignored_nodes_update_on_resume(&self) -> bool {
        self.update_ignored_nodes_on_resume
    }

    /// Clear the config update on resume flag.
    pub fn clear_config_update_on_resume(&mut self) {
        self.update_config_on_resume = false;
    }

    /// Clear the ignored nodes update on resume flag.
    pub fn clear_ignored_nodes_update_on_resume(&mut self) {
        self.update_ignored_nodes_on_resume = false;
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}
