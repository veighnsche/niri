//! Pure helper functions for TTY backend.
//!
//! This module contains stateless helper functions that don't require `&self` or `&mut self`.
//! They are pure computations that take inputs and return outputs without side effects.
//!
//! This follows the pattern established in `src/input/helpers.rs`.

use std::collections::HashSet;
use std::io;
use std::iter::zip;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, ensure, Context};
use niri_config::output::Modeline;
use niri_config::{Config, OutputName};
use niri_ipc::{HSyncPolarity, VSyncPolarity};
use smithay::backend::allocator::format::FormatSet;
use smithay::backend::drm::{DrmDevice, DrmNode, NodeType};
use smithay::output::Mode;
use drm_ffi::drm_mode_modeinfo;
use smithay::reexports::drm::control::{self, connector, crtc, property, Device, ModeFlags, ModeTypeFlags, ResourceHandle};
use smithay::reexports::gbm::Modifier;
use smithay::reexports::wayland_protocols;
use smithay::utils::Transform;
use smithay::wayland::dmabuf::DmabufFeedbackBuilder;
use tracing::{debug, trace, warn};
use wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1::TrancheFlags;

use super::types::{ConnectorProperties, GbmDrmCompositor, SurfaceDmabufFeedback};

// =============================================================================
// DRM Node Discovery
// =============================================================================

/// Get primary and render nodes from a render node path.
pub(super) fn primary_node_from_render_node(path: &Path) -> Option<(DrmNode, DrmNode)> {
    match DrmNode::from_path(path) {
        Ok(node) => {
            if node.ty() == NodeType::Render {
                match node.node_with_type(NodeType::Primary) {
                    Some(Ok(primary_node)) => {
                        return Some((primary_node, node));
                    }
                    Some(Err(err)) => {
                        warn!("error opening primary node for render node {path:?}: {err:?}");
                    }
                    None => {
                        warn!("error opening primary node for render node {path:?}");
                    }
                }
            } else {
                warn!("DRM node {path:?} is not a render node");

                // Gracefully handle misconfiguration on regular desktop systems.
                if let Some(Ok(render_node)) = node.node_with_type(NodeType::Render) {
                    return Some((node, render_node));
                }

                warn!("could not get render node for DRM node {path:?}; proceeding anyway");
                return Some((node, node));
            }
        }
        Err(err) => {
            warn!("error opening {path:?} as DRM node: {err:?}");
        }
    }

    None
}

/// Get primary and render nodes from config.
pub(super) fn primary_node_from_config(config: &Config) -> Option<(DrmNode, DrmNode)> {
    let path = config.debug.render_drm_device.as_ref()?;
    debug!("attempting to use render node from config: {path:?}");

    primary_node_from_render_node(path)
}

/// Get ignored DRM nodes from config.
pub(super) fn ignored_nodes_from_config(config: &Config) -> HashSet<DrmNode> {
    let mut disabled_nodes = HashSet::new();

    for path in &config.debug.ignored_drm_devices {
        if let Some((primary_node, render_node)) = primary_node_from_render_node(path) {
            disabled_nodes.insert(primary_node);
            disabled_nodes.insert(render_node);
        }
    }

    disabled_nodes
}

// =============================================================================
// DRM Property Helpers
// =============================================================================

/// Find a DRM property by name.
pub(super) fn find_drm_property(
    drm: &DrmDevice,
    resource: impl ResourceHandle,
    name: &str,
) -> Option<(property::Handle, property::Info, property::RawValue)> {
    let props = match drm.get_properties(resource) {
        Ok(props) => props,
        Err(err) => {
            warn!("error getting properties: {err:?}");
            return None;
        }
    };

    props.into_iter().find_map(|(handle, value)| {
        let info = drm.get_property(handle).ok()?;
        let n = info.name().to_str().ok()?;

        (n == name).then_some((handle, info, value))
    })
}

