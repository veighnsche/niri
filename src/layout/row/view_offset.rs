// TEAM_007: View offset calculation extracted from mod.rs
//!
//! This module handles view offset (horizontal scroll) calculation and animation.
//! Ported from ScrollingSpace with support for CenterFocusedColumn modes.

use std::cmp::min;

use niri_config::CenterFocusedColumn;

use super::Row;
use crate::animation::Animation;
use crate::layout::animated_value::AnimatedValue;
use crate::layout::{LayoutElement, SizingMode};

impl<W: LayoutElement> Row<W> {
    // =========================================================================
    // View offset queries
    // =========================================================================

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

    // =========================================================================
    // Column X position calculation
    // =========================================================================

    /// Returns the X position of the column at the given index.
    pub(crate) fn column_x(&self, idx: usize) -> f64 {
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
    // View offset computation
    // =========================================================================

    pub(crate) fn compute_new_view_offset_fit(
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

    pub(crate) fn compute_new_view_offset_centered(
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

    pub(crate) fn compute_new_view_offset_for_column_fit(
        &self,
        target_x: Option<f64>,
        idx: usize,
    ) -> f64 {
        let col = &self.columns[idx];
        self.compute_new_view_offset_fit(
            target_x,
            self.column_x(idx),
            col.width(),
            col.sizing_mode(),
        )
    }

    pub(crate) fn compute_new_view_offset_for_column_centered(
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

    pub(crate) fn compute_new_view_offset_for_column(
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

    // =========================================================================
    // View offset animation
    // =========================================================================

    pub(crate) fn animate_view_offset(&mut self, idx: usize, new_view_offset: f64) {
        self.animate_view_offset_with_config(
            idx,
            new_view_offset,
            self.options.animations.horizontal_view_movement.0,
        );
    }

    pub(crate) fn animate_view_offset_with_config(
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

    pub(crate) fn animate_view_offset_to_column_centered(
        &mut self,
        target_x: Option<f64>,
        idx: usize,
        config: niri_config::Animation,
    ) {
        let new_view_offset = self.compute_new_view_offset_for_column_centered(target_x, idx);
        self.animate_view_offset_with_config(idx, new_view_offset, config);
    }

    pub(crate) fn animate_view_offset_to_column_with_config(
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
    pub(crate) fn animate_view_offset_to_column(
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
}

// =========================================================================
// Standalone helper functions
// =========================================================================

/// Computes the new view offset to show a column.
///
/// This helper is used by `compute_new_view_offset_fit` to determine the
/// optimal scroll position.
pub(crate) fn compute_new_view_offset(
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
