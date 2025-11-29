//! TEAM_006: Canvas2D module for 2D tiling layout.
//!
//! Canvas2D is an infinite 2D surface containing multiple [`Row`]s.
//! It replaces `Workspace` entirely.
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
//!
//! ## Module Structure
//!
//! ```text
//! canvas/
//! ├── mod.rs              - Core struct and accessors
//! ├── canvas_floating.rs  - Canvas2D floating integration methods
//! ├── navigation.rs       - Row/column focus navigation
//! ├── operations.rs       - Add/remove/find windows
//! ├── render.rs           - Rendering
//! └── floating/           - FloatingSpace module (TEAM_063)
//!     ├── mod.rs          - FloatingSpace struct, Data, core impl
//!     ├── operations.rs   - add/remove tile, movement
//!     ├── render.rs       - render elements, close animations
//!     └── resize.rs       - resize handling, presets
//! ```

// TEAM_063: Renamed canvas_floating to avoid conflict with floating/ submodule
mod canvas_floating;
mod navigation;
mod operations;
mod render;

// TEAM_063: FloatingSpace consolidated into canvas/floating/
pub mod floating;

use std::collections::BTreeMap;
use std::rc::Rc;

use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::row::{Row, RowRenderElement};
// TEAM_063: Import from new location
use floating::{FloatingSpace, FloatingSpaceRenderElement};
use super::tile::Tile;
use super::LayoutElement;
use super::Options;
// TEAM_060: Using RowId directly instead of WorkspaceId alias
use super::row_types::RowId;
use crate::animation::Clock;
use crate::niri_render_elements;