/// Get a DRM property value by handle.
pub(super) fn get_drm_property(
    drm: &DrmDevice,
    resource: impl ResourceHandle,
    prop: property::Handle,
) -> Option<property::RawValue> {
    let props = match drm.get_properties(resource) {
        Ok(props) => props,
        Err(err) => {
            warn!("error getting properties: {err:?}");
            return None;
        }
    };

    props
        .into_iter()
        .find_map(|(handle, value)| (handle == prop).then_some(value))
}

// =============================================================================
// Mode Calculations
// =============================================================================

/// Calculate the refresh interval from a DRM mode.
pub(super) fn refresh_interval(mode: control::Mode) -> Duration {
    let clock = mode.clock() as u64;
    let htotal = mode.hsync().2 as u64;
    let vtotal = mode.vsync().2 as u64;

    let mut numerator = htotal * vtotal * 1_000_000;
    let mut denominator = clock;

    if mode.flags().contains(ModeFlags::INTERLACE) {
        denominator *= 2;
    }

    if mode.flags().contains(ModeFlags::DBLSCAN) {
        numerator *= 2;
    }

    if mode.vscan() > 1 {
        numerator *= mode.vscan() as u64;
    }

    let refresh_interval = (numerator + denominator / 2) / denominator;
    Duration::from_nanos(refresh_interval)
}

/// Calculate a DRM mode from a modeline configuration.
pub fn calculate_drm_mode_from_modeline(modeline: &Modeline) -> anyhow::Result<control::Mode> {
    ensure!(
        modeline.hdisplay < modeline.hsync_start,
        "hdisplay {} must be < hsync_start {}",
        modeline.hdisplay,
        modeline.hsync_start
    );
    ensure!(
        modeline.hsync_start < modeline.hsync_end,
        "hsync_start {} must be < hsync_end {}",
        modeline.hsync_start,
        modeline.hsync_end
    );
    ensure!(
        modeline.hsync_end < modeline.htotal,
        "hsync_end {} must be < htotal {}",
        modeline.hsync_end,
        modeline.htotal
    );
    ensure!(
        modeline.vdisplay < modeline.vsync_start,
        "vdisplay {} must be < vsync_start {}",
        modeline.vdisplay,
        modeline.vsync_start
    );
    ensure!(
        modeline.vsync_start < modeline.vsync_end,
        "vsync_start {} must be < vsync_end {}",
        modeline.vsync_start,
        modeline.vsync_end
    );
    ensure!(
        modeline.vsync_end < modeline.vtotal,
        "vsync_end {} must be < vtotal {}",
        modeline.vsync_end,
        modeline.vtotal
    );

    let pixel_clock_kilo_hertz = modeline.clock * 1000.0;
    // Calculated as documented in the CVT 1.2 standard:
    // https://app.box.com/s/vcocw3z73ta09txiskj7cnk6289j356b/file/93518784646
    let vrefresh_hertz = (pixel_clock_kilo_hertz * 1000.0)
        / (modeline.htotal as u64 * modeline.vtotal as u64) as f64;
    ensure!(
        vrefresh_hertz.is_finite(),
        "calculated refresh rate is not finite"
    );
    let vrefresh_rounded = vrefresh_hertz.round() as u32;

    let flags = match modeline.hsync_polarity {
        HSyncPolarity::PHSync => ModeFlags::PHSYNC,
        HSyncPolarity::NHSync => ModeFlags::NHSYNC,
    } | match modeline.vsync_polarity {
        VSyncPolarity::PVSync => ModeFlags::PVSYNC,
        VSyncPolarity::NVSync => ModeFlags::NVSYNC,
    };

    let mode_name = format!(
        "{}x{}@{:.2}",
        modeline.hdisplay, modeline.vdisplay, vrefresh_hertz
    );
    let name = modeinfo_name_slice_from_string(&mode_name);

    // https://www.kernel.org/doc/html/v6.17/gpu/drm-uapi.html#c.drm_mode_modeinfo
    Ok(control::Mode::from(drm_mode_modeinfo {
        clock: pixel_clock_kilo_hertz.round() as u32,
        hdisplay: modeline.hdisplay,
        hsync_start: modeline.hsync_start,
        hsync_end: modeline.hsync_end,
        htotal: modeline.htotal,
        vdisplay: modeline.vdisplay,
        vsync_start: modeline.vsync_start,
        vsync_end: modeline.vsync_end,
        vtotal: modeline.vtotal,
        vrefresh: vrefresh_rounded,
        flags: flags.bits(),
        name,
        // Defaults
        type_: drm_ffi::DRM_MODE_TYPE_USERDEF,
        hskew: 0,
        vscan: 0,
    }))
}

