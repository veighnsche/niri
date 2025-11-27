// TEAM_013: Configuration methods extracted from monitor.rs
//!
//! This module contains configuration and output update methods.

use std::rc::Rc;

use crate::layout::monitor::Monitor;
use crate::layout::workspace_types::compute_working_area;
use crate::layout::{LayoutElement, Options};
use crate::utils::output_size;

impl<W: LayoutElement> Monitor<W> {
    // =========================================================================
    // Configuration updates
    // =========================================================================

    pub fn update_config(&mut self, base_options: Rc<Options>) {
        let options =
            Rc::new(Options::clone(&base_options).with_merged_layout(self.layout_config.as_ref()));

        if self.options.layout.empty_workspace_above_first
            != options.layout.empty_workspace_above_first
            && self.canvas.rows().count() > 1
        {
            if options.layout.empty_workspace_above_first {
                self.add_workspace_top();
            } else if self.workspace_switch.is_none() && self.active_workspace_idx() != 0 {
                // TODO: TEAM_024: Implement row removal in canvas
                // self.canvas.remove_row(0);
            }
        }

        for ws in self.canvas.workspaces_mut() {
            ws.update_config(options.clone());
        }

        self.insert_hint_element
            .update_config(options.layout.insert_hint);

        self.base_options = base_options;
        self.options = options;
    }

    pub fn update_layout_config(&mut self, layout_config: Option<niri_config::LayoutPart>) -> bool {
        if self.layout_config == layout_config {
            return false;
        }

        self.layout_config = layout_config;
        self.update_config(self.base_options.clone());

        true
    }

    pub fn update_shaders(&mut self) {
        for ws in self.canvas.workspaces_mut() {
            ws.update_shaders();
        }

        self.insert_hint_element.update_shaders();
    }

    // =========================================================================
    // Output size updates
    // =========================================================================

    pub fn update_output_size(&mut self) {
        self.scale = self.output.current_scale();
        self.view_size = output_size(&self.output);
        self.working_area = compute_working_area(&self.output);

        for ws in self.canvas.workspaces_mut() {
            ws.update_output_size();
        }
    }
}
