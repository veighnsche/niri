// TEAM_002: TileData and WindowHeight extracted from scrolling.rs

use smithay::utils::{Logical, Size};

use crate::layout::tile::Tile;
use crate::layout::LayoutElement;
use crate::utils::ResizeEdge;

/// Height of a window in a column.
///
/// Every window but one in a column must be `Auto`-sized so that the total height can add up to
/// the workspace height. Resizing a window converts all other windows to `Auto`, weighted to
/// preserve their visual heights at the moment of the conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowHeight {
    /// Automatically computed *tile* height, distributed across the column according to weights.
    Auto { weight: f64 },
    /// Fixed *window* height in logical pixels.
    Fixed(f64),
    /// One of the preset heights (tile or window).
    Preset(usize),
}

impl WindowHeight {
    pub(crate) const fn auto_1() -> Self {
        Self::Auto { weight: 1. }
    }
}

/// Extra per-tile data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileData {
    /// Requested height of the window.
    pub(crate) height: WindowHeight,
    /// Cached actual size of the tile.
    pub(crate) size: Size<f64, Logical>,
    /// Cached whether the tile is being interactively resized by its left edge.
    pub(crate) interactively_resizing_by_left_edge: bool,
}

impl TileData {
    pub fn new<W: LayoutElement>(tile: &Tile<W>, height: WindowHeight) -> Self {
        let mut rv = Self {
            height,
            size: Size::default(),
            interactively_resizing_by_left_edge: false,
        };
        rv.update(tile);
        rv
    }

    pub fn update<W: LayoutElement>(&mut self, tile: &Tile<W>) {
        self.size = tile.tile_size();
        self.interactively_resizing_by_left_edge = tile
            .window()
            .interactive_resize_data()
            .is_some_and(|data| data.edges.contains(ResizeEdge::LEFT));
    }
}