/// Calculate a DRM mode using CVT (Coordinated Video Timings).
pub fn calculate_mode_cvt(width: u16, height: u16, refresh: f64) -> control::Mode {
    // Cross-checked with sway's implementation:
    // https://gitlab.freedesktop.org/wlroots/wlroots/-/blob/22528542970687720556035790212df8d9bb30bb/backend/drm/util.c#L251

    let options = libdisplay_info::cvt::Options {
        red_blank_ver: libdisplay_info::cvt::ReducedBlankingVersion::None,
        h_pixels: width as i32,
        v_lines: height as i32,
        ip_freq_rqd: refresh,

        // Defaults
        video_opt: false,
        vblank: 0f64,
        additional_hblank: 0,
        early_vsync_rqd: false,
        int_rqd: false,
        margins_rqd: false,
    };
    let cvt_timing = libdisplay_info::cvt::Timing::compute(options);

    let hsync_start = width + cvt_timing.h_front_porch as u16;
    let vsync_start = (cvt_timing.v_lines_rnd + cvt_timing.v_front_porch) as u16;
    let hsync_end = hsync_start + cvt_timing.h_sync as u16;
    let vsync_end = vsync_start + cvt_timing.v_sync as u16;

    let htotal = hsync_end + cvt_timing.h_back_porch as u16;
    let vtotal = vsync_end + cvt_timing.v_back_porch as u16;

    let clock = f64::round(cvt_timing.act_pixel_freq * 1000f64) as u32;
    let vrefresh = f64::round(cvt_timing.act_frame_rate) as u32;

    let flags = drm_ffi::DRM_MODE_FLAG_NHSYNC | drm_ffi::DRM_MODE_FLAG_PVSYNC;

    let mode_name = format!("{width}x{height}@{:.2}", cvt_timing.act_frame_rate);
    let name = modeinfo_name_slice_from_string(&mode_name);

    let drm_ffi_mode = drm_ffi::drm_sys::drm_mode_modeinfo {
        clock,

        hdisplay: width,
        hsync_start,
        hsync_end,
        htotal,

        vdisplay: height,
        vsync_start,
        vsync_end,
        vtotal,

        vrefresh,

        flags,
        type_: drm_ffi::DRM_MODE_TYPE_USERDEF,
        name,

        // Defaults
        hskew: 0,
        vscan: 0,
    };

    control::Mode::from(drm_ffi_mode)
}

/// Convert a string to a mode name slice (max 31 chars + null terminator).
fn modeinfo_name_slice_from_string(mode_name: &str) -> [core::ffi::c_char; 32] {
    let mut name: [core::ffi::c_char; 32] = [0; 32];

    for (a, b) in zip(&mut name[..31], mode_name.as_bytes()) {
        // Can be u8 on aarch64 and i8 on x86_64.
        *a = *b as _;
    }

    name
}

