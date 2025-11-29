// TEAM_063: FloatingSpace consolidated into canvas/floating/
//!
//! Space for floating windows.
//!
//! ## Module Structure
//!
//! ```text
//! canvas/floating/
//! ├── mod.rs        - FloatingSpace struct, Data, core impl
//! ├── operations.rs - add/remove tile, movement
//! ├── render.rs     - render elements, close animations
//! └── resize.rs     - resize handling, presets
//! ```

mod operations;
mod render;
mod resize;

use std::iter::zip;
use std::rc::Rc;

use niri_config::RelativeTo;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::animation::Clock;
use crate::layout::elements::closing_window::ClosingWindow;
use crate::layout::tile::Tile;
use crate::layout::types::InteractiveResize;
use crate::layout::{LayoutElement, Options, SizeFrac};

// Re-export render element type
pub use render::FloatingSpaceRenderElement;

/// By how many logical pixels the directional move commands move floating windows.
pub const DIRECTIONAL_MOVE_PX: f64 = 50.;

/// Space for floating windows.
#[derive(Debug)]
pub struct FloatingSpace<W: LayoutElement> {
    /// Tiles in top-to-bottom order.
    pub(crate) tiles: Vec<Tile<W>>,

    /// Extra per-tile data.
    pub(crate) data: Vec<Data>,

    /// Id of the active window.
    ///
    /// The active window is not necessarily the topmost window. Focus-follows-mouse should
    /// activate a window, but not bring it to the top, because that's very annoying.
    ///
    /// This is always set to `Some()` when `tiles` isn't empty.
    pub(crate) active_window_id: Option<W::Id>,

    /// Ongoing interactive resize.
    pub(crate) interactive_resize: Option<InteractiveResize<W>>,

    /// Windows in the closing animation.
    pub(crate) closing_windows: Vec<ClosingWindow>,

    /// View size for this space.
    pub(crate) view_size: Size<f64, Logical>,

    /// Working area for this space.
    pub(crate) working_area: Rectangle<f64, Logical>,

    /// Scale of the output the space is on (and rounds its sizes to).
    pub(crate) scale: f64,

    /// Clock for driving animations.
    pub(crate) clock: Clock,

    /// Configurable properties of the layout.
    pub(crate) options: Rc<Options>,
}

/// Extra per-tile data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Data {
    /// Position relative to the working area.
    pub(crate) pos: Point<f64, SizeFrac>,

    /// Cached position in logical coordinates.
    ///
    /// Not rounded to physical pixels.
    pub(crate) logical_pos: Point<f64, Logical>,

    /// Cached actual size of the tile.
    pub(crate) size: Size<f64, Logical>,

    /// Working area used for conversions.
    pub(crate) working_area: Rectangle<f64, Logical>,
}

impl Data {
    pub fn new<W: LayoutElement>(
        working_area: Rectangle<f64, Logical>,
        tile: &Tile<W>,
        logical_pos: Point<f64, Logical>,
    ) -> Self {
        let mut rv = Self {
            pos: Point::default(),
            logical_pos: Point::default(),
            size: Size::default(),
            working_area,
        };
        rv.update(tile);
        rv.set_logical_pos(logical_pos);
        rv
    }

    pub fn scale_by_working_area(
        area: Rectangle<f64, Logical>,
        pos: Point<f64, SizeFrac>,
    ) -> Point<f64, Logical> {
        let mut logical_pos = Point::from((pos.x, pos.y));
        logical_pos.x *= area.size.w;
        logical_pos.y *= area.size.h;
        logical_pos += area.loc;
        logical_pos
    }

    pub fn logical_to_size_frac_in_working_area(
        area: Rectangle<f64, Logical>,
        logical_pos: Point<f64, Logical>,
    ) -> Point<f64, SizeFrac> {
        let pos = logical_pos - area.loc;
        let mut pos = Point::from((pos.x, pos.y));
        pos.x /= f64::max(area.size.w, 1.0);
        pos.y /= f64::max(area.size.h, 1.0);
        pos
    }

    fn recompute_logical_pos(&mut self) {
        let mut logical_pos = Self::scale_by_working_area(self.working_area, self.pos);

        // Make sure the window doesn't go too much off-screen. Numbers taken from Mutter.
        let min_on_screen_hor = f64::clamp(self.size.w / 4., 10., 75.);
        let min_on_screen_ver = f64::clamp(self.size.h / 4., 10., 75.);
        let max_off_screen_hor = f64::max(0., self.size.w - min_on_screen_hor);
        let max_off_screen_ver = f64::max(0., self.size.h - min_on_screen_ver);

        logical_pos -= self.working_area.loc;
        logical_pos.x = f64::max(logical_pos.x, -max_off_screen_hor);
        logical_pos.y = f64::max(logical_pos.y, -max_off_screen_ver);
        logical_pos.x = f64::min(
            logical_pos.x,
            self.working_area.size.w - self.size.w + max_off_screen_hor,
        );
        logical_pos.y = f64::min(
            logical_pos.y,
            self.working_area.size.h - self.size.h + max_off_screen_ver,
        );
        logical_pos += self.working_area.loc;

        self.logical_pos = logical_pos;
    }

