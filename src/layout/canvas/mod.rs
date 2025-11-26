//! TEAM_006: Canvas2D module for 2D tiling layout.
//!
//! Canvas2D is an infinite 2D surface containing multiple [`Row`]s.
//! It replaces `Workspace` entirely.
//!
//! ## Structure
//!
//! ```text
//! Canvas2D
//! ├── rows: BTreeMap<i32, Row<W>>   // Sparse row storage
//! ├── active_row_idx: i32           // Current row
//! ├── floating: FloatingSpace<W>    // Floating windows
//! ├── camera_x: AnimatedValue       // Horizontal camera position
//! └── camera_y: AnimatedValue       // Vertical camera position
//! ```
//!
//! ## Row Indexing
//!
//! - Row `0` is the origin (where windows open by default)
//! - Negative indices are rows above the origin
//! - Positive indices are rows below the origin
//!
//! ## Module Structure
//!
//! ```text
//! canvas/
//! ├── mod.rs        - Core struct and accessors
//! ├── navigation.rs - Row/column focus navigation
//! ├── operations.rs - Add/remove/find windows
//! ├── render.rs     - Rendering
//! └── floating.rs   - Floating window operations
//! ```

mod floating;
mod navigation;
mod operations;
mod render;

use std::collections::BTreeMap;
use std::rc::Rc;

use smithay::output::Output;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::animated_value::AnimatedValue;
use super::floating::{FloatingSpace, FloatingSpaceRenderElement};
use super::row::{Row, RowRenderElement};
use super::LayoutElement;
use super::Options;
use crate::animation::Clock;
use crate::niri_render_elements;

// TEAM_007: Canvas2D render element type
// TEAM_009: Added FloatingSpace render element
niri_render_elements! {
    Canvas2DRenderElement<R> => {
        Row = RowRenderElement<R>,
        Floating = FloatingSpaceRenderElement<R>,
    }
}

/// A 2D infinite canvas containing multiple rows of columns.
///
/// This is the main layout structure for 2D tiling mode, replacing
/// `Workspace` which only supports a single row.
#[derive(Debug)]
pub struct Canvas2D<W: LayoutElement> {
    /// Rows indexed by row number.
    ///
    /// - `0` = origin row (default for new windows)
    /// - Negative = rows above origin
    /// - Positive = rows below origin
    pub(crate) rows: BTreeMap<i32, Row<W>>,

    /// Currently active row index.
    pub(crate) active_row_idx: i32,

    /// Floating windows layer.
    // TEAM_009: Integrated FloatingSpace
    pub(crate) floating: FloatingSpace<W>,

    /// Whether floating mode is active.
    pub(crate) floating_is_active: bool,

    /// Camera X position (horizontal scroll).
    pub(crate) camera_x: AnimatedValue,

    /// Camera Y position (vertical scroll).
    pub(crate) camera_y: AnimatedValue,

    /// The output this canvas is on.
    output: Option<Output>,

    /// View size for the canvas.
    pub(crate) view_size: Size<f64, Logical>,

    /// Working area for the canvas.
    pub(crate) working_area: Rectangle<f64, Logical>,

    /// Parent area (working area excluding struts).
    pub(crate) parent_area: Rectangle<f64, Logical>,

    /// Scale of the output.
    pub(crate) scale: f64,

    /// Clock for animations.
    pub(crate) clock: Clock,

    /// Layout options.
    pub(crate) options: Rc<Options>,
}

impl<W: LayoutElement> Canvas2D<W> {
    /// Creates a new empty canvas.
    pub fn new(
        output: Option<Output>,
        view_size: Size<f64, Logical>,
        parent_area: Rectangle<f64, Logical>,
        working_area: Rectangle<f64, Logical>,
        scale: f64,
        clock: Clock,
        options: Rc<Options>,
    ) -> Self {
        // Create origin row (row 0)
        let mut rows = BTreeMap::new();
        rows.insert(
            0,
            Row::new(
                0,
                view_size,
                parent_area,
                scale,
                clock.clone(),
                options.clone(),
            ),
        );

        // TEAM_009: Create FloatingSpace
        let floating = FloatingSpace::new(
            view_size,
            working_area,
            scale,
            clock.clone(),
            options.clone(),
        );

        Self {
            rows,
            active_row_idx: 0,
            floating,
            floating_is_active: false,
            camera_x: AnimatedValue::new(0.),
            camera_y: AnimatedValue::new(0.),
            output,
            view_size,
            working_area,
            parent_area,
            scale,
            clock,
            options,
        }
    }

    // =========================================================================
    // Basic Accessors
    // =========================================================================

    /// Returns the output this canvas is on.
    pub fn output(&self) -> Option<&Output> {
        self.output.as_ref()
    }

    /// Returns whether the canvas has any tiled windows.
    pub fn has_tiled_windows(&self) -> bool {
        self.rows.values().any(|row| !row.is_empty())
    }

    /// Returns whether the canvas has any floating windows.
    pub fn has_floating_windows(&self) -> bool {
        self.floating.tiles().next().is_some()
    }

    /// Returns whether floating mode is active.
    pub fn floating_is_active(&self) -> bool {
        self.floating_is_active
    }

    /// Returns a reference to the floating space.
    pub fn floating(&self) -> &FloatingSpace<W> {
        &self.floating
    }

    /// Returns a mutable reference to the floating space.
    pub fn floating_mut(&mut self) -> &mut FloatingSpace<W> {
        &mut self.floating
    }

    /// Returns the active row index.
    pub fn active_row_idx(&self) -> i32 {
        self.active_row_idx
    }

    /// Returns the active row, if any.
    pub fn active_row(&self) -> Option<&Row<W>> {
        self.rows.get(&self.active_row_idx)
    }

    /// Returns a mutable reference to the active row.
    pub fn active_row_mut(&mut self) -> Option<&mut Row<W>> {
        self.rows.get_mut(&self.active_row_idx)
    }

    /// Returns an iterator over all rows.
    pub fn rows(&self) -> impl Iterator<Item = (i32, &Row<W>)> {
        self.rows.iter().map(|(&idx, row)| (idx, row))
    }

    /// Returns the current camera position.
    pub fn camera_position(&self) -> Point<f64, Logical> {
        Point::from((self.camera_x.current(), self.camera_y.current()))
    }

    // =========================================================================
    // Animation
    // =========================================================================

    /// Advances all animations.
    pub fn advance_animations(&mut self) {
        for row in self.rows.values_mut() {
            row.advance_animations();
        }
        // TEAM_009: Added floating animation
        self.floating.advance_animations();
    }

    /// Returns whether any animations are ongoing.
    pub fn are_animations_ongoing(&self) -> bool {
        self.rows.values().any(|row| row.are_animations_ongoing())
            || self.floating.are_animations_ongoing()
            || self.camera_x.is_animation_ongoing()
            || self.camera_y.is_animation_ongoing()
    }
}