/// Pick the best mode for a connector based on configuration.
pub(super) fn pick_mode(
    connector: &connector::Info,
    target: Option<niri_config::output::Mode>,
) -> Option<(control::Mode, bool)> {
    let mut mode = None;
    let mut fallback = false;

    if let Some(target) = target {
        let target_mode = target.mode;

        if target.custom {
            if let Some(refresh) = target_mode.refresh {
                let custom_mode =
                    calculate_mode_cvt(target_mode.width, target_mode.height, refresh);
                return Some((custom_mode, false));
            } else {
                warn!("ignoring custom mode without refresh rate");
            }
        }

        let refresh = target_mode.refresh.map(|r| (r * 1000.).round() as i32);
        for m in connector.modes() {
            if m.size() != (target.mode.width, target.mode.height) {
                continue;
            }

            // Interlaced modes don't appear to work.
            if m.flags().contains(ModeFlags::INTERLACE) {
                continue;
            }

            if let Some(refresh) = refresh {
                // If refresh is set, only pick modes with matching refresh.
                let wl_mode = Mode::from(*m);
                if wl_mode.refresh == refresh {
                    mode = Some(m);
                }
            } else if let Some(curr) = mode {
                // If refresh isn't set, pick the mode with the highest refresh.
                if curr.vrefresh() < m.vrefresh() {
                    mode = Some(m);
                }
            } else {
                mode = Some(m);
            }
        }

        if mode.is_none() {
            fallback = true;
        }
    }

    if mode.is_none() {
        // Pick a preferred mode.
        for m in connector.modes() {
            if !m.mode_type().contains(ModeTypeFlags::PREFERRED) {
                continue;
            }

            if let Some(curr) = mode {
                if curr.vrefresh() < m.vrefresh() {
                    mode = Some(m);
                }
            } else {
                mode = Some(m);
            }
        }
    }

    if mode.is_none() {
        // Last attempt.
        mode = connector.modes().first();
    }

    mode.map(|m| (*m, fallback))
}

// =============================================================================
// EDID Helpers
// =============================================================================

/// Get EDID info from a connector.
pub(super) fn get_edid_info(
    device: &DrmDevice,
    connector: connector::Handle,
) -> anyhow::Result<libdisplay_info::info::Info> {
    let (_, info, value) =
        find_drm_property(device, connector, "EDID").context("no EDID property")?;
    let blob = info
        .value_type()
        .convert_value(value)
        .as_blob()
        .context("EDID was not blob type")?;
    let data = device
        .get_property_blob(blob)
        .context("error getting EDID blob value")?;
    libdisplay_info::info::Info::parse_edid(&data).context("error parsing EDID")
}

/// Create an output name from connector and EDID info.
pub(super) fn make_output_name(
    device: &DrmDevice,
    connector: connector::Handle,
    connector_name: String,
) -> OutputName {
    let info = get_edid_info(device, connector)
        .map_err(|err| warn!("error getting EDID info for {connector_name}: {err:?}"))
        .ok();
    OutputName {
        connector: connector_name,
        make: info.as_ref().and_then(|info| info.make()),
        model: info.as_ref().and_then(|info| info.model()),
        serial: info.as_ref().and_then(|info| info.serial()),
    }
}

// =============================================================================
// HDR/VRR/Gamma Helpers
// =============================================================================

const DRM_MODE_COLORIMETRY_DEFAULT: u64 = 0;

/// Reset HDR properties on a connector.
pub(super) fn reset_hdr(props: &ConnectorProperties) -> anyhow::Result<()> {
    let (info, value) = props.find(c"HDR_OUTPUT_METADATA")?;
    let property::ValueType::Blob = info.value_type() else {
        bail!("wrong property type")
    };

    if *value != 0 {
        props
            .device
            .set_property(props.connector, info.handle(), 0)
            .context("error setting property")?;
    }

    let (info, value) = props.find(c"Colorspace")?;
    let property::ValueType::Enum(_) = info.value_type() else {
        bail!("wrong property type")
    };
    if *value != DRM_MODE_COLORIMETRY_DEFAULT {
        props
            .device
            .set_property(props.connector, info.handle(), DRM_MODE_COLORIMETRY_DEFAULT)
            .context("error setting property")?;
    }

    Ok(())
}

/// Check if a connector supports VRR.
pub(super) fn is_vrr_capable(device: &DrmDevice, connector: connector::Handle) -> Option<bool> {
    let (_, info, value) = find_drm_property(device, connector, "vrr_capable")?;
    info.value_type().convert_value(value).as_boolean()
}

