//! Output management subsystem for TTY backend.
//!
//! Handles IPC output reporting and resume state flags.
//! The actual output configuration logic remains in mod.rs due to
//! tight coupling with device management and niri state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::backend::IpcOutputMap;

/// Output management subsystem.
///
/// OWNS:
/// - IPC output map for external queries
/// - Resume update flags
pub struct OutputManager {
    /// IPC output map shared with external queries.
    ipc_outputs: Arc<Mutex<IpcOutputMap>>,
    /// The output config had changed, but the session is paused.
    update_config_on_resume: bool,
    /// The ignored nodes have changed, but the session is paused.
    update_ignored_nodes_on_resume: bool,
}

impl OutputManager {
    /// Create a new OutputManager.
    pub fn new() -> Self {
        Self {
            ipc_outputs: Arc::new(Mutex::new(HashMap::new())),
            update_config_on_resume: false,
            update_ignored_nodes_on_resume: false,
        }
    }

    // === IPC ===

    /// Get a clone of the IPC outputs map.
    pub fn ipc_outputs(&self) -> Arc<Mutex<IpcOutputMap>> {
        Arc::clone(&self.ipc_outputs)
    }

    /// Update the IPC outputs map with new data.
    pub fn set_ipc_outputs(&self, outputs: IpcOutputMap) {
        let mut guard = self.ipc_outputs.lock().unwrap();
        *guard = outputs;
    }

    // === Resume Flags ===

    /// Mark that output config needs to be updated on resume.
    pub fn mark_config_update_on_resume(&mut self) {
        self.update_config_on_resume = true;
    }

    /// Mark that ignored nodes need to be updated on resume.
    pub fn mark_ignored_nodes_update_on_resume(&mut self) {
        self.update_ignored_nodes_on_resume = true;
    }

    /// Check if output config needs to be updated on resume.
    pub fn needs_config_update_on_resume(&self) -> bool {
        self.update_config_on_resume
    }

    /// Check if ignored nodes need to be updated on resume.
    pub fn needs_ignored_nodes_update_on_resume(&self) -> bool {
        self.update_ignored_nodes_on_resume
    }

    /// Clear the config update on resume flag.
    pub fn clear_config_update_on_resume(&mut self) {
        self.update_config_on_resume = false;
    }

    /// Clear the ignored nodes update on resume flag.
    pub fn clear_ignored_nodes_update_on_resume(&mut self) {
        self.update_ignored_nodes_on_resume = false;
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}
