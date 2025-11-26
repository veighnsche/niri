// TEAM_013: LEGACY workspace compatibility layer
//!
//! **THIS ENTIRE FILE IS LEGACY CODE AND WILL BE DELETED.**
//!
//! All workspace-related methods are here for easy removal when
//! Canvas2D fully replaces workspaces.

use smithay::utils::{Logical, Size};

// TEAM_014: Removed OverviewProgress import (Part 3)
use crate::layout::monitor::{Monitor, WorkspaceSwitch};
use crate::layout::workspace::Workspace;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // LEGACY: Workspace accessors
    // =========================================================================

    pub fn active_workspace_idx(&self) -> usize {
        self.active_workspace_idx
    }

    // TEAM_020: Migrate to canvas - this method should eventually be removed
    pub fn active_workspace_ref(&self) -> &Workspace<W> {
        &self.workspaces[self.active_workspace_idx]
    }

    pub fn find_named_workspace(&self, workspace_name: &str) -> Option<&Workspace<W>> {
        self.workspaces.iter().find(|ws| {
            ws.name
                .as_ref()
                .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
        })
    }

    pub fn find_named_workspace_index(&self, workspace_name: &str) -> Option<usize> {
        self.workspaces.iter().position(|ws| {
            ws.name
                .as_ref()
                .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
        })
    }

    // TEAM_020: Migrate to canvas - this method should eventually be removed
    pub fn active_workspace(&mut self) -> &mut Workspace<W> {
        &mut self.workspaces[self.active_workspace_idx]
    }

    // TEAM_021: Canvas-based workspace iteration for migration
    pub fn workspaces(&self) -> impl Iterator<Item = (usize, &Workspace<W>)> {
        // For now, return legacy workspaces but also provide canvas iteration
        self.workspaces.iter().enumerate()
    }

    pub fn workspaces_mut(&mut self) -> impl Iterator<Item = &mut Workspace<W>> + '_ {
        self.workspaces.iter_mut()
    }

    // TEAM_021: Canvas-first methods - try canvas first, fallback to workspace
    pub fn canvas_workspaces(&self) -> impl Iterator<Item = (i32, &crate::layout::row::Row<W>)> {
        self.canvas().workspaces()
    }

    pub fn canvas_workspaces_mut(&mut self) -> impl Iterator<Item = &mut crate::layout::row::Row<W>> + '_ {
        self.canvas_mut().workspaces_mut()
    }

    pub fn canvas_active_workspace(&self) -> Option<&crate::layout::row::Row<W>> {
        self.canvas().active_workspace()
    }

    pub fn canvas_active_workspace_mut(&mut self) -> Option<&mut crate::layout::row::Row<W>> {
        self.canvas_mut().active_workspace_mut()
    }

    pub fn into_workspaces(mut self) -> Vec<Workspace<W>> {
        self.workspaces.retain(|ws| ws.has_windows_or_name());

        for ws in &mut self.workspaces {
            ws.set_output(None);
        }

        self.workspaces
    }

    // =========================================================================
    // TEAM_020: Window query methods (migrating to canvas-only)
    // =========================================================================

    /// Returns all windows on this monitor (tiled and floating).
    /// TEAM_020: Now uses canvas only
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        self.canvas.windows()
    }

    /// Returns whether this monitor contains the given window.
    /// TEAM_020: Now uses canvas only
    pub fn has_window(&self, window: &W::Id) -> bool {
        self.canvas.has_window(window)
    }

    /// Returns the active window on this monitor.
    /// TEAM_020: Now uses canvas instead of workspace
    pub fn active_window(&self) -> Option<&W> {
        self.canvas.active_window()
    }

    // =========================================================================
    // TEAM_020: Canvas workspace iteration methods
    // These methods replace workspace iteration with canvas iteration
    // =========================================================================

    /// Canvas equivalent of workspaces() - provides iteration over canvas rows
    /// TEAM_020: Use this to replace workspaces() calls
    pub fn canvas_rows(&self) -> impl Iterator<Item = (Option<&Monitor<W>>, i32, &crate::layout::row::Row<W>)> + '_ {
        self.canvas.rows().map(move |(idx, row)| (Some(self), idx, row))
    }

    /// Canvas equivalent of workspaces_mut() - provides mutable iteration over canvas rows
    /// TEAM_020: Use this to replace workspaces_mut() calls
    pub fn canvas_rows_mut(&mut self) -> impl Iterator<Item = &mut crate::layout::row::Row<W>> + '_ {
        self.canvas.rows_mut()
    }

    /// Find window in canvas (replaces workspace.find_window patterns)
    /// TEAM_020: Use this to replace workspace window finding
    pub fn canvas_find_window(&self, id: &W::Id) -> Option<(i32, &crate::layout::row::Row<W>, &crate::layout::tile::Tile<W>)> {
        self.canvas.find_window(id)
    }

    /// Find window in canvas (mutable, replaces workspace.find_window patterns)
    /// TEAM_020: Use this to replace workspace window finding
    pub fn canvas_find_window_mut(&mut self, id: &W::Id) -> Option<(i32, &mut crate::layout::tile::Tile<W>)> {
        self.canvas.find_window_mut(id)
    }

    // =========================================================================
    // LEGACY: Animation methods (workspace-based)
    // =========================================================================

    pub fn advance_animations(&mut self) {
        match &mut self.workspace_switch {
            Some(WorkspaceSwitch::Animation(anim)) => {
                if anim.is_done() {
                    self.workspace_switch = None;
                    self.clean_up_workspaces();
                }
            }
            Some(WorkspaceSwitch::Gesture(gesture)) => {
                if let Some(last_time) = &mut gesture.dnd_last_event_time {
                    let now = self.clock.now_unadjusted();
                    if *last_time != now {
                        *last_time = now;
                        gesture.dnd_nonzero_start_time = None;
                    }
                }

                if let Some(anim) = &mut gesture.animation {
                    if anim.is_done() {
                        gesture.animation = None;
                    }
                }
            }
            None => (),
        }

        for ws in &mut self.workspaces {
            ws.advance_animations();
        }
    }

    pub(in crate::layout) fn are_animations_ongoing(&self) -> bool {
        self.workspace_switch
            .as_ref()
            .is_some_and(|s| s.is_animation_ongoing())
            || self.workspaces.iter().any(|ws| ws.are_animations_ongoing())
    }

    pub fn are_transitions_ongoing(&self) -> bool {
        self.workspace_switch.is_some()
            || self
                .workspaces
                .iter()
                .any(|ws| ws.are_transitions_ongoing())
    }

    // TEAM_014: Removed overview methods (Part 3)
    // overview_zoom, set_overview_progress, overview_progress_value are no longer needed

    // =========================================================================
    // LEGACY: Helper methods (workspace geometry)
    // =========================================================================

    pub(super) fn workspace_size(&self, zoom: f64) -> Size<f64, Logical> {
        let ws_size = self.view_size.upscale(zoom);
        let scale = self.scale.fractional_scale();
        ws_size.to_physical_precise_ceil(scale).to_logical(scale)
    }

    pub(super) fn workspace_gap(&self, zoom: f64) -> f64 {
        let scale = self.scale.fractional_scale();
        let gap = self.view_size.h * 0.1 * zoom;
        crate::utils::round_logical_in_physical_max1(scale, gap)
    }

    pub(super) fn workspace_size_with_gap(&self, zoom: f64) -> Size<f64, Logical> {
        let gap = self.workspace_gap(zoom);
        self.workspace_size(zoom) + Size::from((0., gap))
    }

    pub(super) fn previous_workspace_idx(&self) -> Option<usize> {
        let id = self.previous_workspace_id?;
        self.workspaces.iter().position(|w| w.id() == id)
    }
}