/// Get the panel orientation from connector properties.
pub(super) fn get_panel_orientation(props: &ConnectorProperties) -> anyhow::Result<Transform> {
    let (info, value) = props.find(c"panel orientation")?;
    match info.value_type().convert_value(*value) {
        property::Value::Enum(Some(val)) => match val.value() {
            // "Normal"
            0 => Ok(Transform::Normal),
            // "Upside Down"
            1 => Ok(Transform::_180),
            // "Left Side Up"
            2 => Ok(Transform::_90),
            // "Right Side Up"
            3 => Ok(Transform::_270),
            _ => bail!("panel orientation has invalid value: {:?}", val),
        },
        _ => bail!("panel orientation has wrong value type"),
    }
}

/// Set gamma for a CRTC using the legacy API.
pub fn set_gamma_for_crtc(
    device: &DrmDevice,
    crtc: crtc::Handle,
    ramp: Option<&[u16]>,
) -> anyhow::Result<()> {
    let _span = tracy_client::span!("set_gamma_for_crtc");

    let info = device.get_crtc(crtc).context("error getting crtc info")?;
    let gamma_length = info.gamma_length() as usize;

    ensure!(gamma_length != 0, "setting gamma is not supported");

    let mut temp;
    let ramp = if let Some(ramp) = ramp {
        ensure!(ramp.len() == gamma_length * 3, "wrong gamma length");
        ramp
    } else {
        let _span = tracy_client::span!("generate linear gamma");

        // The legacy API provides no way to reset the gamma, so set a linear one manually.
        temp = vec![0u16; gamma_length * 3];

        let (red, rest) = temp.split_at_mut(gamma_length);
        let (green, blue) = rest.split_at_mut(gamma_length);
        let denom = gamma_length as u64 - 1;
        for (i, ((r, g), b)) in zip(zip(red, green), blue).enumerate() {
            let value = (0xFFFFu64 * i as u64 / denom) as u16;
            *r = value;
            *g = value;
            *b = value;
        }

        &temp
    };

    let (red, ramp) = ramp.split_at(gamma_length);
    let (green, blue) = ramp.split_at(gamma_length);

    device
        .set_gamma(crtc, red, green, blue)
        .context("error setting gamma")?;

    Ok(())
}

// =============================================================================
// DmaBuf Feedback
// =============================================================================

