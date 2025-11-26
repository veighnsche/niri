//! Snapshot types for golden testing.
//!
//! These types capture layout state for comparison between
//! golden (original) and refactored code.
//!
//! ## RTL Mirroring
//!
//! All snapshots are YAML-parsable so they can be mirrored for RTL:
//! - Negate all X values
//! - Negate animation `from`/`to` for X-axis animations
//! - Leading edge ↔ trailing edge swap

use serde::Serialize;
use smithay::utils::{Logical, Rectangle, Size};

/// Snapshot of scrolling layout state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScrollingSnapshot {
    /// Columns in the scrolling layout.
    pub columns: Vec<ColumnSnapshot>,
    /// Index of the active column.
    pub active_column_idx: usize,
    /// Current view offset (camera X position).
    pub view_offset: f64,
    /// Working area rectangle.
    pub working_area: RectSnapshot,
    /// View size.
    pub view_size: SizeSnapshot,
    /// Active animations (empty if none or after CompleteAnimations).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<AnimationTimelineSnapshot>,
}

/// Snapshot of a single column.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ColumnSnapshot {
    /// Visual X position of column left edge.
    pub x: f64,
    /// Visual width of column.
    pub width: f64,
    /// Tiles from top to bottom.
    pub tiles: Vec<TileSnapshot>,
    /// Index of the active tile within this column.
    pub active_tile_idx: usize,
    /// Whether the column is full-width.
    pub is_full_width: bool,
    /// Whether the column is in fullscreen mode.
    pub is_fullscreen: bool,
}

/// Snapshot of a single tile.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TileSnapshot {
    /// Visual X position.
    pub x: f64,
    /// Visual Y position.
    pub y: f64,
    /// Visual width.
    pub width: f64,
    /// Visual height.
    pub height: f64,
}

/// Rectangle snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RectSnapshot {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl From<Rectangle<f64, Logical>> for RectSnapshot {
    fn from(rect: Rectangle<f64, Logical>) -> Self {
        Self {
            x: rect.loc.x,
            y: rect.loc.y,
            w: rect.size.w,
            h: rect.size.h,
        }
    }
}

/// Size snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SizeSnapshot {
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl From<Size<f64, Logical>> for SizeSnapshot {
    fn from(size: Size<f64, Logical>) -> Self {
        Self {
            w: size.w,
            h: size.h,
        }
    }
}

// ============================================================================
// Animation Timeline Snapshots
// ============================================================================

/// Timeline snapshot for a single animated value.
///
/// Captures animation parameters in YAML-parsable format for RTL mirroring.
/// For X-axis animations, `from` and `to` should be negated for RTL.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnimationTimelineSnapshot {
    /// What this animation controls.
    /// Examples: "view_offset", "column_0_width", "tile_0_0_height"
    pub target: String,
    /// Starting value.
    pub from: f64,
    /// Target value.
    pub to: f64,
    /// Animation kind.
    pub kind: AnimationKindSnapshot,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Which edge is pinned (for resize animations).
    /// In LTR: "left" (trailing) is pinned, "right" (leading) animates.
    /// In RTL: "right" (trailing) is pinned, "left" (leading) animates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_edge: Option<String>,
}

/// Animation kind snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AnimationKindSnapshot {
    /// Easing curve animation.
    Easing {
        /// Curve name (Linear, EaseOutCubic, etc.)
        curve: String,
        /// Duration in milliseconds.
        duration_ms: u64,
    },
    /// Spring physics animation.
    Spring {
        /// Damping ratio (1.0 = critically damped).
        damping_ratio: f64,
        /// Spring stiffness.
        stiffness: f64,
    },
    /// Deceleration animation (for gestures).
    Deceleration {
        /// Initial velocity.
        initial_velocity: f64,
        /// Deceleration rate.
        deceleration_rate: f64,
    },
}

impl AnimationTimelineSnapshot {
    /// Create a view_offset animation timeline.
    pub fn view_offset(from: f64, to: f64, kind: AnimationKindSnapshot, duration_ms: u64) -> Self {
        Self {
            target: "view_offset".to_string(),
            from,
            to,
            kind,
            duration_ms,
            pinned_edge: None,
        }
    }

    /// Create a tile edge animation timeline.
    ///
    /// Edge names use min/max for RTL-safe coordinates:
    /// - `x_min` / `x_max` for horizontal edges (affected by RTL negation)
    /// - `y_min` / `y_max` for vertical edges (unaffected by RTL)
    ///
    /// In LTR: x_min = left edge, x_max = right edge
    /// In RTL: after negating X, x_min/x_max values flip signs but semantics preserved
    pub fn tile_edge(
        column_idx: usize,
        tile_idx: usize,
        edge: &str,
        from: f64,
        to: f64,
        kind: AnimationKindSnapshot,
        duration_ms: u64,
    ) -> Self {
        Self {
            target: format!("tile_{column_idx}_{tile_idx}_{edge}"),
            from,
            to,
            kind,
            duration_ms,
            pinned_edge: None,
        }
    }

    /// Create a column width animation timeline.
    pub fn column_width(
        column_idx: usize,
        from: f64,
        to: f64,
        kind: AnimationKindSnapshot,
        duration_ms: u64,
        pinned_edge: &str,
    ) -> Self {
        Self {
            target: format!("column_{column_idx}_width"),
            from,
            to,
            kind,
            duration_ms,
            pinned_edge: Some(pinned_edge.to_string()),
        }
    }

    /// Create a tile height animation timeline.
    pub fn tile_height(
        column_idx: usize,
        tile_idx: usize,
        from: f64,
        to: f64,
        kind: AnimationKindSnapshot,
        duration_ms: u64,
    ) -> Self {
        Self {
            target: format!("tile_{column_idx}_{tile_idx}_height"),
            from,
            to,
            kind,
            duration_ms,
            pinned_edge: None,
        }
    }

    /// Returns true if this is an X-axis animation (affected by RTL).
    ///
    /// X-axis animations have their `from` and `to` values negated for RTL.
    pub fn is_x_axis(&self) -> bool {
        self.target == "view_offset"
            || self.target.ends_with("_x_min")
            || self.target.ends_with("_x_max")
            || self.target.ends_with("_move_x")
            || self.target.ends_with("_width")
    }

    /// Mirror this animation for RTL.
    ///
    /// For X-axis animations (view_offset, left/right edges), negates from/to.
    /// For pinned_edge, swaps left↔right.
    pub fn mirror_rtl(&self) -> Self {
        let is_x_axis = self.is_x_axis();

        let (from, to) = if is_x_axis {
            (-self.from, -self.to)
        } else {
            (self.from, self.to)
        };

        let pinned_edge = self.pinned_edge.as_ref().map(|edge| match edge.as_str() {
            "left" => "right".to_string(),
            "right" => "left".to_string(),
            other => other.to_string(),
        });

        Self {
            target: self.target.clone(),
            from,
            to,
            kind: self.kind.clone(),
            duration_ms: self.duration_ms,
            pinned_edge,
        }
    }
}
