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
use std::mem;
use std::rc::Rc;
use std::time::Duration;

use monitor::{InsertHint, InsertPosition, InsertWorkspace, MonitorAddWindowTarget};
use niri_config::utils::MergeWith as _;
use niri_config::{
    Config, CornerRadius, LayoutPart, PresetSize, Workspace as WorkspaceConfig, WorkspaceReference,
};
use niri_ipc::{ColumnDisplay, PositionChange, SizeChange, WindowLayout};
use column::Column;
use types::{ColumnWidth, ScrollDirection};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::utils::RescaleRenderElement;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::output::{self, Output};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Scale, Serial, Size, Transform};
use tile::{Tile, TileRenderElement};
// TEAM_021: Use minimal workspace types after Canvas2D migration
use workspace_types::{WorkspaceAddWindowTarget, WorkspaceId, OutputId, compute_working_area};

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
use crate::rubber_band::RubberBand;
use crate::utils::transaction::{Transaction, TransactionBlocker};
use crate::utils::{
    ensure_min_max_size_maybe_zero, output_matches_name, output_size,
    round_logical_in_physical_max1, ResizeEdge,
};
use crate::window::ResolvedWindowRules;

pub mod closing_window;
// TEAM_002: Column module extracted from scrolling.rs
pub mod column;
pub mod floating;
pub mod focus_ring;
pub mod insert_hint_element;
pub mod monitor;
pub mod opening_window;
pub mod scrolling;
pub mod shadow;
pub mod tab_indicator;
pub mod tile;
// TEAM_003: Shared types for layout modules
pub mod types;
// TEAM_005: AnimatedValue abstraction for view offset and camera
pub mod animated_value;
// TEAM_006: Row module for 2D canvas layout
pub mod row;
// TEAM_006: Canvas2D module for 2D tiling layout
pub mod canvas;
pub mod workspace_types;  // TEAM_021: Minimal workspace types for external compatibility
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
    ///
    /// The workspace id does not necessarily point to a valid workspace. If it doesn't, then it is
    /// simply ignored.
    last_active_workspace_id: HashMap<String, WorkspaceId>,
    /// TEAM_039: Counter for generating unique workspace IDs
    workspace_id_counter: u64,
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum InteractiveMoveState<W: LayoutElement> {
    /// Initial rubberbanding; the window remains in the layout.
    Starting {
        /// The window we're moving.
        window_id: W::Id,
        /// Current pointer delta from the starting location.
        pointer_delta: Point<f64, Logical>,
        /// Pointer location within the visual window geometry as ratio from geometry size.
        ///
        /// This helps the pointer remain inside the window as it resizes.
        pointer_ratio_within_window: (f64, f64),
    },
    /// Moving; the window is no longer in the layout.
    Moving(InteractiveMoveData<W>),
}

#[derive(Debug)]
struct InteractiveMoveData<W: LayoutElement> {
    /// The window being moved.
    pub(self) tile: Tile<W>,
    /// Output where the window is currently located/rendered.
    pub(self) output: Output,
    /// Current pointer position within output.
    pub(self) pointer_pos_within_output: Point<f64, Logical>,
    /// Window column width.
    pub(self) width: ColumnWidth,
    /// Whether the window column was full-width.
    pub(self) is_full_width: bool,
    /// Whether the window targets the floating layout.
    pub(self) is_floating: bool,
    /// Pointer location within the visual window geometry as ratio from geometry size.
    ///
    /// This helps the pointer remain inside the window as it resizes.
    pub(self) pointer_ratio_within_window: (f64, f64),
    /// Config overrides for the output where the window is currently located.
    ///
    /// Cached here to be accessible while an output is removed.
    pub(self) output_config: Option<niri_config::LayoutPart>,
    /// Config overrides for the workspace where the window is currently located.
    ///
    /// To avoid sudden window changes when starting an interactive move, it will remember the
    /// config overrides for the workspace where the move originated from. As soon as the window
    /// moves over some different workspace though, this override will reset.
    pub(self) workspace_config: Option<(WorkspaceId, niri_config::LayoutPart)>,
}

#[derive(Debug)]
pub struct DndData<W: LayoutElement> {
    /// Output where the pointer is currently located.
    output: Output,
    /// Current pointer position within output.
    pointer_pos_within_output: Point<f64, Logical>,
    /// Ongoing DnD hold to activate something.
    hold: Option<DndHold<W>>,
}

#[derive(Debug)]
struct DndHold<W: LayoutElement> {
    /// Time when we started holding on the target.
    start_time: Duration,
    target: DndHoldTarget<W::Id>,
}

#[derive(Debug, PartialEq, Eq)]
enum DndHoldTarget<WindowId> {
    Window(WindowId),
    Workspace(WorkspaceId),
}

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
}

