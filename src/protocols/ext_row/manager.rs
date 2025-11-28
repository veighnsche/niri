//! ext-row manager implementation

use std::collections::HashMap;

use smithay::reexports::wayland_server::backend::ClientId;
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Dispatch, GlobalDispatch, Resource};
use tracing::info;

use super::ext_row_group_v1;
use super::ext_row_handle_v1;
use super::ext_row_manager_v1::{
    self, ExtRowManagerV1, Request, RowState,
};
use super::{Action, BookmarkId, CameraBookmark, CameraState, ExtRowHandler, RowData, RowGeometry};

/// Global data for the ext-row manager protocol.
pub struct ExtRowGlobalData {
    pub version: u32,
}

impl ExtRowGlobalData {
    pub fn new() -> Self {
        Self { version: 1 }
    }
}

/// Refresh the ext-row protocol state.
pub fn refresh<D>(state: &mut D)
where
    D: ExtRowHandler + Dispatch<ExtRowManagerV1, ()>,
    D: Dispatch<ext_row_handle_v1::ExtRowHandleV1, ExtRowManagerV1>,
    D: Dispatch<ext_row_group_v1::ExtRowGroupV1, ExtRowManagerV1>,
    D: GlobalDispatch<ExtRowManagerV1, ExtRowGlobalData>,
    D: 'static,
{
    let _span = tracy_client::span!("ext_row::refresh");

    let protocol_state = state.ext_row_manager_state();

    // Remove rows that no longer exist.
    let mut seen_rows = HashMap::new();
    // TODO: Iterate over actual rows from the layout system
    // for (row_id, row) in state.niri.layout.rows() {
    //     seen_rows.insert(row_id, output.clone());
    // }

    protocol_state.rows.retain(|id, row_data| {
        if seen_rows.contains_key(id) {
            true
        } else {
            info!("Removing row {} from protocol", id);
            // Remove row instances
            for instance in &row_data.instances {
                instance.removed();
            }
            false
        }
    });

    // Update existing rows and create new ones.
    // TODO: Iterate over actual rows from the layout system
    // for (row_id, row) in state.niri.layout.rows() {
    //     refresh_row(protocol_state, row_id, row);
    // }

    // Update row groups (outputs).
    // TODO: Iterate over actual outputs
    // for output in state.niri.layout.outputs() {
    //     refresh_row_group(protocol_state, output);
    // }
}

