//! TEAM_006: Row module for 2D canvas layout.
//!
//! A Row is a horizontal strip of columns — the fundamental horizontal
//! layout primitive for the 2D canvas.
//!
//! ## Design (Option B: Clean Slate)
//!
//! Per Rule 0 (Quality > Speed), Row owns its columns directly rather than
//! wrapping ScrollingSpace. This avoids indirection and technical debt.
//!
//! ## Structure
//!
//! ```text
//! Row
//! ├── columns: Vec<Column<W>>     // Owned directly
//! ├── view_offset_x: AnimatedValue // Horizontal scroll
//! ├── row_index: i32              // Position in canvas
//! └── y_offset: f64               // Computed from row_index
//! ```
//!
//! ## Migration Status
//!
//! This module is being built as a clean-slate implementation based on
//! ScrollingSpace. See TEAM_006 notes for progress.

// TEAM_006: Row implementation in progress.
// This is a placeholder — the full implementation will mirror ScrollingSpace
// but with row-specific additions (row_index, y_offset).
//
// The full implementation requires porting ~3800 lines from scrolling.rs.
// This is being done incrementally to ensure correctness.

use std::iter::zip;
use std::rc::Rc;

use niri_config::Struts;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::closing_window::ClosingWindow;
use super::column::Column;
use super::tile::Tile;
use super::types::InteractiveResize;
use super::{LayoutElement, Options};
use crate::animation::Clock;

/// Amount of touchpad movement to scroll the view for the width of one working area.
const VIEW_GESTURE_WORKING_AREA_MOVEMENT: f64 = 1200.;

/// Extra per-column data.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ColumnData {
    /// Cached actual column width.
    width: f64,
}

/// A horizontal row of columns in the 2D canvas.
///
/// Row is the core horizontal layout primitive. It manages columns,
/// horizontal scrolling, and focus within the row.
#[derive(Debug)]
pub struct Row<W: LayoutElement> {
    // =========================================================================
    // Row-specific fields (not in ScrollingSpace)
    // =========================================================================
    
    /// Row index in the canvas.
    ///
    /// - `0` = origin row (where windows open by default)
    /// - Negative = rows above origin
    /// - Positive = rows below origin
    row_index: i32,

    /// Y offset from canvas origin, computed as `row_index * row_height`.
    y_offset: f64,

    // =========================================================================
    // Column management (from ScrollingSpace)
    // =========================================================================
    
    /// Columns of windows in this row.
    columns: Vec<Column<W>>,

    /// Extra per-column data (cached widths).
    data: Vec<ColumnData>,

    /// Index of the currently active column.
    active_column_idx: usize,

    /// Ongoing interactive resize.
    interactive_resize: Option<InteractiveResize<W>>,

    // =========================================================================
    // View/scroll state (from ScrollingSpace, renamed for clarity)
    // =========================================================================
    
    /// Horizontal view offset from the active column.
    view_offset_x: AnimatedValue,

    /// Whether to activate the previous column upon removal.
    activate_prev_column_on_removal: Option<f64>,

    /// View offset to restore after unfullscreening/unmaximizing.
    view_offset_to_restore: Option<f64>,

    /// Windows in the closing animation.
    closing_windows: Vec<ClosingWindow>,

    // =========================================================================
    // Layout configuration (from ScrollingSpace)
    // =========================================================================
    
    /// View size for this row.
    view_size: Size<f64, Logical>,

    /// Working area (accounts for struts and exclusive zones).
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

impl<W: LayoutElement> Row<W> {
    /// Creates a new empty row at the specified index.
    pub fn new(
        row_index: i32,
        view_size: Size<f64, Logical>,
        parent_area: Rectangle<f64, Logical>,
        scale: f64,
        clock: Clock,
        options: Rc<Options>,
    ) -> Self {
        let working_area = compute_working_area(parent_area, scale, options.layout.struts);
        let y_offset = row_index as f64 * view_size.h;

        Self {
            // Row-specific
            row_index,
            y_offset,
            
            // Column management
            columns: Vec::new(),
            data: Vec::new(),
            active_column_idx: 0,
            interactive_resize: None,
            
            // View state
            view_offset_x: AnimatedValue::new(0.),
            activate_prev_column_on_removal: None,
            view_offset_to_restore: None,
            closing_windows: Vec::new(),
            
            // Configuration
            view_size,
            working_area,
            parent_area,
            scale,
            clock,
            options,
        }
    }