// TEAM_007: Canvas2D render element type
// TEAM_009: Added FloatingSpace render element
niri_render_elements! {
    Canvas2DRenderElement<R> => {
        Row = RowRenderElement<R>,
        Floating = FloatingSpaceRenderElement<R>,
    }
}

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
    pub(crate) rows: BTreeMap<i32, Row<W>>,

    /// Currently active row index.
    pub(crate) active_row_idx: i32,

    /// Previously active row index for back-and-forth navigation.
    /// TEAM_018: Added for previous row tracking
    pub(crate) previous_row_idx: i32,

    /// Floating windows layer.
    // TEAM_009: Integrated FloatingSpace
    pub(crate) floating: FloatingSpace<W>,

    /// Whether floating mode is active.
    pub(crate) floating_is_active: bool,

    /// Camera X position (horizontal scroll).
    pub(crate) camera_x: AnimatedValue,

    /// Camera Y position (vertical scroll).
    pub(crate) camera_y: AnimatedValue,

    /// The output this canvas is on.
    output: Option<Output>,

    /// View size for the canvas.
    pub(crate) view_size: Size<f64, Logical>,

    /// Working area for the canvas.
    pub(crate) working_area: Rectangle<f64, Logical>,

    /// Parent area (working area excluding struts).
    pub(crate) parent_area: Rectangle<f64, Logical>,

    /// Scale of the output.
    pub(crate) scale: f64,

    /// Clock for animations.
    pub(crate) clock: Clock,

    /// Layout options.
    pub(crate) options: Rc<Options>,
    
    /// TEAM_039: Counter for generating unique row IDs for rows
    /// TEAM_055: Renamed from workspace_id_counter to row_id_counter
    row_id_counter: u64,
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
        initial_workspace_id: RowId,
    ) -> Self {
        // Create origin row (row 0)
        let mut rows = BTreeMap::new();
        rows.insert(
            0,
            Row::new(
                0,
                initial_workspace_id,
                view_size,
                parent_area,
                scale,
                clock.clone(),
                options.clone(),
            ),
        );

        // TEAM_009: Create FloatingSpace
        let floating = FloatingSpace::new(
            view_size,
            working_area,
            scale,
            clock.clone(),
            options.clone(),
        );

        Self {
            rows,
            active_row_idx: 0,
            previous_row_idx: 0, // TEAM_018: Initialize previous row tracking
            floating,
            floating_is_active: false,
            camera_x: AnimatedValue::new(0.),
            camera_y: AnimatedValue::new(0.),
            output,
            view_size,
            working_area,
            parent_area,
            scale,
            clock,
            options,
            // TEAM_055: Renamed from workspace_id_counter to row_id_counter
            row_id_counter: initial_workspace_id.0,
        }
    }

    // =========================================================================
    // Basic Accessors
    // =========================================================================

    /// Returns the output this canvas is on.
    pub fn output(&self) -> Option<&Output> {
        self.output.as_ref()
    }

    /// Creates a tile from a window.
    /// TEAM_010: Added for Monitor.add_window() migration
    pub fn make_tile(&self, window: W) -> Tile<W> {
        Tile::new(
            window,
            self.view_size,
            self.scale,
            self.clock.clone(),
            self.options.clone(),
        )
    }

    /// Returns whether the canvas has any tiled windows.
    pub fn has_tiled_windows(&self) -> bool {
        self.rows.values().any(|row| !row.is_empty())
    }

    /// Returns whether the canvas has any floating windows.
    pub fn has_floating_windows(&self) -> bool {
        self.floating.tiles().next().is_some()
    }

    /// Returns whether floating mode is active.
    pub fn floating_is_active(&self) -> bool {
        self.floating_is_active
    }

    /// Returns a reference to the floating space.
    pub fn floating(&self) -> &FloatingSpace<W> {
        &self.floating
    }

    /// Returns a mutable reference to the floating space.
    pub fn floating_mut(&mut self) -> &mut FloatingSpace<W> {
        &mut self.floating
    }

    /// Returns the active row index.
    pub fn active_row_idx(&self) -> i32 {
        self.active_row_idx
    }

    /// Returns the current camera position for rendering.
    /// TEAM_007: Implemented camera offset support for render elements
    pub fn camera_position(&self) -> Point<f64, Logical> {
        Point::from((self.camera_x.current(), self.camera_y.current()))
    }

    /// Returns the active row, if any.
    pub fn active_row(&self) -> Option<&Row<W>> {
        self.rows.get(&self.active_row_idx)
    }

    /// Returns a mutable reference to the active row.
    pub fn active_row_mut(&mut self) -> Option<&mut Row<W>> {
        self.rows.get_mut(&self.active_row_idx)
    }

    /// TEAM_033: Returns a reference to a row by index.
    pub fn row(&self, idx: i32) -> Option<&Row<W>> {
        self.rows.get(&idx)
    }

    /// TEAM_033: Returns a mutable reference to a row by index.
    pub fn row_mut(&mut self, idx: i32) -> Option<&mut Row<W>> {
        self.rows.get_mut(&idx)
    }

    /// Returns an iterator over all rows.
    pub fn rows(&self) -> impl Iterator<Item = (i32, &Row<W>)> {
        self.rows.iter().map(|(&idx, row)| (idx, row))
    }

    /// Returns a mutable iterator over all rows with indices.
    /// TEAM_020: Added for workspace iteration migration
    /// TEAM_033: Updated to return (i32, &mut Row) tuples like rows()
    pub fn rows_mut(&mut self) -> impl Iterator<Item = (i32, &mut Row<W>)> + '_ {
        self.rows.iter_mut().map(|(&idx, row)| (idx, row))
    }

    // =========================================================================
    // Animation
    // =========================================================================

    /// Advances all animations.
    pub fn advance_animations(&mut self) {
        for row in self.rows.values_mut() {
            row.advance_animations();
        }
        // TEAM_009: Added floating animation
        self.floating.advance_animations();
    }

    /// Returns whether any animations are ongoing.
    pub fn are_animations_ongoing(&self) -> bool {
        self.rows.values().any(|row| row.are_animations_ongoing())
            || self.floating.are_animations_ongoing()
            || self.camera_x.is_animation_ongoing()
            || self.camera_y.is_animation_ongoing()
    }

    // =========================================================================
    // Golden Snapshot Testing
    // TEAM_010: Added for test compatibility after Monitor refactor
    // =========================================================================

    /// Creates a snapshot of the active row's layout state for golden testing.
    ///
    /// This delegates to the active row's snapshot() method to ensure golden
    /// tests produce the same output as they did when going through
    /// Workspace → ScrollingSpace.
    #[cfg(test)]
    pub fn snapshot(&self) -> crate::layout::snapshot::ScrollingSnapshot {
        if let Some(row) = self.active_row() {
            row.snapshot()
        } else {
            // Return empty snapshot if no active row
            use crate::layout::snapshot::{RectSnapshot, ScrollingSnapshot, SizeSnapshot};
            ScrollingSnapshot {
                columns: Vec::new(),
                active_column_idx: 0,
                view_offset: 0.0,
                working_area: RectSnapshot::from(self.working_area),
                view_size: SizeSnapshot::from(self.view_size),
                animations: Vec::new(),
            }
        }
    }

    /// Creates a full canvas snapshot including both tiled and floating state.
    ///
    /// This is used for golden tests that need to verify floating window behavior.
    #[cfg(test)]
    pub fn canvas_snapshot(&self) -> crate::layout::snapshot::CanvasSnapshot {
        crate::layout::snapshot::CanvasSnapshot {
            tiled: self.snapshot(),
            floating: self.floating.snapshot(),
            floating_is_active: self.floating_is_active,
        }
    }

    /// Remove a row by index and return it.
    /// TEAM_025: Implemented proper row removal with active row adjustment
    /// TEAM_055: Modified to return the removed row
    pub fn remove_row(&mut self, row_idx: i32) -> Option<Row<W>> {
        // Remove the row and get it back
        let removed = self.rows.remove(&row_idx);
        
        // Adjust active_row_idx if necessary
        if self.active_row_idx == row_idx {
            // Find the next available row to activate
            if let Some((&next_idx, _)) = self.rows.iter().next() {
                self.active_row_idx = next_idx;
            } else {
                // No rows left, reset to origin
                self.active_row_idx = 0;
            }
        } else if self.active_row_idx > row_idx {
            // Active row was after the removed one, adjust index
            // Find the next available row at or after the current active index
            if let Some((&next_idx, _)) = self.rows.range(self.active_row_idx..).next() {
                self.active_row_idx = next_idx;
            } else {
                // No rows after current index, find the highest available
                if let Some((&highest_idx, _)) = self.rows.iter().next_back() {
                    self.active_row_idx = highest_idx;
                } else {
                    self.active_row_idx = 0;
                }
            }
        }
        
        removed
    }
    
    /// Insert a row at a specific index.
    /// TEAM_055: Added for workspace transfer between monitors
    pub fn insert_row(&mut self, row_idx: i32, row: Row<W>) {
        self.rows.insert(row_idx, row);
    }
}