/// Refresh a specific row's protocol state.
fn refresh_row<D>(
    protocol_state: &mut super::ExtRowManagerState,
    row_id: super::RowId,
    // TODO: Add actual row parameter
) -> bool
where
    D: ExtRowHandler + Dispatch<ExtRowManagerV1, ()>,
    D: Dispatch<ext_row_handle_v1::ExtRowHandleV1, ExtRowManagerV1>,
{
    let mut state = RowState::empty();
    
    // TODO: Determine actual row state from layout system
    // if row.is_active() {
    //     state |= RowState::Active;
    // }
    // if row.is_focused() {
    //     state |= RowState::Focused;
    // }
    // if row.has_urgent_window() {
    //     state |= RowState::Urgent;
    // }

    let geometry = RowGeometry {
        x: 0.0, // TODO: Get from actual row
        y: 0.0, // TODO: Get from actual row
        width: 1920.0, // TODO: Get from actual row
        height: 100.0, // TODO: Get from actual row
    };

    let window_count = 0; // TODO: Get from actual row
    let active_window = None; // TODO: Get from actual row

    match protocol_state.rows.entry(row_id) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            // Existing row, check if anything changed.
            let row_data = entry.get_mut();
            let mut changed = false;

            if row_data.state != state {
                changed = true;
                row_data.state = state;
                for instance in &row_data.instances {
                    instance.state(state);
                }
            }

            // Check geometry changes
            if row_data.geometry.x != geometry.x
                || row_data.geometry.y != geometry.y
                || row_data.geometry.width != geometry.width
                || row_data.geometry.height != geometry.height
            {
                changed = true;
                row_data.geometry = geometry.clone();
                for instance in &row_data.instances {
                    instance.geometry_changed(
                        geometry.x as f64 * 256.0,
                        geometry.y as f64 * 256.0,
                        geometry.width as f64 * 256.0,
                        geometry.height as f64 * 256.0,
                    );
                }
            }

            if row_data.window_count != window_count {
                changed = true;
                row_data.window_count = window_count;
                for instance in &row_data.instances {
                    instance.window_count_changed(window_count as u32);
                }
            }

            if row_data.active_window != active_window {
                changed = true;
                row_data.active_window = active_window.clone();
                for instance in &row_data.instances {
                    if let Some(ref surface) = active_window {
                        instance.active_window_changed(surface);
                    } else {
                        instance.active_window_changed(None);
                    }
                }
            }

            changed
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            // New row, start tracking it.
            let mut data = RowData {
                instances: Vec::new(),
                state,
                geometry,
                window_count,
                active_window,
            };

            // Create row instances for each manager.
            for (manager, _) in &protocol_state.instances {
                let manager: &ExtRowManagerV1 = manager;
                let row = data.add_instance::<D>(manager);
                info!("Created new row instance for row {}", row_id);
                
                // Send initial properties
                row.id(row_id.0);
                // row.name(name); // TODO: Send if named
                row.geometry(
                    geometry.x as f64 * 256.0,
                    geometry.y as f64 * 256.0,
                    geometry.width as f64 * 256.0,
                    geometry.height as f64 * 256.0,
                );
                row.state(state);
                row.window_count(window_count as u32);
                if let Some(ref surface) = active_window {
                    row.active_window_changed(surface);
                }
            }

            entry.insert(data);
            true
        }
    }
}

/// Refresh a row group (output) protocol state.
fn refresh_row_group<D>(
    protocol_state: &mut super::ExtRowManagerState,
    output: &WlOutput,
) -> bool
where
    D: ExtRowHandler + Dispatch<ExtRowManagerV1, ()>,
    D: Dispatch<ext_row_group_v1::ExtRowGroupV1, ExtRowManagerV1>,
{
    if protocol_state.row_groups.contains_key(output) {
        // Existing row group, update camera state.
        let group_data = protocol_state.row_groups.get_mut(output).unwrap();
        
        // TODO: Get actual camera state from layout system
        let camera = CameraState {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };

        if group_data.camera.x != camera.x
            || group_data.camera.y != camera.y
            || group_data.camera.zoom != camera.zoom
        {
            group_data.camera = camera.clone();
            for instance in &group_data.instances {
                instance.camera_changed(
                    camera.x as f64 * 256.0,
                    camera.y as f64 * 256.0,
                    camera.zoom as f64 * 256.0,
                );
            }
            return true;
        }
    } else {
        // New row group, start tracking it.
        let mut data = super::RowGroupData {
            instances: Vec::new(),
            camera: CameraState::default(),
            visible_rows: Vec::new(),
        };

        // Create row group handle for each manager instance.
        for (manager, _) in &protocol_state.instances {
            let manager: &ExtRowManagerV1 = manager;
            let _group = data.add_instance::<D>(manager, output);
        }

        protocol_state.row_groups.insert(output.clone(), data);
        return true;
    }

    false
}

impl RowData {
    /// Add a new instance for this row.
    pub fn add_instance<D>(
        &mut self,
        manager: &ExtRowManagerV1,
    ) -> ext_row_handle_v1::ExtRowHandleV1
    where
        D: Dispatch<ext_row_handle_v1::ExtRowHandleV1, ExtRowManagerV1>,
        D: 'static,
    {
        let row = manager
            .client()
            .create_resource::<ext_row_handle_v1::ExtRowHandleV1, _, D>(
                manager.version(),
                manager.clone(),
                (),
            )
            .unwrap();

        manager.row(&row);
        
        // Set row capabilities
        row.capabilities(
            ext_row_handle_v1::RowCapabilities::Focus as u32
                | ext_row_handle_v1::RowCapabilities::Name as u32,
        );

        self.instances.push(row.clone());
        row
    }
}

