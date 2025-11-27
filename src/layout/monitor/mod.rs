// TEAM_013: Monitor module - modular refactor of monitor.rs
//!
//! This module contains the Monitor struct which manages a single output's layout.
//! 
//! The Monitor owns:
//! - A Canvas2D for 2D layout (new system)
//! - Workspaces for legacy compatibility (being phased out)
//! - Insert hint state for window placement
//! - Overview state (being phased out)

use std::rc::Rc;

use niri_config::LayoutPart;
use smithay::backend::renderer::element::utils::{RelocateRenderElement, RescaleRenderElement};
use smithay::output::Output;
use smithay::utils::{Logical, Rectangle, Size};

use crate::animation::Clock;
use crate::layout::canvas::Canvas2D;
use crate::layout::insert_hint_element::InsertHintElement;
use crate::layout::workspace_types::{WorkspaceId, compute_working_area};
use crate::layout::row::Row;
use crate::layout::{LayoutElement, Options};
use crate::niri_render_elements;
use crate::utils::output_size;

// TEAM_013: Submodules
mod types;
// TEAM_021: Legacy workspace modules removed - functionality migrated to Canvas2D
mod navigation;       // LEGACY: Workspace navigation
mod render;           // LEGACY: Workspace rendering
mod hit_test;         // LEGACY: Workspace hit testing
mod config;
mod gestures;         // LEGACY: Workspace gestures

// TEAM_013: Re-exports
// TEAM_014: Removed OverviewProgress from re-exports (Part 3)
pub use types::{
    InsertHint, InsertHintRenderLoc, InsertPosition, InsertWorkspace, MonitorAddWindowTarget,
    WorkspaceSwitch, WorkspaceSwitchGesture,
    WORKSPACE_DND_EDGE_SCROLL_MOVEMENT, WORKSPACE_GESTURE_MOVEMENT, WORKSPACE_GESTURE_RUBBER_BAND,
};

// TEAM_013: Phase 1.5.3 — Monitor now uses Canvas2D instead of workspaces.
// Workspaces are KEPT for gradual migration. One infinite canvas per output.
#[derive(Debug)]
pub struct Monitor<W: LayoutElement> {
    /// Output for this monitor.
    pub(in crate::layout) output: Output,
    /// Cached name of the output.
    output_name: String,
    /// Latest known scale for this output.
    scale: smithay::output::Scale,
    /// Latest known size for this output.
    view_size: Size<f64, Logical>,
    /// Latest known working area for this output.
    ///
    /// Not rounded to physical pixels.
    // FIXME: since this is used for things like DnD scrolling edges in the overview, ideally this
    // should only consider overlay and top layer-shell surfaces. However, Smithay doesn't easily
    // let you do this at the moment.
    working_area: Rectangle<f64, Logical>,

    // =========================================================================
    // TEAM_010: Canvas2D replaces workspaces
    // =========================================================================

    /// The 2D canvas containing all windows on this output.
    pub(in crate::layout) canvas: Canvas2D<W>,

    // TEAM_022: Legacy workspace fields removed - Canvas2D is now sole layout system
    /// LEGACY: In-progress switch between rows (was workspaces).
    pub(in crate::layout) workspace_switch: Option<WorkspaceSwitch>,

    // =========================================================================
    // Shared state
    // =========================================================================

    /// Indication where an interactively-moved window is about to be placed.
    pub(in crate::layout) insert_hint: Option<InsertHint>,
    /// Insert hint element for rendering.
    insert_hint_element: InsertHintElement,
    /// Location to render the insert hint element.
    insert_hint_render_loc: Option<InsertHintRenderLoc>,
    // TEAM_014: Removed overview_open and overview_progress fields (Part 3)
    /// Clock for driving animations.
    pub(in crate::layout) clock: Clock,
    /// Configurable properties of the layout as received from the parent layout.
    pub(in crate::layout) base_options: Rc<Options>,
    /// Configurable properties of the layout.
    pub(in crate::layout) options: Rc<Options>,
    /// Layout config overrides for this monitor.
    layout_config: Option<niri_config::LayoutPart>,
}