    // =========================================================================
    // Row-specific accessors
    // =========================================================================

    /// Returns the row index.
    pub fn row_index(&self) -> i32 {
        self.row_index
    }

    /// Returns the Y offset from canvas origin.
    pub fn y_offset(&self) -> f64 {
        self.y_offset
    }

    /// Returns the row height (same as view height).
    pub fn row_height(&self) -> f64 {
        self.view_size.h
    }

    // =========================================================================
    // Basic queries (from ScrollingSpace)
    // =========================================================================

    /// Returns whether this row has no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns an iterator over the columns.
    pub fn columns(&self) -> impl Iterator<Item = &Column<W>> {
        self.columns.iter()
    }

    /// Returns the active column index.
    pub fn active_column_idx(&self) -> usize {
        self.active_column_idx
    }

    /// Returns the active column, if any.
    pub fn active_column(&self) -> Option<&Column<W>> {
        self.columns.get(self.active_column_idx)
    }

    /// Returns a mutable reference to the active column.
    pub fn active_column_mut(&mut self) -> Option<&mut Column<W>> {
        self.columns.get_mut(self.active_column_idx)
    }

    /// Returns the current horizontal view offset.
    pub fn view_offset_x(&self) -> f64 {
        self.view_offset_x.current()
    }

    // =========================================================================
    // Configuration
    // =========================================================================

    /// Updates configuration when output changes.
    pub fn update_config(
        &mut self,
        view_size: Size<f64, Logical>,
        parent_area: Rectangle<f64, Logical>,
        scale: f64,
        options: Rc<Options>,
    ) {
        let working_area = compute_working_area(parent_area, scale, options.layout.struts);

        for (column, data) in zip(&mut self.columns, &mut self.data) {
            column.update_config(view_size, working_area, parent_area, scale, options.clone());
            data.update(column);
        }

        self.view_size = view_size;
        self.working_area = working_area;
        self.parent_area = parent_area;
        self.scale = scale;
        self.options = options;
        self.y_offset = self.row_index as f64 * view_size.h;

        // Apply always-center and such right away.
        if !self.columns.is_empty() && !self.view_offset_x.is_gesture() {
            self.animate_view_offset_to_column(None, self.active_column_idx, None);
        }
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Focuses the column to the left.
    pub fn focus_left(&mut self) -> bool {
        if self.columns.is_empty() || self.active_column_idx == 0 {
            return false;
        }

        self.activate_prev_column_on_removal = None;
        self.view_offset_to_restore = None;

        let new_idx = self.active_column_idx - 1;
        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);
        true
    }

    /// Focuses the column to the right.
    pub fn focus_right(&mut self) -> bool {
        if self.columns.is_empty() || self.active_column_idx + 1 >= self.columns.len() {
            return false;
        }

        self.activate_prev_column_on_removal = None;
        self.view_offset_to_restore = None;

        let new_idx = self.active_column_idx + 1;
        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);
        true
    }

    /// Focuses a specific column by index.
    pub fn focus_column(&mut self, idx: usize) {
        if idx >= self.columns.len() {
            return;
        }

        if idx != self.active_column_idx {
            self.activate_prev_column_on_removal = None;
            self.view_offset_to_restore = None;
        }

        self.active_column_idx = idx;
        self.animate_view_offset_to_column(None, idx, None);
    }

    // =========================================================================
    // Animation
    // =========================================================================

