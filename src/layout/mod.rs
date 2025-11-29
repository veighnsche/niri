//! Window layout logic.
//!
//! Niri implements scrollable tiling with dynamic workspaces. The scrollable tiling is mostly
//! orthogonal to any particular workspace system, though outputs living in separate coordinate
//! spaces suggest per-output workspaces.
//!
//! I chose a dynamic workspace system because I think it works very well. In particular, it works
//! naturally across outputs getting added and removed, since workspaces can move between outputs
//! as necessary.
//!
//! In the layout, one output (the first one to be added) is designated as *primary*. This is where
//! workspaces from disconnected outputs will move. Currently, the primary output has no other
//! distinction from other outputs.
//!
//! Where possible, niri tries to follow these principles with regards to outputs:
//!
//! 1. Disconnecting and reconnecting the same output must not change the layout.
//!    * This includes both secondary outputs and the primary output.
//! 2. Connecting an output must not change the layout for any workspaces that were never on that
//!    output.
//!
//! Therefore, we implement the following logic: every workspace keeps track of which output it
//! originated on—its *original output*. When an output disconnects, its workspaces are appended to
//! the (potentially new) primary output, but remember their original output. Then, if the original
//! output connects again, all workspaces originally from there move back to that output.
//!
//! In order to avoid surprising behavior, if the user creates or moves any new windows onto a
//! workspace, it forgets its original output, and its current output becomes its original output.
//! Imagine a scenario: the user works with a laptop and a monitor at home, then takes their laptop
//! with them, disconnecting the monitor, and keeps working as normal, using the second monitor's
//! workspace just like any other. Then they come back, reconnect the second monitor, and now we
//! don't want an unassuming workspace to end up on it.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use monitor::{InsertPosition, MonitorAddWindowTarget};
use niri_config::utils::MergeWith as _;
// TEAM_055: Renamed from Workspace to RowConfig
use niri_config::Config;
use niri_ipc::{PositionChange, SizeChange};
// TEAM_021: Use minimal row types after Canvas2D migration
// TEAM_055: Renamed from workspace_types to row_types
use row_types::RowId;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::output::{self, Output};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Scale, Serial, Size, Transform};
use tile::Tile;
use types::{ColumnWidth, ScrollDirection};

// TEAM_060: Removed WorkspaceId type alias - using RowId directly
pub use self::monitor::MonitorRenderElement;
use self::monitor::{Monitor, WorkspaceSwitch};
// DEPRECATED(overview): Removed Animation and SwipeTracker imports (no longer needed)
use crate::animation::Clock;
// TEAM_003: ScrollDirection now imported from types module above
use crate::niri_render_elements;
use crate::render_helpers::offscreen::OffscreenData;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::snapshot::RenderSnapshot;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::{BakedBuffer, RenderTarget, SplitElements};
use crate::utils::transaction::{Transaction, TransactionBlocker};
use crate::utils::{
    ensure_min_max_size_maybe_zero, output_matches_name, output_size,
    round_logical_in_physical_max1, ResizeEdge,
};
use crate::window::ResolvedWindowRules;

// TEAM_002: Column module extracted from scrolling.rs
pub mod column;
// TEAM_062: Render elements grouped into elements/ module
pub mod elements;
// TEAM_063: FloatingSpace consolidated into canvas/floating/
// Re-export for backwards compatibility
pub use canvas::floating;
pub mod monitor;
// TEAM_062: scrolling.rs moved to deprecated/ - replaced by Row
pub mod tile;
// TEAM_003: Shared types for layout modules
pub mod types;
// TEAM_005: AnimatedValue abstraction for view offset and camera
pub mod animated_value;
// TEAM_006: Row module for 2D canvas layout
pub mod row;
// TEAM_006: Canvas2D module for 2D tiling layout
pub mod canvas;
// TEAM_055: Renamed from workspace_types to row_types
pub mod row_types; // TEAM_021: Minimal row types for external compatibility
                   // TEAM_063: Layout implementation split into submodules
                   // TEAM_064: Interactive move and DnD types moved to layout_impl/types.rs
mod layout_impl;
// Re-export internal types for use in this module
// Re-export public types
pub use layout_impl::types::DndData;
use layout_impl::types::InteractiveMoveState;
// DEPRECATED: workspace module removed - functionality migrated to Canvas2D

// TEAM_004: Golden snapshot infrastructure
// TEAM_010: Extended with animation timeline snapshots
#[cfg(test)]
pub mod snapshot;

#[cfg(test)]
mod tests;

/// Size changes up to this many pixels don't animate.
pub const RESIZE_ANIMATION_THRESHOLD: f64 = 10.;

/// Pointer needs to move this far to pull a window from the layout.
const INTERACTIVE_MOVE_START_THRESHOLD: f64 = 256. * 256.;

/// Opacity of interactively moved tiles targeting the scrolling layout.
const INTERACTIVE_MOVE_ALPHA: f64 = 0.75;

// TEAM_014: Removed OVERVIEW_GESTURE_MOVEMENT and OVERVIEW_GESTURE_RUBBER_BAND (Part 3)

/// Size-relative units.
pub struct SizeFrac;

niri_render_elements! {
    LayoutElementRenderElement<R> => {
        Wayland = WaylandSurfaceRenderElement<R>,
        SolidColor = SolidColorRenderElement,
    }
}

pub type LayoutElementRenderSnapshot =
    RenderSnapshot<BakedBuffer<TextureBuffer<GlesTexture>>, BakedBuffer<SolidColorBuffer>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizingMode {
    Normal,
    Maximized,
    Fullscreen,
}

pub trait LayoutElement {
    /// Type that can be used as a unique ID of this element.
    type Id: PartialEq + std::fmt::Debug + Clone;

    /// Unique ID of this element.
    fn id(&self) -> &Self::Id;

    /// Visual size of the element.
    ///
    /// This is what the user would consider the size, i.e. excluding CSD shadows and whatnot.
    /// Corresponds to the Wayland window geometry size.
    fn size(&self) -> Size<i32, Logical>;

    /// Returns the location of the element's buffer relative to the element's visual geometry.
    ///
    /// I.e. if the element has CSD shadows, its buffer location will have negative coordinates.
    fn buf_loc(&self) -> Point<i32, Logical>;

    /// Checks whether a point is in the element's input region.
    ///
    /// The point is relative to the element's visual geometry.
    fn is_in_input_region(&self, point: Point<f64, Logical>) -> bool;

