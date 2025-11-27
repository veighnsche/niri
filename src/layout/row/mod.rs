// TEAM_006: Row module for 2D canvas layout.
// TEAM_007: Refactored into submodules per Rule 7.
//!
//! A Row is a horizontal strip of columns — the fundamental horizontal
//! layout primitive for the 2D canvas.
//!
//! ## Design (Option B: Clean Slate)
//!
//! Per Rule 0 (Quality > Speed), Row owns its columns directly rather than
//! wrapping ScrollingSpace. This avoids indirection and technical debt.
//!
//! ## Module Structure
//!
//! ```text
//! row/
//! ├── mod.rs          - Core struct and basic accessors
//! ├── view_offset.rs  - View offset calculation and animation
//! ├── navigation.rs   - Focus left/right/column
//! ├── operations/     - Column operations (refactored by TEAM_008)
//! │   ├── mod.rs      - Re-exports
//! │   ├── add.rs      - Add tile/column
//! │   ├── remove.rs   - Remove tile/column
//! │   ├── move_col.rs - Move column left/right
//! │   └── consume.rs  - Consume/expel window
//! ├── layout.rs       - Tile positions, config update
//! ├── render.rs       - Rendering
//! ├── gesture.rs      - Gesture-based scrolling
//! └── resize.rs       - Interactive resize
//! ```

mod gesture;
mod layout;
mod navigation;
mod operations;
mod render;
mod resize;
mod view_offset;

pub use render::RowRenderElement;

use std::rc::Rc;

use niri_config::{Struts, Border};
use niri_config::utils::MergeWith;
use niri_ipc::{ColumnDisplay, SizeChange};
use crate::layout::workspace_types::WorkspaceId;
use smithay::utils::{Logical, Point, Rectangle, Size};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::backend::renderer::gles::GlesRenderer;

use crate::utils::ResizeEdge;

