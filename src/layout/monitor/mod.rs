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
use crate::layout::workspace::{compute_working_area, Workspace, WorkspaceId};
use crate::layout::{LayoutElement, Options};
use crate::niri_render_elements;
use crate::utils::output_size;

// TEAM_013: Submodules
mod types;
mod workspace_compat; // LEGACY: All workspace code here for easy deletion
mod workspace_ops;    // LEGACY: Workspace operations
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

    // =========================================================================
    // LEGACY: Keep workspaces temporarily for incremental migration
    // TODO(TEAM_013): Remove after all methods are migrated to Canvas2D
    // =========================================================================

    /// LEGACY: Workspaces (kept for gradual migration)
    pub(in crate::layout) workspaces: Vec<Workspace<W>>,
    /// LEGACY: Index of the currently active workspace.
    pub(in crate::layout) active_workspace_idx: usize,
    /// LEGACY: ID of the previously active workspace.
    pub(in crate::layout) previous_workspace_id: Option<WorkspaceId>,
    /// LEGACY: In-progress switch between workspaces.
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

// TEAM_013: Render element types
niri_render_elements! {
    MonitorInnerRenderElement<R> => {
        Workspace = smithay::backend::renderer::element::utils::CropRenderElement<crate::layout::workspace::WorkspaceRenderElement<R>>,
        InsertHint = smithay::backend::renderer::element::utils::CropRenderElement<crate::layout::insert_hint_element::InsertHintRenderElement>,
        UncroppedInsertHint = crate::layout::insert_hint_element::InsertHintRenderElement,
        Shadow = crate::render_helpers::shadow::ShadowRenderElement,
        SolidColor = crate::render_helpers::solid_color::SolidColorRenderElement,
    }
}

pub type MonitorRenderElement<R> =
    RelocateRenderElement<RescaleRenderElement<MonitorInnerRenderElement<R>>>;

impl<W: LayoutElement> Monitor<W> {
    pub fn new(
        output: Output,
        mut workspaces: Vec<Workspace<W>>,
        ws_id_to_activate: Option<WorkspaceId>,
        clock: Clock,
        base_options: Rc<Options>,
        layout_config: Option<LayoutPart>,
    ) -> Self {
        let options =
            Rc::new(Options::clone(&base_options).with_merged_layout(layout_config.as_ref()));

        let scale = output.current_scale();
        let view_size = output_size(&output);
        let working_area = compute_working_area(&output);

        // Prepare the workspaces: set output, empty first, empty last.
        let mut active_workspace_idx = 0;

        for (idx, ws) in workspaces.iter_mut().enumerate() {
            assert!(ws.has_windows_or_name());

            ws.set_output(Some(output.clone()));
            ws.update_config(options.clone());

            if ws_id_to_activate.is_some_and(|id| ws.id() == id) {
                active_workspace_idx = idx;
            }
        }

        if options.layout.empty_workspace_above_first && !workspaces.is_empty() {
            let ws = Workspace::new(output.clone(), clock.clone(), options.clone());
            workspaces.insert(0, ws);
            active_workspace_idx += 1;
        }

        let ws = Workspace::new(output.clone(), clock.clone(), options.clone());
        workspaces.push(ws);

        // TEAM_010: Create Canvas2D for 2D layout mode
        let canvas = Canvas2D::new(
            Some(output.clone()),
            view_size,
            working_area, // parent_area = working_area
            working_area,
            scale.fractional_scale(),
            clock.clone(),
            options.clone(),
        );

        Self {
            output_name: output.name(),
            output,
            scale,
            view_size,
            working_area,
            // TEAM_010: Canvas2D is the new layout primitive
            canvas,
            // LEGACY: Keep workspaces for gradual migration
            workspaces,
            active_workspace_idx,
            previous_workspace_id: None,
            insert_hint: None,
            insert_hint_element: InsertHintElement::new(options.layout.insert_hint),
            insert_hint_render_loc: None,
            // TEAM_014: Removed overview_open and overview_progress (Part 3)
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

}