    /// Renders the element at the given visual location.
    ///
    /// The element should be rendered in such a way that its visual geometry ends up at the given
    /// location.
    fn render<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        location: Point<f64, Logical>,
        scale: Scale<f64>,
        alpha: f32,
        target: RenderTarget,
    ) -> SplitElements<LayoutElementRenderElement<R>>;

    /// Renders the non-popup parts of the element.
    fn render_normal<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        location: Point<f64, Logical>,
        scale: Scale<f64>,
        alpha: f32,
        target: RenderTarget,
    ) -> Vec<LayoutElementRenderElement<R>> {
        self.render(renderer, location, scale, alpha, target).normal
    }

    /// Renders the popups of the element.
    fn render_popups<R: NiriRenderer>(
        &self,
        renderer: &mut R,
        location: Point<f64, Logical>,
        scale: Scale<f64>,
        alpha: f32,
        target: RenderTarget,
    ) -> Vec<LayoutElementRenderElement<R>> {
        self.render(renderer, location, scale, alpha, target).popups
    }

    /// Requests the element to change its size.
    ///
    /// The size request is stored and will be continuously sent to the element on any further
    /// state changes.
    fn request_size(
        &mut self,
        size: Size<i32, Logical>,
        mode: SizingMode,
        animate: bool,
        transaction: Option<Transaction>,
    );

    /// Requests the element to change size once, clearing the request afterwards.
    fn request_size_once(&mut self, size: Size<i32, Logical>, animate: bool) {
        self.request_size(size, SizingMode::Normal, animate, None);
    }

    fn min_size(&self) -> Size<i32, Logical>;
    fn max_size(&self) -> Size<i32, Logical>;
    fn is_wl_surface(&self, wl_surface: &WlSurface) -> bool;
    fn has_ssd(&self) -> bool;
    fn set_preferred_scale_transform(&self, scale: output::Scale, transform: Transform);
    fn output_enter(&self, output: &Output);
    fn output_leave(&self, output: &Output);
    fn set_offscreen_data(&self, data: Option<OffscreenData>);
    fn set_activated(&mut self, active: bool);
    fn set_active_in_column(&mut self, active: bool);
    fn set_floating(&mut self, floating: bool);
    fn set_bounds(&self, bounds: Size<i32, Logical>);
    fn is_ignoring_opacity_window_rule(&self) -> bool;

    fn is_urgent(&self) -> bool;

    fn configure_intent(&self) -> ConfigureIntent;
    fn send_pending_configure(&mut self);

    /// The element's current sizing mode.
    ///
    /// This will *not* switch immediately after a [`LayoutElement::request_size()`] call.
    fn sizing_mode(&self) -> SizingMode;

    /// The sizing mode that we're requesting the element to assume.
    ///
    /// This *will* switch immediately after a [`LayoutElement::request_size()`] call.
    fn pending_sizing_mode(&self) -> SizingMode;

    /// Size previously requested through [`LayoutElement::request_size()`].
    fn requested_size(&self) -> Option<Size<i32, Logical>>;

    /// Non-fullscreen size that we expect this window has or will shortly have.
    ///
    /// This can be different from [`requested_size()`](LayoutElement::requested_size()). For
    /// example, for floating windows this will generally return the current window size, rather
    /// than the last size that we requested, since we want floating windows to be able to change
    /// size freely. But not always: if we just requested a floating window to resize and it hasn't
    /// responded to it yet, this will return the newly requested size.
    ///
    /// This function should never return a 0 size component. `None` means there's no known
    /// expected size (for example, the window is fullscreen).
    ///
    /// The default impl is for testing only, it will not preserve the window's own size changes.
    fn expected_size(&self) -> Option<Size<i32, Logical>> {
        if self.sizing_mode().is_fullscreen() {
            return None;
        }

        let mut requested = self.requested_size().unwrap_or_default();
        let current = self.size();
        if requested.w == 0 {
            requested.w = current.w;
        }
        if requested.h == 0 {
            requested.h = current.h;
        }
        Some(requested)
    }

    fn is_pending_windowed_fullscreen(&self) -> bool {
        false
    }
    fn request_windowed_fullscreen(&mut self, value: bool) {
        let _ = value;
    }

    fn is_child_of(&self, parent: &Self) -> bool;

    fn rules(&self) -> &ResolvedWindowRules;

    /// Runs periodic clean-up tasks.
    fn refresh(&self);

    fn take_animation_snapshot(&mut self) -> Option<LayoutElementRenderSnapshot>;

    fn set_interactive_resize(&mut self, data: Option<InteractiveResizeData>);
    fn cancel_interactive_resize(&mut self);
    fn interactive_resize_data(&self) -> Option<InteractiveResizeData>;

    fn on_commit(&mut self, serial: Serial);
}

#[derive(Debug)]
pub struct Layout<W: LayoutElement> {
    /// Monitors and workspaes in the layout.
    monitor_set: MonitorSet<W>,
    /// Whether the layout should draw as active.
    ///
    /// This normally indicates that the layout has keyboard focus, but not always. E.g. when the
    /// screenshot UI is open, it keeps the layout drawing as active.
    is_active: bool,
    /// Map from monitor name to id of its last active workspace.
    ///
    /// This data is stored upon monitor removal and is used to restore the active workspace when
    /// the monitor is reconnected.
    // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
    /// The row id does not necessarily point to a valid row. If it doesn't, then it is
    /// simply ignored.
    last_active_row_id: HashMap<String, RowId>,
    /// TEAM_039: Counter for generating unique row IDs
    /// TEAM_055: Renamed from workspace_id_counter to row_id_counter
    row_id_counter: u64,
    /// Ongoing interactive move.
    interactive_move: Option<InteractiveMoveState<W>>,
    /// Ongoing drag-and-drop operation.
    dnd: Option<DndData<W>>,
    /// Clock for driving animations.
    clock: Clock,
    /// Time that we last updated render elements for.
    update_render_elements_time: Duration,
    // TEAM_014: Removed overview_open and overview_progress (Part 3)
    /// Configurable properties of the layout.
    options: Rc<Options>,
}

#[derive(Debug)]
enum MonitorSet<W: LayoutElement> {
    /// At least one output is connected.
    Normal {
        /// Connected monitors.
        monitors: Vec<Monitor<W>>,
        /// Index of the primary monitor.
        primary_idx: usize,
        /// Index of the active monitor.
        active_monitor_idx: usize,
    },
    /// No outputs are connected, and this is the canvas.
    NoOutputs {
        /// The canvas.
        canvas: crate::layout::canvas::Canvas2D<W>,
    },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Options {
    pub layout: niri_config::Layout,
    pub animations: niri_config::Animations,
    pub gestures: niri_config::Gestures,
    // Debug flags.
    pub disable_resize_throttling: bool,
    pub disable_transactions: bool,
    pub deactivate_unfocused_windows: bool,
}

// TEAM_064: InteractiveMoveState, InteractiveMoveData, DndData, DndHold, DndHoldTarget
// moved to layout_impl/types.rs

#[derive(Debug, Clone, Copy)]
pub struct InteractiveResizeData {
    pub(self) edges: ResizeEdge,
}

#[derive(Debug, Clone, Copy)]
pub enum ConfigureIntent {
    /// A configure is not needed (no changes to server pending state).
    NotNeeded,
    /// A configure is throttled (due to resizing too fast for example).
    Throttled,
    /// Can send the configure if it isn't throttled externally (only size changed).
    CanSend,
    /// Should send the configure regardless of external throttling (something other than size
    /// changed).
    ShouldSend,
}

/// Tile that was just removed from the layout.
pub struct RemovedTile<W: LayoutElement> {
    tile: Tile<W>,
    /// Width of the column the tile was in.
    width: ColumnWidth,
    /// Whether the column the tile was in was full-width.
    is_full_width: bool,
    /// Whether the tile was floating.
    is_floating: bool,
    /// Whether the tile was maximized (to preserve state across workspace moves).
    is_maximized: bool,
}

impl<W: LayoutElement> RemovedTile<W> {
    // TEAM_008: Added constructor for use by Row module
    pub(crate) fn new(
        tile: Tile<W>,
        width: ColumnWidth,
        is_full_width: bool,
        is_floating: bool,
        is_maximized: bool,
    ) -> Self {
        Self {
            tile,
            width,
            is_full_width,
            is_floating,
            is_maximized,
        }
    }