    /// Advances all animations.
    pub fn advance_animations(&mut self) {
        // Advance view offset animation
        if let AnimatedValue::Animation(anim) = &self.view_offset_x {
            if anim.is_done() {
                self.view_offset_x = AnimatedValue::Static(anim.to());
            }
        }

        // Advance gesture animations
        if let AnimatedValue::Gesture(gesture) = &mut self.view_offset_x {
            if let Some(anim) = &gesture.animation {
                if anim.is_done() {
                    gesture.current_view_offset += anim.to();
                    gesture.animation = None;
                }
            }
        }

        // Advance column animations
        for col in &mut self.columns {
            col.advance_animations();
        }

        // Advance closing window animations
        for win in &mut self.closing_windows {
            win.advance_animations();
        }
        self.closing_windows.retain(|win| win.are_animations_ongoing());
    }

    /// Returns whether any animations are ongoing.
    pub fn are_animations_ongoing(&self) -> bool {
        self.view_offset_x.is_animation_ongoing()
            || self.columns.iter().any(|col| col.are_animations_ongoing())
            || !self.closing_windows.is_empty()
    }

    // =========================================================================
    // Internal: View offset calculation
    // =========================================================================

    /// Animates the view offset to show the specified column.
    fn animate_view_offset_to_column(
        &mut self,
        _prev_col_x: Option<f64>,
        col_idx: usize,
        _new_col_width: Option<f64>,
    ) {
        // TODO(TEAM_006): Port full animate_view_offset_to_column from ScrollingSpace
        // Current: simple static offset. Need: animated transitions, centering logic
        if !self.columns.is_empty() {
            let col_x = self.column_x(col_idx);
            self.view_offset_x = AnimatedValue::new(-col_x);
        }
    }

    /// Returns the X position of the column at the given index.
    fn column_x(&self, idx: usize) -> f64 {
        let gaps = self.options.layout.gaps;
        let mut x = 0.;
        for i in 0..idx {
            if let Some(data) = self.data.get(i) {
                x += data.width + gaps;
            }
        }
        x
    }

    // =========================================================================
    // Tiles query
    // =========================================================================

    /// Returns tiles with their render positions, offset by the row's Y position.
    pub fn tiles_with_render_positions(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, Point<f64, Logical>, bool)> + '_ {
        let view_offset = self.view_offset_x.current();
        let y_offset = self.y_offset;
        let active_col_idx = self.active_column_idx;

        self.columns
            .iter()
            .enumerate()
            .flat_map(move |(col_idx, col)| {
                let col_x = self.column_x(col_idx) + view_offset;
                let is_active_col = col_idx == active_col_idx;

                // tiles() returns (tile, tile_offset) pairs
                col.tiles().enumerate().map(move |(tile_idx, (tile, tile_offset))| {
                    let tile_pos = Point::from((
                        col_x + tile_offset.x,
                        y_offset + tile_offset.y,
                    ));
                    let is_active = is_active_col && tile_idx == col.active_tile_idx;
                    (tile, tile_pos, is_active)
                })
            })
    }
}

impl ColumnData {
    fn update<W: LayoutElement>(&mut self, column: &Column<W>) {
        self.width = column.width();
    }
}

/// Computes the working area from parent area and struts.
fn compute_working_area(
    parent_area: Rectangle<f64, Logical>,
    scale: f64,
    struts: Struts,
) -> Rectangle<f64, Logical> {
    // Port from scrolling.rs
    let mut area = parent_area;
    
    let round = |x: f64| (x * scale).round() / scale;
    
    area.loc.x += round(struts.left.0 as f64);
    area.loc.y += round(struts.top.0 as f64);
    area.size.w -= round(struts.left.0 as f64) + round(struts.right.0 as f64);
    area.size.h -= round(struts.top.0 as f64) + round(struts.bottom.0 as f64);
    
    area
}

// TODO(TEAM_006): Port add_window from ScrollingSpace
// TODO(TEAM_006): Port remove_window from ScrollingSpace
// TODO(TEAM_006): Port consume_or_expel_window_left/right from ScrollingSpace
// TODO(TEAM_006): Port move_column_left/right from ScrollingSpace
// TODO(TEAM_006): Port gesture handling (view_offset_gesture_begin, etc.)
// TODO(TEAM_006): Port render_elements from ScrollingSpace
// TODO(TEAM_006): Port interactive_resize_begin/update/end from ScrollingSpace
// See docs/2d-canvas-plan/TODO.md for full list
