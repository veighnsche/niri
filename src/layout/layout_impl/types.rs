// TEAM_064: Interactive move and DnD data types extracted from mod.rs
//!
//! This module contains internal data types for interactive move and drag-and-drop operations.

use std::time::Duration;

use smithay::output::Output;
use smithay::utils::{Logical, Point, Scale};

use crate::layout::row_types::RowId;
use crate::layout::tile::Tile;
use crate::layout::{ColumnWidth, LayoutElement};

/// State of an ongoing interactive window move.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum InteractiveMoveState<W: LayoutElement> {
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

/// Data for a window that is being interactively moved.
#[derive(Debug)]
pub(crate) struct InteractiveMoveData<W: LayoutElement> {
    /// The window being moved.
    pub(crate) tile: Tile<W>,
    /// Output where the window is currently located/rendered.
    pub(crate) output: Output,
    /// Current pointer position within output.
    pub(crate) pointer_pos_within_output: Point<f64, Logical>,
    /// Window column width.
    pub(crate) width: ColumnWidth,
    /// Whether the window column was full-width.
    pub(crate) is_full_width: bool,
    /// Whether the window targets the floating layout.
    pub(crate) is_floating: bool,
    /// Pointer location within the visual window geometry as ratio from geometry size.
    ///
    /// This helps the pointer remain inside the window as it resizes.
    pub(crate) pointer_ratio_within_window: (f64, f64),
    /// Config overrides for the output where the window is currently located.
    ///
    /// Cached here to be accessible while an output is removed.
    pub(crate) output_config: Option<niri_config::LayoutPart>,
    /// Config overrides for the workspace where the window is currently located.
    ///
    /// To avoid sudden window changes when starting an interactive move, it will remember the
    /// config overrides for the workspace where the move originated from. As soon as the window
    /// moves over some different workspace though, this override will reset.
    pub(crate) workspace_config: Option<(RowId, niri_config::LayoutPart)>,
}

/// Data for an ongoing drag-and-drop operation.
#[derive(Debug)]
pub struct DndData<W: LayoutElement> {
    /// Output where the pointer is currently located.
    pub(crate) output: Output,
    /// Current pointer position within output.
    pub(crate) pointer_pos_within_output: Point<f64, Logical>,
    /// Ongoing DnD hold to activate something.
    pub(crate) hold: Option<DndHold<W>>,
}

/// Data for a DnD hold operation (holding over a target to activate it).
#[derive(Debug)]
pub(crate) struct DndHold<W: LayoutElement> {
    /// Time when we started holding on the target.
    pub(crate) start_time: Duration,
    /// The target we're holding over.
    pub(crate) target: DndHoldTarget<W::Id>,
}

/// Target of a DnD hold operation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DndHoldTarget<WindowId> {
    /// Holding over a window.
    Window(WindowId),
    /// Holding over a workspace/row.
    Workspace(RowId),
}

impl<W: LayoutElement> InteractiveMoveState<W> {
    /// Returns a reference to the move data if in Moving state.
    pub(crate) fn moving(&self) -> Option<&InteractiveMoveData<W>> {
        match self {
            InteractiveMoveState::Moving(move_) => Some(move_),
            _ => None,
        }
    }

    /// Returns a mutable reference to the move data if in Moving state.
    pub(crate) fn moving_mut(&mut self) -> Option<&mut InteractiveMoveData<W>> {
        match self {
            InteractiveMoveState::Moving(move_) => Some(move_),
            _ => None,
        }
    }
}

impl<W: LayoutElement> InteractiveMoveData<W> {
    /// Computes the render location of the tile being moved.
    pub(crate) fn tile_render_location(&self, zoom: f64) -> Point<f64, Logical> {
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

impl<W: LayoutElement> DndData<W> {
    /// Creates a new DndData.
    pub(crate) fn new(output: Output, pointer_pos_within_output: Point<f64, Logical>) -> Self {
        Self {
            output,
            pointer_pos_within_output,
            hold: None,
        }
    }

    /// Returns the output where the pointer is located.
    pub(crate) fn output(&self) -> &Output {
        &self.output
    }

    /// Returns the pointer position within the output.
    pub(crate) fn pointer_pos_within_output(&self) -> Point<f64, Logical> {
        self.pointer_pos_within_output
    }

    /// Returns a mutable reference to the hold data.
    pub(crate) fn hold_mut(&mut self) -> &mut Option<DndHold<W>> {
        &mut self.hold
    }
}

impl<W: LayoutElement> DndHold<W> {
    /// Creates a new DndHold.
    pub(crate) fn new(start_time: Duration, target: DndHoldTarget<W::Id>) -> Self {
        Self { start_time, target }
    }

    /// Returns the start time.
    pub(crate) fn start_time(&self) -> Duration {
        self.start_time
    }

    /// Returns a reference to the target.
    pub(crate) fn target(&self) -> &DndHoldTarget<W::Id> {
        &self.target
    }
}