    /// Returns the tile.
    pub fn tile(self) -> Tile<W> {
        self.tile
    }

    /// Returns the width of the column the tile was in.
    pub fn width(&self) -> ColumnWidth {
        self.width
    }

    /// Returns whether the column the tile was in was full-width.
    pub fn is_full_width(&self) -> bool {
        self.is_full_width
    }

    /// Returns whether the tile was floating.
    pub fn is_floating(&self) -> bool {
        self.is_floating
    }

    /// Returns whether the tile was maximized.
    pub fn is_maximized(&self) -> bool {
        self.is_maximized
    }

    /// Returns a reference to the window ID without consuming the tile.
    pub fn window_id(&self) -> &W::Id {
        self.tile.window().id()
    }

    /// Destructures into components.
    pub fn into_parts(self) -> (Tile<W>, ColumnWidth, bool, bool, bool) {
        (
            self.tile,
            self.width,
            self.is_full_width,
            self.is_floating,
            self.is_maximized,
        )
    }
}

/// Whether to activate a newly added window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActivateWindow {
    /// Activate unconditionally.
    Yes,
    /// Activate based on heuristics.
    #[default]
    Smart,
    /// Do not activate.
    No,
}

/// Where to put a newly added window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AddWindowTarget<'a, W: LayoutElement> {
    /// No particular preference.
    #[default]
    Auto,
    /// On this output.
    Output(&'a Output),
    /// On this workspace.
    Workspace(RowId),
    /// Next to this existing window.
    NextTo(&'a W::Id),
}

/// Type of the window hit from `window_under()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitType {
    /// The hit is within a window's input region and can be used for sending events to it.
    Input {
        /// Position of the window's buffer.
        win_pos: Point<f64, Logical>,
    },
    /// The hit can activate a window, but it is not in the input region so cannot send events.
    ///
    /// For example, this could be clicking on a tile border outside the window.
    Activate {
        /// Whether the hit was on the tab indicator.
        is_tab_indicator: bool,
    },
}

// TEAM_014: Removed OverviewProgress and OverviewGesture types (Part 3)

impl SizingMode {
    #[must_use]
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    #[must_use]
    pub fn is_fullscreen(&self) -> bool {
        matches!(self, Self::Fullscreen)
    }

    #[must_use]
    pub fn is_maximized(&self) -> bool {
        matches!(self, Self::Maximized)
    }
}

// TEAM_064: InteractiveMoveState and InteractiveMoveData impl blocks moved to layout_impl/types.rs

impl ActivateWindow {
    pub fn map_smart(self, f: impl FnOnce() -> bool) -> bool {
        match self {
            ActivateWindow::Yes => true,
            ActivateWindow::Smart => f(),
            ActivateWindow::No => false,
        }
    }
}

impl HitType {
    pub fn offset_win_pos(mut self, offset: Point<f64, Logical>) -> Self {
        match &mut self {
            HitType::Input { win_pos } => *win_pos += offset,
            HitType::Activate { .. } => (),
        }
        self
    }

    pub fn hit_tile<W: LayoutElement>(
        tile: &Tile<W>,
        tile_pos: Point<f64, Logical>,
        point: Point<f64, Logical>,
    ) -> Option<(&W, Self)> {
        let pos_within_tile = point - tile_pos;
        tile.hit(pos_within_tile)
            .map(|hit| (tile.window(), hit.offset_win_pos(tile_pos)))
    }

    pub fn to_activate(self) -> Self {
        match self {
            HitType::Input { .. } => HitType::Activate {
                is_tab_indicator: false,
            },
            HitType::Activate { .. } => self,
        }
    }
}

impl Options {
    fn from_config(config: &Config) -> Self {
        Self {
            layout: config.layout.clone(),
            animations: config.animations.clone(),
            gestures: config.gestures,
            disable_resize_throttling: config.debug.disable_resize_throttling,
            disable_transactions: config.debug.disable_transactions,
            deactivate_unfocused_windows: config.debug.deactivate_unfocused_windows,
        }
    }

    fn with_merged_layout(mut self, part: Option<&niri_config::LayoutPart>) -> Self {
        if let Some(part) = part {
            self.layout.merge_with(part);
        }
        self
    }

    fn adjusted_for_scale(mut self, scale: f64) -> Self {
        self.layout.gaps = round_logical_in_physical_max1(scale, self.layout.gaps);
        self
    }
}

// TEAM_014: Removed OverviewProgress impl block (Part 3)

impl<W: LayoutElement> Layout<W> {
    pub fn new(clock: Clock, config: &Config) -> Self {
        Self::with_options_and_workspaces(clock, config, Options::from_config(config))
    }

    /// TEAM_039: Generate a unique row ID
    /// TEAM_055: Renamed from next_workspace_id to next_row_id
    pub fn next_row_id(&mut self) -> RowId {
        self.row_id_counter += 1;
        RowId(self.row_id_counter)
    }

    pub fn with_options(clock: Clock, options: Options) -> Self {
        let opts = Rc::new(options);

        // Create a Canvas2D for the NoOutputs variant
        // TEAM_039: Use 1280x720 as default size to match original Workspace behavior
        let view_size = Size::from((1280.0, 720.0));
        let parent_area = Rectangle::from_loc_and_size((0.0, 0.0), view_size);
        let working_area = parent_area;
        let scale = 1.0;

        // Generate unique workspace ID for the initial row
        let initial_workspace_id = RowId(1);

        let canvas = crate::layout::canvas::Canvas2D::new(
            None,
            view_size,
            parent_area,
            working_area,
            scale,
            clock.clone(),
            opts.clone(),
            initial_workspace_id,
        );

        Self {
            monitor_set: MonitorSet::NoOutputs { canvas },
            is_active: true,
            // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
            last_active_row_id: HashMap::new(),
            row_id_counter: 1, // TEAM_039: Start at 1 since we used ID 1 for initial row
            interactive_move: None,
            dnd: None,
            clock,
            update_render_elements_time: Duration::ZERO,
            // TEAM_014: Removed overview_open and overview_progress (Part 3)
            options: opts,
        }
    }

