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

use std::cmp::min;
use std::iter::zip;
use std::rc::Rc;

use niri_config::{CenterFocusedColumn, Struts};
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::closing_window::ClosingWindow;
use super::column::Column;
use super::tile::Tile;
use super::types::{ColumnWidth, InteractiveResize};
use super::{LayoutElement, Options, SizingMode};
use crate::animation::{Animation, Clock};

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

    /// Returns whether this row contains the given window.
    pub fn contains(&self, window: &W::Id) -> bool {
        self.columns.iter().any(|col| col.contains(window))
    }

    /// Finds the column containing the given window.
    pub fn find_column(&self, window: &W::Id) -> Option<usize> {
        self.columns.iter().position(|col| col.contains(window))
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
    // View offset calculation (ported from ScrollingSpace)
    // =========================================================================

    // TEAM_007: Full view offset logic ported from scrolling.rs

    /// Returns whether this row centers the focused column.
    pub fn is_centering_focused_column(&self) -> bool {
        self.options.layout.center_focused_column == CenterFocusedColumn::Always
            || (self.options.layout.always_center_single_column && self.columns.len() <= 1)
    }

    /// Returns the current view position (column X + view offset).
    pub fn view_pos(&self) -> f64 {
        self.column_x(self.active_column_idx) + self.view_offset_x.current()
    }

    /// Returns the target view position (end of animation).
    pub fn target_view_pos(&self) -> f64 {
        self.column_x(self.active_column_idx) + self.view_offset_x.target()
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

    fn compute_new_view_offset_fit(
        &self,
        target_x: Option<f64>,
        col_x: f64,
        width: f64,
        mode: SizingMode,
    ) -> f64 {
        if mode.is_fullscreen() {
            return 0.;
        }

        let (area, padding) = if mode.is_maximized() {
            (self.parent_area, 0.)
        } else {
            (self.working_area, self.options.layout.gaps)
        };

        let target_x = target_x.unwrap_or_else(|| self.target_view_pos());

        let new_offset =
            compute_new_view_offset(target_x + area.loc.x, area.size.w, col_x, width, padding);

        // Non-fullscreen windows are always offset at least by the working area position.
        new_offset - area.loc.x
    }

    fn compute_new_view_offset_centered(
        &self,
        target_x: Option<f64>,
        col_x: f64,
        width: f64,
        mode: SizingMode,
    ) -> f64 {
        if mode.is_fullscreen() {
            return self.compute_new_view_offset_fit(target_x, col_x, width, mode);
        }

        let area = if mode.is_maximized() {
            self.parent_area
        } else {
            self.working_area
        };

        // Columns wider than the view are left-aligned (the fit code can deal with that).
        if area.size.w <= width {
            return self.compute_new_view_offset_fit(target_x, col_x, width, mode);
        }

        -(area.size.w - width) / 2. - area.loc.x
    }

    fn compute_new_view_offset_for_column_fit(&self, target_x: Option<f64>, idx: usize) -> f64 {
        let col = &self.columns[idx];
        self.compute_new_view_offset_fit(
            target_x,
            self.column_x(idx),
            col.width(),
            col.sizing_mode(),
        )
    }

    fn compute_new_view_offset_for_column_centered(
        &self,
        target_x: Option<f64>,
        idx: usize,
    ) -> f64 {
        let col = &self.columns[idx];
        self.compute_new_view_offset_centered(
            target_x,
            self.column_x(idx),
            col.width(),
            col.sizing_mode(),
        )
    }

    fn compute_new_view_offset_for_column(
        &self,
        target_x: Option<f64>,
        idx: usize,
        prev_idx: Option<usize>,
    ) -> f64 {
        if self.is_centering_focused_column() {
            return self.compute_new_view_offset_for_column_centered(target_x, idx);
        }

        match self.options.layout.center_focused_column {
            CenterFocusedColumn::Always => {
                self.compute_new_view_offset_for_column_centered(target_x, idx)
            }
            CenterFocusedColumn::OnOverflow => {
                let Some(prev_idx) = prev_idx else {
                    return self.compute_new_view_offset_for_column_fit(target_x, idx);
                };

                // Activating the same column.
                if prev_idx == idx {
                    return self.compute_new_view_offset_for_column_fit(target_x, idx);
                }

                // Always take the left or right neighbor of the target as the source.
                let source_idx = if prev_idx > idx {
                    min(idx + 1, self.columns.len() - 1)
                } else {
                    idx.saturating_sub(1)
                };

                let source_col_x = self.column_x(source_idx);
                let source_col_width = self.columns[source_idx].width();

                let target_col_x = self.column_x(idx);
                let target_col_width = self.columns[idx].width();

                // NOTE: This logic won't work entirely correctly with small fixed-size maximized
                // windows (they have a different area and padding).
                let total_width = if source_col_x < target_col_x {
                    // Source is left from target.
                    target_col_x - source_col_x + target_col_width
                } else {
                    // Source is right from target.
                    source_col_x - target_col_x + source_col_width
                } + self.options.layout.gaps * 2.;

                // If it fits together, do a normal animation, otherwise center the new column.
                if total_width <= self.working_area.size.w {
                    self.compute_new_view_offset_for_column_fit(target_x, idx)
                } else {
                    self.compute_new_view_offset_for_column_centered(target_x, idx)
                }
            }
            CenterFocusedColumn::Never => {
                self.compute_new_view_offset_for_column_fit(target_x, idx)
            }
        }
    }

    fn animate_view_offset(&mut self, idx: usize, new_view_offset: f64) {
        self.animate_view_offset_with_config(
            idx,
            new_view_offset,
            self.options.animations.horizontal_view_movement.0,
        );
    }

    fn animate_view_offset_with_config(
        &mut self,
        idx: usize,
        new_view_offset: f64,
        config: niri_config::Animation,
    ) {
        let new_col_x = self.column_x(idx);
        let old_col_x = self.column_x(self.active_column_idx);
        let offset_delta = old_col_x - new_col_x;
        self.view_offset_x.offset(offset_delta);

        let pixel = 1. / self.scale;
        if (self.view_offset_x.current() - new_view_offset).abs() < pixel {
            self.view_offset_x = AnimatedValue::Static(new_view_offset);
            return;
        }

        match &mut self.view_offset_x {
            AnimatedValue::Gesture(gesture) => {
                gesture.animation = Some(Animation::new(
                    self.clock.clone(),
                    0.,
                    new_view_offset - gesture.current_view_offset,
                    0.,
                    config,
                ));
            }
            _ => {
                // FIXME: also compute and use current velocity.
                self.view_offset_x = AnimatedValue::Animation(Animation::new(
                    self.clock.clone(),
                    self.view_offset_x.current(),
                    new_view_offset,
                    0.,
                    config,
                ));
            }
        }
    }

    fn animate_view_offset_to_column_centered(
        &mut self,
        target_x: Option<f64>,
        idx: usize,
        config: niri_config::Animation,
    ) {
        let new_view_offset = self.compute_new_view_offset_for_column_centered(target_x, idx);
        self.animate_view_offset_with_config(idx, new_view_offset, config);
    }

    fn animate_view_offset_to_column_with_config(
        &mut self,
        target_x: Option<f64>,
        idx: usize,
        prev_idx: Option<usize>,
        config: niri_config::Animation,
    ) {
        let new_view_offset = self.compute_new_view_offset_for_column(target_x, idx, prev_idx);
        self.animate_view_offset_with_config(idx, new_view_offset, config);
    }

    /// Animates the view offset to show the specified column.
    fn animate_view_offset_to_column(
        &mut self,
        target_x: Option<f64>,
        idx: usize,
        prev_idx: Option<usize>,
    ) {
        self.animate_view_offset_to_column_with_config(
            target_x,
            idx,
            prev_idx,
            self.options.animations.horizontal_view_movement.0,
        )
    }

    // =========================================================================
    // Column/Tile Operations
    // =========================================================================

    /// Adds a tile as a new column.
    ///
    /// If `col_idx` is None, inserts after the active column.
    pub fn add_tile(
        &mut self,
        col_idx: Option<usize>,
        tile: Tile<W>,
        activate: bool,
        width: ColumnWidth,
        is_full_width: bool,
    ) {
        let column = Column::new_with_tile(
            tile,
            self.view_size,
            self.working_area,
            self.parent_area,
            self.scale,
            width,
            is_full_width,
        );

        self.add_column(col_idx, column, activate);
    }

    /// Adds a column at the specified index.
    ///
    /// If `idx` is None, inserts after the active column.
    pub fn add_column(&mut self, idx: Option<usize>, mut column: Column<W>, activate: bool) {
        let was_empty = self.columns.is_empty();

        let idx = idx.unwrap_or_else(|| {
            if was_empty {
                0
            } else {
                self.active_column_idx + 1
            }
        });

        column.update_config(
            self.view_size,
            self.working_area,
            self.parent_area,
            self.scale,
            self.options.clone(),
        );

        self.data.insert(idx, ColumnData { width: column.width() });
        self.columns.insert(idx, column);

        if activate {
            // If this is the first window on an empty row, skip animation.
            if was_empty {
                self.view_offset_x = AnimatedValue::new(0.);
            }

            let prev_offset = (!was_empty && idx == self.active_column_idx + 1)
                .then(|| self.view_offset_x.stationary());

            self.active_column_idx = idx;
            self.animate_view_offset_to_column(None, idx, None);
            self.activate_prev_column_on_removal = prev_offset;
        } else if !was_empty && idx <= self.active_column_idx {
            self.active_column_idx += 1;
        }

        // TODO(TEAM_006): Animate movement of other columns (port from ScrollingSpace)
    }

    /// Removes the column at the specified index.
    pub fn remove_column(&mut self, idx: usize) -> Column<W> {
        let column = self.columns.remove(idx);
        self.data.remove(idx);

        if self.columns.is_empty() {
            self.active_column_idx = 0;
            self.activate_prev_column_on_removal = None;
        } else if idx < self.active_column_idx {
            self.active_column_idx -= 1;
        } else if idx == self.active_column_idx {
            // Activate previous or next column
            if let Some(prev_offset) = self.activate_prev_column_on_removal.take() {
                if self.active_column_idx > 0 {
                    self.active_column_idx -= 1;
                }
                self.view_offset_x = AnimatedValue::new(prev_offset);
            } else {
                self.active_column_idx = self.active_column_idx.min(self.columns.len() - 1);
            }
            self.animate_view_offset_to_column(None, self.active_column_idx, None);
        }

        // TODO(TEAM_006): Animate movement of other columns (port from ScrollingSpace)

        column
    }

    /// Moves the active column to the left.
    pub fn move_left(&mut self) -> bool {
        if self.active_column_idx == 0 {
            return false;
        }
        self.move_column_to(self.active_column_idx - 1);
        true
    }

    /// Moves the active column to the right.
    pub fn move_right(&mut self) -> bool {
        let new_idx = self.active_column_idx + 1;
        if new_idx >= self.columns.len() {
            return false;
        }
        self.move_column_to(new_idx);
        true
    }

    /// Moves the active column to a specific index.
    fn move_column_to(&mut self, new_idx: usize) {
        if self.active_column_idx == new_idx {
            return;
        }

        let current_col_x = self.column_x(self.active_column_idx);

        let column = self.columns.remove(self.active_column_idx);
        let data = self.data.remove(self.active_column_idx);
        self.columns.insert(new_idx, column);
        self.data.insert(new_idx, data);

        // Preserve the camera position when moving.
        let view_offset_delta = -self.column_x(self.active_column_idx) + current_col_x;
        self.view_offset_x.offset(view_offset_delta);

        self.active_column_idx = new_idx;
        self.animate_view_offset_to_column(None, new_idx, None);

        // TODO(TEAM_006): Animate column movement (port from ScrollingSpace)
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

// TEAM_007: Ported from scrolling.rs
fn compute_new_view_offset(
    cur_x: f64,
    view_width: f64,
    new_col_x: f64,
    new_col_width: f64,
    gaps: f64,
) -> f64 {
    // If the column is wider than the view, always left-align it.
    if view_width <= new_col_width {
        return 0.;
    }

    // Compute the padding in case it needs to be smaller due to large tile width.
    let padding = ((view_width - new_col_width) / 2.).clamp(0., gaps);

    // Compute the desired new X with padding.
    let new_x = new_col_x - padding;
    let new_right_x = new_col_x + new_col_width + padding;

    // If the column is already fully visible, leave the view as is.
    if cur_x <= new_x && new_right_x <= cur_x + view_width {
        return -(new_col_x - cur_x);
    }

    // Otherwise, prefer the alignment that results in less motion from the current position.
    let dist_to_left = (cur_x - new_x).abs();
    let dist_to_right = ((cur_x + view_width) - new_right_x).abs();
    if dist_to_left <= dist_to_right {
        -padding
    } else {
        -(view_width - padding - new_col_width)
    }
}

// TODO(TEAM_006): Port add_window from ScrollingSpace
// TODO(TEAM_006): Port remove_window from ScrollingSpace
// TODO(TEAM_006): Port consume_or_expel_window_left/right from ScrollingSpace
// TODO(TEAM_006): Port move_column_left/right from ScrollingSpace
// TODO(TEAM_006): Port gesture handling (view_offset_gesture_begin, etc.)
// TODO(TEAM_006): Port render_elements from ScrollingSpace
// TODO(TEAM_006): Port interactive_resize_begin/update/end from ScrollingSpace
// See docs/2d-canvas-plan/TODO.md for full list
