// TEAM_063: Layout output management operations
//!
//! Methods for adding and removing outputs.

use std::mem;

use niri_config::LayoutPart;
use smithay::output::Output;

use super::super::column::Column;
use super::super::{Layout, LayoutElement, Monitor, MonitorSet};

impl<W: LayoutElement> Layout<W> {
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

                if primary.row_switch.is_some() {
                    primary.stop_row_switch();
                    stopped_primary_ws_switch = true;
                }

                // If we stopped a row switch, then we might need to clean up rows.
                // Also if empty_row_above_first is set and there are only 2 rows left,
                // both will be empty and one of them needs to be removed. clean_up_workspaces
                // takes care of this.

                if stopped_primary_ws_switch
                    || (primary.options.layout.empty_row_above_first
                        && primary.canvas.rows().count() == 2)
                {
                    // TEAM_021: Use canvas-first cleanup if possible, fallback to workspace
                    if primary.canvas().has_windows() {
                        primary.canvas_mut().clean_up_workspaces();
                    } else if primary.canvas.rows().count() == 2 {
                        // TEAM_057: Both rows are empty, remove the non-origin row
                        // When empty_row_above_first is set, we keep row 0 (origin) and remove
                        // the other empty row (typically row -1)
                        let non_origin_idx = primary
                            .canvas
                            .rows()
                            .find(|(idx, _)| *idx != 0)
                            .map(|(idx, _)| idx);
                        if let Some(idx) = non_origin_idx {
                            primary.canvas_mut().remove_row(idx);
                        }
                    }
                }

                workspaces.reverse();

                // Create the new monitor with the output
                // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
                let _row_id_to_activate = self.last_active_row_id.remove(&output.name());

                // TEAM_035: Add type annotation to help compiler infer W
                let monitor: Monitor<W> = Monitor::new(
                    output,
                    self.clock.clone(),
                    self.options.clone(),
                    layout_config,
                    self.next_row_id(),
                );
                // DEPRECATED(overview): Removed overview state sync
                monitors.push(monitor);

                MonitorSet::Normal {
                    monitors,
                    primary_idx,
                    active_monitor_idx,
                }
            }
            MonitorSet::NoOutputs { canvas: _ } => {
                // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
                let _row_id_to_activate = self.last_active_row_id.remove(&output.name());

                let monitor = Monitor::new(
                    output,
                    self.clock.clone(),
                    self.options.clone(),
                    layout_config,
                    self.next_row_id(),
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

                // TEAM_033: Store active row ID before consuming monitor
                // TEAM_055: Renamed from active_ws_id to active_row_id
                let output_name = monitor.output_name().clone();
                let active_row_id = monitor
                    .canvas
                    .rows()
                    .nth(monitor.active_row_idx())
                    .map(|(_, ws)| ws.id())
                    .unwrap_or_else(|| crate::layout::row_types::RowId::specific(0));

                // TEAM_055: Renamed from last_active_workspace_id to last_active_row_id
                self.last_active_row_id.insert(output_name, active_row_id);

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
                    // TEAM_033: Destructure tuple from workspaces_mut()
                    for (_, row) in canvas.rows_mut() {
                        row.update_config(view_size, working_area, scale, options.clone());
                    }

                    // TEAM_052: Clean up empty unnamed rows when transitioning to NoOutputs
                    // Only rows with windows or names should remain
                    canvas.cleanup_empty_rows();

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
}
