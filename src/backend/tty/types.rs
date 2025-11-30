//! Type definitions for TTY backend.
//!
//! This module contains all type aliases, structs, and constants used by the TTY backend.

use std::num::NonZeroU64;
use std::time::Duration;

use niri_config::OutputName;
use smithay::backend::allocator::gbm::GbmAllocator;
use smithay::backend::allocator::Fourcc;
use smithay::backend::drm::compositor::DrmCompositor;
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::drm::{DrmDevice, DrmDeviceFd, DrmNode};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::renderer::multigpu::{MultiFrame, MultiRenderer};
use smithay::backend::renderer::RendererSuper;
use smithay::desktop::utils::OutputPresentationFeedback;
use smithay::reexports::drm::control::{connector, crtc, property};
use smithay::wayland::dmabuf::DmabufFeedback;

use crate::backend::OutputId;

// =============================================================================
// Public Type Aliases
// =============================================================================

pub type TtyRenderer<'render> = MultiRenderer<
    'render,
    'render,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

pub type TtyFrame<'render, 'frame, 'buffer> = MultiFrame<
    'render,
    'render,
    'frame,
    'buffer,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

pub type TtyRendererError<'render> = <TtyRenderer<'render> as RendererSuper>::Error;

// =============================================================================
// Internal Type Aliases
// =============================================================================

pub(super) type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    (OutputPresentationFeedback, Duration),
    DrmDeviceFd,
>;

// =============================================================================
// Constants
// =============================================================================

pub(super) const SUPPORTED_COLOR_FORMATS: [Fourcc; 4] = [
    Fourcc::Xrgb8888,
    Fourcc::Xbgr8888,
    Fourcc::Argb8888,
    Fourcc::Abgr8888,
];

// =============================================================================
// Public Structs
// =============================================================================

/// Dmabuf feedback for a surface (render vs scanout paths).
pub struct SurfaceDmabufFeedback {
    pub render: DmabufFeedback,
    pub scanout: DmabufFeedback,
}

// =============================================================================
// Internal Structs
// =============================================================================

/// Per-output state stored in Output user data.
#[derive(Debug, Clone, Copy)]
pub(super) struct TtyOutputState {
    pub node: DrmNode,
    pub crtc: crtc::Handle,
}

/// A connected, but not necessarily enabled, crtc.
#[derive(Debug, Clone)]
pub struct CrtcInfo {
    pub(crate) id: OutputId,
    pub(crate) name: OutputName,
}

impl CrtcInfo {
    /// Create a new CrtcInfo.
    pub fn new(id: OutputId, name: OutputName) -> Self {
        Self { id, name }
    }

    /// Get the output ID.
    pub fn id(&self) -> OutputId {
        self.id
    }

    /// Get the output name.
    pub fn name(&self) -> &OutputName {
        &self.name
    }
}

/// Surface state for a connected output.
pub(super) struct Surface {
    pub name: OutputName,
    pub compositor: GbmDrmCompositor,
    pub connector: connector::Handle,
    pub dmabuf_feedback: Option<SurfaceDmabufFeedback>,
    pub gamma_props: Option<GammaProps>,
    /// Gamma change to apply upon session resume.
    pub pending_gamma_change: Option<Option<Vec<u16>>>,
    /// Tracy frame that goes from vblank to vblank.
    pub vblank_frame: Option<tracy_client::Frame>,
    /// Frame name for the VBlank frame.
    pub vblank_frame_name: tracy_client::FrameName,
    /// Plot name for the time since presentation plot.
    pub time_since_presentation_plot_name: tracy_client::PlotName,
    /// Plot name for the presentation misprediction plot.
    pub presentation_misprediction_plot_name: tracy_client::PlotName,
    pub sequence_delta_plot_name: tracy_client::PlotName,
}

/// Gamma LUT properties for a CRTC.
pub(super) struct GammaProps {
    pub crtc: crtc::Handle,
    pub gamma_lut: property::Handle,
    pub gamma_lut_size: property::Handle,
    pub previous_blob: Option<NonZeroU64>,
}

/// Connector properties for configuration.
pub(super) struct ConnectorProperties<'a> {
    pub device: &'a DrmDevice,
    pub connector: connector::Handle,
    pub properties: Vec<(property::Info, property::RawValue)>,
}
// Note: impl ConnectorProperties is in mod.rs
