// TEAM_013: Types extracted from monitor.rs for modular refactor
//!
//! This module contains type definitions used by the Monitor struct.

use std::time::Duration;

use niri_config::CornerRadius;
use smithay::utils::{Logical, Point};

use crate::animation::Animation;
use crate::input::swipe_tracker::SwipeTracker;
// TEAM_055: Renamed from workspace_types to row_types, WorkspaceId to RowId
use crate::layout::row_types::RowId as WorkspaceId;
use crate::layout::LayoutElement;

// TEAM_013: Re-export InsertPosition from types module
pub use crate::layout::types::InsertPosition;

/// Amount of touchpad movement to scroll the height of one workspace.
pub const WORKSPACE_GESTURE_MOVEMENT: f64 = 300.;

/// Rubber band configuration for workspace gestures.
pub const WORKSPACE_GESTURE_RUBBER_BAND: crate::rubber_band::RubberBand =
    crate::rubber_band::RubberBand {
        stiffness: 0.5,
        limit: 0.05,
    };

/// Amount of DnD edge scrolling to scroll the height of one workspace.
///
/// This constant is tied to the default dnd-edge-workspace-switch max-speed setting.
pub const WORKSPACE_DND_EDGE_SCROLL_MOVEMENT: f64 = 1500.;

/// In-progress switch between workspaces.
#[derive(Debug)]
pub enum WorkspaceSwitch {
    Animation(Animation),
    Gesture(WorkspaceSwitchGesture),
}

/// Gesture state for workspace switching.
#[derive(Debug)]
pub struct WorkspaceSwitchGesture {
    /// Index of the workspace where the gesture was started.
    pub center_idx: usize,
    /// Fractional workspace index where the gesture was started.
    ///
    /// Can differ from center_idx when starting a gesture in the middle between workspaces, for
    /// example by "catching" an animation.
    pub start_idx: f64,
    /// Current, fractional workspace index.
    pub current_idx: f64,
    /// Animation for the extra offset to the current position.
    ///
    /// For example, if there's a workspace switch during a DnD scroll.
    pub animation: Option<Animation>,
    pub tracker: SwipeTracker,
    /// Whether the gesture is controlled by the touchpad.
    pub is_touchpad: bool,
    /// Whether the gesture is clamped to +-1 workspace around the center.
    pub is_clamped: bool,

    /// If this gesture is for drag-and-drop scrolling, this is the last event's unadjusted
    /// timestamp.
    pub dnd_last_event_time: Option<Duration>,
    /// Time when the drag-and-drop scroll delta became non-zero, used for debouncing.
    ///
    /// If `None` then the scroll delta is currently zero.
    pub dnd_nonzero_start_time: Option<Duration>,
}

/// Which workspace to insert a window into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertWorkspace {
    Existing(WorkspaceId),
    NewAt(usize),
}

/// Hint for where a window will be inserted.
#[derive(Debug)]
pub struct InsertHint {
    pub workspace: InsertWorkspace,
    pub position: InsertPosition,
    pub corner_radius: CornerRadius,
}

/// Location to render the insert hint element.
#[derive(Debug, Clone, Copy)]
pub struct InsertHintRenderLoc {
    pub workspace: InsertWorkspace,
    pub location: Point<f64, Logical>,
}

// TEAM_014: Removed OverviewProgress enum (Part 3)

/// Where to put a newly added window.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum MonitorAddWindowTarget<'a, W: LayoutElement> {
    /// No particular preference.
    #[default]
    Auto,
    /// On this workspace.
    Workspace {
        /// Id of the target workspace.
        id: WorkspaceId,
        /// Override where the window will open as a new column.
        column_idx: Option<usize>,
    },
    /// Next to this existing window.
    NextTo(&'a W::Id),
}

impl<'a, W: LayoutElement> Copy for MonitorAddWindowTarget<'a, W> {}

impl<'a, W: LayoutElement> Clone for MonitorAddWindowTarget<'a, W> {
    fn clone(&self) -> Self {
        *self
    }
}

// =========================================================================
// Impl blocks for types
// =========================================================================

impl WorkspaceSwitch {
    pub fn current_idx(&self) -> f64 {
        match self {
            WorkspaceSwitch::Animation(anim) => anim.value(),
            WorkspaceSwitch::Gesture(gesture) => {
                gesture.current_idx + gesture.animation.as_ref().map_or(0., |anim| anim.value())
            }
        }
    }

    pub fn target_idx(&self) -> f64 {
        match self {
            WorkspaceSwitch::Animation(anim) => anim.to(),
            WorkspaceSwitch::Gesture(gesture) => gesture.current_idx,
        }
    }

    pub fn offset(&mut self, delta: isize) {
        match self {
            WorkspaceSwitch::Animation(anim) => anim.offset(delta as f64),
            WorkspaceSwitch::Gesture(gesture) => {
                if delta >= 0 {
                    gesture.center_idx += delta as usize;
                } else {
                    gesture.center_idx -= (-delta) as usize;
                }
                gesture.start_idx += delta as f64;
                gesture.current_idx += delta as f64;
            }
        }
    }

    pub fn is_animation_ongoing(&self) -> bool {
        match self {
            WorkspaceSwitch::Animation(_) => true,
            WorkspaceSwitch::Gesture(gesture) => gesture.animation.is_some(),
        }
    }
}

impl WorkspaceSwitchGesture {
    pub fn min_max(&self, workspace_count: usize) -> (f64, f64) {
        if self.is_clamped {
            let min = self.center_idx.saturating_sub(1) as f64;
            let max = (self.center_idx + 1).min(workspace_count - 1) as f64;
            (min, max)
        } else {
            (0., (workspace_count - 1) as f64)
        }
    }

    pub fn animate_from(
        &mut self,
        from: f64,
        clock: crate::animation::Clock,
        config: niri_config::Animation,
    ) {
        let current = self.animation.as_ref().map_or(0., Animation::value);
        self.animation = Some(Animation::new(clock, from + current, 0., 0., config));
    }
}

impl InsertWorkspace {
    pub fn existing_id(self) -> Option<WorkspaceId> {
        match self {
            InsertWorkspace::Existing(id) => Some(id),
            InsertWorkspace::NewAt(_) => None,
        }
    }
}

// TEAM_014: Removed OverviewProgress impl blocks (Part 3)