use super::animated_value::AnimatedValue;
use super::closing_window::ClosingWindow;
use super::column::Column;
use super::tile::Tile;
use super::types::{InteractiveResize};
use super::{LayoutElement, Options, ConfigureIntent};
use crate::animation::Clock;
use crate::utils::transaction::TransactionBlocker;

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

    /// Optional name for this row (replaces workspace naming).
    /// 
    /// Used for user-identifiable rows in the 2D canvas.
    name: Option<String>,

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

        Self {
            row_index,
            y_offset: row_index as f64 * view_size.h,
            name: None,
            view_size,
            parent_area,
            working_area,
            scale,
            clock,
            options,
            columns: Vec::new(),
            data: Vec::new(),
            active_column_idx: 0,
            interactive_resize: None,
            view_offset_x: AnimatedValue::new(0.0),
            activate_prev_column_on_removal: None,
            view_offset_to_restore: None,
            closing_windows: Vec::new(),
        }
    }

    // =========================================================================
    // Row-specific accessors
    // =========================================================================

    /// Returns the row index.
    pub fn row_index(&self) -> i32 {
        self.row_index
    }

    /// Get the row index (alias for row_index).
    /// TEAM_031: Added for compatibility with workspace patterns
    pub fn idx(&self) -> i32 {
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

    /// Returns the row's name, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the row's name.
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// Returns the workspace ID for this row.
    /// TEAM_024: Added for workspace compatibility during Canvas2D migration
    pub fn id(&self) -> crate::layout::workspace_types::WorkspaceId {
        crate::layout::workspace_types::WorkspaceId::from_row_index(self.row_index)
    }

    /// Sets the row index (used internally by canvas for reordering).
    pub(crate) fn set_row_index(&mut self, row_index: i32) {
        self.row_index = row_index;
    }

    /// Sets the Y offset (used internally by canvas for reordering).
    pub(crate) fn set_y_offset(&mut self, y_offset: f64) {
        self.y_offset = y_offset;
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

    /// Returns a reference to the view offset animated value.
    /// TEAM_035: Added for test compatibility
    pub fn view_offset(&self) -> &AnimatedValue {
        &self.view_offset_x
    }

    /// Returns whether this row contains the given window id.
    pub fn has_window(&self, window: &W::Id) -> bool {
        self.columns.iter().any(|col| col.contains(window))
    }

    /// Returns whether this row contains the given window id.
    /// Alias for has_window for canvas compatibility.
    pub fn contains(&self, window: &W::Id) -> bool {
        self.has_window(window)
    }

    /// Returns whether the given window is floating in this row.
    /// TEAM_024: Added for workspace compatibility - always false for tiled rows
    pub fn is_floating(&self, _id: &W::Id) -> bool {
        false // Rows only contain tiled windows, floating windows are in Canvas2D.floating
    }

    /// Finds the column containing the given window.
    pub fn find_column(&self, window: &W::Id) -> Option<usize> {
        self.columns.iter().position(|col| col.contains(window))
    }

    /// Returns all tiles in this row.
    /// TEAM_010: Added for Canvas2D.windows() migration
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<W>> + '_ {
        self.columns.iter().flat_map(|col| col.tiles_iter())
    }

    /// Returns all tiles in this row (mutable).
    /// TEAM_010: Added for Canvas2D.windows_mut() migration
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<W>> + '_ {
        self.columns.iter_mut().flat_map(|col| col.tiles_iter_mut())
    }

    /// Returns all windows in this row.
    /// TEAM_024: Added for workspace compatibility
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        self.tiles().map(|tile| tile.window())
    }

    /// Returns all windows in this row (mutable).
    /// TEAM_024: Added for workspace compatibility
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut W> + '_ {
        self.tiles_mut().map(|tile| tile.window_mut())
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

    /// Refreshes all windows in this row, updating their configuration and state.
    ///
    /// This method handles window activation, configuration, and bounds updates.
    /// It's the Row equivalent of the old ScrollingSpace::refresh() method.
    ///
    /// # Arguments
    /// * `is_active` - Whether this row is on the active monitor
    /// * `is_focused` - Whether this row is the focused row
    pub fn refresh(&mut self, is_active: bool, is_focused: bool) {
        for (col_idx, col) in self.columns.iter_mut().enumerate() {
            let mut col_resize_data = None;
            if let Some(resize) = &self.interactive_resize {
                if col.contains(&resize.window) {
                    col_resize_data = Some(resize.data);
                }
            }

            let is_tabbed = col.display_mode == ColumnDisplay::Tabbed;
            let extra_size = Size::new(0.0, 0.0); // TEAM_027: TODO - calculate proper extra_size

            // If transactions are disabled, also disable combined throttling, for more intuitive
            // behavior. In tabbed display mode, only one window is visible, so individual
            // throttling makes more sense.
            let individual_throttling = self.options.disable_transactions || is_tabbed;

            let intent = if self.options.disable_resize_throttling {
                ConfigureIntent::CanSend
            } else if individual_throttling {
                // In this case, we don't use combined throttling, but rather compute throttling
                // individually below.
                ConfigureIntent::CanSend
            } else {
                col.tiles_iter()
                    .fold(ConfigureIntent::NotNeeded, |intent, tile| {
                        match (intent, tile.window().configure_intent()) {
                            (_, ConfigureIntent::ShouldSend) => ConfigureIntent::ShouldSend,
                            (ConfigureIntent::NotNeeded, tile_intent) => tile_intent,
                            (ConfigureIntent::CanSend, ConfigureIntent::Throttled) => {
                                ConfigureIntent::Throttled
                            }
                            (intent, _) => intent,
                        }
                    })
            };

            let active_tile_idx = col.active_tile_idx;
            for (tile_idx, tile) in col.tiles_iter_mut().enumerate() {
                let win = tile.window_mut();

                let active_in_column = Some(active_tile_idx) == Some(tile_idx);
                win.set_active_in_column(active_in_column);
                win.set_floating(false);

                let mut active = is_active && self.active_column_idx == col_idx;
                if self.options.deactivate_unfocused_windows {
                    active &= active_in_column && is_focused;
                } else {
                    // In tabbed mode, all tabs have activated state to reduce unnecessary
                    // animations when switching tabs.
                    active &= active_in_column || is_tabbed;
                }
                win.set_activated(active);

                win.set_interactive_resize(col_resize_data);

                let border_config = self.options.layout.border.merged_with(&win.rules().border);
                let bounds = compute_toplevel_bounds(
                    border_config,
                    self.working_area.size,
                    extra_size,
                    self.options.layout.gaps,
                );
                win.set_bounds(bounds);

                let intent = if individual_throttling {
                    win.configure_intent()
                } else {
                    intent
                };

                if matches!(
                    intent,
                    ConfigureIntent::CanSend | ConfigureIntent::ShouldSend
                ) {
                    win.send_pending_configure();
                }

                win.refresh();
            }
        }
    }

    // =========================================================================
    // Golden Snapshot Testing
    // TEAM_010: Added snapshot() for golden test compatibility
    // =========================================================================

    /// Returns self for test compatibility.
    /// 
    /// In the old architecture, Workspace had a scrolling() method that returned
    /// the ScrollingSpace. In Canvas2D, Row IS the scrolling space, so this just
    /// returns self.
    /// 
    /// TEAM_035: Added for test compatibility
    #[cfg(test)]
    pub fn scrolling(&self) -> &Self {
        self
    }

    /// Verifies internal invariants for testing.
    /// TEAM_035: Added for test compatibility
    #[cfg(test)]
    pub fn verify_invariants(&self, _move_win_id: Option<&W::Id>) {
        assert!(self.view_size.w > 0.);
        assert!(self.view_size.h > 0.);
        assert!(self.scale > 0.);

        if !self.columns.is_empty() {
            assert!(self.active_column_idx < self.columns.len());
        }

        for col in &self.columns {
            col.verify_invariants();
        }
    }

    /// Creates a snapshot of this row's layout state for golden testing.
    ///
    /// This produces the same format as ScrollingSpace.snapshot() to ensure
    /// golden tests pass after the Monitor refactor.
    ///
    /// # Arguments
    /// * `is_active` - Whether this row is on the active monitor
    /// * `is_focused` - Whether this row is the focused row
    #[cfg(test)]
    pub fn snapshot(&self) -> crate::layout::snapshot::ScrollingSnapshot {
        use crate::layout::snapshot::{
            AnimationTimelineSnapshot, RectSnapshot, ScrollingSnapshot, SizeSnapshot,
        };

        let columns = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let col_x = self.column_x(idx);
                col.snapshot(col_x)
            })
            .collect();

        // TEAM_010: Capture ALL active animations
        let mut animations = Vec::new();

        // 1. Capture view_offset animation (camera X movement)
        if let AnimatedValue::Animation(anim) = &self.view_offset_x {
            let kind = Self::extract_animation_kind(anim);
            animations.push(AnimationTimelineSnapshot::view_offset(
                anim.from(),
                anim.to(),
                kind,
                anim.duration().as_millis() as u64,
            ));
        }

        // 2. Capture all column and tile animations
        for (col_idx, column) in self.columns.iter().enumerate() {
            let col_x = self.column_x(col_idx);

            // Column move animation affects all tiles' X position
            if let Some((anim, from_offset)) = column.move_animation() {
                let kind = Self::extract_animation_kind(anim);
                // Column move is an offset, not absolute position
                animations.push(AnimationTimelineSnapshot {
                    target: format!("column_{col_idx}_move_x"),
                    from: from_offset,
                    to: 0.0, // Animation moves toward 0 offset
                    kind,
                    duration_ms: anim.duration().as_millis() as u64,
                    pinned_edge: None,
                });
            }

            // Iterate through tiles for individual animations
            for (tile, tile_idx) in column.tiles_with_animations() {
                let tile_offset = column.tile_offset(tile_idx);
                let tile_size = tile.tile_size();

                // Tile resize animation → affects width (right edge) and height (bottom edge)
                if let Some(anim) = tile.resize_animation() {
                    if let Some((_, tile_size_from)) = tile.resize_animation_from_sizes() {
                        let kind = Self::extract_animation_kind(anim);

                        // Width animation: x_max (right edge in LTR) moves
                        if (tile_size_from.w - tile_size.w).abs() > 0.1 {
                            animations.push(AnimationTimelineSnapshot::tile_edge(
                                col_idx,
                                tile_idx,
                                "x_max",
                                col_x + tile_offset.x + tile_size_from.w,
                                col_x + tile_offset.x + tile_size.w,
                                kind.clone(),
                                anim.duration().as_millis() as u64,
                            ));
                        }

                        // Height animation: y_max (bottom edge) moves
                        if (tile_size_from.h - tile_size.h).abs() > 0.1 {
                            animations.push(AnimationTimelineSnapshot::tile_edge(
                                col_idx,
                                tile_idx,
                                "y_max",
                                tile_offset.y + tile_size_from.h,
                                tile_offset.y + tile_size.h,
                                kind,
                                anim.duration().as_millis() as u64,
                            ));
                        }
                    }
                }

                // Tile move_x animation → x_min and x_max edges move together
                if let Some((anim, from_x)) = tile.move_x_animation_with_from() {
                    let kind = Self::extract_animation_kind(anim);
                    let current_x = col_x + tile_offset.x;
                    let from_abs_x = current_x + from_x; // from is relative offset

                    // x_min edge (left in LTR)
                    animations.push(AnimationTimelineSnapshot::tile_edge(
                        col_idx,
                        tile_idx,
                        "x_min",
                        from_abs_x,
                        current_x,
                        kind.clone(),
                        anim.duration().as_millis() as u64,
                    ));

                    // x_max edge (right in LTR) moves same amount
                    animations.push(AnimationTimelineSnapshot::tile_edge(
                        col_idx,
                        tile_idx,
                        "x_max",
                        from_abs_x + tile_size.w,
                        current_x + tile_size.w,
                        kind,
                        anim.duration().as_millis() as u64,
                    ));
                }

                // Tile move_y animation → y_min and y_max edges move together
                if let Some((anim, from_y)) = tile.move_y_animation_with_from() {
                    let kind = Self::extract_animation_kind(anim);
                    let current_y = tile_offset.y;
                    let from_abs_y = current_y + from_y; // from is relative offset

                    // y_min edge (top)
                    animations.push(AnimationTimelineSnapshot::tile_edge(
                        col_idx,
                        tile_idx,
                        "y_min",
                        from_abs_y,
                        current_y,
                        kind.clone(),
                        anim.duration().as_millis() as u64,
                    ));

                    // y_max edge (bottom) moves same amount
                    animations.push(AnimationTimelineSnapshot::tile_edge(
                        col_idx,
                        tile_idx,
                        "y_max",
                        from_abs_y + tile_size.h,
                        current_y + tile_size.h,
                        kind,
                        anim.duration().as_millis() as u64,
                    ));
                }
            }
        }

        ScrollingSnapshot {
            columns,
            active_column_idx: self.active_column_idx,
            view_offset: self.view_offset_x.current(),
            working_area: RectSnapshot::from(self.working_area),
            view_size: SizeSnapshot::from(self.view_size),
            animations,
        }
    }

    // TEAM_010: Helper to extract animation kind for snapshots
    #[cfg(test)]
    fn extract_animation_kind(anim: &crate::animation::Animation) -> crate::layout::snapshot::AnimationKindSnapshot {
        use crate::layout::snapshot::AnimationKindSnapshot;

        if let Some(curve_name) = anim.easing_curve_name() {
            return AnimationKindSnapshot::Easing {
                curve: curve_name.to_string(),
                duration_ms: anim.duration().as_millis() as u64,
            };
        }

        if let Some(params) = anim.spring_params() {
            // Calculate damping_ratio from damping
            // damping = damping_ratio * 2 * sqrt(mass * stiffness)
            // For mass=1: damping_ratio = damping / (2 * sqrt(stiffness))
            let damping_ratio = params.damping / (2.0 * params.stiffness.sqrt());
            return AnimationKindSnapshot::Spring {
                damping_ratio: (damping_ratio * 100.0).round() / 100.0,
                stiffness: (params.stiffness * 10.0).round() / 10.0,
            };
        }

        if let Some((initial_velocity, deceleration_rate)) = anim.deceleration_params() {
            return AnimationKindSnapshot::Deceleration {
                initial_velocity: (initial_velocity * 10.0).round() / 10.0,
                deceleration_rate: (deceleration_rate * 1000.0).round() / 1000.0,
            };
        }

        // Fallback
        AnimationKindSnapshot::Easing {
            curve: "Unknown".to_string(),
            duration_ms: anim.duration().as_millis() as u64,
        }
    }
}