// TEAM_022: Render element types - now uses Canvas2D instead of Workspace
niri_render_elements! {
    MonitorInnerRenderElement<R> => {
        Canvas = smithay::backend::renderer::element::utils::CropRenderElement<
            crate::layout::canvas::Canvas2DRenderElement<R>>,
        InsertHint = smithay::backend::renderer::element::utils::CropRenderElement<crate::layout::insert_hint_element::InsertHintRenderElement>,
        UncroppedInsertHint = crate::layout::insert_hint_element::InsertHintRenderElement,
        Shadow = crate::render_helpers::shadow::ShadowRenderElement,
        SolidColor = crate::render_helpers::solid_color::SolidColorRenderElement,
    }
}

pub type MonitorRenderElement<R> =
    RelocateRenderElement<RescaleRenderElement<MonitorInnerRenderElement<R>>>;

impl<W: LayoutElement> Monitor<W> {
    // TEAM_022: Simplified constructor - Canvas2D is now the sole layout system
    pub fn new(
        output: Output,
        clock: Clock,
        base_options: Rc<Options>,
        layout_config: Option<LayoutPart>,
        initial_workspace_id: WorkspaceId,
    ) -> Self {
        let options =
            Rc::new(Options::clone(&base_options).with_merged_layout(layout_config.as_ref()));

        let scale = output.current_scale();
        let view_size = output_size(&output);
        let working_area = compute_working_area(&output);

        // TEAM_022: Create Canvas2D as the sole layout system
        let canvas = Canvas2D::new(
            Some(output.clone()),
            view_size,
            working_area, // parent_area = working_area
            working_area,
            scale.fractional_scale(),
            clock.clone(),
            options.clone(),
            initial_workspace_id,
        );

        Self {
            output_name: output.name(),
            output,
            scale,
            view_size,
            working_area,
            // TEAM_022: Canvas2D is the sole layout system
            canvas,
            insert_hint: None,
            insert_hint_element: InsertHintElement::new(options.layout.insert_hint),
            insert_hint_render_loc: None,
            // TEAM_022: Keep workspace_switch for row switching animations
            workspace_switch: None,
            clock,
            base_options,
            options,
            layout_config,
        }
    }

    // =========================================================================
    // Output accessors
    // =========================================================================

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn output_name(&self) -> &String {
        &self.output_name
    }

    pub fn scale(&self) -> smithay::output::Scale {
        self.scale
    }

    pub fn view_size(&self) -> Size<f64, Logical> {
        self.view_size
    }

    pub fn working_area(&self) -> Rectangle<f64, Logical> {
        self.working_area
    }

    // TEAM_022: Returns active row index (was workspace index)
    pub fn active_workspace_idx(&self) -> usize {
        self.canvas.active_row_idx() as usize
    }

    // TEAM_022: Returns the number of rows (was workspace count)
    pub fn workspace_count(&self) -> usize {
        self.canvas.rows().count()
    }

    // TEAM_022: Returns active row as active_workspace for compatibility
    pub fn active_workspace(&mut self) -> Option<&mut Row<W>> {
        self.canvas.active_row_mut()
    }

    // TEAM_022: Returns active row ref as active_workspace_ref for compatibility
    pub fn active_workspace_ref(&self) -> Option<&Row<W>> {
        self.canvas.active_row()
    }

    // TEAM_021: Legacy workspace compatibility method
    pub fn are_transitions_ongoing(&self) -> bool {
        // Check canvas for ongoing transitions
        self.canvas.are_transitions_ongoing()
    }

    // TEAM_021: Legacy workspace compatibility method
    pub fn move_workspace_to_idx(&mut self, _old_idx: usize, _new_idx: usize) {
        // Empty stub - workspace movement is now handled by canvas
    }

    pub fn layout_config(&self) -> Option<&niri_config::LayoutPart> {
        self.layout_config.as_ref()
    }

    // =========================================================================
    // TEAM_010: Canvas2D accessors
    // =========================================================================

    /// Returns a reference to the canvas.
    pub fn canvas(&self) -> &Canvas2D<W> {
        &self.canvas
    }

