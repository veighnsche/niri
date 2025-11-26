//! Snapshot types for golden testing.
//!
//! These types capture layout state for comparison between
//! golden (original) and refactored code.
//!
//! TEAM_004: Created as part of Phase 0.5.A infrastructure.

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
