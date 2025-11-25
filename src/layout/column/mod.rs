// TEAM_002: Column module extracted from scrolling.rs
//! Column layout management.
//!
//! A column is a vertical stack of tiles (windows) within a scrolling workspace.

mod core;
mod layout;
mod operations;
mod render;
mod sizing;
#[cfg(test)]
mod tests;
mod tile_data;

use std::rc::Rc;

use niri_config::PresetSize;
use niri_ipc::ColumnDisplay;
use smithay::utils::{Logical, Rectangle, Size};

pub use self::tile_data::{TileData, WindowHeight};
use super::tab_indicator::TabIndicator;
use super::tile::Tile;
use super::workspace::ResolvedSize;
use super::{LayoutElement, Options};
use crate::animation::{Animation, Clock};

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

#[derive(Debug)]
pub(crate) struct MoveAnimation {
    pub anim: Animation,
    pub from: f64,
}

/// A column of tiled windows.
#[derive(Debug)]
pub struct Column<W: LayoutElement> {
    /// Tiles in this column. Must be non-empty.
    pub(crate) tiles: Vec<Tile<W>>,
    /// Extra per-tile data. Must have the same number of elements as `tiles`.
    pub(crate) data: Vec<TileData>,
    /// Index of the currently active tile.
    pub(crate) active_tile_idx: usize,
    /// Desired width of this column.
    pub(crate) width: ColumnWidth,
    /// Currently selected preset width index.
    pub(crate) preset_width_idx: Option<usize>,
    /// Whether this column is full-width.
    pub(crate) is_full_width: bool,
    /// Whether this column is going to be fullscreen.
    pub(crate) is_pending_fullscreen: bool,
    /// Whether this column is going to be maximized.
    pub(crate) is_pending_maximized: bool,
    /// How this column displays and arranges windows.
    pub(crate) display_mode: ColumnDisplay,
    /// Tab indicator for the tabbed display mode.
    pub(crate) tab_indicator: TabIndicator,
    /// Animation of the render offset during window swapping.
    pub(crate) move_animation: Option<MoveAnimation>,
    /// Latest known view size for this column's workspace.
    pub(crate) view_size: Size<f64, Logical>,
    /// Latest known working area for this column's workspace.
    pub(crate) working_area: Rectangle<f64, Logical>,
    /// Working area excluding struts.
    pub(crate) parent_area: Rectangle<f64, Logical>,
    /// Scale of the output the column is on.
    pub(crate) scale: f64,
    /// Clock for driving animations.
    pub(crate) clock: Clock,
    /// Configurable properties of the layout.
    pub(crate) options: Rc<Options>,
}

pub(crate) fn resolve_preset_size(
    preset: PresetSize,
    options: &Options,
    view_size: f64,
    extra_size: f64,
) -> ResolvedSize {
    match preset {
        PresetSize::Proportion(proportion) => ResolvedSize::Tile(
            (view_size - options.layout.gaps) * proportion - options.layout.gaps - extra_size,
        ),
        PresetSize::Fixed(width) => ResolvedSize::Window(f64::from(width)),
    }
}