    /// Returns a mutable reference to the canvas.
    pub fn canvas_mut(&mut self) -> &mut Canvas2D<W> {
        &mut self.canvas
    }

    // =========================================================================
    // TEAM_022: Legacy workspace compatibility methods
    // These route to Canvas2D operations
    // =========================================================================

    /// Find a named row/workspace by name.
    pub fn find_named_workspace(&self, name: &str) -> Option<&Row<W>> {
        for (_, row) in self.canvas.rows() {
            if let Some(row_name) = row.name() {
                if row_name == name {
                    return Some(row);
                }
            }
        }
        None
    }

    /// Get active window from canvas.
    pub fn active_window(&self) -> Option<&W> {
        self.canvas.active_window()
    }

    /// Clean up empty rows in the canvas.
    pub fn clean_up_workspaces(&mut self) {
        self.canvas.cleanup_empty_rows();
    }

    /// Add a workspace/row at top.
    pub fn add_workspace_top(&mut self) {
        // TEAM_022: Create a new row above current
        // For now, just ensure row -1 exists
        self.canvas.ensure_row(-1);
    }

    /// Add a workspace/row at bottom.
    pub fn add_workspace_bottom(&mut self) {
        // Find the max row index and add one below
        let max_idx = self.canvas.rows().map(|(i, _)| i).max().unwrap_or(0);
        self.canvas.ensure_row(max_idx + 1);
    }

    /// Get workspace size with gap.
    pub fn workspace_size_with_gap(&self, _zoom: f64) -> f64 {
        self.view_size.h
    }

    /// Convert monitor into workspaces (for output removal).
    /// TEAM_022: Returns empty since we don't use workspace Vec anymore.
    pub fn into_workspaces(self) -> Vec<()> {
        Vec::new()
    }

    /// TEAM_033: Convert monitor into its canvas (consumes self).
    /// Used when the last monitor is removed to preserve window state.
    pub fn into_canvas(self) -> Canvas2D<W> {
        self.canvas
    }

    /// Append workspaces from another monitor.
    pub fn append_workspaces(&mut self, _workspaces: Vec<()>) {
        // TEAM_022: No-op - workspace Vec is no longer used
    }

    /// TEAM_033: Append canvas from another monitor.
    /// Used when a monitor is removed and windows need to be transferred.
    pub fn append_canvas(&mut self, other_canvas: Canvas2D<W>) {
        // Transfer all rows from other canvas to this one
        for (idx, row) in other_canvas.rows {
            // TODO(TEAM_033): Properly merge rows - for now just append as new rows
            let new_idx = self.canvas.rows.keys().max().unwrap_or(&-1) + 1;
            // Rows need to be updated with new output info
            let mut row = row;
            // Update row with this monitor's config
            row.update_config(
                self.view_size,
                self.working_area,
                self.scale.fractional_scale(),
                self.options.clone(),
            );
            self.canvas.rows.insert(new_idx, row);
        }
    }

    /// Add a column to a row.
    pub fn add_column(
        &mut self,
        _row_idx: usize,
        _column: crate::layout::column::Column<W>,
        _activate: bool,
    ) {
        // TEAM_022: TODO - implement proper column addition to canvas
    }

    /// Resolve add window target.
    pub fn resolve_add_window_target(
        &self,
        _target: &MonitorAddWindowTarget<W>,
    ) -> (i32, Option<usize>) {
        // TEAM_022: Return active row for now
        (self.canvas.active_row_idx(), None)
    }

    /// Add a window to the monitor.
    pub fn add_window(
        &mut self,
        window: W,
        target: Option<MonitorAddWindowTarget<W>>,
        activate: crate::layout::ActivateWindow,
        _width: Option<crate::layout::types::ColumnWidth>,
        _is_full_width: bool,
    ) -> Option<()> {
        // TEAM_022: Add window to canvas
        let (row_idx, _col_idx) = if let Some(target) = target {
            self.resolve_add_window_target(&target)
        } else {
            (self.canvas.active_row_idx(), None)
        };

        // Create tile and add to row
        let tile = self.canvas.make_tile(window);
        let width = crate::layout::types::ColumnWidth::Proportion(1.0);
        // TEAM_039: Use map_smart to properly handle ActivateWindow::Smart
        // Smart should activate unless there's a reason not to (like pending fullscreen)
        let should_activate = activate.map_smart(|| true);
        self.canvas.add_tile_to_row(
            row_idx,
            tile,
            should_activate,
            width,
            false,
        );
        Some(())
    }

