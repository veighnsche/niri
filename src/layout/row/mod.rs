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

mod column_data;
mod fullscreen;
mod gesture;
mod hit_test;
mod layout;
mod navigation;
mod operations;
mod render;
mod resize;
mod sizing;
mod state;
mod view_offset;

use std::cmp::max;
use std::rc::Rc;

use column_data::ColumnData;
use niri_config::utils::MergeWith;
use niri_config::{Border, PresetSize, Struts};
use niri_ipc::{ColumnDisplay, SizeChange};
pub use render::RowRenderElement;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Serial, Size};
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::SurfaceCachedState;

use super::animated_value::AnimatedValue;
use super::column::{resolve_preset_size, Column};
use super::elements::closing_window::ClosingWindow;
use super::elements::tab_indicator::TabIndicator;
use super::tile::Tile;
use super::types::{InteractiveResize, ResolvedSize};
use super::{ConfigureIntent, LayoutElement, Options};
use crate::animation::Clock;
// TEAM_060: Using RowId directly instead of WorkspaceId alias
use crate::layout::row_types::RowId;
use crate::utils::transaction::TransactionBlocker;
use crate::utils::{
    ensure_min_max_size, ensure_min_max_size_maybe_zero, send_scale_transform, ResizeEdge,
};
use crate::window::ResolvedWindowRules;