impl ColumnData {
    fn update<W: LayoutElement>(&mut self, column: &Column<W>) {
        self.width = column.width();
    }
}

/// Computes the working area from parent area and struts.
pub(crate) fn compute_working_area(
    parent_area: Rectangle<f64, Logical>,
    scale: f64,
    struts: Struts,
) -> Rectangle<f64, Logical> {
    let mut area = parent_area;
    
    let round = |x: f64| (x * scale).round() / scale;
    
    area.loc.x += round(struts.left.0 as f64);
    area.loc.y += round(struts.top.0 as f64);
    area.size.w -= round(struts.left.0 as f64) + round(struts.right.0 as f64);
    area.size.h -= round(struts.top.0 as f64) + round(struts.bottom.0 as f64);
    
    area
}

// TEAM_022: Missing workspace-compatibility methods for Row
impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // Window configuration methods (migrated from Workspace)
    // =========================================================================

    /// Configure a new window with defaults.
    /// TEAM_022: Stub implementation for compatibility
    pub fn configure_new_window<R>(
        &self,
        _window: &W::Id,
        _width: Option<niri_config::PresetSize>,
        _height: Option<niri_config::PresetSize>,
        _is_floating: bool,
        _rules: &R,
    ) {
        // TEAM_022: TODO - implement proper window configuration
        // For now, this is a no-op as windows are configured when added
    }

    /// Resolve the default width for a window.
    /// TEAM_022: Stub implementation
    pub fn resolve_default_width<R>(&self, rules: R, is_floating: bool) -> Option<niri_config::PresetSize> {
        // Return None for auto width
        None
    }

    /// Resolve the default height for a window.
    /// TEAM_022: Stub implementation
    pub fn resolve_default_height<R>(&self, rules: R, is_floating: bool) -> Option<niri_config::PresetSize> {
        // Return None for auto height
        None
    }

    /// Move focus down within the active column.
    /// TEAM_022: Returns false if cannot move down
    pub fn focus_down(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.focus_down()
        } else {
            false
        }
    }

    /// Move focus up within the active column.
    /// TEAM_022: Returns false if cannot move up
    pub fn focus_up(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.focus_up()
        } else {
            false
        }
    }

    /// Move window down within the active column.
    /// TEAM_022: Returns false if cannot move
    pub fn move_down(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.move_down()
        } else {
            false
        }
    }

    /// Move window up within the active column.
    /// TEAM_022: Returns false if cannot move
    pub fn move_up(&mut self) -> bool {
        if let Some(col) = self.active_column_mut() {
            col.move_up()
        } else {
            false
        }
    }

    /// Check if any transitions are ongoing.
    pub fn are_transitions_ongoing(&self) -> bool {
        self.are_animations_ongoing()
    }

    // TEAM_024: Workspace compatibility methods - these are mostly no-ops for rows
    // since floating windows are handled at the Canvas2D level
    
    pub fn set_window_height(&mut self, _window: Option<&W::Id>, _height: super::SizeChange) {
        // Rows don't control individual window heights - this is a no-op
    }
    
    // TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn reset_window_height(&mut self, _window: Option<&W::Id>) {
        // Rows don't control individual window heights - this is a no-op
    }
    
    // TEAM_035: Updated signature to take no arguments (uses active column)
    pub fn expand_column_to_available_width(&mut self) {
        // TODO: TEAM_024: Implement column width expansion if needed
    }
    
    // TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn toggle_window_floating(&mut self, _window: Option<&W::Id>) {
        // Floating is handled at Canvas2D level - this is a no-op for rows
    }
    
    // TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn set_window_floating(&mut self, _window: Option<&W::Id>, _floating: bool) {
        // Floating is handled at Canvas2D level - this is a no-op for rows
    }
    
    pub fn focus_floating(&mut self) -> bool {
        // Rows don't contain floating windows - always false
        false
    }
    
    pub fn focus_tiling(&mut self) -> bool {
        // Focus the first tiled window in the active column
        if let Some(column) = self.active_column_mut() {
            let active_tile_idx = column.active_tile_idx;
            if let Some(tile) = column.tiles_iter_mut().nth(active_tile_idx) {
                tile.window_mut().set_activated(true);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    pub fn switch_focus_floating_tiling(&mut self) -> bool {
        // Rows only have tiled windows - just focus tiling
        self.focus_tiling()
    }
    
    pub fn set_fullscreen(&mut self, _id: &W::Id, _is_fullscreen: bool) {
        // TODO: TEAM_024: Implement fullscreen state if needed
    }
    
    pub fn toggle_fullscreen(&mut self, _id: &W::Id) {
        // TODO: TEAM_024: Implement fullscreen toggle if needed
    }
    
    pub fn set_maximized(&mut self, _id: &W::Id, _maximize: bool) {
        // TODO: TEAM_024: Implement maximized state if needed
    }
    
    pub fn toggle_maximized(&mut self, _id: &W::Id) {
        // TODO: TEAM_024: Implement maximized toggle if needed
    }
    
    pub fn active_window(&self) -> Option<&W> {
        self.active_column()
            .and_then(|col| col.tiles_iter().nth(col.active_tile_idx))
            .map(|tile| tile.window())
    }
    
    pub fn activate_window(&mut self, _window: &W::Id) -> bool {
        // TODO: TEAM_024: Implement window activation if needed
        false
    }
    
    // TEAM_035: Updated signature to accept window ID and return bool
    pub fn start_open_animation(&mut self, _window: &W::Id) -> bool {
        // TODO: TEAM_024: Implement open animation if needed
        false
    }
    
    pub fn layout_config(&self) -> Option<niri_config::LayoutPart> {
        // Rows don't have individual layout configs - this comes from the monitor/canvas
        None
    }

    /// Get the current output for this row.
    /// TEAM_022: Stub implementation - rows don't directly track outputs
    pub fn current_output(&self) -> Option<Output> {
        // TEAM_022: TODO - rows should get output from monitor/canvas
        None
    }

    /// Get mutable reference to the active window.
    /// TEAM_022: Stub implementation
    pub fn active_window_mut(&mut self) -> Option<&mut W> {
        // TEAM_022: TODO - implement active window logic
        if let Some(col) = self.active_column_mut() {
            let active_tile_idx = col.active_tile_idx;
            col.tiles_iter_mut().nth(active_tile_idx).map(|tile| tile.window_mut())
        } else {
            None
        }
    }

    /// Check if this row is urgent.
    /// TEAM_022: Stub implementation
    pub fn is_urgent(&self) -> bool {
        // TEAM_022: TODO - implement urgency detection
        false
    }

    /// Find window under the given point.
    /// TEAM_036: Implemented based on ScrollingSpace::window_under
    pub fn window_under(&self, pos: Point<f64, Logical>) -> Option<(&W, super::HitType)> {
        let scale = self.scale;
        let view_off = Point::from((-self.view_pos(), 0.));
        
        for (col, col_x) in self.columns_in_render_order() {
            let col_off = Point::from((col_x, 0.));
            let col_render_off = col.render_offset();

            // Hit the tab indicator.
            if col.display_mode == ColumnDisplay::Tabbed && col.sizing_mode().is_normal() {
                let col_pos = view_off + col_off + col_render_off;
                let col_pos = col_pos.to_physical_precise_round(scale).to_logical(scale);

                if let Some(idx) = col.tab_indicator.hit(
                    col.tab_indicator_area(),
                    col.tiles.len(),
                    scale,
                    pos - col_pos,
                ) {
                    let hit = super::HitType::Activate {
                        is_tab_indicator: true,
                    };
                    return Some((col.tiles[idx].window(), hit));
                }
            }

            for (tile, tile_off, visible) in col.tiles_in_render_order() {
                if !visible {
                    continue;
                }

                let tile_pos =
                    view_off + col_off + col_render_off + tile_off + tile.render_offset();
                // Round to physical pixels.
                let tile_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                if let Some(rv) = super::HitType::hit_tile(tile, tile_pos, pos) {
                    return Some(rv);
                }
            }
        }

        None
    }

    /// Find resize edges under the given point.
    /// TEAM_036: Implemented based on original Workspace::resize_edges_under
    pub fn resize_edges_under(&self, pos: Point<f64, Logical>) -> Option<ResizeEdge> {
        let scale = self.scale;
        let view_off = Point::from((-self.view_pos(), 0.));
        
        for (col, col_x) in self.columns_in_render_order() {
            let col_off = Point::from((col_x, 0.));
            let col_render_off = col.render_offset();

            for (tile, tile_off, visible) in col.tiles_in_render_order() {
                if !visible {
                    continue;
                }

                let tile_pos =
                    view_off + col_off + col_render_off + tile_off + tile.render_offset();
                // Round to physical pixels.
                let tile_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                let pos_within_tile = pos - tile_pos;
                
                // Check if point is within this tile
                if tile.hit(pos_within_tile).is_some() {
                    let size = tile.tile_size().to_f64();
                    
                    // Determine resize edges based on position within tile (thirds)
                    let mut edges = ResizeEdge::empty();
                    if pos_within_tile.x < size.w / 3. {
                        edges |= ResizeEdge::LEFT;
                    } else if 2. * size.w / 3. < pos_within_tile.x {
                        edges |= ResizeEdge::RIGHT;
                    }
                    if pos_within_tile.y < size.h / 3. {
                        edges |= ResizeEdge::TOP;
                    } else if 2. * size.h / 3. < pos_within_tile.y {
                        edges |= ResizeEdge::BOTTOM;
                    }
                    return Some(edges);
                }
            }
        }

        None
    }

    /// Update shaders for all tiles in the row.
    /// TEAM_031: Added for monitor config compatibility
    pub fn update_shaders(&mut self) {
        for column in &mut self.columns {
            for tile in &mut column.tiles {
                tile.update_shaders();
            }
        }
    }

    /// Update output size for all tiles in the row.
    /// TEAM_031: Added for monitor config compatibility
    pub fn update_output_size(&mut self) {
        for column in &mut self.columns {
            for tile in &mut column.tiles {
                tile.update_window();
            }
        }
    }

    /// Get the visual rectangle of the active tile.
    /// TEAM_022: Stub implementation
    pub fn active_tile_visual_rectangle(&self) -> Option<Rectangle<f64, Logical>> {
        // TEAM_022: TODO - implement active tile visual rectangle
        self.active_column()
            .and_then(|col| col.tiles_iter().nth(col.active_tile_idx))
            .map(|tile| Rectangle::from_loc_and_size(Point::new(0.0, 0.0), tile.tile_size()))
    }

    /// Check if this row has any windows.
    /// TEAM_033: Added for workspace cleanup logic
    pub fn has_windows(&self) -> bool {
        self.columns().count() > 0
    }

    /// Check if this row has any windows or a name.
    /// TEAM_022: Stub implementation
    pub fn has_windows_or_name(&self) -> bool {
        self.has_windows() || self.name().is_some()
    }

    /// Update a window in this row.
    /// TEAM_022: Stub implementation
    pub fn update_window(&mut self, _window: &W::Id) {
        // TEAM_022: TODO - implement window update
    }

    /// Update the layout config for this row.
    /// TEAM_022: Stub implementation
    pub fn update_layout_config(&mut self, _config: Option<niri_config::LayoutPart>) {
        // TEAM_022: TODO - rows don't have individual layout configs
    }

    /// Resolve scrolling width for a window ID.
    /// TEAM_025: Stub implementation
    pub fn resolve_scrolling_width(&self, _window: &W::Id, _width: Option<niri_config::PresetSize>) -> f64 {
        // TEAM_025: TODO - implement proper scrolling width resolution
        1.0
    }

    /// Make a tile for a window.
    /// TEAM_025: Stub implementation
    pub fn make_tile(&mut self, _window: W, _activate: bool) {
        // TEAM_025: TODO - implement tile creation
    }

    /// Handle descendants added.
    /// TEAM_025: Stub implementation
    pub fn descendants_added(&mut self, _id: &W::Id) -> bool {
        // TEAM_025: TODO - implement descendants handling
        false
    }

    /// Compute scroll amount needed to activate a window.
    /// Adapted from ScrollingSpace::scroll_amount_to_activate
    pub fn scroll_amount_to_activate(&self, window: &W::Id) -> f64 {
        let column_idx = self
            .columns
            .iter()
            .position(|col| col.contains(window))
            .unwrap();

        if self.active_column_idx == column_idx {
            return 0.;
        }

        // Compute the scroll amount needed to bring the column to view
        let target_x = self.column_x(column_idx);
        let current_x = self.view_offset_x();
        target_x - current_x
    }

    /// Find a Wayland surface.
    /// TEAM_036: Implemented - searches all tiles for matching surface
    pub fn find_wl_surface(&self, wl_surface: &WlSurface) -> Option<&W> {
        self.tiles()
            .find(|tile| tile.window().is_wl_surface(wl_surface))
            .map(|tile| tile.window())
    }

    /// Find a Wayland surface mutably.
    /// TEAM_036: Implemented - searches all tiles for matching surface
    pub fn find_wl_surface_mut(&mut self, wl_surface: &WlSurface) -> Option<&mut W> {
        for column in &mut self.columns {
            for tile in &mut column.tiles {
                if tile.window().is_wl_surface(wl_surface) {
                    return Some(tile.window_mut());
                }
            }
        }
        None
    }

    /// Get popup target rectangle.
    /// TEAM_025: Stub implementation
    /// TEAM_035: Updated return type to Rectangle<f64, Logical>
    pub fn popup_target_rect(&self, _window: &W::Id) -> Option<Rectangle<f64, Logical>> {
        // TEAM_025: TODO - implement popup target rect
        None
    }

    /// Activate window without raising.
    /// TEAM_025: Stub implementation
    /// TEAM_035: Updated return type to bool
    pub fn activate_window_without_raising(&mut self, _window: &W::Id) -> bool {
        // TEAM_025: TODO - implement activation without raising
        false
    }

    /// Get tiles with IPC layouts.
    /// TEAM_025: Stub implementation
    /// TEAM_035: Updated return type to iterator of (tile, layout) tuples
    pub fn tiles_with_ipc_layouts(&self) -> impl Iterator<Item = (&Tile<W>, niri_ipc::WindowLayout)> {
        // TEAM_025: TODO - implement IPC layout generation
        // For now, return tiles with empty layouts
        self.tiles().map(|tile| {
            let layout = niri_ipc::WindowLayout {
                pos_in_scrolling_layout: None,
                tile_size: (0.0, 0.0),
                window_size: (0, 0),
                tile_pos_in_workspace_view: None,
                window_offset_in_tile: (0.0, 0.0),
            };
            (tile, layout)
        })
    }

    /// Expel window from column to floating.
    /// TEAM_028: Stub implementation
    pub fn expel_from_column(&mut self) {
        // TEAM_028: TODO - implement window expulsion to floating
    }

    /// Swap window in given direction.
    /// TEAM_028: Stub implementation
    pub fn swap_window_in_direction(&mut self, _direction: super::types::ScrollDirection) {
        // TEAM_028: TODO - implement window swapping
    }

    /// Toggle column tabbed display mode.
    /// TEAM_028: Stub implementation
    pub fn toggle_column_tabbed_display(&mut self) {
        // TEAM_028: TODO - implement tabbed display toggle
    }

    /// Set column display mode.
    /// TEAM_028: Stub implementation
    pub fn set_column_display(&mut self, _display: ColumnDisplay) {
        // TEAM_028: TODO - implement column display setting
    }

    /// Center the active column.
    /// TEAM_028: Stub implementation
    pub fn center_column(&mut self) {
        // TEAM_028: TODO - implement column centering
    }

    /// Center all visible columns.
    /// TEAM_028: Stub implementation
    pub fn center_visible_columns(&mut self) {
        // TEAM_028: TODO - implement visible columns centering
    }

    /// Toggle width configuration.
    /// TEAM_028: Stub implementation
    pub fn toggle_width(&mut self, _forwards: bool) {
        // TEAM_028: TODO - implement width toggle
    }

    /// Toggle window width.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn toggle_window_width(&mut self, _window: Option<&W::Id>, _forwards: bool) {
        // TEAM_028: TODO - implement window width toggle
    }

    /// Toggle window height.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn toggle_window_height(&mut self, _window: Option<&W::Id>, _forwards: bool) {
        // TEAM_028: TODO - implement window height toggle
    }

    /// Toggle full width for active column.
    /// TEAM_028: Stub implementation
    pub fn toggle_full_width(&mut self) {
        // TEAM_028: TODO - implement full width toggle
    }

    /// Set column width.
    /// TEAM_028: Stub implementation
    pub fn set_column_width(&mut self, _change: SizeChange) {
        // TEAM_028: TODO - implement column width setting
    }

    /// Set window width.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept Option<&W::Id>
    pub fn set_window_width(&mut self, _window: Option<&W::Id>, _change: SizeChange) {
        // TEAM_028: TODO - implement window width setting
    }

    /// Get scrolling insert position.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated return type to InsertPosition
    pub fn scrolling_insert_position(&self, _pos: Point<f64, Logical>) -> super::types::InsertPosition {
        // TEAM_028: TODO - implement insert position calculation
        super::types::InsertPosition::NewColumn(0)
    }

    /// Store unmap snapshot if empty.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept &W::Id
    pub fn store_unmap_snapshot_if_empty(&mut self, _renderer: &mut GlesRenderer, _window: &W::Id) {
        // TEAM_028: TODO - implement unmap snapshot storage
    }

    /// Clear unmap snapshot.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept &W::Id
    pub fn clear_unmap_snapshot(&mut self, _window: &W::Id) {
        // TEAM_028: TODO - implement unmap snapshot clearing
    }

    /// Start close animation for window.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated signature to accept &W::Id
    pub fn start_close_animation_for_window(&mut self, _renderer: &mut GlesRenderer, _window: &W::Id, _blocker: TransactionBlocker) {
        // TEAM_028: TODO - implement close animation
    }

    /// Start close animation for a tile with snapshot.
    /// TEAM_033: Added for interactive move window closing
    pub fn start_close_animation_for_tile(
        &mut self,
        _renderer: &mut GlesRenderer,
        _snapshot: crate::layout::tile::TileRenderSnapshot,
        _tile_size: Size<f64, Logical>,
        _tile_pos: Point<f64, Logical>,
        _blocker: TransactionBlocker,
    ) {
        // TODO(TEAM_033): Implement proper close animation with snapshot
        // This requires ClosingWindow infrastructure similar to ScrollingSpace
    }

    /// Convert logical position to size fraction for floating windows.
    /// TEAM_035: Stub implementation for compatibility
    pub fn floating_logical_to_size_frac(&self, pos: Point<f64, Logical>) -> Point<f64, super::SizeFrac> {
        // TODO: Implement proper conversion using working area
        // For now, just convert the coordinates
        Point::from((pos.x, pos.y))
    }
}

/// Computes the toplevel bounds for windows in a row.
fn compute_toplevel_bounds(
    border_config: Border,
    working_area_size: Size<f64, Logical>,
    extra_size: Size<f64, Logical>,
    gaps: f64,
) -> Size<i32, Logical> {
    let mut border = 0.;
    if !border_config.off {
        border = border_config.width * 2.;
    }

    Size::from((
        f64::max(working_area_size.w - gaps * 2. - extra_size.w - border, 1.),
        f64::max(working_area_size.h - gaps * 2. - extra_size.h - border, 1.),
    ))
    .to_i32_floor()
}

// See docs/2d-canvas-plan/TODO.md for remaining work
