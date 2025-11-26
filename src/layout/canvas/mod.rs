//! TEAM_006: Canvas2D module for 2D tiling layout.
//!
//! Canvas2D is an infinite 2D surface containing multiple [`Row`]s.
//! It replaces `Workspace` for 2D mode.
//!
//! ## Structure
//!
//! ```text
//! Canvas2D
//! ├── rows: BTreeMap<i32, Row<W>>   // Sparse row storage
//! ├── active_row_idx: i32           // Current row
//! ├── floating: FloatingSpace<W>    // Floating windows
//! ├── camera_x: AnimatedValue       // Horizontal camera position
//! └── camera_y: AnimatedValue       // Vertical camera position
//! ```
//!
//! ## Row Indexing
//!
//! - Row `0` is the origin (where windows open by default)
//! - Negative indices are rows above the origin
//! - Positive indices are rows below the origin

use std::collections::BTreeMap;
use std::rc::Rc;

use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::row::Row;
use super::tile::Tile;
use super::{LayoutElement, Options};
use crate::animation::Clock;

// TEAM_006: FloatingSpace temporarily removed until Row has full functionality.
// Will be re-added when Canvas2D is more complete.

/// A 2D infinite canvas containing multiple rows of columns.
///
/// This is the main layout structure for 2D tiling mode, replacing
/// `Workspace` which only supports a single row.
#[derive(Debug)]
pub struct Canvas2D<W: LayoutElement> {
    /// Rows indexed by row number.
    ///
    /// - `0` = origin row (default for new windows)
    /// - Negative = rows above origin
    /// - Positive = rows below origin
    rows: BTreeMap<i32, Row<W>>,

    /// Currently active row index.
    active_row_idx: i32,

    // TEAM_006: Floating layer will be added when Row is more complete
    // floating: FloatingSpace<W>,
    // floating_is_active: bool,

    /// Camera X position (horizontal scroll).
    camera_x: AnimatedValue,

    /// Camera Y position (vertical scroll).
    camera_y: AnimatedValue,

    /// The output this canvas is on.
    output: Option<Output>,

    /// View size for the canvas.
    view_size: Size<f64, Logical>,

    /// Working area for the canvas.
    working_area: Rectangle<f64, Logical>,

    /// Parent area (working area excluding struts).
    parent_area: Rectangle<f64, Logical>,

    /// Scale of the output.
    scale: f64,

    /// Clock for animations.
    clock: Clock,

    /// Layout options.
    options: Rc<Options>,
}

impl<W: LayoutElement> Canvas2D<W> {
    /// Creates a new empty canvas.
    pub fn new(
        output: Option<Output>,
        view_size: Size<f64, Logical>,
        parent_area: Rectangle<f64, Logical>,
        working_area: Rectangle<f64, Logical>,
        scale: f64,
        clock: Clock,
        options: Rc<Options>,
    ) -> Self {
        // Create origin row (row 0)
        let mut rows = BTreeMap::new();
        rows.insert(
            0,
            Row::new(0, view_size, parent_area, scale, clock.clone(), options.clone()),
        );

        Self {
            rows,
            active_row_idx: 0,
            camera_x: AnimatedValue::new(0.),
            camera_y: AnimatedValue::new(0.),
            output,
            view_size,
            working_area,
            parent_area,
            scale,
            clock,
            options,
        }
    }

    /// Returns the output this canvas is on.
    pub fn output(&self) -> Option<&Output> {
        self.output.as_ref()
    }

    /// Returns whether the canvas has any tiled windows.
    pub fn has_tiled_windows(&self) -> bool {
        self.rows.values().any(|row| !row.is_empty())
    }

    // TEAM_006: floating_is_active will be added when FloatingSpace is integrated

    /// Returns the active row index.
    pub fn active_row_idx(&self) -> i32 {
        self.active_row_idx
    }

    /// Returns the active row, if any.
    pub fn active_row(&self) -> Option<&Row<W>> {
        self.rows.get(&self.active_row_idx)
    }

    /// Returns a mutable reference to the active row.
    pub fn active_row_mut(&mut self) -> Option<&mut Row<W>> {
        self.rows.get_mut(&self.active_row_idx)
    }