    /// Add a tile to the monitor.
    pub fn add_tile(
        &mut self,
        tile: crate::layout::tile::Tile<W>,
        target: MonitorAddWindowTarget<W>,
        activate: crate::layout::ActivateWindow,
        _animate: bool,
        width: crate::layout::types::ColumnWidth,
        is_full_width: bool,
        _is_floating: bool,
    ) {
        let (row_idx, _col_idx) = self.resolve_add_window_target(&target);
        self.canvas.add_tile_to_row(
            row_idx,
            tile,
            activate == crate::layout::ActivateWindow::Yes,
            width,
            is_full_width,
        );
    }

    /// Previous workspace index (for workspace switching).
    pub fn previous_workspace_idx(&self) -> Option<usize> {
        // TEAM_022: Not implemented for canvas
        None
    }

    /// Activate a workspace/row by index.
    pub fn activate_workspace(&mut self, idx: usize) {
        self.canvas.focus_row(idx as i32);
    }

    // =========================================================================
    // TEAM_031: Missing Monitor Methods Implementation
    // =========================================================================

    /// Check if monitor contains a specific window by ID.
    /// TEAM_035: Updated to accept &W::Id instead of &W
    pub fn has_window(&self, window: &W::Id) -> bool {
        self.canvas.contains(window)
    }

    /// Advance animations on the monitor.
    pub fn advance_animations(&mut self) {
        self.canvas.advance_animations();
    }

    /// Check if any animations are ongoing.
    pub fn are_animations_ongoing(&self) -> bool {
        self.canvas.are_animations_ongoing()
    }

    /// Remove name from a row/workspace by WorkspaceId.
    /// TEAM_033: Updated to take WorkspaceId and return bool
    pub fn unname_workspace(&mut self, id: crate::layout::workspace_types::WorkspaceId) -> bool {
        // Find row with matching ID first (immutable)
        let found_idx = self.canvas.rows().find_map(|(idx, row)| {
            if row.id() == id {
                Some(idx)
            } else {
                None
            }
        });

        // Then mutate (mutable borrow no longer conflicts)
        if let Some(idx) = found_idx {
            if let Some(row) = self.canvas.row_mut(idx) {
                row.set_name(None);
            }
            return true;
        }
        false
    }

    /// Remove name from a row/workspace by index (internal use).
    pub fn unname_workspace_by_idx(&mut self, idx: usize) {
        if let Some(row) = self.canvas.row_mut(idx as i32) {
            row.set_name(None);
        }
    }

    /// Stop workspace/row switching animation.
    pub fn stop_workspace_switch(&mut self) {
        self.workspace_switch = None;
    }

    /// Remove a row/workspace by index.
    pub fn remove_workspace_by_idx(&mut self, idx: usize) {
        self.canvas.remove_row(idx as i32);
    }

    /// Insert a workspace/row at specific index.
    pub fn insert_workspace(&mut self, idx: usize) {
        self.canvas.ensure_row(idx as i32);
    }

    /// Activate workspace/row with animation config.
    /// TEAM_035: Updated to accept Option<Animation>
    pub fn activate_workspace_with_anim_config(&mut self, idx: usize, _config: Option<niri_config::Animation>) {
        self.canvas.focus_row(idx as i32);
    }

    /// Verifies internal invariants for testing.
    /// TEAM_035: Added for test compatibility
    #[cfg(test)]
    pub fn verify_invariants(&self) {
        // Basic canvas invariants
        assert!(self.canvas.rows().count() > 0 || !self.canvas.has_windows());
    }

    // TEAM_033: Removed duplicate into_canvas - kept the one defined earlier
}