// =========================================================================
// LEGACY: Test-only methods
// =========================================================================

#[cfg(test)]
impl<W: LayoutElement> Monitor<W> {
    pub(in crate::layout) fn verify_invariants(&self) {
        use approx::assert_abs_diff_eq;
        use crate::layout::Options;

        let options =
            Options::clone(&self.base_options).with_merged_layout(self.layout_config.as_ref());
        assert_eq!(&*self.options, &options);

        assert!(
            !self.workspaces.is_empty(),
            "monitor must have at least one workspace"
        );
        assert!(self.active_workspace_idx < self.workspaces.len());

        if let Some(WorkspaceSwitch::Animation(anim)) = &self.workspace_switch {
            let before_idx = anim.from() as usize;
            let after_idx = anim.to() as usize;

            assert!(before_idx < self.workspaces.len());
            assert!(after_idx < self.workspaces.len());
        }

        assert!(
            !self.workspaces.last().unwrap().has_windows(),
            "monitor must have an empty workspace in the end"
        );
        if self.options.layout.empty_workspace_above_first {
            assert!(
                !self.workspaces.first().unwrap().has_windows(),
                "first workspace must be empty when empty_workspace_above_first is set"
            )
        }

        assert!(
            self.workspaces.last().unwrap().name.is_none(),
            "monitor must have an unnamed workspace in the end"
        );
        if self.options.layout.empty_workspace_above_first {
            assert!(
                self.workspaces.first().unwrap().name.is_none(),
                "first workspace must be unnamed when empty_workspace_above_first is set"
            )
        }

        if self.options.layout.empty_workspace_above_first {
            assert!(
                self.workspaces.len() != 2,
                "if empty_workspace_above_first is set there must be just 1 or 3+ workspaces"
            )
        }

        let pre_skip = if self.options.layout.empty_workspace_above_first {
            1
        } else {
            0
        };
        if self.workspace_switch.is_none() {
            for (idx, ws) in self
                .workspaces
                .iter()
                .enumerate()
                .skip(pre_skip)
                .rev()
                .skip(1)
            {
                if idx != self.active_workspace_idx {
                    assert!(
                        ws.has_windows_or_name(),
                        "non-active workspace can't be empty and unnamed except the last one"
                    );
                }
            }
        }

        for workspace in &self.workspaces {
            assert_eq!(self.clock, workspace.clock);

            assert_eq!(
                self.scale().integer_scale(),
                workspace.scale().integer_scale()
            );
            assert_eq!(
                self.scale().fractional_scale(),
                workspace.scale().fractional_scale()
            );
            assert_eq!(self.view_size, workspace.view_size());
            assert_eq!(self.working_area, workspace.working_area());

            assert_eq!(
                workspace.base_options, self.options,
                "workspace options must be synchronized with monitor"
            );
        }

        let scale = self.scale().fractional_scale();
        let iter = self.workspaces_with_render_geo();
        for (_ws, ws_geo) in iter {
            let pos = ws_geo.loc;
            let rounded_pos = pos.to_physical_precise_round(scale).to_logical(scale);

            assert_abs_diff_eq!(pos.x, rounded_pos.x, epsilon = 1e-5);
            assert_abs_diff_eq!(pos.y, rounded_pos.y, epsilon = 1e-5);
        }
    }
}