impl super::RowGroupData {
    /// Add a new instance for this row group.
    pub fn add_instance<D>(
        &mut self,
        manager: &ExtRowManagerV1,
        output: &WlOutput,
    ) -> ext_row_group_v1::ExtRowGroupV1
    where
        D: Dispatch<ext_row_group_v1::ExtRowGroupV1, ExtRowManagerV1>,
        D: 'static,
    {
        let group = manager
            .client()
            .create_resource::<ext_row_group_v1::ExtRowGroupV1, _, D>(
                manager.version(),
                manager.clone(),
                (),
            )
            .unwrap();

        manager.output_group(&group);
        group.output(output.clone());
        
        // Set group capabilities
        group.capabilities(
            ext_row_group_v1::GroupCapabilities::CameraControl as u32
                | ext_row_group_v1::GroupCapabilities::Bookmarks as u32,
        );

        // Send initial camera state
        group.camera(
            self.camera.x as f64 * 256.0,
            self.camera.y as f64 * 256.0,
            self.camera.zoom as f64 * 256.0,
        );

        self.instances.push(group.clone());
        group
    }
}

impl<D> GlobalDispatch<ExtRowManagerV1, ExtRowGlobalData, D> for super::ExtRowManagerState
where
    D: GlobalDispatch<ExtRowManagerV1, ExtRowGlobalData>,
    D: Dispatch<ExtRowManagerV1, ()>,
    D: Dispatch<ext_row_handle_v1::ExtRowHandleV1, ExtRowManagerV1>,
    D: Dispatch<ext_row_group_v1::ExtRowGroupV1, ExtRowManagerV1>,
    D: ExtRowHandler,
    D: 'static,
{
    fn bind(
        state: &mut D,
        handle: &smithay::reexports::wayland_server::DisplayHandle,
        client: &smithay::reexports::wayland_server::Client,
        resource: New<ExtRowManagerV1>,
        global_data: &ExtRowGlobalData,
        data_init: &mut (),
    ) {
        let manager = resource.instantiate_client(client, data_init).unwrap();
        let protocol_state = state.ext_row_manager_state();
        protocol_state.add_manager::<D>(manager.clone());

        // Send existing rows to the new client.
        for (row_id, row_data) in &protocol_state.rows {
            let row = row_data.add_instance::<D>(&manager);
            
            // Send current state
            row.id(row_id.0);
            row.geometry(
                row_data.geometry.x as f64 * 256.0,
                row_data.geometry.y as f64 * 256.0,
                row_data.geometry.width as f64 * 256.0,
                row_data.geometry.height as f64 * 256.0,
            );
            row.state(row_data.state);
            row.window_count_changed(row_data.window_count as u32);
            if let Some(ref surface) = row_data.active_window {
                row.active_window_changed(surface);
            }
        }

        // Create row groups for all outputs.
        for (output, group_data) in &mut protocol_state.row_groups {
            let group = group_data.add_instance::<D>(&manager, output);
            
            // Send current camera state
            group.camera(
                group_data.camera.x as f64 * 256.0,
                group_data.camera.y as f64 * 256.0,
                group_data.camera.zoom as f64 * 256.0,
            );
        }
    }

    fn can_view(client: Client, global_data: &ExtRowGlobalData) -> bool {
        // TODO: Implement access control if needed
        let _ = (client, global_data);
        true
    }
}