    pub fn update_config(&mut self, working_area: Rectangle<f64, Logical>) {
        if self.working_area == working_area {
            return;
        }

        self.working_area = working_area;
        self.recompute_logical_pos();
    }

    pub fn update<W: LayoutElement>(&mut self, tile: &Tile<W>) {
        let size = tile.tile_size();
        if self.size == size {
            return;
        }

        self.size = size;
        self.recompute_logical_pos();
    }

    pub fn set_logical_pos(&mut self, logical_pos: Point<f64, Logical>) {
        self.pos = Self::logical_to_size_frac_in_working_area(self.working_area, logical_pos);

        // This will clamp the logical position to the current working area.
        self.recompute_logical_pos();
    }

    pub fn center(&self) -> Point<f64, Logical> {
        self.logical_pos + self.size.downscale(2.)
    }

    #[cfg(test)]
    fn verify_invariants(&self) {
        let mut temp = *self;
        temp.recompute_logical_pos();
        assert_eq!(
            self.logical_pos, temp.logical_pos,
            "cached logical pos must be up to date"
        );
    }
}

impl<W: LayoutElement> FloatingSpace<W> {
    pub fn new(
        view_size: Size<f64, Logical>,
        working_area: Rectangle<f64, Logical>,
        scale: f64,
        clock: Clock,
        options: Rc<Options>,
    ) -> Self {
        Self {
            tiles: Vec::new(),
            data: Vec::new(),
            active_window_id: None,
            interactive_resize: None,
            closing_windows: Vec::new(),
            view_size,
            working_area,
            scale,
            clock,
            options,
        }
    }

    pub fn update_config(
        &mut self,
        view_size: Size<f64, Logical>,
        working_area: Rectangle<f64, Logical>,
        scale: f64,
        options: Rc<Options>,
    ) {
        for (tile, data) in zip(&mut self.tiles, &mut self.data) {
            tile.update_config(view_size, scale, options.clone());
            data.update(tile);
            data.update_config(working_area);
        }

        self.view_size = view_size;
        self.working_area = working_area;
        self.scale = scale;
        self.options = options;
    }

    pub fn update_shaders(&mut self) {
        for tile in &mut self.tiles {
            tile.update_shaders();
        }
    }

    pub fn advance_animations(&mut self) {
        for tile in &mut self.tiles {
            tile.advance_animations();
        }

        self.closing_windows.retain_mut(|closing| {
            closing.advance_animations();
            closing.are_animations_ongoing()
        });
    }

    pub fn are_animations_ongoing(&self) -> bool {
        self.tiles.iter().any(Tile::are_animations_ongoing) || !self.closing_windows.is_empty()
    }

    pub fn are_transitions_ongoing(&self) -> bool {
        self.tiles.iter().any(Tile::are_transitions_ongoing) || !self.closing_windows.is_empty()
    }

