// TEAM_013: LEGACY workspace compatibility layer
//!
//! **THIS ENTIRE FILE IS LEGACY CODE AND WILL BE DELETED.**
//!
//! All workspace-related methods are here for easy removal when
//! Canvas2D fully replaces workspaces.

use smithay::utils::{Logical, Size};

use crate::layout::monitor::{Monitor, OverviewProgress, WorkspaceSwitch};
use crate::layout::workspace::Workspace;
use crate::layout::LayoutElement;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // LEGACY: Workspace accessors
    // =========================================================================

    pub fn active_workspace_idx(&self) -> usize {
        self.active_workspace_idx
    }

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

    pub fn active_workspace(&mut self) -> &mut Workspace<W> {
        &mut self.workspaces[self.active_workspace_idx]
    }

    pub fn into_workspaces(mut self) -> Vec<Workspace<W>> {
        self.workspaces.retain(|ws| ws.has_windows_or_name());

        for ws in &mut self.workspaces {
            ws.set_output(None);
        }

        self.workspaces
    }

    // =========================================================================
    // LEGACY: Window query methods (workspace-based)
    // TODO(TEAM_013): Replace with canvas-only versions
    // =========================================================================

    /// Returns all windows on this monitor (tiled and floating).
    pub fn windows(&self) -> impl Iterator<Item = &W> + '_ {
        let canvas_windows = self.canvas.windows();
        let workspace_windows = self.workspaces.iter().flat_map(|ws| ws.windows());
        canvas_windows.chain(workspace_windows)
    }

    /// Returns whether this monitor contains the given window.
    pub fn has_window(&self, window: &W::Id) -> bool {
        self.canvas.contains_any(window)
            || self.workspaces.iter().any(|ws| ws.has_window(window))
    }

    /// Returns the active window on this monitor.
    pub fn active_window(&self) -> Option<&W> {
        self.active_workspace_ref().active_window()
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

    // =========================================================================
    // LEGACY: Overview methods
    // =========================================================================

    pub fn overview_zoom(&self) -> f64 {
        let progress = self.overview_progress.as_ref().map(|p| p.value());
        crate::layout::compute_overview_zoom(&self.options, progress)
    }

    pub(in crate::layout) fn set_overview_progress(
        &mut self,
        progress: Option<&crate::layout::OverviewProgress>,
    ) {
        let prev_render_idx = self.workspace_render_idx();
        self.overview_progress = progress.map(OverviewProgress::from);
        let new_render_idx = self.workspace_render_idx();

        if prev_render_idx != new_render_idx {
            if let Some(WorkspaceSwitch::Animation(anim)) = &mut self.workspace_switch {
                *anim = anim.restarted(prev_render_idx, anim.to(), 0.);
            }
        }
    }

    #[cfg(test)]
    pub(in crate::layout) fn overview_progress_value(&self) -> Option<f64> {
        self.overview_progress.as_ref().map(|p| p.value())
    }

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
