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
// TEAM_003: Import ColumnWidth and ResolvedSize from types module
pub use super::types::ColumnWidth;
use super::types::ResolvedSize;
use super::{LayoutElement, Options};
use crate::animation::{Animation, Clock};

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

// TEAM_004: Snapshot method for golden testing
// TEAM_010: Extended with animation capture
#[cfg(test)]
impl<W: LayoutElement> Column<W> {
    /// Create a snapshot of this column for golden testing.
    ///
    /// The `column_x` parameter is the X position of this column's left edge.
    pub fn snapshot(&self, column_x: f64) -> crate::layout::snapshot::ColumnSnapshot {
        use crate::layout::snapshot::{ColumnSnapshot, TileSnapshot};

        let tiles: Vec<TileSnapshot> = self
            .tiles()
            .map(|(tile, offset)| {
                let tile_size = tile.tile_size();
                TileSnapshot {
                    x: column_x + offset.x,
                    y: offset.y,
                    width: tile_size.w,
                    height: tile_size.h,
                }
            })
            .collect();

        ColumnSnapshot {
            x: column_x,
            width: self.width(),
            tiles,
            active_tile_idx: self.active_tile_idx,
            is_full_width: self.is_full_width,
            is_fullscreen: self.sizing_mode().is_fullscreen(),
        }
    }

    /// Returns the column's move animation if present (animation, from_offset).
    pub fn move_animation(&self) -> Option<(&crate::animation::Animation, f64)> {
        self.move_animation.as_ref().map(|m| (&m.anim, m.from))
    }

    /// Returns iterator over tiles with their animations for golden testing.
    pub fn tiles_with_animations(&self) -> impl Iterator<Item = (&super::tile::Tile<W>, usize)> {
        self.tiles.iter().enumerate().map(|(idx, tile)| (tile, idx))
    }
}