/// Build dmabuf feedback for a surface.
pub(super) fn surface_dmabuf_feedback(
    compositor: &GbmDrmCompositor,
    primary_formats: FormatSet,
    primary_render_node: DrmNode,
    surface_render_node: Option<DrmNode>,
    surface_scanout_node: DrmNode,
) -> Result<SurfaceDmabufFeedback, io::Error> {
    let surface = compositor.surface();
    let planes = surface.planes();

    let primary_plane_formats = surface.plane_info().formats.clone();
    let primary_or_overlay_plane_formats = primary_plane_formats
        .iter()
        .chain(planes.overlay.iter().flat_map(|p| p.formats.iter()))
        .copied()
        .collect::<FormatSet>();

    // We limit the scan-out trache to formats we can also render from so that there is always a
    // fallback render path available in case the supplied buffer can not be scanned out directly.
    let mut primary_scanout_formats = primary_plane_formats
        .intersection(&primary_formats)
        .copied()
        .collect::<Vec<_>>();
    let mut primary_or_overlay_scanout_formats = primary_or_overlay_plane_formats
        .intersection(&primary_formats)
        .copied()
        .collect::<Vec<_>>();

    // HACK: AMD iGPU + dGPU systems share some modifiers between the two, and yet cross-device
    // buffers produce a glitched scanout if the modifier is not Linear...
    //
    // Also limit scan-out formats to Linear if we have a device without a render node (i.e.
    // we're rendering on a different device).
    if surface_render_node != Some(primary_render_node) {
        primary_scanout_formats.retain(|f| f.modifier == Modifier::Linear);
        primary_or_overlay_scanout_formats.retain(|f| f.modifier == Modifier::Linear);
    }

    let builder = DmabufFeedbackBuilder::new(primary_render_node.dev_id(), primary_formats);

    trace!(
        "primary scanout formats: {}, overlay adds: {}",
        primary_scanout_formats.len(),
        primary_or_overlay_scanout_formats.len() - primary_scanout_formats.len(),
    );

    // Prefer the primary-plane-only formats, then primary-or-overlay-plane formats. This will
    // increase the chance of scanning out a client even with our disabled-by-default overlay
    // planes.
    let scanout = builder
        .clone()
        .add_preference_tranche(
            surface_scanout_node.dev_id(),
            Some(TrancheFlags::Scanout),
            primary_scanout_formats,
        )
        .add_preference_tranche(
            surface_scanout_node.dev_id(),
            Some(TrancheFlags::Scanout),
            primary_or_overlay_scanout_formats,
        )
        .build()?;

    // If this is the primary node surface, send scanout formats in both tranches to avoid
    // duplication.
    let render = if surface_render_node == Some(primary_render_node) {
        scanout.clone()
    } else {
        builder.build()?
    };

    Ok(SurfaceDmabufFeedback { render, scanout })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use niri_config::output::Modeline;
    use niri_ipc::{HSyncPolarity, VSyncPolarity};

    use super::{calculate_drm_mode_from_modeline, calculate_mode_cvt};

    #[test]
    fn test_calculate_drmmode_from_modeline() {
        let modeline1 = Modeline {
            clock: 173.0,
            hdisplay: 1920,
            vdisplay: 1080,
            hsync_start: 2048,
            hsync_end: 2248,
            htotal: 2576,
            vsync_start: 1083,
            vsync_end: 1088,
            vtotal: 1120,
            hsync_polarity: HSyncPolarity::NHSync,
            vsync_polarity: VSyncPolarity::PVSync,
        };
        assert_debug_snapshot!(calculate_drm_mode_from_modeline(&modeline1).unwrap(), @"Mode {
    name: \"1920x1080@59.96\",
    clock: 173000,
    size: (
        1920,
        1080,
    ),
    hsync: (
        2048,
        2248,
        2576,
    ),
    vsync: (
        1083,
        1088,
        1120,
    ),
    hskew: 0,
    vscan: 0,
    vrefresh: 60,
    mode_type: ModeTypeFlags(
        USERDEF,
    ),
}");
        let modeline2 = Modeline {
            clock: 452.5,
            hdisplay: 1920,
            vdisplay: 1080,
            hsync_start: 2088,
            hsync_end: 2296,
            htotal: 2672,
            vsync_start: 1083,
            vsync_end: 1088,
            vtotal: 1177,
            hsync_polarity: HSyncPolarity::NHSync,
            vsync_polarity: VSyncPolarity::PVSync,
        };
        assert_debug_snapshot!(calculate_drm_mode_from_modeline(&modeline2).unwrap(), @"Mode {
    name: \"1920x1080@143.88\",
    clock: 452500,
    size: (
        1920,
        1080,
    ),
    hsync: (
        2088,
        2296,
        2672,
    ),
    vsync: (
        1083,
        1088,
        1177,
    ),
    hskew: 0,
    vscan: 0,
    vrefresh: 144,
    mode_type: ModeTypeFlags(
        USERDEF,
    ),
}");
    }

    #[test]
    fn test_calc_cvt() {
        // Crosschecked with other calculators like the cvt commandline utility.
        assert_debug_snapshot!(calculate_mode_cvt(1920, 1080, 60.0), @"Mode {
    name: \"1920x1080@59.96\",
    clock: 173000,
    size: (
        1920,
        1080,
    ),
    hsync: (
        2048,
        2248,
        2576,
    ),
    vsync: (
        1083,
        1088,
        1120,
    ),
    hskew: 0,
    vscan: 0,
    vrefresh: 60,
    mode_type: ModeTypeFlags(
        USERDEF,
    ),
}");
        assert_debug_snapshot!(calculate_mode_cvt(1920, 1080, 144.0), @"Mode {
    name: \"1920x1080@143.88\",
    clock: 452500,
    size: (
        1920,
        1080,
    ),
    hsync: (
        2088,
        2296,
        2672,
    ),
    vsync: (
        1083,
        1088,
        1177,
    ),
    hskew: 0,
    vscan: 0,
    vrefresh: 144,
    mode_type: ModeTypeFlags(
        USERDEF,
    ),
}");
    }
}