impl<W: LayoutElement> RemovedTile<W> {
    // TEAM_008: Added constructor for use by Row module
    pub(crate) fn new(
        tile: Tile<W>,
        width: ColumnWidth,
        is_full_width: bool,
        is_floating: bool,
    ) -> Self {
        Self {
            tile,
            width,
            is_full_width,
            is_floating,
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

    /// Destructures into components.
    pub fn into_parts(self) -> (Tile<W>, ColumnWidth, bool, bool) {
        (self.tile, self.width, self.is_full_width, self.is_floating)
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
    Workspace(WorkspaceId),
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

impl<W: LayoutElement> InteractiveMoveState<W> {
    fn moving(&self) -> Option<&InteractiveMoveData<W>> {
        match self {
            InteractiveMoveState::Moving(move_) => Some(move_),
            _ => None,
        }
    }

    fn moving_mut(&mut self) -> Option<&mut InteractiveMoveData<W>> {
        match self {
            InteractiveMoveState::Moving(move_) => Some(move_),
            _ => None,
        }
    }
}

impl<W: LayoutElement> InteractiveMoveData<W> {
    fn tile_render_location(&self, zoom: f64) -> Point<f64, Logical> {
        let scale = Scale::from(self.output.current_scale().fractional_scale());
        let window_size = self.tile.window_size();
        let pointer_offset_within_window = Point::from((
            window_size.w * self.pointer_ratio_within_window.0,
            window_size.h * self.pointer_ratio_within_window.1,
        ));
        let pos = self.pointer_pos_within_output
            - (pointer_offset_within_window + self.tile.window_loc() - self.tile.render_offset())
                .upscale(zoom);
        // Round to physical pixels.
        pos.to_physical_precise_round(scale).to_logical(scale)
    }
}

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

    /// TEAM_039: Generate a unique workspace ID
    pub fn next_workspace_id(&mut self) -> WorkspaceId {
        self.workspace_id_counter += 1;
        WorkspaceId(self.workspace_id_counter)
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
        let initial_workspace_id = WorkspaceId(1);
        
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
            last_active_workspace_id: HashMap::new(),
            workspace_id_counter: 1,  // TEAM_039: Start at 1 since we used ID 1 for initial row
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
        let initial_workspace_id = WorkspaceId(1);
        
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
            last_active_workspace_id: HashMap::new(),
            workspace_id_counter: 1,  // TEAM_039: Start at 1 since we used ID 1 for initial row
            interactive_move: None,
            dnd: None,
            clock,
            update_render_elements_time: Duration::ZERO,
            // TEAM_014: Removed overview_open and overview_progress (Part 3)
            options: opts,
        }
    }

    pub fn add_output(&mut self, output: Output, layout_config: Option<LayoutPart>) {
        self.monitor_set = match mem::take(&mut self.monitor_set) {
            MonitorSet::Normal {
                mut monitors,
                primary_idx,
                active_monitor_idx,
            } => {
                let primary = &mut monitors[primary_idx];

                let mut stopped_primary_ws_switch = false;

                // TEAM_024: In Canvas2D system, we don't move workspaces between monitors
                // Each monitor has its own canvas, so no workspace migration needed
                // TEAM_035: Add type annotation for empty vec and make mutable
                let mut workspaces: Vec<crate::layout::row::Row<W>> = vec![];

                if primary.workspace_switch.is_some() {
                    primary.stop_workspace_switch();
                    stopped_primary_ws_switch = true;
                }

                // If we stopped a workspace switch, then we might need to clean up workspaces.
                // Also if empty_workspace_above_first is set and there are only 2 workspaces left,
                // both will be empty and one of them needs to be removed. clean_up_workspaces
                // takes care of this.

                if stopped_primary_ws_switch
                    || (primary.options.layout.empty_workspace_above_first
                        && primary.canvas.rows().count() == 2)
                {
                    // TEAM_021: Use canvas-first cleanup if possible, fallback to workspace
                    if primary.canvas().has_windows() {
                        primary.canvas_mut().clean_up_workspaces();
                    } else if primary.canvas.rows().count() == 2 {
                        // Both rows are empty, remove one
                        // TODO: TEAM_024: Implement canvas cleanup logic
                    }
                }

                workspaces.reverse();

                // Create the new monitor with the output
                let _ws_id_to_activate = self.last_active_workspace_id.remove(&output.name());

                // TEAM_035: Add type annotation to help compiler infer W
                let monitor: Monitor<W> = Monitor::new(
                    output,
                    self.clock.clone(),
                    self.options.clone(),
                    layout_config,
                    self.next_workspace_id(),
                );
                // DEPRECATED(overview): Removed overview state sync
                monitors.push(monitor);

                MonitorSet::Normal {
                    monitors,
                    primary_idx,
                    active_monitor_idx,
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                let ws_id_to_activate = self.last_active_workspace_id.remove(&output.name());

                let monitor = Monitor::new(
                    output,
                    self.clock.clone(),
                    self.options.clone(),
                    layout_config,
                    self.next_workspace_id(),
                );
                // DEPRECATED(overview): Removed overview state sync

                MonitorSet::Normal {
                    monitors: vec![monitor],
                    primary_idx: 0,
                    active_monitor_idx: 0,
                }
            }
        }
    }

    pub fn remove_output(&mut self, output: &Output) {
        self.monitor_set = match mem::take(&mut self.monitor_set) {
            MonitorSet::Normal {
                mut monitors,
                mut primary_idx,
                mut active_monitor_idx,
            } => {
                let idx = monitors
                    .iter()
                    .position(|mon| &mon.output == output)
                    .expect("trying to remove non-existing output");
                let monitor = monitors.remove(idx);

                // TEAM_033: Store active workspace ID before consuming monitor
                let output_name = monitor.output_name().clone();
                let active_ws_id = monitor.canvas.workspaces()
                    .nth(monitor.active_workspace_idx())
                    .map(|(_, ws)| ws.id())
                    .unwrap_or_else(|| crate::layout::workspace_types::WorkspaceId::specific(0));
                
                self.last_active_workspace_id.insert(output_name, active_ws_id);

                if monitors.is_empty() {
                    // Removed the last monitor.
                    // TEAM_033: Get config values before consuming monitor
                    let view_size = monitor.view_size();
                    let working_area = monitor.working_area();
                    let scale = monitor.scale().fractional_scale();
                    let options = self.options.clone();
                    
                    // Convert monitor to canvas
                    let mut canvas = monitor.into_canvas();
                    
                    // Update all rows with layout options
                    // TEAM_033: Destructure tuple from rows_mut()
                    for (_, row) in canvas.rows_mut() {
                        row.update_config(view_size, working_area, scale, options.clone());
                    }

                    MonitorSet::NoOutputs { canvas }
                } else {
                    // TEAM_033: Convert monitor to canvas for transfer
                    let removed_canvas = monitor.into_canvas();
                    
                    if primary_idx >= idx {
                        // Update primary_idx to either still point at the same monitor, or at some
                        // other monitor if the primary has been removed.
                        primary_idx = primary_idx.saturating_sub(1);
                    }
                    if active_monitor_idx >= idx {
                        // Update active_monitor_idx to either still point at the same monitor, or
                        // at some other monitor if the active monitor has
                        // been removed.
                        active_monitor_idx = active_monitor_idx.saturating_sub(1);
                    }

                    let primary = &mut monitors[primary_idx];
                    primary.append_canvas(removed_canvas);

                    MonitorSet::Normal {
                        monitors,
                        primary_idx,
                        active_monitor_idx,
                    }
                }
            }
            MonitorSet::NoOutputs { .. } => {
                panic!("tried to remove output when there were already none")
            }
        }
    }

    pub fn add_column_by_idx(
        &mut self,
        monitor_idx: usize,
        workspace_idx: usize,
        column: Column<W>,
        activate: bool,
    ) {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            panic!()
        };

        monitors[monitor_idx].add_column(workspace_idx, column, activate);

        if activate {
            *active_monitor_idx = monitor_idx;
        }
    }

    /// Adds a new window to the layout.
    ///
    /// Returns an output that the window was added to, if there were any outputs.
    #[allow(clippy::too_many_arguments)]
    pub fn add_window(
        &mut self,
        window: W,
        target: AddWindowTarget<W>,
        width: Option<PresetSize>,
        height: Option<PresetSize>,
        is_full_width: bool,
        is_floating: bool,
        activate: ActivateWindow,
    ) -> Option<&Output> {
        let scrolling_height = height.map(SizeChange::from);
        let id = window.id().clone();

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let (mon_idx, target) = match target {
                    AddWindowTarget::Auto => (*active_monitor_idx, MonitorAddWindowTarget::Auto),
                    AddWindowTarget::Output(output) => {
                        let mon_idx = monitors
                            .iter()
                            .position(|mon| mon.output == *output)
                            .unwrap();

                        (mon_idx, MonitorAddWindowTarget::Auto)
                    }
                    AddWindowTarget::Workspace(ws_id) => {
                        let mon_idx = monitors
                            .iter()
                            .position(|mon| mon.canvas.workspaces().any(|(_, ws)| ws.id() == ws_id))
                            .unwrap();

                        (
                            mon_idx,
                            MonitorAddWindowTarget::Workspace {
                                id: ws_id,
                                column_idx: None,
                            },
                        )
                    }
                    AddWindowTarget::NextTo(next_to) => {
                        if let Some(output) = self
                            .interactive_move
                            .as_ref()
                            .and_then(|move_| {
                                if let InteractiveMoveState::Moving(move_) = move_ {
                                    Some(move_)
                                } else {
                                    None
                                }
                            })
                            .filter(|move_| next_to == move_.tile.window().id())
                            .map(|move_| move_.output.clone())
                        {
                            // The next_to window is being interactively moved.
                            let mon_idx = monitors
                                .iter()
                                .position(|mon| mon.output == output)
                                .unwrap_or(*active_monitor_idx);

                            (mon_idx, MonitorAddWindowTarget::Auto)
                        } else {
                            let mon_idx = monitors
                                .iter()
                                .position(|mon| {
                                    mon.canvas.workspaces().any(|(_, ws)| ws.has_window(next_to))
                                })
                                .unwrap();
                            (mon_idx, MonitorAddWindowTarget::NextTo(next_to))
                        }
                    }
                };
                let mon = &mut monitors[mon_idx];

                let (ws_idx, _) = mon.resolve_add_window_target(&target);
                let ws = &mon.canvas.workspaces().nth(ws_idx as usize).unwrap().1;
                // TEAM_039: resolve_scrolling_width now takes &W and returns ColumnWidth
                let scrolling_width = Some(ws.resolve_scrolling_width(&window, width));

                mon.add_window(
                    window,
                    Some(target),
                    activate,
                    scrolling_width,
                    is_full_width,
                );

                if activate.map_smart(|| false) {
                    *active_monitor_idx = mon_idx;
                }

                // Set the default height for scrolling windows.
                if !is_floating {
                    if let Some(change) = scrolling_height {
                        // Find the window in the canvas and set its height
                        if let Some((_, _, tile)) = mon.canvas.find_window(&id) {
                            let window = tile.window();
                            // TODO: TEAM_023: Implement window height setting on canvas/row
                            // For now, we'll skip this as the API needs to be adapted
                        }
                    }
                }

                Some(&mon.output)
            }
            MonitorSet::NoOutputs { canvas } => {
                let (ws_idx, target) = match target {
                    AddWindowTarget::Auto => {
                        // In Canvas2D, we always add to the active row (row 0 by default)
                        (0, WorkspaceAddWindowTarget::Auto)
                    }
                    AddWindowTarget::Output(_) => panic!(),
                    AddWindowTarget::Workspace(ws_id) => {
                        // Find the row with the given workspace ID
                        // TODO: TEAM_023: Implement proper workspace ID to row mapping
                        let ws_idx = 0; // Default to origin row for now
                        (ws_idx, WorkspaceAddWindowTarget::Auto)
                    }
                    AddWindowTarget::NextTo(next_to) => {
                        if self
                            .interactive_move
                            .as_ref()
                            .and_then(|move_| {
                                if let InteractiveMoveState::Moving(move_) = move_ {
                                    Some(move_)
                                } else {
                                    None
                                }
                            })
                            .filter(|move_| next_to == move_.tile.window().id())
                            .is_some()
                        {
                            // The next_to window is being interactively moved. If there are no
                            // other windows, we may have no workspaces at all.
                            // In Canvas2D, we always have at least the origin row
                            (0, WorkspaceAddWindowTarget::Auto)
                        } else {
                            // Find the row that contains the next_to window
                            if let Some((ws_idx, _, tile)) = canvas.find_window(&next_to) {
                                (ws_idx, WorkspaceAddWindowTarget::NextTo(tile.window()))
                            } else {
                                // Default to origin row if not found
                                (0, WorkspaceAddWindowTarget::Auto)
                            }
                        }
                    }
                };
                // TEAM_033: Fixed tile creation and add_tile arguments
                // First ensure the row exists
                let ws = canvas.ensure_row(ws_idx);
                // TEAM_039: resolve_scrolling_width now takes &W and returns ColumnWidth
                let scrolling_width = ws.resolve_scrolling_width(&window, width);
                
                // Create tile using canvas's make_tile (returns proper Tile<W>)
                let tile = canvas.make_tile(window);
                let activate_bool = activate.map_smart(|| false);
                
                // Get the row again (ensure_row may have modified the canvas)
                if let Some(ws) = canvas.row_mut(ws_idx) {
                    ws.add_tile(
                        None,
                        tile,
                        activate_bool,
                        scrolling_width,
                        is_full_width,
                    );

                    // Set the default height for scrolling windows.
                    if !is_floating {
                        if let Some(change) = scrolling_height {
                            ws.set_window_height(Some(&id), change);
                        }
                    }
                }

                None
            }
        }
    }

    pub fn remove_window(
        &mut self,
        window: &W::Id,
        transaction: Transaction,
    ) -> Option<RemovedTile<W>> {
        if let Some(state) = &self.interactive_move {
            match state {
                InteractiveMoveState::Starting { window_id, .. } => {
                    if window_id == window {
                        self.interactive_move_end(window);
                    }
                }
                InteractiveMoveState::Moving(move_) => {
                    if move_.tile.window().id() == window {
                        let Some(InteractiveMoveState::Moving(move_)) =
                            self.interactive_move.take()
                        else {
                            unreachable!()
                        };

                        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
                        for mon in self.monitors_mut() {
                            mon.dnd_scroll_gesture_end();
                            mon.canvas_mut().dnd_scroll_gesture_end();
                        }

                        return Some(RemovedTile {
                            tile: move_.tile,
                            width: move_.width,
                            is_full_width: move_.is_full_width,
                            is_floating: false,
                        });
                    }
                }
            }
        }

        // TEAM_033: Restructured to avoid borrow checker issues
        // First find the window location, then perform operations
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // First pass: find which row has the window (immutable borrow)
                    let found_ws_idx = mon.canvas.workspaces().enumerate().find_map(|(idx, (_, ws))| {
                        if ws.has_window(window) {
                            Some(idx)
                        } else {
                            None
                        }
                    });

                    if let Some(ws_idx) = found_ws_idx {
                        // Now we can mutably access the specific row
                        let removed = mon.canvas.row_mut(ws_idx as i32)
                            .expect("row should exist")
                            .remove_tile(window, transaction);

                        // Get state we need for cleanup checks
                        let ws_has_windows = mon.canvas.row(ws_idx as i32)
                            .map(|ws| ws.has_windows_or_name())
                            .unwrap_or(false);
                        let active_idx = mon.active_workspace_idx();
                        let ws_count = mon.canvas.workspaces().count();
                        let switch_in_progress = mon.workspace_switch.is_some();

                        // Clean up empty workspaces that are not active and not last.
                        if !ws_has_windows
                            && ws_idx != active_idx
                            && ws_idx != ws_count - 1
                            && !switch_in_progress
                        {
                            mon.canvas.remove_row(ws_idx as i32);
                        }

                        if ws_idx < active_idx {
                            mon.canvas.active_row_idx -= 1;
                        }

                        // Special case handling when empty_workspace_above_first is set and all
                        // workspaces are empty.
                        if mon.options.layout.empty_workspace_above_first
                            && mon.canvas.workspaces().count() == 2
                            && !switch_in_progress
                        {
                            let ws0_empty = mon.canvas.row(0).map(|ws| !ws.has_windows_or_name()).unwrap_or(true);
                            let ws1_empty = mon.canvas.row(1).map(|ws| !ws.has_windows_or_name()).unwrap_or(true);
                            if ws0_empty && ws1_empty {
                                mon.canvas.remove_row(1);
                                mon.canvas.active_row_idx = 0;
                            }
                        }
                        return Some(removed);
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // First pass: find which row has the window
                let found_ws_idx = canvas.workspaces().enumerate().find_map(|(idx, (_, ws))| {
                    if ws.has_window(window) {
                        Some(idx)
                    } else {
                        None
                    }
                });

                if let Some(ws_idx) = found_ws_idx {
                    let removed = canvas.row_mut(ws_idx as i32)
                        .expect("row should exist")
                        .remove_tile(window, transaction);

                    // Clean up empty workspaces.
                    let ws_has_windows = canvas.row(ws_idx as i32)
                        .map(|ws| ws.has_windows_or_name())
                        .unwrap_or(false);
                    if !ws_has_windows {
                        canvas.remove_row(ws_idx as i32);
                    }

                    return Some(removed);
                }
            }
        }