    // =========================================================================
    // Tile Accessors
    // =========================================================================

    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.tiles.iter()
    }

    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.tiles.iter_mut()
    }

    pub fn tiles_with_offsets(&self) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>)> + '_ {
        let offsets = self.data.iter().map(|d| d.logical_pos);
        zip(&self.tiles, offsets)
    }

    pub fn tiles_with_offsets_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut Tile<W>, Point<f64, Logical>)> + '_ {
        let offsets = self.data.iter().map(|d| d.logical_pos);
        zip(&mut self.tiles, offsets)
    }

    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>)> {
        let scale = self.scale;
        self.tiles_with_offsets().map(move |(tile, offset)| {
            let pos = offset + tile.render_offset();
            // Round to physical pixels.
            let pos = pos.to_physical_precise_round(scale).to_logical(scale);
            (tile, pos)
        })
    }

    pub fn tiles_with_render_positions_mut(
        &mut self,
        round: bool,
    ) -> impl Iterator<Item = (&mut Tile<W>, Point<f64, Logical>)> {
        let scale = self.scale;
        self.tiles_with_offsets_mut().map(move |(tile, offset)| {
            let mut pos = offset + tile.render_offset();
            // Round to physical pixels.
            if round {
                pos = pos.to_physical_precise_round(scale).to_logical(scale);
            }
            (tile, pos)
        })
    }

    // =========================================================================
    // Window Queries
    // =========================================================================

    pub(crate) fn idx_of(&self, id: &W::Id) -> Option<usize> {
        self.tiles.iter().position(|tile| tile.window().id() == id)
    }

    pub(crate) fn contains(&self, id: &W::Id) -> bool {
        self.idx_of(id).is_some()
    }

    pub fn active_window(&self) -> Option<&W> {
        let id = self.active_window_id.as_ref()?;
        self.tiles
            .iter()
            .find(|tile| tile.window().id() == id)
            .map(Tile::window)
    }

    pub fn active_window_mut(&mut self) -> Option<&mut W> {
        let id = self.active_window_id.as_ref()?;
        self.tiles
            .iter_mut()
            .find(|tile| tile.window().id() == id)
            .map(Tile::window_mut)
    }

    pub fn has_window(&self, id: &W::Id) -> bool {
        self.tiles.iter().any(|tile| tile.window().id() == id)
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    // =========================================================================
    // Focus/Activation
    // =========================================================================

    pub fn activate_window_without_raising(&mut self, id: &W::Id) -> bool {
        if !self.contains(id) {
            return false;
        }

        self.active_window_id = Some(id.clone());
        true
    }

    pub fn activate_window(&mut self, id: &W::Id) -> bool {
        let Some(idx) = self.idx_of(id) else {
            return false;
        };

        self.raise_window(idx, 0);
        self.active_window_id = Some(id.clone());
        self.bring_up_descendants_of(0);

        true
    }

    pub(crate) fn raise_window(&mut self, from_idx: usize, to_idx: usize) {
        assert!(to_idx <= from_idx);

        let tile = self.tiles.remove(from_idx);
        let data = self.data.remove(from_idx);
        self.tiles.insert(to_idx, tile);
        self.data.insert(to_idx, data);
    }

    fn focus_directional(
        &mut self,
        distance: impl Fn(Point<f64, Logical>, Point<f64, Logical>) -> f64,
    ) -> bool {
        let Some(active_id) = &self.active_window_id else {
            return false;
        };
        let active_idx = self.idx_of(active_id).unwrap();
        let center = self.data[active_idx].center();

        let result = zip(&self.tiles, &self.data)
            .filter(|(tile, _)| tile.window().id() != active_id)
            .map(|(tile, data)| (tile, distance(center, data.center())))
            .filter(|(_, dist)| *dist > 0.)
            .min_by(|(_, dist_a), (_, dist_b)| f64::total_cmp(dist_a, dist_b));
        if let Some((tile, _)) = result {
            let id = tile.window().id().clone();
            self.activate_window(&id);
            true
        } else {
            false
        }
    }

    pub fn focus_left(&mut self) -> bool {
        self.focus_directional(|focus, other| focus.x - other.x)
    }

    pub fn focus_right(&mut self) -> bool {
        self.focus_directional(|focus, other| other.x - focus.x)
    }

    pub fn focus_up(&mut self) -> bool {
        self.focus_directional(|focus, other| focus.y - other.y)
    }

    pub fn focus_down(&mut self) -> bool {
        self.focus_directional(|focus, other| other.y - focus.y)
    }

    pub fn focus_leftmost(&mut self) {
        let result = self
            .tiles_with_offsets()
            .min_by(|(_, pos_a), (_, pos_b)| f64::total_cmp(&pos_a.x, &pos_b.x));
        if let Some((tile, _)) = result {
            let id = tile.window().id().clone();
            self.activate_window(&id);
        }
    }

    pub fn focus_rightmost(&mut self) {
        let result = self
            .tiles_with_offsets()
            .max_by(|(_, pos_a), (_, pos_b)| f64::total_cmp(&pos_a.x, &pos_b.x));
        if let Some((tile, _)) = result {
            let id = tile.window().id().clone();
            self.activate_window(&id);
        }
    }

    pub fn focus_topmost(&mut self) {
        let result = self
            .tiles_with_offsets()
            .min_by(|(_, pos_a), (_, pos_b)| f64::total_cmp(&pos_a.y, &pos_b.y));
        if let Some((tile, _)) = result {
            let id = tile.window().id().clone();
            self.activate_window(&id);
        }
    }

    pub fn focus_bottommost(&mut self) {
        let result = self
            .tiles_with_offsets()
            .max_by(|(_, pos_a), (_, pos_b)| f64::total_cmp(&pos_a.y, &pos_b.y));
        if let Some((tile, _)) = result {
            let id = tile.window().id().clone();
            self.activate_window(&id);
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    pub fn clamp_within_working_area(
        &self,
        pos: Point<f64, Logical>,
        size: Size<f64, Logical>,
    ) -> Point<f64, Logical> {
        use crate::utils::clamp_preferring_top_left_in_area;
        let mut rect = Rectangle::new(pos, size);
        clamp_preferring_top_left_in_area(self.working_area, &mut rect);
        rect.loc
    }

    pub fn scale_by_working_area(&self, pos: Point<f64, SizeFrac>) -> Point<f64, Logical> {
        Data::scale_by_working_area(self.working_area, pos)
    }

    pub fn logical_to_size_frac(&self, logical_pos: Point<f64, Logical>) -> Point<f64, SizeFrac> {
        Data::logical_to_size_frac_in_working_area(self.working_area, logical_pos)
    }

    pub fn stored_or_default_tile_pos(&self, tile: &Tile<W>) -> Option<Point<f64, Logical>> {
        let pos = tile.floating_pos.map(|pos| self.scale_by_working_area(pos));
        pos.or_else(|| {
            tile.window().rules().default_floating_position.map(|pos| {
                let relative_to = pos.relative_to;
                let size = tile.tile_size();
                let area = self.working_area;

                let mut pos = Point::from((pos.x.0, pos.y.0));
                if relative_to == RelativeTo::TopRight
                    || relative_to == RelativeTo::BottomRight
                    || relative_to == RelativeTo::Right
                {
                    pos.x = area.size.w - size.w - pos.x;
                }
                if relative_to == RelativeTo::BottomLeft
                    || relative_to == RelativeTo::BottomRight
                    || relative_to == RelativeTo::Bottom
                {
                    pos.y = area.size.h - size.h - pos.y;
                }
                if relative_to == RelativeTo::Top || relative_to == RelativeTo::Bottom {
                    pos.x += area.size.w / 2.0 - size.w / 2.0
                }
                if relative_to == RelativeTo::Left || relative_to == RelativeTo::Right {
                    pos.y += area.size.h / 2.0 - size.h / 2.0
                }

                pos + self.working_area.loc
            })
        })
    }

    pub fn working_area(&self) -> Rectangle<f64, Logical> {
        self.working_area
    }

    #[cfg(test)]
    pub fn view_size(&self) -> Size<f64, Logical> {
        self.view_size
    }

    #[cfg(test)]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    #[cfg(test)]
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    #[cfg(test)]
    pub fn options(&self) -> &Rc<Options> {
        &self.options
    }

    // =========================================================================
    // Test Helpers
    // =========================================================================

    #[cfg(test)]
    pub fn verify_invariants(&self) {
        use crate::layout::SizingMode;

        assert!(self.scale > 0.);
        assert!(self.scale.is_finite());
        assert_eq!(self.tiles.len(), self.data.len());

        for (i, (tile, data)) in zip(&self.tiles, &self.data).enumerate() {
            assert!(Rc::ptr_eq(&self.options, &tile.options));
            assert_eq!(self.view_size, tile.view_size());
            assert_eq!(self.clock, tile.clock);
            assert_eq!(self.scale, tile.scale());
            tile.verify_invariants();

            if let Some(idx) = tile.floating_preset_width_idx {
                assert!(idx < self.options.layout.preset_column_widths.len());
            }
            if let Some(idx) = tile.floating_preset_height_idx {
                assert!(idx < self.options.layout.preset_window_heights.len());
            }

            assert_eq!(
                tile.window().pending_sizing_mode(),
                SizingMode::Normal,
                "floating windows cannot be maximized or fullscreen"
            );

            data.verify_invariants();

            let mut data2 = *data;
            data2.update(tile);
            data2.update_config(self.working_area);
            assert_eq!(data, &data2, "tile data must be up to date");

            for tile_below in &self.tiles[i + 1..] {
                assert!(
                    !tile_below.window().is_child_of(tile.window()),
                    "children must be stacked above parents"
                );
            }
        }

        if let Some(id) = &self.active_window_id {
            assert!(!self.tiles.is_empty());
            assert!(self.contains(id), "active window must be present in tiles");
        } else {
            assert!(self.tiles.is_empty());
        }

        if let Some(resize) = &self.interactive_resize {
            assert!(
                self.contains(&resize.window),
                "interactive resize window must be present in tiles"
            );
        }
    }

    /// Create a snapshot of the floating layout for golden testing.
    #[cfg(test)]
    pub fn snapshot(&self) -> crate::layout::snapshot::FloatingSnapshot {
        use crate::layout::snapshot::{FloatingSnapshot, FloatingWindowSnapshot};

        let windows: Vec<FloatingWindowSnapshot> = self
            .tiles
            .iter()
            .zip(self.data.iter())
            .map(|(tile, data)| {
                let size = tile.tile_size();
                FloatingWindowSnapshot {
                    x: data.logical_pos.x,
                    y: data.logical_pos.y,
                    width: size.w,
                    height: size.h,
                }
            })
            .collect();

        let active_window_idx = self.active_window_id.as_ref().and_then(|active_id| {
            self.tiles.iter().position(|tile| tile.window().id() == active_id)
        });

        FloatingSnapshot {
            windows,
            active_window_idx,
        }
    }
}
