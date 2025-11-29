//! Pure data types for the Niri compositor.
//!
//! These types have no behavior beyond basic accessors and are used
//! throughout the compositor state.

use std::cell::RefCell;

use calloop::RegistrationToken;
use smithay::desktop::{LayerSurface, PopupGrab, Window};
use smithay::output::{Output, WeakOutput};
use smithay::reexports::wayland_protocols::ext::session_lock::v1::server::ext_session_lock_v1::ExtSessionLockV1;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};
use smithay::wayland::session_lock::SessionLocker;

use crate::layout::HitType;
use crate::window::mapped::MappedId;

// Forward declaration - State is defined in this module's parent
use super::State;

// =============================================================================
// PointerVisibility
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerVisibility {
    /// The pointer is visible.
    #[default]
    Visible,
    /// The pointer is invisible, but retains its focus.
    ///
    /// This state is set temporarily after auto-hiding the pointer to keep tooltips open and grabs
    /// ongoing.
    Hidden,
    /// The pointer is invisible and cannot focus.
    ///
    /// Corresponds to a fully disabled pointer, for example after a touchscreen input, or after
    /// the pointer contents changed in a Hidden state.
    Disabled,
}

impl PointerVisibility {
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Visible)
    }
}

// =============================================================================
// DndIcon
// =============================================================================

#[derive(Debug)]
pub struct DndIcon {
    pub surface: WlSurface,
    pub offset: Point<i32, Logical>,
}

// =============================================================================
// CenterCoords
// =============================================================================

pub enum CenterCoords {
    Separately,
    Both,
    // Force centering even if the cursor is already in the rectangle.
    BothAlways,
}

// =============================================================================
// RedrawState
// =============================================================================

#[derive(Debug, Default)]
pub enum RedrawState {
    /// The compositor is idle.
    #[default]
    Idle,
    /// A redraw is queued.
    Queued,
    /// We submitted a frame to the KMS and waiting for it to be presented.
    WaitingForVBlank { redraw_needed: bool },
    /// We did not submit anything to KMS and made a timer to fire at the estimated VBlank.
    WaitingForEstimatedVBlank(RegistrationToken),
    /// A redraw is queued on top of the above.
    WaitingForEstimatedVBlankAndQueued(RegistrationToken),
}

impl RedrawState {
    pub fn queue_redraw(self) -> Self {
        match self {
            RedrawState::Idle => RedrawState::Queued,
            RedrawState::WaitingForEstimatedVBlank(token) => {
                RedrawState::WaitingForEstimatedVBlankAndQueued(token)
            }

            // A redraw is already queued.
            value @ (RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)) => {
                value
            }

            // We're waiting for VBlank, request a redraw afterwards.
            RedrawState::WaitingForVBlank { .. } => RedrawState::WaitingForVBlank {
                redraw_needed: true,
            },
        }
    }
}

// =============================================================================
// LockState and LockRenderState
// =============================================================================

#[derive(Debug, Default)]
pub enum LockState {
    #[default]
    Unlocked,
    WaitingForSurfaces {
        confirmation: SessionLocker,
        deadline_token: RegistrationToken,
    },
    Locking(SessionLocker),
    Locked(ExtSessionLockV1),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LockRenderState {
    /// The output displays a normal session frame.
    Unlocked,
    /// The output displays a locked frame.
    Locked,
}

// =============================================================================
// KeyboardFocus
// =============================================================================

// The surfaces here are always toplevel surfaces focused as far as niri's logic is concerned, even
// when popup grabs are active (which means the real keyboard focus is on a popup descending from
// that toplevel surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardFocus {
    // Layout is focused by default if there's nothing else to focus.
    Layout { surface: Option<WlSurface> },
    LayerShell { surface: WlSurface },
    LockScreen { surface: Option<WlSurface> },
    ScreenshotUi,
    ExitConfirmDialog,
    Overview,
    Mru,
}

impl KeyboardFocus {
    pub fn surface(&self) -> Option<&WlSurface> {
        match self {
            KeyboardFocus::Layout { surface } => surface.as_ref(),
            KeyboardFocus::LayerShell { surface } => Some(surface),
            KeyboardFocus::LockScreen { surface } => surface.as_ref(),
            KeyboardFocus::ScreenshotUi => None,
            KeyboardFocus::ExitConfirmDialog => None,
            KeyboardFocus::Overview => None,
            KeyboardFocus::Mru => None,
        }
    }

    pub fn into_surface(self) -> Option<WlSurface> {
        match self {
            KeyboardFocus::Layout { surface } => surface,
            KeyboardFocus::LayerShell { surface } => Some(surface),
            KeyboardFocus::LockScreen { surface } => surface,
            KeyboardFocus::ScreenshotUi => None,
            KeyboardFocus::ExitConfirmDialog => None,
            KeyboardFocus::Overview => None,
            KeyboardFocus::Mru => None,
        }
    }

    pub fn is_layout(&self) -> bool {
        matches!(self, KeyboardFocus::Layout { .. })
    }

    pub fn is_overview(&self) -> bool {
        matches!(self, KeyboardFocus::Overview)
    }
}

// =============================================================================
// PointContents
// =============================================================================

#[derive(Default, Clone, PartialEq)]
pub struct PointContents {
    // Output under point.
    pub output: Option<Output>,
    // Surface under point and its location in the global coordinate space.
    //
    // Can be `None` even when `window` is set, for example when the pointer is over the niri
    // border around the window.
    pub surface: Option<(WlSurface, Point<f64, Logical>)>,
    // If surface belongs to a window, this is that window.
    pub window: Option<(Window, HitType)>,
    // If surface belongs to a layer surface, this is that layer surface.
    pub layer: Option<LayerSurface>,
    // Pointer is over a hot corner.
    pub hot_corner: bool,
}

// =============================================================================
// CastTarget
// =============================================================================

#[derive(Clone, PartialEq, Eq)]
pub enum CastTarget {
    // Dynamic cast before selecting anything.
    Nothing,
    Output(WeakOutput),
    Window { id: u64 },
}

// =============================================================================
// PopupGrabState
// =============================================================================

pub struct PopupGrabState {
    pub root: WlSurface,
    pub grab: PopupGrab<State>,
    pub has_keyboard_grab: bool,
}

// =============================================================================
// PendingMruCommit
// =============================================================================

/// Pending update to a window's focus timestamp.
#[derive(Debug)]
pub struct PendingMruCommit {
    pub id: MappedId,
    pub token: RegistrationToken,
    pub stamp: std::time::Duration,
}

// =============================================================================
// SurfaceFrameThrottlingState (internal)
// =============================================================================

// Not related to the one in Smithay.
//
// This state keeps track of when a surface last received a frame callback.
pub(crate) struct SurfaceFrameThrottlingState {
    /// Output and sequence that the frame callback was last sent at.
    pub last_sent_at: RefCell<Option<(Output, u32)>>,
}

impl Default for SurfaceFrameThrottlingState {
    fn default() -> Self {
        Self {
            last_sent_at: RefCell::new(None),
        }
    }
}