    fn with_options_and_workspaces(clock: Clock, config: &Config, options: Options) -> Self {
        let opts = Rc::new(options);

        // Create a Canvas2D for the NoOutputs variant
        // TEAM_039: Use 1280x720 as default size to match original Workspace behavior
        let view_size = Size::from((1280.0, 720.0));
        let parent_area = Rectangle::from_loc_and_size((0.0, 0.0), view_size);
        let working_area = parent_area;
        let scale = 1.0;

        // Generate unique workspace ID for the initial row
        let initial_workspace_id = RowId(1);

        let canvas = crate::layout::canvas::Canvas2D::new(
            None,
            view_size,
            parent_area,
            working_area,
            scale,
            clock.clone(),
            opts.clone(),
            initial_workspace_id,
        );

        // TODO: TEAM_023: Apply workspace config to canvas rows if needed
        // For now, we just create the default canvas with origin row

        Layout {
            monitor_set: MonitorSet::NoOutputs { canvas },
            is_active: true,
            // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
            last_active_row_id: HashMap::new(),
            row_id_counter: 1, // TEAM_039: Start at 1 since we used ID 1 for initial row
            interactive_move: None,
            dnd: None,
            clock,
            update_render_elements_time: Duration::ZERO,
            // TEAM_014: Removed overview_open and overview_progress (Part 3)
            options: opts,
        }
    }

    // TEAM_063: add_output, remove_output, add_column_by_idx moved to layout_impl/output_ops.rs
    // TEAM_063: add_window, remove_window, descendants_added, update_window,
    // find_window_and_output, find_window_and_output_mut moved to layout_impl/window_ops.rs

    // TEAM_063: find_workspace_by_id, find_workspace_by_name, find_row_by_name,
    // find_workspace_by_ref, unname_workspace, unname_workspace_by_ref,
    // unname_workspace_by_id moved to layout_impl/row_management.rs

    // TEAM_063: popup_target_rect moved to layout_impl/queries.rs

    pub fn update_output_size(&mut self, output: &Output) {
        let _span = tracy_client::span!("Layout::update_output_size");

        let Some(mon) = self.monitor_for_output_mut(output) else {
            error!("monitor missing in update_output_size()");
            return;
        };

        mon.update_output_size();
    }

    // TEAM_063: scroll_amount_to_activate moved to layout_impl/queries.rs
    // TEAM_063: should_trigger_focus_follows_mouse_on moved to layout_impl/queries.rs
    // TEAM_063: activate_window, activate_window_without_raising, active_output,
    // active_row, active_row_mut, windows_for_output, windows_for_output_mut,
    // with_windows, with_windows_mut, active_monitor_mut, active_monitor_ref,
    // monitors, monitors_mut moved to layout_impl/focus.rs
    // TEAM_063: canvas_snapshot moved to layout_impl/queries.rs

