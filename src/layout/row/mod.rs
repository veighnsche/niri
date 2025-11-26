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

use niri_config::Struts;
use smithay::utils::{Logical, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::closing_window::ClosingWindow;
use super::column::Column;
use super::types::InteractiveResize;
use super::{LayoutElement, Options};
use crate::animation::Clock;

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
    // Golden Snapshot Testing
    // TEAM_010: Added snapshot() for golden test compatibility
    // =========================================================================

    /// Creates a snapshot of this row's layout state for golden testing.
    ///
    /// This produces the same format as ScrollingSpace.snapshot() to ensure
    /// golden tests pass after the Monitor refactor.
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

// See docs/2d-canvas-plan/TODO.md for remaining work