        None
    }

    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn descendants_added(&mut self, id: &W::Id) -> bool {
        // Check canvas rows first
        for mon in self.monitors_mut() {
            if mon.canvas.descendants_added(id) {
                return true;
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        for ws in self.workspaces_mut() {
            if ws.descendants_added(id) {
                return true;
            }
        }

        false
    }

    pub fn update_window(&mut self, window: &W::Id, serial: Option<Serial>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().id() == window {
                // Do this before calling update_window() so it can get up-to-date info.
                if let Some(serial) = serial {
                    move_.tile.window_mut().on_commit(serial);
                }

                move_.tile.update_window();
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_044: Check floating space first
                    if mon.canvas.floating.has_window(window) {
                        mon.canvas.floating.update_window(window, serial);
                        return;
                    }
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(window) {
                            ws.update_window(window, serial);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // TEAM_044: Check floating space first
                if canvas.floating.has_window(window) {
                    canvas.floating.update_window(window, serial);
                    return;
                }
                for (_, ws) in canvas.workspaces_mut() {
                    if ws.has_window(window) {
                        ws.update_window(window, serial);
                        return;
                    }
                }
            }
        }
    }

    pub fn find_workspace_by_id(&self, id: WorkspaceId) -> Option<(i32, &crate::layout::row::Row<W>)> {
        match &self.monitor_set {
            MonitorSet::Normal { ref monitors, .. } => {
                for mon in monitors {
                    if let Some((row_idx, row)) = mon
                        .canvas
                        .workspaces()
                        .find(|(_, w)| w.id() == id)
                    {
                        return Some((row_idx, row));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                if let Some((row_idx, row)) =
                    canvas.workspaces().find(|(_, w)| w.id() == id)
                {
                    return Some((row_idx, row));
                }
            }
        }

        None
    }

    pub fn find_workspace_by_name(&self, workspace_name: &str) -> Option<(i32, &crate::layout::row::Row<W>)> {
        match &self.monitor_set {
            MonitorSet::Normal { ref monitors, .. } => {
                for mon in monitors {
                    if let Some((row_idx, row)) =
                        mon.canvas.workspaces().find(|(_, w)| {
                            w.name()
                                .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
                        })
                    {
                        return Some((row_idx, row));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                if let Some((row_idx, row)) = canvas.workspaces().find(|(_, w)| {
                    w.name()
                        .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
                }) {
                    return Some((row_idx, row));
                }
            }
        }

        None
    }

    pub fn find_workspace_by_ref(
        &mut self,
        reference: WorkspaceReference,
    ) -> Option<&mut crate::layout::row::Row<W>> {
        if let WorkspaceReference::Index(index) = reference {
            self.active_monitor_mut().and_then(|m| {
                let row_idx = index.saturating_sub(1) as i32;
                m.canvas.rows_mut().find(|(idx, row)| *idx == row_idx).map(|(_, row)| row)
            })
        } else {
            // Find the workspace by name or id across all monitors
            for monitor in self.monitors_mut() {
                if let Some((_, row)) = monitor.canvas.rows_mut().find(|(idx, row)| {
                    match &reference {
                        WorkspaceReference::Name(ref_name) => row
                            .name()
                            .as_ref()
                            .is_some_and(|name| name.eq_ignore_ascii_case(ref_name)),
                        WorkspaceReference::Id(id) => row.id().get() == *id,
                        WorkspaceReference::Index(_) => unreachable!(),
                    }
                }) {
                    return Some(row);
                }
            }
            None
        }
    }

    pub fn unname_workspace(&mut self, workspace_name: &str) {
        self.unname_workspace_by_ref(WorkspaceReference::Name(workspace_name.into()));
    }

    pub fn unname_workspace_by_ref(&mut self, reference: WorkspaceReference) {
        let id = self.find_workspace_by_ref(reference).map(|ws| ws.id());
        if let Some(id) = id {
            self.unname_workspace_by_id(id);
        }
    }

    pub fn unname_workspace_by_id(&mut self, id: WorkspaceId) {
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    if mon.unname_workspace(id) {
                        return;
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_033: Find row first, then operate to avoid borrow issues
                let found_idx = canvas.rows().find_map(|(idx, ws)| {
                    if ws.id() == id {
                        Some((idx, ws.has_windows()))
                    } else {
                        None
                    }
                });

                if let Some((idx, has_windows)) = found_idx {
                    if let Some(row) = canvas.row_mut(idx) {
                        row.set_name(None);
                    }

                    // Clean up empty workspaces.
                    if !has_windows {
                        canvas.remove_row(idx);
                    }
                }
            }
        }
    }

    pub fn find_window_and_output(&self, wl_surface: &WlSurface) -> Option<(&W, Option<&Output>)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().is_wl_surface(wl_surface) {
                return Some((move_.tile.window(), Some(&move_.output)));
            }
        }

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces() {
                        if let Some(window) = ws.find_wl_surface(wl_surface) {
                            return Some((window, Some(&mon.output)));
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Use workspaces() not workspaces_mut() since we have &self
                for (_, ws) in canvas.workspaces() {
                    if let Some(window) = ws.find_wl_surface(wl_surface) {
                        return Some((window, None));
                    }
                }
            }
        }

        None
    }

    pub fn find_window_and_output_mut(
        &mut self,
        wl_surface: &WlSurface,
    ) -> Option<(&mut W, Option<&Output>)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if move_.tile.window().is_wl_surface(wl_surface) {
                return Some((move_.tile.window_mut(), Some(&move_.output)));
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if let Some(window) = ws.find_wl_surface_mut(wl_surface) {
                            return Some((window, Some(&mon.output)));
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                for (_, ws) in canvas.workspaces_mut() {
                    if let Some(window) = ws.find_wl_surface_mut(wl_surface) {
                        return Some((window, None));
                    }
                }
            }
        }

        None
    }

    /// Computes the window-geometry-relative target rect for popup unconstraining.
    ///
    /// We will try to fit popups inside this rect.
    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn popup_target_rect(&self, window: &W::Id) -> Rectangle<f64, Logical> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                // Follow the scrolling layout logic and fit the popup horizontally within the
                // window geometry.
                let width = move_.tile.window_size().w;
                let height = output_size(&move_.output).h;
                let mut target = Rectangle::from_size(Size::from((width, height)));
                // FIXME: ideally this shouldn't include the tile render offset, but the code
                // duplication would be a bit annoying for this edge case.
                target.loc.y -= move_.tile_render_location(1.).y;
                return target;
            }
        }

        // Try canvas first
        for mon in self.monitors() {
            if let Some(window_ref) = mon.canvas.find_window(window) {
                return mon.canvas.popup_target_rect(window_ref.2.window());
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        self.workspaces()
            .find_map(|(_, _, ws)| ws.popup_target_rect(window))
            .unwrap()
    }

    pub fn update_output_size(&mut self, output: &Output) {
        let _span = tracy_client::span!("Layout::update_output_size");

        let Some(mon) = self.monitor_for_output_mut(output) else {
            error!("monitor missing in update_output_size()");
            return;
        };

        mon.update_output_size();
    }

    pub fn scroll_amount_to_activate(&self, window: &W::Id) -> f64 {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return 0.;
            }
        }

        for mon in self.monitors() {
            for (_, ws) in mon.canvas.workspaces() {
                if ws.has_window(window) {
                    return ws.scroll_amount_to_activate(window);
                }
            }
        }

        0.
    }

    pub fn should_trigger_focus_follows_mouse_on(&self, window: &W::Id) -> bool {
        // During an animation, it's easy to trigger focus-follows-mouse on the previous workspace,
        // especially when clicking to switch workspace on a bar of some kind. This cancels the
        // workspace switch, which is annoying and not intended.
        //
        // This function allows focus-follows-mouse to trigger only on the animation target
        // workspace.
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return true;
            }
        }

        let MonitorSet::Normal { monitors, .. } = &self.monitor_set else {
            return true;
        };

        let (mon, ws_idx) = monitors
            .iter()
            .find_map(|mon| {
                mon.canvas.workspaces()
                    .position(|(_, ws)| ws.has_window(window))
                    .map(|ws_idx| (mon, ws_idx))
            })
            .unwrap();

        // During a gesture, focus-follows-mouse does not cause any unintended workspace switches.
        if let Some(WorkspaceSwitch::Gesture(_)) = mon.workspace_switch {
            return true;
        }

        ws_idx == mon.active_workspace_idx()
    }

    pub fn activate_window(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return;
        };

        // TEAM_033: Restructure to avoid borrow issues - find first, then operate
        for (monitor_idx, mon) in monitors.iter_mut().enumerate() {
            // First find the workspace with the window (immutable scan)
            let found_ws_idx = mon.canvas.workspaces().enumerate().find_map(|(idx, (_, ws))| {
                if ws.has_window(window) {
                    Some(idx)
                } else {
                    None
                }
            });

            if let Some(workspace_idx) = found_ws_idx {
                // Now activate the window
                if let Some(ws) = mon.canvas.row_mut(workspace_idx as i32) {
                    if ws.activate_window(window) {
                        *active_monitor_idx = monitor_idx;

                        // If currently in the middle of a vertical swipe between the target workspace
                        // and some other, don't switch the workspace.
                        match &mon.workspace_switch {
                            Some(WorkspaceSwitch::Gesture(gesture))
                                if gesture.current_idx.floor() == workspace_idx as f64
                                    || gesture.current_idx.ceil() == workspace_idx as f64 => {}
                            _ => mon.switch_workspace(workspace_idx),
                        }

                        return;
                    }
                }
            }
        }
    }

    pub fn activate_window_without_raising(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return;
        };

        for (monitor_idx, mon) in monitors.iter_mut().enumerate() {
            // TEAM_035: Find workspace index first, then switch to avoid double borrow
            let found_ws_idx = mon.canvas.workspaces_mut()
                .enumerate()
                .find_map(|(workspace_idx, (_, ws))| {
                    if ws.activate_window_without_raising(window) {
                        Some(workspace_idx)
                    } else {
                        None
                    }
                });

            if let Some(workspace_idx) = found_ws_idx {
                *active_monitor_idx = monitor_idx;

                // If currently in the middle of a vertical swipe between the target workspace
                // and some other, don't switch the workspace.
                match &mon.workspace_switch {
                    Some(WorkspaceSwitch::Gesture(gesture))
                        if gesture.current_idx.floor() == workspace_idx as f64
                            || gesture.current_idx.ceil() == workspace_idx as f64 => {}
                    _ => mon.switch_workspace(workspace_idx),
                }

                return;
            }
        }
    }

    pub fn active_output(&self) -> Option<&Output> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        Some(&monitors[*active_monitor_idx].output)
    }

    pub fn active_workspace(&self) -> Option<&crate::layout::row::Row<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        let mon = &monitors[*active_monitor_idx];
        mon.canvas.active_workspace()
    }

    pub fn active_workspace_mut(&mut self) -> Option<&mut crate::layout::row::Row<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            return None;
        };

        let mon = &mut monitors[*active_monitor_idx];
        mon.canvas.active_workspace_mut()
    }

    pub fn windows_for_output(&self, output: &Output) -> impl Iterator<Item = &W> + '_ {
        let MonitorSet::Normal { monitors, .. } = &self.monitor_set else {
            panic!()
        };

        let moving_window = self
            .interactive_move
            .as_ref()
            .and_then(|x| x.moving())
            .filter(|move_| move_.output == *output)
            .map(|move_| move_.tile.window())
            .into_iter();

        let mon = monitors.iter().find(|mon| &mon.output == output).unwrap();
        let mon_windows = mon.canvas.workspaces().flat_map(|(_, ws)| ws.windows());

        moving_window.chain(mon_windows)
    }

    pub fn windows_for_output_mut(&mut self, output: &Output) -> impl Iterator<Item = &mut W> + '_ {
        let MonitorSet::Normal { monitors, .. } = &mut self.monitor_set else {
            panic!()
        };

        let moving_window = self
            .interactive_move
            .as_mut()
            .and_then(|x| x.moving_mut())
            .filter(|move_| move_.output == *output)
            .map(|move_| move_.tile.window_mut())
            .into_iter();

        let mon = monitors
            .iter_mut()
            .find(|mon| &mon.output == output)
            .unwrap();
        // TEAM_035: Extract row from tuple
        let mon_windows = mon.canvas.workspaces_mut().flat_map(|(_, ws)| ws.windows_mut());

        moving_window.chain(mon_windows)
    }

    pub fn with_windows(
        &self,
        mut f: impl FnMut(&W, Option<&Output>, Option<WorkspaceId>, WindowLayout),
    ) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            // We don't fill any positions for interactively moved windows.
            let layout = move_.tile.ipc_layout_template();
            f(move_.tile.window(), Some(&move_.output), None, layout);
        }

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces() {
                        for (tile, layout) in ws.tiles_with_ipc_layouts() {
                            f(tile.window(), Some(&mon.output), Some(ws.id()), layout);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Use workspaces() not workspaces_mut() since we have &self
                for (_, ws) in canvas.workspaces() {
                    for (tile, layout) in ws.tiles_with_ipc_layouts() {
                        f(tile.window(), None, Some(ws.id()), layout);
                    }
                }
            }
        }
    }

    pub fn with_windows_mut(&mut self, mut f: impl FnMut(&mut W, Option<&Output>)) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            f(move_.tile.window_mut(), Some(&move_.output));
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        for win in ws.windows_mut() {
                            f(win, Some(&mon.output));
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                for (_, ws) in canvas.workspaces_mut() {
                    for win in ws.windows_mut() {
                        f(win, None);
                    }
                }
            }
        }
    }

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

    pub fn active_monitor_mut(&mut self) -> Option<&mut Monitor<W>> {
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

    pub fn active_monitor_ref(&self) -> Option<&Monitor<W>> {
        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        Some(&monitors[*active_monitor_idx])
    }

    pub fn monitors(&self) -> impl Iterator<Item = &Monitor<W>> + '_ {
        let monitors = if let MonitorSet::Normal { monitors, .. } = &self.monitor_set {
            &monitors[..]
        } else {
            &[][..]
        };

        monitors.iter()
    }

    pub fn monitors_mut(&mut self) -> impl Iterator<Item = &mut Monitor<W>> + '_ {
        let monitors = if let MonitorSet::Normal { monitors, .. } = &mut self.monitor_set {
            &mut monitors[..]
        } else {
            &mut [][..]
        };

        monitors.iter_mut()
    }

    pub fn monitor_for_output(&self, output: &Output) -> Option<&Monitor<W>> {
        self.monitors().find(|mon| &mon.output == output)
    }

    pub fn monitor_for_output_mut(&mut self, output: &Output) -> Option<&mut Monitor<W>> {
        self.monitors_mut().find(|mon| &mon.output == output)
    }

    pub fn monitor_for_workspace(&self, workspace_name: &str) -> Option<&Monitor<W>> {
        self.monitors().find(|monitor| {
            monitor.canvas.workspaces().any(|(idx, ws)| {
                ws.name()
                    .is_some_and(|name| name.eq_ignore_ascii_case(workspace_name))
            })
        })
    }

    pub fn outputs(&self) -> impl Iterator<Item = &Output> + '_ {
        self.monitors().map(|mon| &mon.output)
    }

    pub fn move_left(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_left();
        }
    }

    pub fn move_right(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_right();
        }
    }

    pub fn move_column_to_first(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_column_to_first();
        }
    }

    pub fn move_column_to_last(&mut self) {
        if let Some(monitor) = self.active_monitor() {
            monitor.canvas_mut().move_column_to_last();
        }
    }

    pub fn move_column_left_or_to_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.move_left() {
                return false;
            }
        }

        self.move_column_to_output(output, None, true);
        true
    }

    pub fn move_column_right_or_to_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.move_right() {
                return false;
            }
        }

        self.move_column_to_output(output, None, true);
        true
    }

    pub fn move_column_to_index(&mut self, index: usize) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.move_column_to_index(index);
    }

    pub fn move_down(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.move_down();
    }

    pub fn move_up(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.move_up();
    }

    // TEAM_012: Renamed from move_down_or_to_workspace_down
    pub fn move_down_or_to_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_down_or_to_workspace_down();
    }

    // TEAM_012: Renamed from move_up_or_to_workspace_up
    pub fn move_up_or_to_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_up_or_to_workspace_up();
    }

    pub fn consume_or_expel_window_left(&mut self, window: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.consume_or_expel_window_left(window);
    }

    pub fn consume_or_expel_window_right(&mut self, window: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.consume_or_expel_window_right(window);
    }

    pub fn focus_left(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_left();
    }

    pub fn focus_right(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_right();
    }

    pub fn focus_column_first(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_column_first();
    }

    pub fn focus_column_last(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_column_last();
    }

    pub fn focus_column_right_or_first(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_column_right_or_first();
    }

    pub fn focus_column_left_or_last(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_column_left_or_last();
    }

    pub fn focus_column(&mut self, index: usize) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_column(index);
    }

    pub fn focus_window_up_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.focus_up() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_window_down_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.focus_down() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_column_left_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.focus_left() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_column_right_or_output(&mut self, output: &Output) -> bool {
        if let Some(workspace) = self.active_workspace_mut() {
            if workspace.focus_right() {
                return false;
            }
        }

        self.focus_output(output);
        true
    }

    pub fn focus_window_in_column(&mut self, index: u8) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        // TEAM_040: Fixed - pass u8 directly
        workspace.focus_window_in_column(index);
    }

    pub fn focus_down(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_down();
    }

    pub fn focus_up(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_up();
    }

    pub fn focus_down_or_left(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_down_or_left();
    }

    pub fn focus_down_or_right(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_down_or_right();
    }

    pub fn focus_up_or_left(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_up_or_left();
    }

    pub fn focus_up_or_right(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_up_or_right();
    }

    // TEAM_012: Renamed from focus_window_or_workspace_down
    pub fn focus_window_or_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.focus_window_or_workspace_down();
    }

    // TEAM_012: Renamed from focus_window_or_workspace_up
    pub fn focus_window_or_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.focus_window_or_workspace_up();
    }

    pub fn focus_window_top(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_window_top();
    }

    pub fn focus_window_bottom(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_window_bottom();
    }

    pub fn focus_window_down_or_top(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_window_down_or_top();
    }

    pub fn focus_window_up_or_bottom(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_window_up_or_bottom();
    }

    // TEAM_012: Renamed from move_to_workspace_up
    pub fn move_to_row_up(&mut self, focus: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_to_workspace_up(focus);
    }

    // TEAM_012: Renamed from move_to_workspace_down
    pub fn move_to_row_down(&mut self, focus: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_to_workspace_down(focus);
    }

    pub fn move_to_workspace(
        &mut self,
        window: Option<&W::Id>,
        idx: usize,
        activate: ActivateWindow,
    ) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
                return;
            }
        }

        let monitor = if let Some(window) = window {
            match &mut self.monitor_set {
                MonitorSet::Normal { monitors, .. } => monitors
                    .iter_mut()
                    .find(|mon| mon.has_window(window))
                    .unwrap(),
                MonitorSet::NoOutputs { .. } => {
                    return;
                }
            }
        } else {
            let Some(monitor) = self.active_monitor() else {
                return;
            };
            monitor
        };
        monitor.move_to_workspace(window, idx, activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace_up
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_column_to_row_up(&mut self, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_column_to_row_up(activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace_down
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn move_column_to_row_down(&mut self, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().move_column_to_row_down(activate);
    }

    // TEAM_012: Renamed from move_column_to_workspace
    pub fn move_column_to_row(&mut self, idx: usize, activate: bool) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.move_column_to_workspace(idx, activate);
    }

    // TEAM_012: Renamed from switch_workspace_up
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn focus_row_up(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().focus_row_up();
    }

    // TEAM_012: Renamed from switch_workspace_down
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn focus_row_down(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.canvas_mut().focus_row_down();
    }

    // TEAM_012: Renamed from switch_workspace
    pub fn focus_row(&mut self, idx: usize) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_workspace(idx);
    }

    // TEAM_012: Renamed from switch_workspace_auto_back_and_forth
    pub fn focus_row_auto_back_and_forth(&mut self, idx: usize) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_workspace_auto_back_and_forth(idx);
    }

    // TEAM_012: Renamed from switch_workspace_previous
    pub fn focus_previous_position(&mut self) {
        let Some(monitor) = self.active_monitor() else {
            return;
        };
        monitor.switch_workspace_previous();
    }

    pub fn consume_into_column(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.consume_into_column();
    }

    pub fn expel_from_column(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.expel_from_column();
    }

    pub fn swap_window_in_direction(&mut self, direction: ScrollDirection) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.swap_window_in_direction(direction);
    }

    pub fn toggle_column_tabbed_display(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.toggle_column_tabbed_display();
    }

    pub fn set_column_display(&mut self, display: ColumnDisplay) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.set_column_display(display);
    }

    pub fn center_column(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.center_column();
    }

    pub fn center_window(&mut self, id: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if id.is_none() || id == Some(move_.tile.window().id()) {
                return;
            }
        }

        // Find the monitor that contains the window and call canvas.center_window
        if let Some(id) = id {
            for monitor in self.monitors_mut() {
                if monitor.canvas().has_window(id) {
                    monitor.canvas_mut().center_window(Some(id));
                    break;
                }
            }
        } else {
            // Center active window on active monitor
            if let Some(monitor) = self.active_monitor() {
                monitor.canvas_mut().center_window(None);
            }
        }
    }

    pub fn center_visible_columns(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.center_visible_columns();
    }

    pub fn focus(&self) -> Option<&W> {
        self.focus_with_output().map(|(win, _out)| win)
    }

    pub fn focus_with_output(&self) -> Option<(&W, &Output)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            return Some((move_.tile.window(), &move_.output));
        }

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &self.monitor_set
        else {
            return None;
        };

        let mon = &monitors[*active_monitor_idx];
        mon.active_window().map(|win| (win, &mon.output))
    }

    pub fn interactive_moved_window_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&W, HitType)> {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.output == *output {
                // TEAM_014: Removed overview zoom handling (Part 3)
                let tile_pos = move_.tile_render_location(1.);
                HitType::hit_tile(&move_.tile, tile_pos, pos_within_output)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the window under the cursor and the hit type.
    pub fn window_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<(&W, HitType)> {
        let mon = self.monitor_for_output(output)?;
        mon.window_under(pos_within_output)
    }

    pub fn resize_edges_under(
        &self,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<ResizeEdge> {
        let mon = self.monitor_for_output(output)?;
        mon.resize_edges_under(pos_within_output)
    }

    pub fn workspace_under(
        &self,
        extended_bounds: bool,
        output: &Output,
        pos_within_output: Point<f64, Logical>,
    ) -> Option<&crate::layout::row::Row<W>> {
        if self
            .interactive_moved_window_under(output, pos_within_output)
            .is_some()
        {
            return None;
        }

        let mon = self.monitor_for_output(output)?;
        if extended_bounds {
            mon.workspace_under(pos_within_output).map(|(ws, _)| ws)
        } else {
            mon.workspace_under_narrow(pos_within_output)
        }
    }

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
                for (_, workspace) in canvas.workspaces() {
                    assert!(
                        workspace.has_windows_or_name(),
                        "with no outputs there cannot be empty unnamed workspaces"
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

            for (_, workspace) in monitor.canvas.workspaces() {
                let ws_id = workspace.id();
                // TEAM_039: Debug workspace ID uniqueness
                if seen_workspace_id.contains(&ws_id) {
                    panic!("workspace id must be unique: duplicate ID {:?} found", ws_id);
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

    pub fn advance_animations(&mut self) {
        let _span = tracy_client::span!("Layout::advance_animations");

        let mut dnd_scroll = None;
        let mut is_dnd = false;
        if let Some(dnd) = &self.dnd {
            dnd_scroll = Some((dnd.output.clone(), dnd.pointer_pos_within_output, true));
            is_dnd = true;
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            move_.tile.advance_animations();

            if dnd_scroll.is_none() {
                dnd_scroll = Some((
                    move_.output.clone(),
                    move_.pointer_pos_within_output,
                    !move_.is_floating,
                ));
            }
        }

        // TEAM_014: Removed is_overview_open (Part 3)

        // Scroll the view if needed.
        if let Some((output, pos_within_output, is_scrolling)) = dnd_scroll {
            if let Some(mon) = self.monitor_for_output_mut(&output) {
                let mut scrolled = false;

                // TEAM_014: Removed overview zoom (Part 3) - always 1.0
                scrolled |= mon.dnd_scroll_gesture_scroll(pos_within_output, 1.);

                if is_scrolling {
                    if let Some((ws, geo)) = mon.workspace_under(pos_within_output) {
                        let ws_id = ws.id();
                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon
                            .canvas.workspaces_mut()
                            .find(|(_, ws)| ws.id() == ws_id)
                            .unwrap();
                        // As far as the DnD scroll gesture is concerned, the workspace spans across
                        // the whole monitor horizontally.
                        let ws_pos = Point::from((0., geo.loc.y));
                        // TEAM_014: Removed overview zoom (Part 3) - always 1.0
                        // TEAM_035: Row's dnd_scroll_gesture_scroll only takes delta.x
                        let delta = pos_within_output - ws_pos;
                        scrolled |= ws.dnd_scroll_gesture_scroll(delta.x);
                    }
                }

                if scrolled {
                    // Don't trigger DnD hold while scrolling.
                    if let Some(dnd) = &mut self.dnd {
                        dnd.hold = None;
                    }
                } else if is_dnd {
                    let target = mon
                        .window_under(pos_within_output)
                        .map(|(win, _)| DndHoldTarget::Window(win.id().clone()))
                        .or_else(|| {
                            mon.workspace_under_narrow(pos_within_output)
                                .map(|ws| DndHoldTarget::Workspace(ws.id()))
                        });

                    let dnd = self.dnd.as_mut().unwrap();
                    if let Some(target) = target {
                        let now = self.clock.now_unadjusted();
                        let start_time = if let Some(hold) = &mut dnd.hold {
                            if hold.target != target {
                                hold.start_time = now;
                            }
                            hold.target = target;
                            hold.start_time
                        } else {
                            let hold = dnd.hold.insert(DndHold {
                                start_time: now,
                                target,
                            });
                            hold.start_time
                        };

                        // Delay copied from gnome-shell.
                        let delay = Duration::from_millis(750);
                        if delay <= now.saturating_sub(start_time) {
                            let hold = dnd.hold.take().unwrap();

                            // TEAM_014: Removed overview animation sync (Part 3)
                            let config = None;

                            let mon = self.monitor_for_output_mut(&output).unwrap();

                            let ws_idx = match hold.target {
                                // TEAM_035: Extract row from tuple
                                DndHoldTarget::Window(id) => mon
                                    .canvas
                                    .workspaces_mut()
                                    .position(|(_, ws)| ws.activate_window(&id))
                                    .unwrap(),
                                DndHoldTarget::Workspace(id) => {
                                    mon.canvas.workspaces().position(|(idx, ws)| ws.id() == id).unwrap()
                                }
                            };

                            mon.dnd_scroll_gesture_end();
                            mon.activate_workspace_with_anim_config(ws_idx, config);

                            self.focus_output(&output);

                            // TEAM_014: Removed close_overview call (Part 3)
                        }
                    } else {
                        // No target, reset the hold timer.
                        dnd.hold = None;
                    }
                }
            }
        }

        // TEAM_014: Removed overview_progress animation handling (Part 3)

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    // TEAM_014: Removed set_overview_progress call (Part 3)
                    mon.advance_animations();
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
                    ws.advance_animations();
                }
            }
        }
    }

    pub fn are_animations_ongoing(&self, output: Option<&Output>) -> bool {
        // Keep advancing animations if we might need to scroll the view.
        if let Some(dnd) = &self.dnd {
            if output.map_or(true, |output| *output == dnd.output) {
                return true;
            }
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if output.map_or(true, |output| *output == move_.output) {
                if move_.tile.are_animations_ongoing() {
                    return true;
                }

                // Keep advancing animations if we might need to scroll the view.
                if !move_.is_floating {
                    return true;
                }
            }
        }

        // TEAM_014: Removed overview_progress animation check (Part 3)

        for mon in self.monitors() {
            if output.is_some_and(|output| mon.output != *output) {
                continue;
            }

            if mon.are_animations_ongoing() {
                return true;
            }
        }

        false
    }

    pub fn update_render_elements(&mut self, output: Option<&Output>) {
        let _span = tracy_client::span!("Layout::update_render_elements");

        self.update_render_elements_time = self.clock.now();

        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if output.map_or(true, |output| move_.output == *output) {
                let pos_within_output = move_.tile_render_location(1.);
                let view_rect =
                    Rectangle::new(pos_within_output.upscale(-1.), output_size(&move_.output));
                move_.tile.update_render_elements(true, view_rect);
            }
        }

        self.update_insert_hint(output);

        let MonitorSet::Normal {
            monitors,
            active_monitor_idx,
            ..
        } = &mut self.monitor_set
        else {
            error!("update_render_elements called with no monitors");
            return;
        };

        for (idx, mon) in monitors.iter_mut().enumerate() {
            if output.map_or(true, |output| mon.output == *output) {
                let is_active = self.is_active
                    && idx == *active_monitor_idx
                    && !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_)));
                // TEAM_014: Removed set_overview_progress call (Part 3)
                mon.update_render_elements(is_active);
            }
        }
    }

    pub fn update_shaders(&mut self) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            move_.tile.update_shaders();
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    mon.update_shaders();
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
                    ws.update_shaders();
                }
            }
        }
    }

    fn update_insert_hint(&mut self, output: Option<&Output>) {
        let _span = tracy_client::span!("Layout::update_insert_hint");

        for mon in self.monitors_mut() {
            mon.insert_hint = None;
        }

        if !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_))) {
            return;
        }
        let Some(InteractiveMoveState::Moving(move_)) = self.interactive_move.take() else {
            unreachable!()
        };
        if output.is_some_and(|out| &move_.output != out) {
            self.interactive_move = Some(InteractiveMoveState::Moving(move_));
            return;
        }

        let _span = tracy_client::span!("Layout::update_insert_hint::update");

        if let Some(mon) = self.monitor_for_output_mut(&move_.output) {
            // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
            let (insert_ws, geo) = mon.insert_position(move_.pointer_pos_within_output);
            match insert_ws {
                InsertWorkspace::Existing(ws_id) => {
                    // TEAM_035: Extract row from tuple
                    let (_, ws) = mon
                        .canvas.workspaces_mut()
                        .find(|(_, ws)| ws.id() == ws_id)
                        .unwrap();
                    let pos_within_workspace = move_.pointer_pos_within_output - geo.loc;
                    let position = if move_.is_floating {
                        InsertPosition::Floating
                    } else {
                        ws.scrolling_insert_position(pos_within_workspace)
                    };

                    let rules = move_.tile.window().rules();
                    let border_width = move_.tile.effective_border_width().unwrap_or(0.);
                    let corner_radius = rules
                        .geometry_corner_radius
                        .map_or(CornerRadius::default(), |radius| {
                            radius.expanded_by(border_width as f32)
                        });
                    mon.insert_hint = Some(InsertHint {
                        workspace: insert_ws,
                        position,
                        corner_radius,
                    });
                }
                InsertWorkspace::NewAt(_) => {
                    let position = if move_.is_floating {
                        InsertPosition::Floating
                    } else {
                        InsertPosition::NewColumn(0)
                    };
                    mon.insert_hint = Some(InsertHint {
                        workspace: insert_ws,
                        position,
                        corner_radius: CornerRadius::default(),
                    });
                }
            }
        }

        self.interactive_move = Some(InteractiveMoveState::Moving(move_));
    }

    pub fn ensure_named_workspace(&mut self, ws_config: &WorkspaceConfig) {
        if self.find_workspace_by_name(&ws_config.name.0).is_some() {
            return;
        }

        let clock = self.clock.clone();
        let options = self.options.clone();

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                primary_idx,
                active_monitor_idx,
            } => {
                let mon_idx = ws_config
                    .open_on_output
                    .as_deref()
                    .map(|name| {
                        monitors
                            .iter_mut()
                            .position(|monitor| output_matches_name(&monitor.output, name))
                            .unwrap_or(*primary_idx)
                    })
                    .unwrap_or(*active_monitor_idx);
                let mon = &mut monitors[mon_idx];

                // TEAM_024: Create a named row in the canvas instead of a workspace
                // Find the next available row index (start from 1, above origin)
                let mut row_idx = 1i32;
                while mon.canvas.workspaces().any(|(idx, _)| idx == row_idx) {
                    row_idx += 1;
                }
                
                let row = mon.canvas.ensure_row(row_idx);
                row.set_name(Some(ws_config.name.0.clone()));
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_024: Create a named row in the canvas instead of a workspace
                // Find the next available row index (start from 1, above origin)
                let mut row_idx = 1i32;
                while canvas.workspaces().any(|(idx, _)| idx == row_idx) {
                    row_idx += 1;
                }
                
                let row = canvas.ensure_row(row_idx);
                row.set_name(Some(ws_config.name.0.clone()));
            }
        }
    }

    /// TEAM_020: Migrated to use canvas instead of workspace iteration
    pub fn update_config(&mut self, config: &Config) {
        // Update canvas config for all monitors
        let options = Options::from_config(config);
        for mon in self.monitors_mut() {
            mon.canvas.update_config(Rc::new(options.clone()));
        }

        // TODO(TEAM_020): Eventually remove workspace config entirely
        // For now, keep workspace config for compatibility
        // Update workspace-specific config for all named workspaces.
        for ws in self.workspaces_mut() {
            let Some(name) = ws.name() else { continue };
            if let Some(config) = config.workspaces.iter().find(|w| &w.name.0 == name) {
                ws.update_layout_config(config.layout.clone().map(|x| x.0));
            }
        }

        self.update_options(Options::from_config(config));
    }

    fn update_options(&mut self, options: Options) {
        let options = Rc::new(options);

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            let view_size = output_size(&move_.output);
            let scale = move_.output.current_scale().fractional_scale();
            let options = Options::clone(&options)
                .with_merged_layout(move_.output_config.as_ref())
                .with_merged_layout(move_.workspace_config.as_ref().map(|(_, c)| c))
                .adjusted_for_scale(scale);
            move_.tile.update_config(view_size, scale, Rc::new(options));
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    mon.update_config(options.clone());
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Use canvas properties for update_config
                let view_size = canvas.view_size;
                let parent_area = canvas.parent_area;
                let scale = canvas.scale;
                for (_, ws) in canvas.workspaces_mut() {
                    ws.update_config(view_size, parent_area, scale, options.clone());
                }
            }
        }

        self.options = options;
    }

    pub fn toggle_width(&mut self, forwards: bool) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.toggle_width(forwards);
    }

    pub fn toggle_window_width(&mut self, window: Option<&W::Id>, forwards: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.toggle_window_width(window, forwards);
    }

    pub fn toggle_window_height(&mut self, window: Option<&W::Id>, forwards: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.toggle_window_height(window, forwards);
    }

    pub fn toggle_full_width(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.toggle_full_width();
    }

    pub fn set_column_width(&mut self, change: SizeChange) {
        // TEAM_043: Handle floating windows
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, active_monitor_idx, .. } => {
                let mon = &mut monitors[*active_monitor_idx];
                if mon.canvas.floating_is_active {
                    mon.canvas.floating.set_window_width(None, change, true);
                } else if let Some(row) = mon.canvas.active_row_mut() {
                    row.set_column_width(change);
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                if canvas.floating_is_active {
                    canvas.floating.set_window_width(None, change, true);
                } else if let Some(row) = canvas.active_row_mut() {
                    row.set_column_width(change);
                }
            }
        }
    }

    pub fn set_window_width(&mut self, window: Option<&W::Id>, change: SizeChange) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.set_window_width(window, change);
    }

    pub fn set_window_height(&mut self, window: Option<&W::Id>, change: SizeChange) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.set_window_height(window, change);
    }

    pub fn reset_window_height(&mut self, window: Option<&W::Id>) {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if window.is_none() || window == Some(move_.tile.window().id()) {
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.reset_window_height(window);
    }

    pub fn expand_column_to_available_width(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.expand_column_to_available_width();
    }

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
            MonitorSet::Normal { monitors, active_monitor_idx, .. } => {
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
                    monitors[*active_monitor_idx].canvas.toggle_floating_window_by_id(None);
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
            self.active_workspace_mut()
        };

        let Some(workspace) = workspace else {
            return;
        };
        workspace.set_window_floating(window, floating);
    }

    pub fn focus_floating(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_floating();
    }

    pub fn focus_tiling(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.focus_tiling();
    }

    pub fn switch_focus_floating_tiling(&mut self) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };
        workspace.switch_focus_floating_tiling();
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
                    monitor.canvas_mut().move_floating_window(Some(id), x, y, animate);
                    break;
                }
            }
        } else {
            // Move active floating window on active monitor
            if let Some(monitor) = self.active_monitor() {
                monitor.canvas_mut().move_floating_window(None, x, y, animate);
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
                        mon.canvas.workspaces()
                            .position(|(_, ws)| ws.has_window(window))
                            .map(|ws_idx| (mon_idx, ws_idx))
                    })
                    .unwrap()
            } else {
                let mon_idx = *active_monitor_idx;
                let mon = &monitors[mon_idx];
                (mon_idx, mon.active_workspace_idx())
            };

            let workspace_idx = target_ws_idx.unwrap_or(monitors[new_idx].active_workspace_idx());
            if mon_idx == new_idx && ws_idx == workspace_idx {
                return;
            }

            let mon = &monitors[new_idx];
            if mon.canvas.workspaces().count() <= workspace_idx {
                return;
            }

            let ws_id = mon.canvas.workspaces().nth(workspace_idx).unwrap().1.id();

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
            let ws = mon.canvas.row_mut(ws_idx as i32).expect("workspace should exist");
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
                    .unwrap_or(monitors[new_idx].active_workspace_idx())
                    .min(monitors[new_idx].canvas.workspaces().count() - 1);
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

        let idx = monitors[*active_monitor_idx].active_workspace_idx();
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

        if current.canvas.workspaces().count() <= old_idx {
            return false;
        }

        // Do not do anything if the output is already correct
        if current_idx == target_idx {
            // Just update the original output since this is an explicit movement action.
            // TODO: TEAM_032: original_output field doesn't exist in Row - adapt or remove this functionality
            // current.canvas.workspaces().nth(old_idx).unwrap().original_output = OutputId::new(&current.output);

            return false;
        }

        // Only switch active monitor if the workspace to be moved is the currently focused one on
        // the current monitor.
        let activate =
            current_idx == *active_monitor_idx && old_idx == current.active_workspace_idx();

        // TEAM_035: In Canvas2D, we don't move workspaces between monitors
        // Each monitor has its own canvas. Just ensure the target has a row at the right index.
        let _ws = current.remove_workspace_by_idx(old_idx);
        // TODO: TEAM_032: original_output field doesn't exist in Row - adapt or remove this functionality
        // ws.original_output = OutputId::new(new_output);

        let target = &mut monitors[target_idx];
        let target_idx_row = target.active_workspace_idx() + 1;
        target.insert_workspace(target_idx_row);

        if activate {
            *active_monitor_idx = target_idx;
        }

        activate
    }

    pub fn set_fullscreen(&mut self, id: &W::Id, is_fullscreen: bool) {
        // Check if this is a request to unset the windowed fullscreen state.
        if !is_fullscreen {
            let mut handled = false;
            self.with_windows_mut(|window, _| {
                if window.id() == id && window.is_pending_windowed_fullscreen() {
                    window.request_windowed_fullscreen(false);
                    handled = true;
                }
            });
            if handled {
                return;
            }
        }

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.set_fullscreen(id, is_fullscreen);
                return;
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.set_fullscreen(id, is_fullscreen);
                return;
            }
        }
    }

    pub fn toggle_fullscreen(&mut self, id: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.toggle_fullscreen(id);
                return;
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.toggle_fullscreen(id);
                return;
            }
        }
    }

    pub fn toggle_windowed_fullscreen(&mut self, id: &W::Id) {
        let (_, window) = self.windows().find(|(_, win)| win.id() == id).unwrap();
        if window.pending_sizing_mode().is_fullscreen() {
            // Remove the real fullscreen.
            // TEAM_021: Try canvas first, then fallback to workspace
            for mon in self.monitors_mut() {
                if mon.canvas.has_window(id) {
                    mon.canvas.set_fullscreen(id, false);
                    break;
                }
            }

            // Fallback to workspace iteration for compatibility
            for ws in self.workspaces_mut() {
                if ws.has_window(id) {
                    ws.set_fullscreen(id, false);
                    break;
                }
            }
        }

        // This will switch is_pending_fullscreen() to false right away.
        self.with_windows_mut(|window, _| {
            if window.id() == id {
                window.request_windowed_fullscreen(!window.is_pending_windowed_fullscreen());
            }
        });
    }

    pub fn set_maximized(&mut self, id: &W::Id, maximize: bool) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.set_maximized(id, maximize);
                return;
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.set_maximized(id, maximize);
                return;
            }
        }
    }

    pub fn toggle_maximized(&mut self, id: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == id {
                return;
            }
        }

        // TEAM_020: Try canvas first, then fallback to workspace
        for mon in self.monitors_mut() {
            if mon.canvas.has_window(id) {
                mon.canvas.toggle_maximized(id);
                return;
            }
        }

        // TODO(TEAM_020): Eventually remove workspace check entirely
        // For now, keep workspace check for compatibility
        for ws in self.workspaces_mut() {
            if ws.has_window(id) {
                ws.toggle_maximized(id);
                return;
            }
        }
    }

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
            let active_idx = monitor.active_workspace_idx();
            let is_target_output = &monitor.output == output;
            
            // TEAM_035: Extract row from tuple
            for (idx, (_, ws)) in monitor.canvas.workspaces_mut().enumerate() {
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
            for (_, ws) in monitor.canvas.workspaces_mut() {
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
            for (_, ws) in monitor.canvas.workspaces_mut() {
                if ws.view_offset_gesture_end(is_touchpad) {
                    return Some(monitor.output.clone());
                }
            }
        }

        None
    }

    // TEAM_014: Removed overview_gesture_begin, overview_gesture_update, overview_gesture_end (Part 3)
    // These methods are no longer needed as overview mode is removed.

    pub fn interactive_move_begin(
        &mut self,
        window_id: W::Id,
        output: &Output,
        start_pos_within_output: Point<f64, Logical>,
    ) -> bool {
        if self.interactive_move.is_some() {
            return false;
        }

        let Some((mon, (ws, ws_geo))) = self.monitors().find_map(|mon| {
            mon.workspaces_with_render_geo()
                .find(|(ws, _)| ws.has_window(&window_id))
                .map(|rv| (mon, rv))
        }) else {
            return false;
        };

        if mon.output() != output {
            return false;
        }

        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

        let is_floating = ws.is_floating(&window_id);
        let (tile, tile_offset, _visible) = ws
            .tiles_with_render_positions()
            .find(|(tile, _, _)| tile.window().id() == &window_id)
            .unwrap();
        let window_offset = tile.window_loc();

        let tile_pos = ws_geo.loc + tile_offset;

        let pointer_offset_within_window =
            start_pos_within_output - tile_pos - window_offset;
        let window_size = tile.window_size();
        let pointer_ratio_within_window = (
            f64::clamp(pointer_offset_within_window.x / window_size.w, 0., 1.),
            f64::clamp(pointer_offset_within_window.y / window_size.h, 0., 1.),
        );

        self.interactive_move = Some(InteractiveMoveState::Starting {
            window_id,
            pointer_delta: Point::from((0., 0.)),
            pointer_ratio_within_window,
        });

        // dnd_scroll_gesture_begin removed - was overview-only

        // Lock the view for scrolling interactive move.
        if !is_floating {
            // TEAM_021: Use canvas for DND gestures instead of workspace iteration
            for mon in self.monitors_mut() {
                mon.canvas_mut().dnd_scroll_gesture_begin();
            }
        }

        true
    }

    pub fn interactive_move_update(
        &mut self,
        window: &W::Id,
        delta: Point<f64, Logical>,
        output: Output,
        pointer_pos_within_output: Point<f64, Logical>,
    ) -> bool {
        let Some(state) = self.interactive_move.take() else {
            return false;
        };

        match state {
            InteractiveMoveState::Starting {
                window_id,
                mut pointer_delta,
                pointer_ratio_within_window,
            } => {
                if window_id != *window {
                    self.interactive_move = Some(InteractiveMoveState::Starting {
                        window_id,
                        pointer_delta,
                        pointer_ratio_within_window,
                    });
                    return false;
                }

                // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

                pointer_delta += delta;

                let (cx, cy) = (pointer_delta.x, pointer_delta.y);
                let sq_dist = cx * cx + cy * cy;

                let factor = RubberBand {
                    stiffness: 1.0,
                    limit: 0.5,
                }
                .band(sq_dist / INTERACTIVE_MOVE_START_THRESHOLD);

                let (is_floating, tile, workspace_config) = self
                    .workspaces_mut()
                    .find(|ws| ws.has_window(&window_id))
                    .map(|ws| {
                        let workspace_config = ws.layout_config().map(|c| (ws.id(), c.clone()));
                        (
                            ws.is_floating(&window_id),
                            ws.tiles_mut()
                                .find(|tile| *tile.window().id() == window_id)
                                .unwrap(),
                            workspace_config,
                        )
                    })
                    .unwrap();
                tile.interactive_move_offset = pointer_delta.upscale(factor);

                // Put it back to be able to easily return.
                self.interactive_move = Some(InteractiveMoveState::Starting {
                    window_id: window_id.clone(),
                    pointer_delta,
                    pointer_ratio_within_window,
                });

                if !is_floating && sq_dist < INTERACTIVE_MOVE_START_THRESHOLD {
                    return true;
                }

                let output_config = self
                    .monitors()
                    .find(|mon| mon.output() == &output)
                    .and_then(|mon| mon.layout_config().cloned());

                // If the pointer is currently on the window's own output, then we can animate the
                // window movement from its current (rubberbanded and possibly moved away) position
                // to the pointer. Otherwise, we just teleport it as the layout code is not aware
                // of monitor positions.
                //
                // FIXME: when and if the layout code knows about monitor positions, this will be
                // potentially animatable.
                let mut tile_pos = None;
                if let Some((mon, (ws, ws_geo))) = self.monitors().find_map(|mon| {
                    mon.workspaces_with_render_geo()
                        .find(|(ws, _)| ws.has_window(window))
                        .map(|rv| (mon, rv))
                }) {
                    if mon.output() == &output {
                        let (_, tile_offset, _) = ws
                            .tiles_with_render_positions()
                            .find(|(tile, _, _)| tile.window().id() == window)
                            .unwrap();

                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
                        tile_pos = Some((ws_geo.loc + tile_offset, 1.0));
                    }
                }

                // Clear it before calling remove_window() to avoid running interactive_move_end()
                // in the middle of interactive_move_update() and the confusion that causes.
                self.interactive_move = None;

                // Unset fullscreen before removing the tile. This will restore its size properly,
                // and move it to floating if needed, so we don't have to deal with that here.
                let ws = self
                    .workspaces_mut()
                    .find(|ws| ws.has_window(&window_id))
                    .unwrap();
                ws.set_fullscreen(window, false);
                ws.set_maximized(window, false);

                let RemovedTile {
                    mut tile,
                    width,
                    is_full_width,
                    is_floating,
                } = self.remove_window(window, Transaction::new()).unwrap();

                tile.stop_move_animations();
                tile.interactive_move_offset = Point::from((0., 0.));
                tile.window().output_enter(&output);
                tile.window().set_preferred_scale_transform(
                    output.current_scale(),
                    output.current_transform(),
                );

                let view_size = output_size(&output);
                let scale = output.current_scale().fractional_scale();
                let options = Options::clone(&self.options)
                    .with_merged_layout(output_config.as_ref())
                    .with_merged_layout(workspace_config.as_ref().map(|(_, c)| c))
                    .adjusted_for_scale(scale);
                tile.update_config(view_size, scale, Rc::new(options));

                if is_floating {
                    // TEAM_021: Unlock view using canvas instead of workspace iteration
                    for mon in self.monitors_mut() {
                        mon.canvas_mut().dnd_scroll_gesture_end();
                    }
                } else {
                    // Animate to semitransparent.
                    tile.animate_alpha(
                        1.,
                        INTERACTIVE_MOVE_ALPHA,
                        self.options.animations.window_movement.0,
                    );
                    tile.hold_alpha_animation_after_done();
                }

                let mut data = InteractiveMoveData {
                    tile,
                    output,
                    pointer_pos_within_output,
                    width,
                    is_full_width,
                    is_floating,
                    pointer_ratio_within_window,
                    output_config,
                    workspace_config,
                };

                if let Some((tile_pos, zoom)) = tile_pos {
                    let new_tile_pos = data.tile_render_location(zoom);
                    data.tile
                        .animate_move_from((tile_pos - new_tile_pos).downscale(zoom));
                }

                self.interactive_move = Some(InteractiveMoveState::Moving(data));
            }
            InteractiveMoveState::Moving(mut move_) => {
                if window != move_.tile.window().id() {
                    self.interactive_move = Some(InteractiveMoveState::Moving(move_));
                    return false;
                }

                let mut ws_id = None;
                if let Some(mon) = self.monitor_for_output(&output) {
                    let (insert_ws, _) = mon.insert_position(move_.pointer_pos_within_output);
                    if let InsertWorkspace::Existing(id) = insert_ws {
                        ws_id = Some(id);
                    }
                }

                // If moved over a different workspace, reset the config override.
                let mut update_config = false;
                if let Some((id, _)) = &move_.workspace_config {
                    if Some(*id) != ws_id {
                        move_.workspace_config = None;
                        update_config = true;
                    }
                }

                if output != move_.output {
                    move_.tile.window().output_leave(&move_.output);
                    move_.tile.window().output_enter(&output);
                    move_.tile.window().set_preferred_scale_transform(
                        output.current_scale(),
                        output.current_transform(),
                    );
                    move_.output = output.clone();
                    self.focus_output(&output);

                    move_.output_config = self
                        .monitor_for_output(&output)
                        .and_then(|mon| mon.layout_config().cloned());

                    update_config = true;
                }

                if update_config {
                    let view_size = output_size(&output);
                    let scale = output.current_scale().fractional_scale();
                    let options = Options::clone(&self.options)
                        .with_merged_layout(move_.output_config.as_ref())
                        .with_merged_layout(move_.workspace_config.as_ref().map(|(_, c)| c))
                        .adjusted_for_scale(scale);
                    move_.tile.update_config(view_size, scale, Rc::new(options));
                }

                move_.pointer_pos_within_output = pointer_pos_within_output;

                self.interactive_move = Some(InteractiveMoveState::Moving(move_));
            }
        }

        true
    }

    pub fn interactive_move_end(&mut self, window: &W::Id) {
        let Some(move_) = &self.interactive_move else {
            return;
        };

        let move_ = match move_ {
            InteractiveMoveState::Starting { window_id, .. } => {
                if window_id != window {
                    return;
                }

                let Some(InteractiveMoveState::Starting { window_id, .. }) =
                    self.interactive_move.take()
                else {
                    unreachable!()
                };

                // TEAM_021: Use canvas to find and update window instead of workspace iteration
                for mon in self.monitors_mut() {
                    mon.dnd_scroll_gesture_end();
                    
                    // Try canvas first for window operations
                    if let Some((_row_idx, tile)) = mon.canvas_mut().find_window_mut(&window_id) {
                        let offset = tile.interactive_move_offset;
                        tile.interactive_move_offset = Point::from((0., 0.));
                        tile.animate_move_from(offset);
                        
                        // Unlock view
                        mon.canvas_mut().dnd_scroll_gesture_end();
                        return;
                    }
                }

                // Fallback to workspace iteration for compatibility
                // TEAM_035: Capture is_active before mutable borrow
                let is_active = self.is_active;
                for mon in self.monitors_mut() {
                    let active_ws_idx = mon.active_workspace_idx() as i32;
                    for (ws_idx, ws) in mon.canvas.workspaces_mut() {
                        let is_focused = is_active && ws_idx == active_ws_idx;
                        ws.refresh(is_active, is_focused);
                    }
                }

                for ws in self.workspaces_mut() {
                    if let Some(tile) = ws.tiles_mut().find(|tile| *tile.window().id() == window_id)
                    {
                        let offset = tile.interactive_move_offset;
                        tile.interactive_move_offset = Point::from((0., 0.));
                        tile.animate_move_from(offset);
                    }

                    // Unlock the view on the workspaces, but if the moved window was active,
                    // preserve that.
                    let moved_tile_was_active =
                        ws.active_window().is_some_and(|win| *win.id() == window_id);

                    ws.dnd_scroll_gesture_end();

                    if moved_tile_was_active {
                        ws.activate_window(&window_id);
                    }
                }

                return;
            }
            InteractiveMoveState::Moving(move_) => move_,
        };

        if window != move_.tile.window().id() {
            return;
        }

        let Some(InteractiveMoveState::Moving(mut move_)) = self.interactive_move.take() else {
            unreachable!()
        };

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.dnd_scroll_gesture_end();
        }

        // Unlock the view on the workspaces.
        if !move_.is_floating {
            // TEAM_021: Use canvas for DND gestures instead of workspace iteration
            for mon in self.monitors_mut() {
                mon.canvas_mut().dnd_scroll_gesture_end();
            }

            // Also animate the tile back to opaque.
            move_.tile.animate_alpha(
                INTERACTIVE_MOVE_ALPHA,
                1.,
                self.options.animations.window_movement.0,
            );
        }

        // TEAM_014: Removed overview check (Part 3) - always allow workspace activation
        let allow_to_activate_workspace = true;

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                let (mon, insert_ws, position, offset, zoom) =
                    if let Some(mon) = monitors.iter_mut().find(|mon| mon.output == move_.output) {
                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0

                        let (insert_ws, geo) = mon.insert_position(move_.pointer_pos_within_output);
                        let (position, offset) = match insert_ws {
                            InsertWorkspace::Existing(ws_id) => {
                                // TEAM_035: Extract row from tuple
                                let ws_idx = mon
                                    .canvas.workspaces_mut()
                                    .position(|(_, ws)| ws.id() == ws_id)
                                    .unwrap();

                                let position = if move_.is_floating {
                                    InsertPosition::Floating
                                } else {
                                    let pos_within_workspace =
                                        move_.pointer_pos_within_output - geo.loc;
                                    // TEAM_035: Extract row from tuple
                                    let (_, ws) = mon.canvas.workspaces_mut().nth(ws_idx).unwrap();
                                    ws.scrolling_insert_position(pos_within_workspace)
                                };

                                (position, Some(geo.loc))
                            }
                            InsertWorkspace::NewAt(_) => {
                                let position = if move_.is_floating {
                                    InsertPosition::Floating
                                } else {
                                    InsertPosition::NewColumn(0)
                                };

                                (position, None)
                            }
                        };

                        // TEAM_014: zoom always 1.0 (Part 3)
                        (mon, insert_ws, position, offset, 1.0)
                    } else {
                        let mon = &mut monitors[*active_monitor_idx];
                        // TEAM_014: Removed overview_zoom (Part 3) - always 1.0
                        // No point in trying to use the pointer position on the wrong output.
                        let ws = &mon.canvas.workspaces().nth(0).unwrap().1;
                        let ws_geo = mon.workspaces_render_geo().next().unwrap();

                        let position = if move_.is_floating {
                            InsertPosition::Floating
                        } else {
                            ws.scrolling_insert_position(Point::from((0., 0.)))
                        };

                        let insert_ws = InsertWorkspace::Existing(ws.id());
                        // TEAM_014: zoom always 1.0 (Part 3)
                        (mon, insert_ws, position, Some(ws_geo.loc), 1.0)
                    };

                let win_id = move_.tile.window().id().clone();
                let tile_render_loc = move_.tile_render_location(zoom);

                // TEAM_035: Use canvas.workspaces() instead of mon.workspaces
                let ws_idx = match insert_ws {
                    InsertWorkspace::Existing(ws_id) => mon
                        .canvas
                        .workspaces()
                        .position(|(_, ws)| ws.id() == ws_id)
                        .unwrap(),
                    InsertWorkspace::NewAt(ws_idx) => {
                        if mon.options.layout.empty_workspace_above_first && ws_idx == 0 {
                            // Reuse the top empty workspace.
                            0
                        } else if mon.canvas.workspaces().count() - 1 <= ws_idx {
                            // Reuse the bottom empty workspace.
                            mon.canvas.workspaces().count() - 1
                        } else {
                            // TEAM_035: Use canvas.ensure_row instead of add_workspace_at
                            mon.canvas.ensure_row(ws_idx as i32);
                            ws_idx
                        }
                    }
                };

                match position {
                    InsertPosition::NewColumn(column_idx) => {
                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon.canvas.workspaces().nth(ws_idx).unwrap();
                        let ws_id = ws.id();
                        mon.add_tile(
                            move_.tile,
                            MonitorAddWindowTarget::Workspace {
                                id: ws_id,
                                column_idx: Some(column_idx),
                            },
                            ActivateWindow::Yes,
                            allow_to_activate_workspace,
                            move_.width,
                            move_.is_full_width,
                            false,
                        );
                    }
                    InsertPosition::InColumn(column_idx, _tile_idx) => {
                        // TEAM_035: Use canvas row directly instead of mon.add_tile_to_column
                        // Just add the tile directly to the row
                        if let Some(row) = mon.canvas.row_mut(ws_idx as i32) {
                            row.add_tile(Some(column_idx), move_.tile, true, move_.width, move_.is_full_width);
                        }
                    }
                    InsertPosition::Floating => {
                        let tile_render_loc = move_.tile_render_location(zoom);

                        let mut tile = move_.tile;
                        tile.floating_pos = None;

                        match insert_ws {
                            InsertWorkspace::Existing(_) => {
                                if let Some(offset) = offset {
                                    let pos = (tile_render_loc - offset).downscale(zoom);
                                    // TEAM_035: Extract row from tuple and call method
                                    let (_, ws) = mon.canvas.workspaces().nth(ws_idx).unwrap();
                                    let pos = ws.floating_logical_to_size_frac(pos);
                                    tile.floating_pos = Some(pos);
                                } else {
                                    error!(
                                        "offset unset for inserting a floating tile \
                                         to existing workspace"
                                    );
                                }
                            }
                            InsertWorkspace::NewAt(_) => {
                                // When putting a floating tile on a new workspace, we don't really
                                // have a good pre-existing position.
                            }
                        }

                        // Set the floating size so it takes into account any window resizing that
                        // took place during the move.
                        if let Some(size) = tile.window().expected_size() {
                            tile.floating_window_size = Some(size);
                        }

                        // TEAM_035: Extract row from tuple
                        let (_, ws) = mon.canvas.workspaces().nth(ws_idx).unwrap();
                        let ws_id = ws.id();
                        mon.add_tile(
                            tile,
                            MonitorAddWindowTarget::Workspace {
                                id: ws_id,
                                column_idx: None,
                            },
                            ActivateWindow::Yes,
                            allow_to_activate_workspace,
                            move_.width,
                            move_.is_full_width,
                            true,
                        );
                    }
                }

                // needed because empty_workspace_above_first could have modified the idx
                // TEAM_035: Use tiles_mut() to get mutable reference for animate_move_from
                let (tile, tile_offset, ws_geo) = mon
                    .workspaces_with_render_geo_mut(false)
                    .find_map(|(ws, geo)| {
                        ws.tiles_mut()
                            .find(|tile| tile.window().id() == &win_id)
                            .map(|tile| (tile, Point::from((0.0, 0.0)), geo))
                    })
                    .unwrap();
                let new_tile_render_loc = ws_geo.loc + tile_offset.upscale(zoom);

                tile.animate_move_from((tile_render_loc - new_tile_render_loc).downscale(zoom));
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                // TEAM_024: Use canvas instead of creating workspaces
                // Ensure we have at least the origin row (row 0)
                let row = canvas.ensure_row(0);
                
                // No point in trying to use the pointer position without outputs.
                // TEAM_035: add_tile takes (col_idx, tile, activate, width, is_full_width)
                row.add_tile(
                    None,
                    move_.tile,
                    true,
                    move_.width,
                    move_.is_full_width,
                );
            }
        }
    }

    pub fn interactive_move_is_moving_above_output(&self, output: &Output) -> bool {
        let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move else {
            return false;
        };

        move_.output == *output
    }

    pub fn dnd_update(&mut self, output: Output, pointer_pos_within_output: Point<f64, Logical>) {
        let begin_gesture = self.dnd.is_none();

        self.dnd = Some(DndData {
            output,
            pointer_pos_within_output,
            hold: None,
        });

        if begin_gesture {
            // dnd_scroll_gesture_begin removed - was overview-only

            for ws in self.workspaces_mut() {
                ws.dnd_scroll_gesture_begin();
            }
        }
    }

    pub fn dnd_end(&mut self) {
        if self.dnd.is_none() {
            return;
        }

        self.dnd = None;

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.dnd_scroll_gesture_end();
        }

        // TEAM_021: Use canvas for DND gestures instead of workspace iteration
        for mon in self.monitors_mut() {
            mon.canvas_mut().dnd_scroll_gesture_end();
        }
    }

    pub fn interactive_resize_begin(&mut self, window: W::Id, edges: ResizeEdge) -> bool {
        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(&window) {
                            return ws.interactive_resize_begin(window, edges);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
                    if ws.has_window(&window) {
                        return ws.interactive_resize_begin(window, edges);
                    }
                }
            }
        }

        false
    }

    pub fn interactive_resize_update(
        &mut self,
        window: &W::Id,
        delta: Point<f64, Logical>,
    ) -> bool {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return false;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(window) {
                            return ws.interactive_resize_update(window, delta);
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
                    if ws.has_window(window) {
                        return ws.interactive_resize_update(window, delta);
                    }
                }
            }
        }

        false
    }

    pub fn interactive_resize_end(&mut self, window: &W::Id) {
        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if move_.tile.window().id() == window {
                return;
            }
        }

        match &mut self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                for mon in monitors {
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(window) {
                            ws.interactive_resize_end(Some(window));
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
                    if ws.has_window(window) {
                        ws.interactive_resize_end(Some(window));
                        return;
                    }
                }
            }
        }
    }

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
            let index = monitor.active_workspace_idx();
            (monitor, index)
        };

        monitor.move_workspace_to_idx(old_idx, new_idx);
    }

    // TEAM_012: Renamed from set_workspace_name, simplified to not take reference
    // TEAM_018: Now calls canvas instead of workspace code
    pub fn set_row_name(&mut self, name: String) {
        // TODO(TEAM_018): implement proper duplicate name checking for canvas rows
        // For now, just set the name on the active row
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
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(window) {
                            ws.store_unmap_snapshot_if_empty(renderer, window);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
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
                    for (_, ws) in mon.canvas.workspaces_mut() {
                        if ws.has_window(window) {
                            ws.clear_unmap_snapshot(window);
                            return;
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                for (_, ws) in canvas.workspaces_mut() {
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
                let Some((ws, ws_geo)) = mon.workspace_under(pointer_pos_within_output) else {
                    return;
                };
                let ws_id = ws.id();
                // TEAM_035: Extract row from tuple
                let (_, ws) = mon
                    .canvas
                    .workspaces_mut()
                    .find(|(_, ws)| ws.id() == ws_id)
                    .unwrap();

                let tile_pos = tile_pos - ws_geo.loc;
                ws.start_close_animation_for_tile(renderer, snapshot, tile_size, tile_pos, blocker);
                return;
            }
        }

        // TEAM_033: Destructure tuples from rows_mut()
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

    pub fn render_interactive_move_for_output<'a, R: NiriRenderer + 'a>(
        &'a self,
        renderer: &mut R,
        output: &Output,
        target: RenderTarget,
    ) -> impl Iterator<Item = RescaleRenderElement<TileRenderElement<R>>> + 'a {
        if self.update_render_elements_time != self.clock.now() {
            error!("clock moved between updating render elements and rendering");
        }

        let mut rv = None;

        if let Some(InteractiveMoveState::Moving(move_)) = &self.interactive_move {
            if &move_.output == output {
                let scale = Scale::from(move_.output.current_scale().fractional_scale());
                // Overview mode has been removed, zoom is always 1.0
                let zoom = 1.0;
                let location = move_.tile_render_location(zoom);
                let iter = move_
                    .tile
                    .render(renderer, location, true, target)
                    .map(move |elem| {
                        RescaleRenderElement::from_element(
                            elem,
                            location.to_physical_precise_round(scale),
                            zoom,
                        )
                    });
                rv = Some(iter);
            }
        }

        rv.into_iter().flatten()
    }

    pub fn refresh(&mut self, is_active: bool) {
        let _span = tracy_client::span!("Layout::refresh");

        self.is_active = is_active;

        let mut ongoing_scrolling_dnd = self.dnd.is_some().then_some(true);

        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            let win = move_.tile.window_mut();

            win.set_active_in_column(true);
            win.set_floating(move_.is_floating);
            win.set_activated(true);

            win.set_interactive_resize(None);

            win.set_bounds(output_size(&move_.output).to_i32_round());

            win.send_pending_configure();
            win.refresh();

            ongoing_scrolling_dnd.get_or_insert(!move_.is_floating);
        } else if let Some(InteractiveMoveState::Starting { window_id, .. }) =
            &self.interactive_move
        {
            ongoing_scrolling_dnd.get_or_insert_with(|| {
                let (_, _, ws) = self
                    .workspaces()
                    .find(|(_, _, ws)| ws.has_window(window_id))
                    .unwrap();
                !ws.is_floating(window_id)
            });
        }

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                active_monitor_idx,
                ..
            } => {
                for (idx, mon) in monitors.iter_mut().enumerate() {
                    let is_active = self.is_active
                        && idx == *active_monitor_idx
                        && !matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_)));

                    // DEPRECATED(overview): Removed overview_open checks
                    // Overview is no longer supported, so always end DnD scroll gesture
                    mon.dnd_scroll_gesture_end();

                    // TEAM_043: Refresh all rows in the canvas
                    let active_row_idx = mon.canvas().active_row_idx();
                    let floating_is_active = mon.canvas().floating_is_active;
                    for (row_idx, row) in mon.canvas_mut().workspaces_mut() {
                        let is_focused = is_active && row_idx == active_row_idx && !floating_is_active;
                        row.refresh(is_active, is_focused);
                        row.view_offset_gesture_end(ongoing_scrolling_dnd);
                    }
                    
                    // TEAM_043: Refresh floating space
                    let is_floating_focused = is_active && floating_is_active;
                    mon.canvas_mut().floating.refresh(is_active, is_floating_focused);
                    
                    if let Some(is_scrolling) = ongoing_scrolling_dnd {
                        // Lock or unlock the view for scrolling interactive move.
                        if is_scrolling {
                            // Canvas equivalent: dnd_scroll_gesture_begin on active row
                            if let Some(row) = mon.canvas_mut().active_row_mut() {
                                row.dnd_scroll_gesture_begin();
                            }
                        } else {
                            mon.canvas_mut().dnd_scroll_gesture_end();
                        }
                    }
                }
            }
            MonitorSet::NoOutputs { canvas, .. } => {
                let floating_is_active = canvas.floating_is_active;
                for (_, ws) in canvas.workspaces_mut() {
                    ws.refresh(false, !floating_is_active);
                    ws.view_offset_gesture_end(None);
                }
                // TEAM_043: Refresh floating space
                canvas.floating.refresh(false, floating_is_active);
            }
        }
    }

    pub fn workspaces(
        &self,
    ) -> impl Iterator<Item = (Option<&Monitor<W>>, i32, &crate::layout::row::Row<W>)> + '_ {
        let iter_normal;
        let iter_no_outputs;

        match &self.monitor_set {
            MonitorSet::Normal { monitors, .. } => {
                let it = monitors.iter().flat_map(|mon| {
                    mon.canvas
                        .workspaces()
                        .map(move |(row_idx, row)| (Some(mon), row_idx, row))
                });

                iter_normal = Some(it);
                iter_no_outputs = None;
            }
            MonitorSet::NoOutputs { canvas } => {
                let it = canvas
                    .workspaces()
                    .map(|(row_idx, row)| (None, row_idx, row));

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
                    .flat_map(|mon| mon.canvas.workspaces_mut().map(|(_, row)| row));

                iter_normal = Some(it);
                iter_no_outputs = None;
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_035: Map (i32, &mut Row) to just &mut Row
                let it = canvas.workspaces_mut().map(|(_, row)| row);

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

        let rest = self
            .workspaces()
            .flat_map(|(mon, _, ws)| ws.windows().map(move |win| (mon, win)));

        moving_window.chain(rest)
    }

    pub fn has_window(&self, window: &W::Id) -> bool {
        self.windows().any(|(_, win)| win.id() == window)
    }

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
            WorkspaceId(1),
        );
        
        Self::NoOutputs { canvas }
    }
}