    fn active_monitor(&mut self) -> Option<&mut Monitor<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return None;
        };

        Some(&mut monitors[*active_monitor_idx])
    }

    pub fn monitor_for_output(&self, output: &Output) -> Option<&Monitor<W>> {
        self.monitors().find(|mon| &mon.output == output)
    }

    pub fn monitor_for_output_mut(&mut self, output: &Output) -> Option<&mut Monitor<W>> {
        self.monitors_mut().find(|mon| &mon.output == output)
    }

    pub fn monitor_for_workspace(&self, workspace_name: &str) -> Option<&Monitor<W>> {
        self.monitors().find(|monitor| {
            monitor.canvas.rows().any(|(utils::scale::MIN_LOGICAL_AREA, ws)| {
                ws.name()
                    .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
            })
        })
    }

    pub fn outputs(&self) -> impl Iterator<Item = &Output> + '_ {
        self.monitors().map(|mon| &mon.output)
    }

    // TEAM_063: move_left, move_right, move_column_to_first, move_column_to_last,
    // move_column_left_or_to_output, move_column_right_or_to_output, move_column_to_index,
    // move_down, move_up, move_down_or_to_row_down, move_up_or_to_row_up,
    // consume_or_expel_window_left, consume_or_expel_window_right,
    // focus_left, focus_right, focus_column_first, focus_column_last,
    // focus_column_right_or_first, focus_column_left_or_last, focus_column,
    // focus_window_up_or_output, focus_window_down_or_output,
    // focus_column_left_or_output, focus_column_right_or_output,
    // focus_window_in_column, focus_down, focus_up, focus_down_or_left,
    // focus_down_or_right, focus_up_or_left, focus_up_or_right,
    // focus_window_or_row_down, focus_window_or_row_up, focus_window_top,
    // focus_window_bottom, focus_window_down_or_top, focus_window_up_or_bottom,
    // move_to_row_up, move_to_row_down, move_to_row,
    // move_column_to_row_up, move_column_to_row_down, move_column_to_row,
    // focus_row_up, focus_row_down, focus_row, focus_row_auto_back_and_forth,
    // focus_previous_position, consume_into_column, expel_from_column,
    // swap_window_in_direction, toggle_column_tabbed_display, set_column_display,
    // center_column, center_window, center_visible_columns, focus, focus_with_output,
    // interactive_moved_window_under, window_under, resize_edges_under, workspace_under
    // moved to layout_impl/navigation.rs

    #[cfg(test)]
    fn verify_invariants(&self) {
        use std::collections::HashSet;

        use approx::assert_abs_diff_eq;

        // TEAM_014: overview_zoom always 1.0 now (Part 3)
        let zoom = 1.0;

        let mut move_win_id = None;
        if let Some(state) = &self.interactive_move {
            match state {
                InteractiveMoveState::Starting {
                    window_id,
                    pointer_delta: _,
                    pointer_ratio_within_window: _,
                } => {
                    assert!(
                        self.has_window(window_id),
                        "interactive move must be on an existing window"
                    );
                    move_win_id = Some(window_id.clone());
                }
                InteractiveMoveState::Moving(move_) => {
                    assert_eq!(self.clock, move_.tile.clock);
                    assert!(move_.tile.window().pending_sizing_mode().is_normal());

                    move_.tile.verify_invariants();

                    let scale = move_.output.current_scale().fractional_scale();
                    let options = Options::clone(&self.options)
                        .with_merged_layout(move_.output_config.as_ref())
                        .with_merged_layout(move_.workspace_config.as_ref().map(|(_, c)| c))
                        .adjusted_for_scale(scale);
                    assert_eq!(
                        &*move_.tile.options, &options,
                        "interactive moved tile options must be \
                         base options adjusted for output scale"
                    );

                    let tile_pos = move_.tile_render_location(zoom);
                    let rounded_pos = tile_pos.to_physical_precise_round(scale).to_logical(scale);

                    // Tile position must be rounded to physical pixels.
                    assert_abs_diff_eq!(tile_pos.x, rounded_pos.x, epsilon = 1e-5);
                    assert_abs_diff_eq!(tile_pos.y, rounded_pos.y, epsilon = 1e-5);

                    if let Some(alpha) = &move_.tile.alpha_animation {
                        if move_.is_floating {
                            assert_eq!(
                                alpha.anim.to(),
                                1.,
                                "interactively moved floating tile can animate alpha only to 1"
                            );

                            assert!(
                                !alpha.hold_after_done,
                                "interactively moved floating tile \
                                 cannot have held alpha animation"
                            );
                        } else {
                            assert_ne!(
                                alpha.anim.to(),
                                1.,
                                "interactively moved scrolling tile must animate alpha to not 1"
                            );

                            assert!(
                                alpha.hold_after_done,
                                "interactively moved scrolling tile \
                                 must have held alpha animation"
                            );
                        }
                    }
                }
            }
        }

        let mut seen_workspace_id = HashSet::new();
        let mut seen_workspace_name = Vec::<String>::new();

        // TEAM_035: Updated verify_invariants for Canvas2D architecture
        let (monitors, &primary_idx, &active_monitor_idx) = match &self.monitor_set {
            MonitorSet::Normal {
                monitors,
                primary_idx,
                active_monitor_idx,
            } => (monitors, primary_idx, active_monitor_idx),
            MonitorSet::NoOutputs { canvas } => {
                // Empty unnamed rows should have been cleaned up when transitioning to NoOutputs
                // Exception: row 0 (origin) is always kept even if empty and unnamed
                for (row_idx, workspace) in canvas.rows() {
                    assert!(
                        row_idx == 0 || workspace.has_windows_or_name(),
                        "with no outputs there cannot be empty unnamed workspaces (except row 0)"
                    );

                    // TEAM_035: Row.clock is private, skip this check
                    // assert_eq!(self.clock, workspace.clock);

                    // TEAM_035: Row doesn't have base_options, skip this check
                    // assert_eq!(
                    //     workspace.base_options, self.options,
                    //     "workspace base options must be synchronized with layout"
                    // );

                    assert!(
                        seen_workspace_id.insert(workspace.id()),
                        "workspace id must be unique"
                    );

                    if let Some(name) = workspace.name() {
                        assert!(
                            !seen_workspace_name
                                .iter()
                                .any(|n| n.eq_ignore_ascii_case(name)),
                            "workspace name must be unique"
                        );
                        seen_workspace_name.push(name.to_string());
                    }

                    workspace.verify_invariants(move_win_id.as_ref());
                }

                return;
            }
        };

        assert!(primary_idx < monitors.len());
        assert!(active_monitor_idx < monitors.len());

        let mut saw_view_offset_gesture = false;

        for (_idx, monitor) in monitors.iter().enumerate() {
            assert_eq!(self.clock, monitor.clock);
            assert_eq!(
                monitor.base_options, self.options,
                "monitor base options must be synchronized with layout"
            );

            // TEAM_014: Removed overview state invariant checks (Part 3)

            monitor.verify_invariants();

            // TEAM_035: In Canvas2D, workspaces don't have original_output
            // Each monitor has its own canvas, so workspace migration checks are not applicable

            for (_, workspace) in monitor.canvas.rows() {
                let ws_id = workspace.id();
                // TEAM_039: Debug workspace ID uniqueness
                if seen_workspace_id.contains(&ws_id) {
                    panic!(
                        "workspace id must be unique: duplicate ID {:?} found",
                        ws_id
                    );
                }
                seen_workspace_id.insert(ws_id);

                if let Some(name) = workspace.name() {
                    assert!(
                        !seen_workspace_name
                            .iter()
                            .any(|n| n.eq_ignore_ascii_case(name)),
                        "workspace name must be unique"
                    );
                    seen_workspace_name.push(name.to_string());
                }

                workspace.verify_invariants(move_win_id.as_ref());

                // TEAM_035: Row has view_offset() directly (no scrolling() needed)
                let has_view_offset_gesture = workspace.view_offset().is_gesture();
                if self.dnd.is_some() || self.interactive_move.is_some() {
                    // We'd like to check that all workspaces have the gesture here, furthermore we
                    // want to check that they have the gesture only if the interactive move
                    // targets the scrolling layout. However, we cannot do that because we start
                    // and stop the gesture lazily. Otherwise the gesture code would pollute a lot
                    // of places like adding new workspaces, implicitly moving windows between
                    // floating and tiling on fullscreen, etc.
                    //
                    // assert!(
                    //     has_view_offset_gesture,
                    //     "during an interactive move in the scrolling layout, \
                    //      all workspaces should be in a view offset gesture"
                    // );
                } else if saw_view_offset_gesture {
                    assert!(
                        !has_view_offset_gesture,
                        "only one workspace can have an ongoing view offset gesture"
                    );
                }
                saw_view_offset_gesture = has_view_offset_gesture;
            }
        }
    }

    // TEAM_064: advance_animations, are_animations_ongoing, update_render_elements,
    // update_shaders, update_insert_hint, update_config, update_options
    // moved to layout_impl/render.rs

    // TEAM_063: ensure_named_row moved to layout_impl/row_management.rs

    // TEAM_063: toggle_width, toggle_window_width, toggle_window_height,
    // toggle_full_width, set_column_width, set_window_width, set_window_height,
    // reset_window_height, expand_column_to_available_width
    // moved to layout_impl/resize.rs

    pub fn toggle_window_floating(&mut self, window: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                move_.is_floating = !move_.is_floating;

                // When going to floating, restore the floating window size.
                if move_.is_floating {
                    let floating_size = move_.tile.floating_window_size;
                    let win = move_.tile.window_mut();
                    let mut size =
                        floating_size.unwrap_or_else(|| win.expected_size().unwrap_or_default());

                    // Apply min/max size window rules. If requesting a concrete size, apply
                    // completely; if requesting (0, 0), apply only when min/max results in a fixed
                    // size.
                    let min_size = win.min_size();
                    let max_size = win.max_size();
                    size.w = ensure_min_max_size_maybe_zero(size.w, min_size.w, max_size.w);
                    size.h = ensure_min_max_size_maybe_zero(size.h, min_size.h, max_size.h);

                    win.request_size_once(size, true);

                    // Animate the tile back to opaque.
                    move_.tile.animate_alpha(
                        INTERACTIVE_MOVE_ALPHA,
                        1.,
                        self.options.animations.window_movement.0,
                    );

                    // TEAM_021: Use canvas for DND gestures instead of workspace iteration
                    for mon in self.monitors_mut() {
                        mon.canvas_mut().dnd_scroll_gesture_end();
                    }
                } else {
                    // Animate the tile back to semitransparent.
                    move_.tile.animate_alpha(
                        1.,
                        INTERACTIVE_MOVE_ALPHA,
                        self.options.animations.window_movement.0,
                    );
                    move_.tile.hold_alpha_animation_after_done();
                }

                return;
            }
        }

        // TEAM_040: Use Canvas2D's toggle_floating_window_by_id instead of Row method
        // Floating is handled at the Canvas2D level, not the Row level
        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                if let Some(window) = window {
                    // Find the monitor containing this window
                    for mon in monitors.iter_mut() {
                        if mon.canvas.contains_any(window) {
                            mon.canvas.toggle_floating_window_by_id(Some(window));
                            return;
                        }
                    }
                } else {
                    // Toggle active window on active monitor
                    monitors[*active_monitor_idx]
                        .canvas
                        .toggle_floating_window_by_id(None);
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                canvas.toggle_floating_window_by_id(window);
            }
        }
    }

    pub fn set_window_floating(&mut self, window: Option<&W::Id>, floating: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                if move_.is_floating != floating {
                    self.toggle_window_floating(window);
                }
                return;
            }
        }

        let workspace = if let Some(window) = window {
            Some(
                self.workspaces_mut()
                    .find(|ws| ws.has_window(window))
                    .unwrap(),
            )
        } else {
            self.active_row_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.set_window_floating(window, floating);
    }

    pub fn focus_floating(&mut self) {
        // TEAM_044: Call Canvas2D method to properly set floating_is_active
        let Some(mon) = self.active_monitor_mut() else {
            return;
        };
        mon.canvas.focus_floating();
    }

    pub fn focus_tiling(&mut self) {
        // TEAM_044: Call Canvas2D method to properly set floating_is_active
        let Some(mon) = self.active_monitor_mut() else {
            return;
        };
        mon.canvas.focus_tiling();
    }

    pub fn switch_focus_floating_tiling(&mut self) {
        // TEAM_044: Call Canvas2D method to properly toggle floating_is_active
        let Some(mon) = self.active_monitor_mut() else {
            return;
        };
        mon.canvas.switch_focus_floating_tiling();
    }

    pub fn move_floating_window(
        &mut self,
        id: Option<&W::Id>,
        x: PositionChange,
        y: PositionChange,
        animate: bool,
    ) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if id.is_none() || id == Some(move_.tile.window().id()) {
                return;
            }
        }

        // Find the monitor that contains the window and call canvas.move_floating_window
        if let Some(id) = id {
            for monitor in self.monitors_mut() {
                if monitor.canvas().has_window(id) {
                    monitor
                        .canvas_mut()
                        .move_floating_window(Some(id), x, y, animate);
                    break;
                }
            }
        } else {
            // Move active floating window on active monitor
            if let Some(monitor) = self.active_monitor() {
                monitor
                    .canvas_mut()
                    .move_floating_window(None, x, y, animate);
            }
        }
    }

    pub fn focus_output(&mut self, output: &Output) {
        if let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        {
            for (idx, mon) in monitors.iter().enumerate() {
                if &mon.output == output {
                    *active_monitor_idx = idx;
                    return;
                }
            }
        }
    }

    pub fn move_to_output(
        &mut self,
        window: Option<&W::Id>,
        output: &Output,
        target_ws_idx: Option<usize>,
        activate: ActivateWindow,
    ) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        if let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        {
            let new_idx = monitors
                .iter()
                .position(|mon| &mon.output == output)
                .unwrap();

            let (mon_idx, ws_idx) = if let Some(window) = window {
                monitors
                    .iter()
                    .enumerate()
                    .find_map(|(mon_idx, mon)| {
                        mon.canvas
                            .rows()
                            .position(|(_, ws)| ws.has_window(window))
                            .map(|ws_idx| (mon_idx, ws_idx))
                    })
                    .unwrap()
            } else {
                let mon_idx = *active_monitor_idx;
                let mon = &monitors[mon_idx];
                (mon_idx, mon.active_row_idx())
            };

            let workspace_idx = target_ws_idx.unwrap_or(monitors[new_idx].active_row_idx());
            if mon_idx == new_idx && ws_idx == workspace_idx {
                return;
            }

            let mon = &monitors[new_idx];
            if mon.canvas.rows().count() <= workspace_idx {
                return;
            }

            let ws_id = mon.canvas.rows().nth(workspace_idx).unwrap().1.id();

            let mon = &mut monitors[mon_idx];
            let activate = activate.map_smart(|| {
                window.map_or(true, |win| {
                    mon_idx == *active_monitor_idx
                        && mon.active_window().map(|win| win.id()) == Some(win)
                })
            });
            let activate = if activate {
                ActivateWindow::Yes
            } else {
                ActivateWindow::No
            };

            // TEAM_033: Use row_mut for mutable access instead of workspaces() iterator
            let ws = mon
                .canvas
                .row_mut(ws_idx as i32)
                .expect("workspace should exist");
            let transaction = Transaction::new();
            let mut removed = if let Some(window) = window {
                ws.remove_tile(window, transaction)
            } else if let Some(removed) = ws.remove_active_tile(transaction) {
                removed
            } else {
                return;
            };

            removed.tile.stop_move_animations();

            let mon = &mut monitors[new_idx];
            mon.add_tile(
                removed.tile,
                MonitorAddWindowTarget::Workspace {
                    id: ws_id,
                    column_idx: None,
                },
                activate,
                true,
                removed.width,
                removed.is_full_width,
                removed.is_floating,
            );
            if activate.map_smart(|| false) {
                *active_monitor_idx = new_idx;
            }

            let mon = &mut monitors[mon_idx];
            if mon.workspace_switch.is_none() {
                monitors[mon_idx].clean_up_workspaces();
            }
        }
    }

    pub fn move_column_to_output(
        &mut self,
        output: &Output,
        target_ws_idx: Option<usize>,
        activate: bool,
    ) {
        if let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        {
            let new_idx = monitors
                .iter()
                .position(|mon| &mon.output == output)
                .unwrap();

            let current = &mut monitors[*active_monitor_idx];
            // TEAM_033: Check floating mode on canvas, not row
            if current.canvas.floating_is_active() {
                self.move_to_output(None, output, None, ActivateWindow::Smart);
                return;
            }

            let ws = current.active_workspace();

            if let Some(ws) = ws {
                let Some(column) = ws.remove_active_column() else {
                    return;
                };

                let workspace_idx = target_ws_idx
                    .unwrap_or(monitors[new_idx].active_row_idx())
                    .min(monitors[new_idx].canvas.rows().count() - 1);
                self.add_column_by_idx(new_idx, workspace_idx, column, activate);
            }
        }
    }

    pub fn move_workspace_to_output(&mut self, output: &Output) -> bool {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return false;
        };

        let idx = monitors[*active_monitor_idx].active_row_idx();
        self.move_workspace_to_output_by_id(idx, None, output)
    }

    // FIXME: accept workspace by id
    pub fn move_workspace_to_output_by_id(
        &mut self,
        old_idx: usize,
        old_output: Option<Output>,
        new_output: &Output,
    ) -> bool {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return false;
        };

        let current_idx = if let Some(old_output) = old_output {
            monitors
                .iter()
                .position(|mon| mon.output == old_output)
                .unwrap()
        } else {
            *active_monitor_idx
        };
        let target_idx = monitors
            .iter()
            .position(|mon| mon.output == *new_output)
            .unwrap();

        let current = &mut monitors[current_idx];

        if current.canvas.rows().count() <= old_idx {
            return false;
        }

        // Do not do anything if the output is already correct
        if current_idx == target_idx {
            // Just update the original output since this is an explicit movement action.
            // current.canvas.rows().nth(old_idx).unwrap().original_output =
            // OutputId::new(&current.output);

            return false;
        }

        // Only switch active monitor if the workspace to be moved is the currently focused one on
        // the current monitor.
        let activate = current_idx == *active_monitor_idx && old_idx == current.active_row_idx();

        // TEAM_055: Transfer row from source to target monitor
        // Remove the row from source and get it back
        let removed_row = current.remove_workspace_by_idx(old_idx);

        // TEAM_055: Ensure source canvas has at least one empty row
        if current.canvas.rows().count() == 0 {
            current.canvas.ensure_row(0);
        }

        let target = &mut monitors[target_idx];

        // TEAM_055: Original insert_workspace has "Don't insert past the last empty workspace"
        // logic that adjusts idx to insert BEFORE the last workspace when idx == len.
        // In BTreeMap terms, we want to insert at a key that comes BEFORE existing empty rows.
        // Use key -1 so it sorts before key 0 in BTreeMap iteration order.
        let insert_key = -1i32;

        // Insert the removed row into the target
        if let Some(row) = removed_row {
            target.canvas.insert_row(insert_key, row);
        } else {
            // If no row was removed (shouldn't happen), just ensure a row exists
            target.canvas.ensure_row(insert_key);
        }

        // If activating, set the inserted row as active
        if activate {
            target.canvas.focus_row(insert_key);
            *active_monitor_idx = target_idx;
        }

        activate
    }

    // TEAM_063: set_fullscreen, toggle_fullscreen, toggle_windowed_fullscreen,
    // set_maximized, toggle_maximized moved to layout_impl/fullscreen.rs

    pub fn workspace_switch_gesture_begin(&mut self, output: &Output, is_touchpad: bool) {
        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => unreachable!(),
        };

        for monitor in monitors {
            // Cancel the gesture on other outputs.
            if &monitor.output != output {
                monitor.workspace_switch_gesture_end(None);
                continue;
            }

            monitor.workspace_switch_gesture_begin(is_touchpad);
        }
    }

    pub fn workspace_switch_gesture_update(
        &mut self,
        delta_y: f64,
        timestamp: Duration,
        is_touchpad: bool,
    ) -> Option<Option<Output>> {
        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => return None,
        };

        for monitor in monitors {
            if let Some(refresh) =
                monitor.workspace_switch_gesture_update(delta_y, timestamp, is_touchpad)
            {
                if refresh {
                    return Some(Some(monitor.output.clone()));
                } else {
                    return Some(None);
                }
            }
        }

        None
    }

    pub fn workspace_switch_gesture_end(&mut self, is_touchpad: Option<bool>) -> Option<Output> {
        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => return None,
        };

        for monitor in monitors {
            if monitor.workspace_switch_gesture_end(is_touchpad) {
                return Some(monitor.output.clone());
            }
        }

        None
    }

    pub fn view_offset_gesture_begin(
        &mut self,
        output: &Output,
        workspace_idx: Option<usize>,
        is_touchpad: bool,
    ) {
        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => unreachable!(),
        };

        for monitor in monitors {
            // TEAM_033: Get active index before mutable iteration to avoid borrow conflicts
            let active_idx = monitor.active_row_idx();
            let is_target_output = &monitor.output == output;

            // TEAM_035: Extract row from tuple
            for (idx, (_, ws)) in monitor.canvas.rows_mut().enumerate() {
                // Cancel the gesture on other workspaces.
                if !is_target_output || idx != workspace_idx.unwrap_or(active_idx) {
                    ws.view_offset_gesture_end(None);
                    continue;
                }

                ws.view_offset_gesture_begin(is_touchpad);
            }
        }
    }

    pub fn view_offset_gesture_update(
        &mut self,
        delta_x: f64,
        timestamp: Duration,
        is_touchpad: bool,
    ) -> Option<Option<Output>> {
        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => return None,
        };

        for monitor in monitors {
            for (_, ws) in monitor.canvas.rows_mut() {
                if let Some(refresh) =
                    ws.view_offset_gesture_update(delta_x, timestamp, is_touchpad)
                {
                    if refresh {
                        return Some(Some(monitor.output.clone()));
                    } else {
                        return Some(None);
                    }
                }
            }
        }

        None
    }

    pub fn view_offset_gesture_end(&mut self, is_touchpad: Option<bool>) -> Option<Output> {
        let monitors = match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => monitors,
            MonitorSet::NoOutputs { .. } => return None,
        };

        for monitor in monitors {
            for (_, ws) in monitor.canvas.rows_mut() {
                if ws.view_offset_gesture_end(is_touchpad) {
                    return Some(monitor.output.clone());
                }
            }
        }

        None
    }

    // TEAM_014: Removed overview_gesture_begin, overview_gesture_update, overview_gesture_end (Part
    // 3) These methods are no longer needed as overview mode is removed.

    // TEAM_064: interactive_move_begin, interactive_move_update, interactive_move_end,
    // interactive_move_is_moving_above_output, dnd_update, dnd_end
    // moved to layout_impl/interactive_move.rs

    // TEAM_063: interactive_resize_begin, interactive_resize_update,
    // interactive_resize_end moved to layout_impl/resize.rs

    // TEAM_012: Renamed from move_workspace_down
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_row_down();
    }

    // TEAM_012: Renamed from move_workspace_up
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_row_up();
    }

    // TEAM_012: Renamed from move_workspace_to_idx
    pub fn move_row_to_index(
        &mut self,
        reference: Option<(Option<Output>, usize)>,
        new_idx: usize,
    ) {
        let (monitor, old_idx) = if let Some((output, old_idx)) = reference {
            let monitor = if let Some(output) = output {
                let Some(monitor) = self.monitor_for_output_mut(&output) else {
                    return;
                };
                monitor
            } else {
                // In case a numbered row reference is used, assume the active monitor
                let Some(monitor) = self.active_monitor() else {
                    return;
                };
                monitor
            };

            (monitor, old_idx)
        } else {
            let Some(monitor) = self.active_monitor() else {
                return;
            };
            let index = monitor.active_row_idx();
            (monitor, index)
        };

        monitor.move_workspace_to_idx(old_idx, new_idx);
    }

    // TEAM_012: Renamed from set_workspace_name, simplified to not take reference
    // TEAM_018: Now calls canvas instead of workspace code
    // TEAM_057: Duplicate name checking implemented in canvas/navigation.rs
    pub fn set_row_name(&mut self, name: String) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().set_row_name(Some(name));
    }

    // TEAM_012: Renamed from unset_workspace_name, simplified to not take reference
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn unset_row_name(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().unset_row_name();
    }

    // TEAM_014: Removed set_monitors_overview_state, toggle_overview, open_overview,
    // close_overview, toggle_overview_to_workspace (Part 3)
    // Overview mode is no longer supported.

    pub fn start_open_animation_for_window(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        // TEAM_021: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(window) {
                if mon.canvas.start_open_animation(window) {
                    return;
                }
            }
        }

        // Fallback to workspace iteration for compatibility
        for ws in self.workspaces_mut() {
            if ws.start_open_animation(window) {
                return;
            }
        }
    }

    pub fn store_unmap_snapshot(&mut self, renderer: &mut GlesRenderer, window: &W::Id) {
        let _span = tracy_client::span!("Layout::store_unmap_snapshot");

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().id() == window {
                move_.tile.store_unmap_snapshot_if_empty(renderer);
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(window) {
                            ws.store_unmap_snapshot_if_empty(renderer, window);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(window) {
                        ws.store_unmap_snapshot_if_empty(renderer, window);
                        return;
                    }
                }
            }
        }
    }

    pub fn clear_unmap_snapshot(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().id() == window {
                let _ = move_.tile.take_unmap_snapshot();
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.rows_mut() {
                        if ws.has_window(window) {
                            ws.clear_unmap_snapshot(window);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.rows_mut() {
                    if ws.has_window(window) {
                        ws.clear_unmap_snapshot(window);
                        return;
                    }
                }
            }
        }
    }

    pub fn start_close_animation_for_window(
        &mut self,
        renderer: &mut GlesRenderer,
        window: &W::Id,
        blocker: TransactionBlocker,
    ) {
        let _span = tracy_client::span!("Layout::start_close_animation_for_window");

        // Overview mode has been removed, zoom is always 1.0
        let zoom = 1.0;

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().id() == window {
                let Some(snapshot) = move_.tile.take_unmap_snapshot() else {
                    return;
                };
                let tile_pos = move_.tile_render_location(zoom);
                let tile_size = move_.tile.tile_size();

                let output = move_.output.clone();
                let pointer_pos_within_output = move_.pointer_pos_within_output;
                let Some(mon) = self.monitor_for_output_mut(&output) else {
                    return;
                };
                let Some((ws, ws_geo)) = mon.row_under(pointer_pos_within_output) else {
                    return;
                };
                let ws_id = ws.id();
                // TEAM_035: Extract row from tuple
                let (_, ws) = mon
                    .canvas
                    .rows_mut()
                    .find(|(_, ws)| ws.id() == ws_id)
                    .unwrap();

                let tile_pos = tile_pos - ws_geo.loc;
                ws.start_close_animation_for_tile(renderer, snapshot, tile_size, tile_pos, blocker);
                return;
            }
        }

        // TEAM_033: Destructure tuples from workspaces_mut()
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, row) in mon.canvas.rows_mut() {
                        if row.has_window(window) {
                            row.start_close_animation_for_window(renderer, window, blocker);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, row) in canvas.rows_mut() {
                    if row.has_window(window) {
                        row.start_close_animation_for_window(renderer, window, blocker);
                        return;
                    }
                }
            }
        }
    }

    // TEAM_064: render_interactive_move_for_output, refresh
    // moved to layout_impl/render.rs

    pub fn workspaces(
        &self,
    ) -> impl Iterator<Item = (Option<&Monitor<W>>, i32, &crate::layout::row::Row<W>)> + '_ {
        let iter_normal;
        let iter_no_outputs;

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                let it = monitors.iter().flat_map(|mon| {
                    mon.canvas
                        .rows()
                        .map(move |(row_idx, row)| (Some(mon), row_idx, row))
                });

                iter_normal = Some(it);
                iter_no_outputs = None;
            }
            MonitorSet::NoOutputs { canvas } => {
                let it = canvas.rows().map(|(row_idx, row)| (None, row_idx, row));

                iter_normal = None;
                iter_no_outputs = Some(it);
            }
        }

        let iter_normal = iter_normal.into_iter().flatten();
        let iter_no_outputs = iter_no_outputs.into_iter().flatten();
        iter_normal.chain(iter_no_outputs)
    }

    pub fn workspaces_mut(&mut self) -> impl Iterator<Item = &mut crate::layout::row::Row<W>> + '_ {
        let iter_normal;
        let iter_no_outputs;

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                // TEAM_035: Map (i32, &mut Row) to just &mut Row
                let it = monitors
                    .iter_mut()
                    .flat_map(|mon| mon.canvas.rows_mut().map(|(_, row)| row));

                iter_normal = Some(it);
                iter_no_outputs = None;
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Map (i32, &mut Row) to just &mut Row
                let it = canvas.rows_mut().map(|(_, row)| row);

                iter_normal = None;
                iter_no_outputs = Some(it);
            }
        }

        let iter_normal = iter_normal.into_iter().flatten();
        let iter_no_outputs = iter_no_outputs.into_iter().flatten();
        iter_normal.chain(iter_no_outputs)
    }

    pub fn windows(&self) -> impl Iterator<Item = (Option<&Monitor<W>>, &W)> {
        let moving_window = self
            .interactive_move
            .as_ref()
            .and_then(|x| x.moving())
            .map(|move_| (self.monitor_for_output(&move_.output), move_.tile.window()))
            .into_iter();

        // TEAM_054: Include both tiled and floating windows
        let tiled = self
            .workspaces()
            .flat_map(|(mon, _, ws)| ws.windows().map(move |win| (mon, win)));

        let floating: Box<dyn Iterator<Item = (Option<&Monitor<W>>, &W)>> = match &self.monitor_set
        {
            MonitorSet::Normal { monitors, .. } => Box::new(monitors.iter().flat_map(|mon| {
                mon.canvas
                    .floating
                    .tiles()
                    .map(move |tile| (Some(mon), tile.window()))
            })),
            MonitorSet::NoOutputs { canvas, .. } => {
                Box::new(canvas.floating.tiles().map(|tile| (None, tile.window())))
            }
        };

        moving_window.chain(tiled).chain(floating)
    }

    // TEAM_063: has_window moved to layout_impl/queries.rs

    // TEAM_014: Removed duplicate is_overview_open (Part 3) - see line 2343
}

impl<W: LayoutElement> Default for MonitorSet<W> {
    fn default() -> Self {
        // Create a default Canvas2D for NoOutputs
        let view_size = Size::from((1920.0, 1080.0));
        let parent_area = Rectangle::new(Point::from((0.0, 0.0)), view_size);
        let working_area = parent_area;
        let clock = Clock::with_time(Duration::ZERO);
        let options = Rc::new(Options::default());

        let canvas = crate::layout::canvas::Canvas2D::new(
            None,
            view_size,
            parent_area,
            working_area,
            1.0,
            clock,
            options,
            RowId(1),
        );

        Self::NoOutputs { canvas }
    }
}