// TEAM_064: ColumnData moved to column_data.rs

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

    /// TEAM_039: Unique workspace ID for this row
    workspace_id: RowId,

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
        workspace_id: RowId,
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
            workspace_id,
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

    /// Set the row index.
    /// TEAM_059: Added for renumbering rows after cleanup
    pub fn set_idx(&mut self, idx: i32) {
        self.row_index = idx;
        self.y_offset = idx as f64 * self.view_size.h;
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

    /// Returns the row ID for this row.
    /// TEAM_039: Return the unique row ID stored in the row
    /// TEAM_055: Renamed from workspace ID to row ID
    pub fn id(&self) -> crate::layout::row_types::RowId {
        self.workspace_id
    }

    /// Sets the row index (used internally by canvas for reordering).
    pub(crate) fn set_row_index(&mut self, row_index: i32) {
        self.row_index = row_index;
    }

    /// Sets the Y offset (used internally by canvas for reordering).
    pub(crate) fn set_y_offset(&mut self, y_offset: f64) {
        self.y_offset = y_offset;
    }

    // TEAM_064: Basic queries moved to state.rs:
    // is_empty, column_count, columns, active_column_idx, active_column, active_column_mut,
    // has_window, contains, is_floating, find_column, tiles, tiles_mut, windows, windows_mut

    /// Returns the current horizontal view offset.
    pub fn view_offset_x(&self) -> f64 {
        self.view_offset_x.current()
    }

    /// Returns a reference to the view offset animated value.
    /// TEAM_035: Added for test compatibility
    pub fn view_offset(&self) -> &AnimatedValue {
        &self.view_offset_x
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

        // TEAM_049: Update cached column data with animated widths
        for (col_idx, column) in self.columns.iter().enumerate() {
            self.data[col_idx].update(column);
        }

        // Advance closing window animations
        for win in &mut self.closing_windows {
            win.advance_animations();
        }
        self.closing_windows
            .retain(|win| win.are_animations_ongoing());
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
    fn extract_animation_kind(
        anim: &crate::animation::Animation,
    ) -> crate::layout::snapshot::AnimationKindSnapshot {
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

// TEAM_064: ColumnData impl moved to column_data.rs

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
    /// TEAM_039: Properly implemented - sends scale/transform and sets size/bounds
    pub fn configure_new_window(
        &self,
        window: &smithay::desktop::Window,
        width: Option<PresetSize>,
        height: Option<PresetSize>,
        is_floating: bool,
        rules: &ResolvedWindowRules,
    ) {
        // Send scale and transform to all surfaces
        // TEAM_039: Convert f64 scale to output::Scale
        let scale = smithay::output::Scale::Fractional(self.scale);
        window.with_surfaces(|surface, data| {
            send_scale_transform(surface, data, scale, smithay::utils::Transform::Normal);
        });

        let toplevel = window.toplevel().expect("no x11 support");
        let (min_size, max_size) = with_states(toplevel.wl_surface(), |state| {
            let mut guard = state.cached_state.get::<SurfaceCachedState>();
            let current = guard.current();
            (current.min_size, current.max_size)
        });

        toplevel.with_pending_state(|state| {
            use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;

            if state.states.contains(xdg_toplevel::State::Fullscreen) {
                state.size = Some(self.view_size.to_i32_round());
            } else if state.states.contains(xdg_toplevel::State::Maximized) {
                state.size = Some(self.working_area.size.to_i32_round());
            } else {
                let size =
                    self.new_window_size(width, height, is_floating, rules, (min_size, max_size));
                state.size = Some(size);
            }

            // Set bounds for the window
            state.bounds = Some(self.new_window_toplevel_bounds(rules));
        });
    }

    /// Compute the size for a new window.
    /// TEAM_039: Ported from original Workspace implementation
    pub fn new_window_size(
        &self,
        width: Option<PresetSize>,
        height: Option<PresetSize>,
        is_floating: bool,
        rules: &ResolvedWindowRules,
        (min_size, max_size): (Size<i32, Logical>, Size<i32, Logical>),
    ) -> Size<i32, Logical> {
        // For tiled windows, use scrolling logic
        let mut size = self.scrolling_new_window_size(width, height, rules);

        // Apply min/max size constraints
        let (min_size, max_size) = rules.apply_min_max_size(min_size, max_size);
        size.w = ensure_min_max_size_maybe_zero(size.w, min_size.w, max_size.w);

        // For scrolling (where height is > 0) only ensure fixed height
        if min_size.h == max_size.h {
            size.h = ensure_min_max_size(size.h, min_size.h, max_size.h);
        } else if size.h > 0 {
            // Also always honor min height
            size.h = max(size.h, min_size.h);
        }

        size
    }

    /// Compute the size for a new scrolling window.
    /// TEAM_039: Ported from ScrollingSpace::new_window_size
    fn scrolling_new_window_size(
        &self,
        width: Option<PresetSize>,
        height: Option<PresetSize>,
        rules: &ResolvedWindowRules,
    ) -> Size<i32, Logical> {
        let border = self.options.layout.border.merged_with(&rules.border);

        let display_mode = rules
            .default_column_display
            .unwrap_or(self.options.layout.default_column_display);
        let will_tab = display_mode == ColumnDisplay::Tabbed;
        let extra = if will_tab {
            TabIndicator::new(self.options.layout.tab_indicator).extra_size(1, self.scale)
        } else {
            Size::from((0., 0.))
        };

        let working_size = self.working_area.size;

        let width = if let Some(size) = width {
            let size = match resolve_preset_size(size, &self.options, working_size.w, extra.w) {
                ResolvedSize::Tile(mut size) => {
                    if !border.off {
                        size -= border.width * 2.;
                    }
                    size
                }
                ResolvedSize::Window(size) => size,
            };

            max(1, size.floor() as i32)
        } else {
            0
        };

        let mut full_height = self.working_area.size.h - self.options.layout.gaps * 2.;
        if !border.off {
            full_height -= border.width * 2.;
        }

        let height = if let Some(height) = height {
            let height = match resolve_preset_size(height, &self.options, working_size.h, extra.h) {
                ResolvedSize::Tile(mut size) => {
                    if !border.off {
                        size -= border.width * 2.;
                    }
                    size
                }
                ResolvedSize::Window(size) => size,
            };
            f64::min(height, full_height)
        } else {
            full_height
        };

        Size::from((width, max(height.floor() as i32, 1)))
    }

    /// Compute the toplevel bounds for new windows.
    /// TEAM_039: Ported from ScrollingSpace::new_window_toplevel_bounds
    pub fn new_window_toplevel_bounds(&self, rules: &ResolvedWindowRules) -> Size<i32, Logical> {
        let border_config = self.options.layout.border.merged_with(&rules.border);

        let display_mode = rules
            .default_column_display
            .unwrap_or(self.options.layout.default_column_display);
        let will_tab = display_mode == ColumnDisplay::Tabbed;
        let extra_size = if will_tab {
            TabIndicator::new(self.options.layout.tab_indicator).extra_size(1, self.scale)
        } else {
            Size::from((0., 0.))
        };

        compute_toplevel_bounds(
            border_config,
            self.working_area.size,
            extra_size,
            self.options.layout.gaps,
        )
    }

    /// Resolve the default width for a window.
    /// TEAM_039: Ported from original Workspace implementation
    pub fn resolve_default_width(
        &self,
        default_width: Option<Option<PresetSize>>,
        is_floating: bool,
    ) -> Option<PresetSize> {
        match default_width {
            Some(Some(width)) => Some(width),
            Some(None) => None,
            None if is_floating => None,
            None => self.options.layout.default_column_width,
        }
    }

    /// Resolve the default height for a window.
    /// TEAM_039: Ported from original Workspace implementation
    pub fn resolve_default_height(
        &self,
        default_height: Option<Option<PresetSize>>,
        is_floating: bool,
    ) -> Option<PresetSize> {
        match default_height {
            Some(Some(height)) => Some(height),
            Some(None) => None,
            None if is_floating => None,
            // We don't have a global default at the moment.
            None => None,
        }
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

    pub fn set_window_height(&mut self, window: Option<&W::Id>, change: super::SizeChange) {
        if self.columns.is_empty() {
            return;
        }

        let (col_idx, tile_idx) = if let Some(window) = window {
            // Find the window across all columns
            let mut found = None;
            for (col_idx, col) in self.columns.iter_mut().enumerate() {
                if let Some(tile_idx) = col
                    .tiles
                    .iter()
                    .position(|tile| tile.window().id() == window)
                {
                    found = Some((col_idx, Some(tile_idx)));
                    break;
                }
            }
            found.unwrap_or_else(|| (self.active_column_idx, None))
        } else {
            // Use active column, no specific tile
            (self.active_column_idx, None)
        };

        if let Some(col) = self.columns.get_mut(col_idx) {
            col.set_window_height(change, tile_idx, true);
        }
    }

    // TEAM_064: reset_window_height and expand_column_to_available_width moved to sizing.rs

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

    // TEAM_064: Fullscreen/maximize methods moved to fullscreen.rs:
    // set_fullscreen, toggle_fullscreen, set_maximized, toggle_maximized,
    // get_fullscreen_size_for_window

    // TEAM_064: activate_window moved to navigation.rs

    // TEAM_035: Updated signature to accept window ID and return bool
    pub fn start_open_animation(&mut self, window: &W::Id) -> bool {
        self.columns
            .iter_mut()
            .any(|col| col.start_open_animation(window))
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

    // TEAM_064: active_window_mut, is_urgent moved to state.rs
    // TEAM_064: window_under, resize_edges_under moved to hit_test.rs

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

    // TEAM_064: active_tile_visual_rectangle, has_windows, has_windows_or_name moved to state.rs

    /// Update a window in this row.
    /// Update window state and layout.
    /// TEAM_022: Implemented based on ScrollingSpace::update_window
    /// TEAM_044: Added serial parameter for on_commit handling
    /// TEAM_050: Added view_offset_to_restore logic for fullscreen transitions
    pub fn update_window(&mut self, window: &W::Id, serial: Option<Serial>) {
        let (col_idx, column) = self
            .columns
            .iter_mut()
            .enumerate()
            .find(|(_, col)| col.contains(window))
            .unwrap();

        // TEAM_044: Find the tile and call on_commit before update_window
        let tile = column
            .tiles
            .iter_mut()
            .find(|tile| tile.window().id() == window)
            .unwrap();

        // Do this before calling update_window() so it can get up-to-date info.
        if let Some(serial) = serial {
            tile.window_mut().on_commit(serial);
        }

        let prev_width = self.data[col_idx].width;
        // TEAM_050: Track sizing mode before update for view offset save/restore
        let was_normal = column.sizing_mode().is_normal();

        // TEAM_053: Capture fullscreen size BEFORE update_window if transitioning to normal
        let fullscreen_size_to_preserve = if !was_normal {
            // Check if this window will transition to normal (unfullscreen)
            column
                .tiles
                .iter()
                .find(|tile| tile.window().id() == window)
                .and_then(|tile| tile.window().expected_size())
        } else {
            None
        };

        column.update_window(window);
        self.data[col_idx].update(column);
        column.update_tile_sizes(false);

        let offset = prev_width - self.data[col_idx].width;

        // TEAM_056: Move columns in tandem with resizing
        // When a column to the LEFT of active is resized, animate columns to the left
        // When a column at/after active is resized, animate columns to the right
        if offset != 0. {
            if self.active_column_idx <= col_idx {
                for col in &mut self.columns[col_idx + 1..] {
                    col.animate_move_from_with_config(
                        offset,
                        self.options.animations.window_resize.anim,
                    );
                }
            } else {
                // Resizing a column to the left of active - animate columns to the left
                // including the resized column itself
                for col in &mut self.columns[..=col_idx] {
                    col.animate_move_from_with_config(
                        -offset,
                        self.options.animations.window_resize.anim,
                    );
                }
            }
        }

        // TEAM_050: View offset save/restore for fullscreen transitions
        if col_idx == self.active_column_idx {
            let is_normal = self.columns[col_idx].sizing_mode().is_normal();

            // When the active column goes fullscreen, store the view offset to restore later.
            if was_normal && !is_normal {
                self.view_offset_to_restore = Some(self.view_offset_x.stationary());
            }

            // Upon unfullscreening, restore the view offset and preserve fullscreen size.
            let unfullscreen_offset = if !was_normal && is_normal {
                // TEAM_053: Set the captured fullscreen size to the tile
                if let Some(fullscreen_size) = fullscreen_size_to_preserve {
                    if let Some(tile) = self.columns[col_idx]
                        .tiles
                        .iter_mut()
                        .find(|tile| tile.window().id() == window)
                    {
                        tile.floating_window_size = Some(fullscreen_size);
                    }
                }

                self.view_offset_to_restore.take()
            } else {
                None
            };

            // We might need to move the view to ensure the resized window is still visible.
            // Only do it when the view isn't frozen by an interactive resize or a view gesture.
            if self.interactive_resize.is_none() && !self.view_offset_x.is_gesture() {
                // Restore the view offset upon unfullscreening if needed.
                if let Some(prev_offset) = unfullscreen_offset {
                    self.animate_view_offset_with_config(
                        col_idx,
                        prev_offset,
                        self.options.animations.horizontal_view_movement.0,
                    );
                }

                // Animate to ensure the column is visible
                self.animate_view_offset_to_column(None, col_idx, None);
            }
        }
    }

    /// Update the layout config for this row.
    /// TEAM_022: Stub implementation
    pub fn update_layout_config(&mut self, _config: Option<niri_config::LayoutPart>) {
        // TEAM_022: TODO - rows don't have individual layout configs
    }

    /// Resolve scrolling width for a window.
    /// TEAM_039: Properly implemented - returns ColumnWidth based on preset or window size
    pub fn resolve_scrolling_width(
        &self,
        window: &W,
        width: Option<PresetSize>,
    ) -> crate::layout::types::ColumnWidth {
        let width = width.unwrap_or_else(|| PresetSize::Fixed(window.size().w));
        match width {
            PresetSize::Fixed(fixed) => {
                let mut fixed = f64::from(fixed);

                // Add border width since ColumnWidth includes borders.
                let rules = window.rules();
                let border = self.options.layout.border.merged_with(&rules.border);
                if !border.off {
                    fixed += border.width * 2.;
                }

                crate::layout::types::ColumnWidth::Fixed(fixed)
            }
            PresetSize::Proportion(prop) => crate::layout::types::ColumnWidth::Proportion(prop),
        }
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

    // TEAM_064: find_wl_surface, find_wl_surface_mut moved to state.rs

    /// Get popup target rectangle.
    /// TEAM_025: Stub implementation
    /// TEAM_035: Updated return type to Rectangle<f64, Logical>
    pub fn popup_target_rect(&self, _window: &W::Id) -> Option<Rectangle<f64, Logical>> {
        // TEAM_025: TODO - implement popup target rect
        None
    }

    // TEAM_064: activate_window_without_raising moved to navigation.rs

    /// Get tiles with IPC layouts.
    /// TEAM_025: Stub implementation
    /// TEAM_035: Updated return type to iterator of (tile, layout) tuples
    pub fn tiles_with_ipc_layouts(
        &self,
    ) -> impl Iterator<Item = (&Tile<W>, niri_ipc::WindowLayout)> {
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
    // TEAM_040: Implemented based on ScrollingSpace::toggle_column_tabbed_display
    pub fn toggle_column_tabbed_display(&mut self) {
        if self.columns.is_empty() {
            return;
        }

        let col = &mut self.columns[self.active_column_idx];
        let display = match col.display_mode {
            ColumnDisplay::Normal => ColumnDisplay::Tabbed,
            ColumnDisplay::Tabbed => ColumnDisplay::Normal,
        };

        self.set_column_display(display);
    }

    /// Set column display mode.
    // TEAM_040: Implemented based on ScrollingSpace::set_column_display
    pub fn set_column_display(&mut self, display: ColumnDisplay) {
        if self.columns.is_empty() {
            return;
        }

        let col = &mut self.columns[self.active_column_idx];
        if col.display_mode == display {
            return;
        }

        cancel_resize_for_column(&mut self.interactive_resize, col);
        col.set_column_display(display);

        // With place_within_column, the tab indicator changes the column size immediately.
        self.data[self.active_column_idx].update(col);
        col.update_tile_sizes(true);

        // Disable fullscreen if needed.
        if col.display_mode != ColumnDisplay::Tabbed && col.tiles.len() > 1 {
            let window = col.tiles[col.active_tile_idx].window().id().clone();
            self.set_fullscreen(&window, false);
            self.set_maximized(&window, false);
        }
    }

    /// Center the active column.
    /// TEAM_039: Implemented based on ScrollingSpace::center_column
    pub fn center_column(&mut self) {
        if self.columns.is_empty() {
            return;
        }

        self.animate_view_offset_to_column_centered(
            None,
            self.active_column_idx,
            self.options.animations.horizontal_view_movement.0,
        );

        // Cancel any interactive resize on the active column
        let col = &mut self.columns[self.active_column_idx];
        if let Some(resize) = &self.interactive_resize {
            if col.contains(&resize.window) {
                self.interactive_resize = None;
            }
        }
    }

    /// Center all visible columns.
    /// TEAM_028: Stub implementation
    pub fn center_visible_columns(&mut self) {
        // TEAM_028: TODO - implement visible columns centering
    }

    // TEAM_064: Sizing methods moved to sizing.rs:
    // toggle_width, toggle_window_width, toggle_window_height, toggle_full_width,
    // set_column_width, set_window_width

    /// Get scrolling insert position.
    /// TEAM_028: Stub implementation
    /// TEAM_035: Updated return type to InsertPosition
    pub fn scrolling_insert_position(
        &self,
        _pos: Point<f64, Logical>,
    ) -> super::types::InsertPosition {
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
    pub fn start_close_animation_for_window(
        &mut self,
        _renderer: &mut GlesRenderer,
        _window: &W::Id,
        _blocker: TransactionBlocker,
    ) {
        // TEAM_028: TODO - implement close animation
    }

    /// Start close animation for a tile with snapshot.
    /// TEAM_033: Added for interactive move window closing
    pub fn start_close_animation_for_tile(
        &mut self,
        renderer: &mut GlesRenderer,
        snapshot: crate::layout::tile::TileRenderSnapshot,
        tile_size: Size<f64, Logical>,
        tile_pos: Point<f64, Logical>,
        blocker: TransactionBlocker,
    ) {
        // TEAM_033: Implemented proper close animation with snapshot
        // Based on ScrollingSpace::start_close_animation_for_tile

        let anim = crate::animation::Animation::new(
            self.clock.clone(),
            0.,
            1.,
            0.,
            self.options.animations.window_close.anim,
        );

        let blocker = if self.options.disable_transactions {
            TransactionBlocker::completed()
        } else {
            blocker
        };

        let scale = smithay::utils::Scale::from(self.scale);
        let res = ClosingWindow::new(
            renderer, snapshot, scale, tile_size, tile_pos, blocker, anim,
        );
        match res {
            Ok(closing) => {
                self.closing_windows.push(closing);
            }
            Err(err) => {
                tracing::warn!("error creating a closing window animation: {err:?}");
            }
        }
    }

    /// Convert logical position to size fraction for floating windows.
    /// TEAM_057: Implemented proper conversion using working area
    pub fn floating_logical_to_size_frac(
        &self,
        pos: Point<f64, Logical>,
    ) -> Point<f64, super::SizeFrac> {
        // Convert from logical coordinates to size fraction (0.0 to 1.0 relative to working area)
        let relative_pos = pos - self.working_area.loc;
        Point::from((
            relative_pos.x / f64::max(self.working_area.size.w, 1.0),
            relative_pos.y / f64::max(self.working_area.size.h, 1.0),
        ))
    }

    /// TEAM_057: Compute the area for the insert hint based on position.
    /// Ported from ScrollingSpace::insert_hint_area
    pub fn insert_hint_area(
        &self,
        position: super::InsertPosition,
    ) -> Option<Rectangle<f64, Logical>> {
        use super::InsertPosition;

        let hint_area = match position {
            InsertPosition::NewColumn(column_index) => {
                if column_index == 0 || column_index == self.columns.len() {
                    let size = Size::from((
                        300.,
                        self.working_area.size.h - self.options.layout.gaps * 2.,
                    ));
                    let mut loc = Point::from((
                        self.column_x(column_index),
                        self.working_area.loc.y + self.options.layout.gaps,
                    ));
                    if column_index == 0 && !self.columns.is_empty() {
                        loc.x -= size.w + self.options.layout.gaps;
                    }
                    Rectangle::new(loc, size)
                } else if column_index > self.columns.len() {
                    return None;
                } else {
                    let size = Size::from((
                        300.,
                        self.working_area.size.h - self.options.layout.gaps * 2.,
                    ));
                    let loc = Point::from((
                        self.column_x(column_index) - size.w / 2. - self.options.layout.gaps / 2.,
                        self.working_area.loc.y + self.options.layout.gaps,
                    ));
                    Rectangle::new(loc, size)
                }
            }
            InsertPosition::InColumn(column_index, tile_index) => {
                if column_index >= self.columns.len() {
                    return None;
                }

                let col = &self.columns[column_index];
                if tile_index > col.tiles.len() {
                    return None;
                }

                let top = col.tile_offset(tile_index).y;
                let height = if tile_index == 0 || tile_index == col.tiles.len() {
                    150.
                } else {
                    300.
                };
                let y = if tile_index == 0 {
                    top
                } else if tile_index == col.tiles.len() {
                    top - self.options.layout.gaps - 150.
                } else {
                    top - self.options.layout.gaps / 2. - 150.
                };

                let size = Size::from((self.data[column_index].width, height));
                let loc = Point::from((self.column_x(column_index), y));
                Rectangle::new(loc, size)
            }
            InsertPosition::Floating => return None,
        };

        Some(hint_area)
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

// TEAM_040: Helper function to cancel interactive resize for a column
fn cancel_resize_for_column<W: LayoutElement>(
    interactive_resize: &mut Option<InteractiveResize<W>>,
    column: &mut Column<W>,
) {
    if let Some(resize) = interactive_resize {
        if column.contains(&resize.window) {
            *interactive_resize = None;
        }
    }

    for tile in &mut column.tiles {
        tile.window_mut().cancel_interactive_resize();
    }
}

// See docs/2d-canvas-plan/TODO.md for remaining work
