// TEAM_063: Layout row management operations
//!
//! Methods for finding and managing rows.

use niri_config::WorkspaceReference;
// TEAM_055: Renamed from Workspace to RowConfig
use niri_config::RowConfig as WorkspaceConfig;

use super::super::{
    Layout, LayoutElement, MonitorSet,
    output_matches_name, row_types::RowId,
};

impl<W: LayoutElement> Layout<W> {
    pub fn find_workspace_by_id(&self, id: RowId) -> Option<(i32, &crate::layout::row::Row<W>)> {
        match &self.monitor_set {
            MonitorSet::Normal { ref monitors, .. } => {
                for mon in monitors {
                    if let Some((row_idx, row)) = mon
                        .canvas
                        .rows()
                        .find(|(_, w)| w.id() == id)
                    {
                        return Some((row_idx, row));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                if let Some((row_idx, row)) =
                    canvas.rows().find(|(_, w)| w.id() == id)
                {
                    return Some((row_idx, row));
                }
            }
        }

        None
    }

    // TEAM_056: Backwards compatibility alias for find_row_by_name
    pub fn find_workspace_by_name(&self, name: &str) -> Option<(i32, &crate::layout::row::Row<W>)> {
        self.find_row_by_name(name)
    }

    // TEAM_055: Renamed from find_workspace_by_name to find_row_by_name
    pub fn find_row_by_name(&self, row_name: &str) -> Option<(i32, &crate::layout::row::Row<W>)> {
        match &self.monitor_set {
            MonitorSet::Normal { ref monitors, .. } => {
                for mon in monitors {
                    if let Some((row_idx, row)) =
                        mon.canvas.rows().find(|(_, w)| {
                            w.name()
                                .is_some_and(|name| name.eq_ignore_ascii_case(row_name))
                        })
                    {
                        return Some((row_idx, row));
                    }
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                if let Some((row_idx, row)) = canvas.rows().find(|(_, w)| {
                    w.name()
                        .is_some_and(|name| name.eq_ignore_ascii_case(row_name))
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
                m.canvas.rows_mut().find(|(idx, _)| *idx == row_idx).map(|(_, row)| row)
            })
        } else {
            // Find the workspace by name or id across all monitors
            for monitor in self.monitors_mut() {
                if let Some((_, row)) = monitor.canvas.rows_mut().find(|(_, row)| {
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

    pub fn unname_workspace_by_id(&mut self, id: RowId) {
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

    // TEAM_055: Renamed from ensure_named_workspace to ensure_named_row
    pub fn ensure_named_row(&mut self, row_config: &WorkspaceConfig) {
        if self.find_row_by_name(&row_config.name.0).is_some() {
            return;
        }

        match &mut self.monitor_set {
            MonitorSet::Normal {
                monitors,
                primary_idx,
                active_monitor_idx,
            } => {
                let mon_idx = row_config
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

                // TEAM_055: Create a new row at position -1 (before existing rows)
                // Original behavior: insert new workspace at position 0, which pushes existing down
                // In BTreeMap: use negative key so it comes first in iteration order
                let insert_key = mon.canvas.rows().map(|(idx, _)| idx).min().unwrap_or(0) - 1;
                let row = mon.canvas.ensure_row(insert_key);
                row.set_name(Some(row_config.name.0.clone()));
                
                // TEAM_055: Clean up empty unnamed rows (like original clean_up_workspaces)
                let active_key = mon.canvas.active_row_idx;
                let rows_to_remove: Vec<i32> = mon.canvas.rows()
                    .filter(|(idx, row)| {
                        *idx != active_key && !row.has_windows() && row.name().is_none()
                    })
                    .map(|(idx, _)| idx)
                    .collect();
                for idx in rows_to_remove {
                    mon.canvas.remove_row(idx);
                }
            }
            MonitorSet::NoOutputs { canvas } => {
                // TEAM_055: Create a new row at position -1 (before existing rows)
                // Original behavior: insert new workspace at position 0
                let insert_key = canvas.rows().map(|(idx, _)| idx).min().unwrap_or(0) - 1;
                let row = canvas.ensure_row(insert_key);
                row.set_name(Some(row_config.name.0.clone()));
                
                // TEAM_055: Clean up empty unnamed rows
                let active_key = canvas.active_row_idx;
                let rows_to_remove: Vec<i32> = canvas.rows()
                    .filter(|(idx, row)| {
                        *idx != active_key && !row.has_windows() && row.name().is_none()
                    })
                    .map(|(idx, _)| idx)
                    .collect();
                for idx in rows_to_remove {
                    canvas.remove_row(idx);
                }
            }
        }
    }
}