    /// Returns an iterator over all rows.
    pub fn rows(&self) -> impl Iterator<Item = (i32, &Row<W>)> {
        self.rows.iter().map(|(&idx, row)| (idx, row))
    }

    /// Returns the current camera position.
    pub fn camera_position(&self) -> Point<f64, Logical> {
        Point::from((self.camera_x.current(), self.camera_y.current()))
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Focuses the row above the current row.
    pub fn focus_up(&mut self) -> bool {
        let target_row = self.active_row_idx - 1;
        self.focus_row(target_row)
    }

    /// Focuses the row below the current row.
    pub fn focus_down(&mut self) -> bool {
        let target_row = self.active_row_idx + 1;
        self.focus_row(target_row)
    }

    /// Focuses the column to the left in the active row.
    pub fn focus_left(&mut self) -> bool {
        if let Some(row) = self.active_row_mut() {
            row.focus_left()
        } else {
            false
        }
    }

    /// Focuses the column to the right in the active row.
    pub fn focus_right(&mut self) -> bool {
        if let Some(row) = self.active_row_mut() {
            row.focus_right()
        } else {
            false
        }
    }

    /// Focuses a specific row.
    fn focus_row(&mut self, target_row: i32) -> bool {
        if !self.rows.contains_key(&target_row) {
            return false;
        }

        // Try to maintain the same column index
        let col_idx = self
            .active_row()
            .map(|r| r.active_column_idx())
            .unwrap_or(0);

        self.active_row_idx = target_row;

        // Focus the same column index (or the last one if it doesn't exist)
        if let Some(row) = self.active_row_mut() {
            let max_col = row.column_count().saturating_sub(1);
            row.focus_column(col_idx.min(max_col));
        }

        self.update_camera_y();
        true
    }

    /// Updates the camera Y position to follow the active row.
    fn update_camera_y(&mut self) {
        if let Some(row) = self.active_row() {
            let target_y = row.y_offset();
            // TODO: Use animation config from options
            self.camera_y = AnimatedValue::new(target_y);
        }
    }

    // =========================================================================
    // Row Management
    // =========================================================================

    /// Creates a new row at the specified index if it doesn't exist.
    pub fn ensure_row(&mut self, row_idx: i32) -> &mut Row<W> {
        self.rows.entry(row_idx).or_insert_with(|| {
            Row::new(
                row_idx,
                self.view_size,
                self.parent_area,
                self.scale,
                self.clock.clone(),
                self.options.clone(),
            )
        })
    }

    /// Removes empty rows (except row 0 which is always kept).
    pub fn cleanup_empty_rows(&mut self) {
        self.rows.retain(|&idx, row| idx == 0 || !row.is_empty());
    }

    // =========================================================================
    // Animation
    // =========================================================================

    /// Advances all animations.
    pub fn advance_animations(&mut self) {
        for row in self.rows.values_mut() {
            row.advance_animations();
        }
        // TEAM_006: self.floating.advance_animations() will be added later
    }

    /// Returns whether any animations are ongoing.
    pub fn are_animations_ongoing(&self) -> bool {
        self.rows.values().any(|row| row.are_animations_ongoing())
            // TEAM_006: || self.floating.are_animations_ongoing()
            || self.camera_x.is_animation_ongoing()
            || self.camera_y.is_animation_ongoing()
    }

    // =========================================================================
    // Tiles Query
    // =========================================================================

    /// Returns all tiles with their render positions.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_ {
        let camera_offset = self.camera_position();
        self.rows.values().flat_map(move |row| {
            row.tiles_with_render_positions()
                .map(move |(tile, mut pos, is_active)| {
                    pos.x -= camera_offset.x;
                    pos.y -= camera_offset.y;
                    (tile, pos, is_active)
                })
        })
    }
}

// TODO(TEAM_006): Integrate FloatingSpace (after Row is complete)
// TODO(TEAM_006): Add add_window that routes to correct row
// TODO(TEAM_006): Add remove_window that finds window across rows
// TODO(TEAM_006): Animate camera_y when changing rows (current is instant)
// TODO(TEAM_006): Add render_elements method
// See docs/2d-canvas-plan/TODO.md for full list