impl<D> Dispatch<ExtRowManagerV1, (), D> for super::ExtRowManagerState
where
    D: Dispatch<ExtRowManagerV1, ()>,
    D: Dispatch<ext_row_handle_v1::ExtRowHandleV1, ExtRowManagerV1>,
    D: Dispatch<ext_row_group_v1::ExtRowGroupV1, ExtRowManagerV1>,
    D: ExtRowHandler,
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &smithay::reexports::wayland_server::Client,
        resource: &ExtRowManagerV1,
        request: Request,
        _data: &(),
        _dhandle: &smithay::reexports::wayland_server::DisplayHandle,
        _data_init: &mut (),
    ) {
        match request {
            Request::Commit => {
                let protocol_state = state.ext_row_manager_state();
                if let Some(actions) = protocol_state.instances.get_mut(resource) {
                    for action in actions.drain(..) {
                        match action {
                            Action::Assign(row_id, output) => {
                                // TODO: Implement row assignment to output
                                info!("Assigning row {:?} to output {:?}", row_id, output);
                            }
                            Action::Focus(row_id) => {
                                state.focus_row(row_id);
                            }
                        }
                    }
                }
            }
            Request::Stop => {
                let protocol_state = state.ext_row_manager_state();
                // Clear all pending actions
                if let Some(actions) = protocol_state.instances.get_mut(resource) {
                    actions.clear();
                }
                
                // Stop sending events to this manager
                // TODO: Implement event filtering
            }
            Request::FocusRow { row_id } => {
                state.focus_row(super::RowId(row_id));
            }
            Request::FocusRowAt { x, y } => {
                let x_f64 = x as f64 / 256.0;
                let y_f64 = y as f64 / 256.0;
                state.focus_row_at(x_f64, y_f64);
            }
            Request::SetRowName { row_id, name } => {
                let row_id = super::RowId(row_id);
                let name_opt = name.map(|s| s.into_string());
                state.set_row_name(row_id, name_opt);
            }
            Request::SetCameraPosition { x, y, zoom } => {
                let x_f64 = x as f64 / 256.0;
                let y_f64 = y as f64 / 256.0;
                let zoom_f64 = zoom as f64 / 256.0;
                state.set_camera_position(x_f64, y_f64, zoom_f64);
            }
            Request::MoveCameraBy { delta_x, delta_y, delta_zoom } => {
                let delta_x_f64 = delta_x as f64 / 256.0;
                let delta_y_f64 = delta_y as f64 / 256.0;
                let delta_zoom_f64 = delta_zoom as f64 / 256.0;
                state.move_camera_by(delta_x_f64, delta_y_f64, delta_zoom_f64);
            }
            Request::CenterCameraOnRow { row_id } => {
                state.center_camera_on_row(super::RowId(row_id));
            }
            Request::CreateBookmark { name, x, y, zoom } => {
                let name_opt = name.map(|s| s.into_string());
                let x_f64 = x as f64 / 256.0;
                let y_f64 = y as f64 / 256.0;
                let zoom_f64 = zoom as f64 / 256.0;
                let bookmark_id = state.create_bookmark(name_opt, x_f64, y_f64, zoom_f64);
                
                // Send bookmark created event
                resource.bookmark_created(
                    bookmark_id,
                    name.as_ref().map(|s| s.as_str()),
                    x,
                    y,
                    zoom,
                );
            }
            Request::RemoveBookmark { bookmark_id } => {
                state.remove_bookmark(bookmark_id);
                resource.bookmark_removed(bookmark_id);
            }
            Request::GotoBookmark { bookmark_id } => {
                state.goto_bookmark(bookmark_id);
            }
            Request::FocusRowUp => {
                state.focus_row_up();
            }
            Request::FocusRowDown => {
                state.focus_row_down();
            }
            Request::Destroy => {
                let protocol_state = state.ext_row_manager_state();
                protocol_state.remove_manager(resource);
            }
            _ => {}
        }
    }

    fn destroyed(
        state: &mut D,
        _client: ClientId,
        resource: &ExtRowManagerV1,
        _data: &(),
    ) {
        let protocol_state = state.ext_row_manager_state();
        protocol_state.remove_manager(resource);
    }
}
