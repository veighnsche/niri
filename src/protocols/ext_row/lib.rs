//! ext-row protocol implementation for Canvas2D
//!
//! This is a complete redesign of the ext-workspace protocol for Canvas2D
//! architecture where rows are horizontal layout strips within an infinite
//! canvas, rather than discrete workspace containers.

pub mod ext_row_group_v1;
pub mod ext_row_handle_v1;
pub mod ext_row_manager_v1;

use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::layout::row_types::RowId;

/// Trait for handling ext-row protocol requests.
pub trait ExtRowHandler {
    /// Get the mutable ext-row manager state.
    fn ext_row_manager_state(&mut self) -> &mut ExtRowManagerState;

    /// Focus a specific row by ID.
    fn focus_row(&mut self, row_id: RowId);

    /// Focus row at canvas coordinates.
    fn focus_row_at(&mut self, x: f64, y: f64);

    /// Set a row's name.
    fn set_row_name(&mut self, row_id: RowId, name: Option<String>);

    /// Set camera position and zoom.
    fn set_camera_position(&mut self, x: f64, y: f64, zoom: f64);

    /// Move camera by relative amounts.
    fn move_camera_by(&mut self, delta_x: f64, delta_y: f64, delta_zoom: f64);

    /// Center camera on a specific row.
    fn center_camera_on_row(&mut self, row_id: RowId);

    /// Create a camera bookmark.
    fn create_bookmark(&mut self, name: Option<String>, x: f64, y: f64, zoom: f64) -> BookmarkId;

    /// Remove a camera bookmark.
    fn remove_bookmark(&mut self, bookmark_id: BookmarkId);

    /// Jump to a bookmark.
    fn goto_bookmark(&mut self, bookmark_id: BookmarkId);

    /// Focus the row above current.
    fn focus_row_up(&mut self);

    /// Focus the row below current.
    fn focus_row_down(&mut self);
}

/// Actions queued for the ext-row manager.
#[derive(Debug, Clone)]
pub enum Action {
    /// Assign a row to an output.
    Assign(RowId, WlOutput),
    /// Focus a specific row.
    Focus(RowId),
}

/// Camera bookmark information.
#[derive(Debug, Clone)]
pub struct CameraBookmark {
    /// Unique bookmark identifier.
    pub id: BookmarkId,
    /// Optional bookmark name.
    pub name: Option<String>,
    /// Camera X position.
    pub x: f64,
    /// Camera Y position.
    pub y: f64,
    /// Camera zoom level.
    pub zoom: f64,
}

/// Unique identifier for camera bookmarks.
pub type BookmarkId = u64;

/// State for the ext-row manager protocol.
pub struct ExtRowManagerState {
    /// Manager instances and their queued actions.
    instances: std::collections::HashMap<
        ext_row_manager_v1::ExtRowManagerV1,
        Vec<Action>,
    >,

    /// Row groups (outputs) and their state.
    row_groups: std::collections::HashMap<WlOutput, RowGroupData>,

    /// Rows and their protocol state.
    rows: std::collections::HashMap<RowId, RowData>,

    /// Camera bookmarks.
    bookmarks: std::collections::HashMap<BookmarkId, CameraBookmark>,
}

/// Data associated with a row group (output).
#[derive(Debug)]
pub struct RowGroupData {
    /// Row group instances.
    instances: Vec<ext_row_group_v1::ExtRowGroupV1>,

    /// Current camera state.
    camera: CameraState,

    /// Currently visible rows in this viewport.
    visible_rows: Vec<RowId>,
}

/// Camera state for an output's viewport.
#[derive(Debug, Clone)]
pub struct CameraState {
    /// Camera X position.
    pub x: f64,
    /// Camera Y position.
    pub y: f64,
    /// Camera zoom level.
    pub zoom: f64,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

/// Data associated with a row in the protocol.
#[derive(Debug)]
pub struct RowData {
    /// Row instances.
    instances: Vec<ext_row_handle_v1::ExtRowHandleV1>,

    /// Current row state.
    state: ext_row_handle_v1::State,

    /// Row geometry in canvas coordinates.
    geometry: RowGeometry,

    /// Number of windows in the row.
    window_count: usize,

    /// Active window ID, if any.
    active_window: Option<wl_surface::WlSurface>,
}

/// Row geometry in canvas coordinates.
#[derive(Debug, Clone)]
pub struct RowGeometry {
    /// Row X position in canvas.
    pub x: f64,
    /// Row Y position in canvas.
    pub y: f64,
    /// Row width.
    pub width: f64,
    /// Row height.
    pub height: f64,
}

impl ExtRowManagerState {
    /// Create a new ext-row manager state.
    pub fn new() -> Self {
        Self {
            instances: std::collections::HashMap::new(),
            row_groups: std::collections::HashMap::new(),
            rows: std::collections::HashMap::new(),
            bookmarks: std::collections::HashMap::new(),
        }
    }

    /// Add a new manager instance.
    pub fn add_manager<D>(&mut self, manager: ext_row_manager_v1::ExtRowManagerV1)
    where
        D: smithay::reexports::wayland_server::Dispatch<
            ext_row_manager_v1::ExtRowManagerV1,
            (),
        >,
    {
        self.instances.insert(manager, Vec::new());
    }

    /// Remove a manager instance.
    pub fn remove_manager(&mut self, manager: &ext_row_manager_v1::ExtRowManagerV1) {
        self.instances.remove(manager);
    }

    /// Get or create a row group for an output.
    pub fn get_or_create_row_group(
        &mut self,
        output: &WlOutput,
    ) -> &mut RowGroupData {
        self.row_groups
            .entry(output.clone())
            .or_insert_with(|| RowGroupData {
                instances: Vec::new(),
                camera: CameraState::default(),
                visible_rows: Vec::new(),
            })
    }

    /// Update row geometry.
    pub fn update_row_geometry(&mut self, row_id: RowId, geometry: RowGeometry) {
        if let Some(row_data) = self.rows.get_mut(&row_id) {
            row_data.geometry = geometry;
        }
    }

    /// Update row state.
    pub fn update_row_state(&mut self, row_id: RowId, state: ext_row_handle_v1::State) {
        if let Some(row_data) = self.rows.get_mut(&row_id) {
            row_data.state = state;
        }
    }

    /// Update camera state for an output.
    pub fn update_camera_state(&mut self, output: &WlOutput, camera: CameraState) {
        if let Some(group_data) = self.row_groups.get_mut(output) {
            group_data.camera = camera;
        }
    }

    /// Add a camera bookmark.
    pub fn add_bookmark(&mut self, bookmark: CameraBookmark) {
        self.bookmarks.insert(bookmark.id, bookmark);
    }

    /// Remove a camera bookmark.
    pub fn remove_bookmark(&mut self, bookmark_id: BookmarkId) -> Option<CameraBookmark> {
        self.bookmarks.remove(&bookmark_id)
    }

    /// Get a bookmark by ID.
    pub fn get_bookmark(&self, bookmark_id: BookmarkId) -> Option<&CameraBookmark> {
        self.bookmarks.get(&bookmark_id)
    }

    /// Get all bookmarks.
    pub fn get_bookmarks(&self) -> impl Iterator<Item = &CameraBookmark> {
        self.bookmarks.values()
    }
}

impl Default for ExtRowManagerState {
    fn default() -> Self {
        Self::new()
    }
}
