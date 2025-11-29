// TEAM_003: Shared types for the layout module
//! Shared types used across layout modules.
//!
//! These types are used by Column, ScrollingSpace, FloatingSpace, and will be used
//! by Row and Canvas2D in the future.

use niri_config::PresetSize;
use smithay::utils::{Logical, Size};

use super::{InteractiveResizeData, LayoutElement};

/// Width of a column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    /// Proportion of the current view width.
    Proportion(f64),
    /// Fixed width in logical pixels.
    Fixed(f64),
}

impl From<PresetSize> for ColumnWidth {
    fn from(value: PresetSize) -> Self {
        match value {
            PresetSize::Proportion(p) => Self::Proportion(p.clamp(0., 10000.)),
            PresetSize::Fixed(f) => Self::Fixed(f64::from(f.clamp(1, 100000))),
        }
    }
}

/// Horizontal direction for an operation.
///
/// As operations often have a symmetrical counterpart, e.g. focus-right/focus-left, methods
/// on `ScrollingSpace` can sometimes be factored using the direction of the operation as a
/// parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Left,
    Right,
}

/// Resolved width or height in logical pixels.
#[derive(Debug, Clone, Copy)]
pub enum ResolvedSize {
    /// Size of the tile including borders.
    Tile(f64),
    /// Size of the window excluding borders.
    Window(f64),
}

/// State for an ongoing interactive resize operation.
#[derive(Debug)]
pub struct InteractiveResize<W: LayoutElement> {
    pub window: W::Id,
    pub original_window_size: Size<f64, Logical>,
    pub data: InteractiveResizeData,
}

/// Position where a window should be inserted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertPosition {
    /// Insert as a new column at the given index.
    NewColumn(usize),
    /// Insert in an existing column at (column_idx, tile_idx).
    InColumn(usize, usize),
    /// Insert in the floating layout.
    Floating,
}
