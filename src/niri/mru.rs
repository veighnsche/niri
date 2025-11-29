//! Most Recently Used (MRU) window management for the Niri compositor.
//!
//! This module handles the Alt-Tab style window switcher and MRU tracking.

use smithay::desktop::Window;

use crate::ui::mru::MruCloseRequest;

use super::Niri;

// =============================================================================
// MRU Methods
// =============================================================================

impl Niri {
    /// Closes the MRU window switcher UI.
    ///
    /// Returns the selected window if one was confirmed.
    pub fn close_mru(&mut self, close_request: MruCloseRequest) -> Option<Window> {
        if !self.ui.mru.is_open() {
            return None;
        }
        self.queue_redraw_all();

        let id = self.ui.mru.close(close_request)?;
        self.find_window_by_id(id)
    }

    /// Cancels the MRU window switcher UI without selecting a window.
    pub fn cancel_mru(&mut self) {
        self.close_mru(MruCloseRequest::Cancel);
    }

    /// Apply a pending MRU commit immediately.
    ///
    /// Called for example on keyboard events that reach the active window, which immediately adds
    /// it to the MRU.
    pub fn mru_apply_keyboard_commit(&mut self) {
        let Some(pending) = self.ui.pending_mru_commit.take() else {
            return;
        };
        self.event_loop.remove(pending.token);

        if let Some(window) = self
            .layout
            .workspaces_mut()
            .flat_map(|ws| ws.tiles_mut().map(|tile| tile.window_mut()))
            .find(|w| w.id() == pending.id)
        {
            window.set_focus_timestamp(pending.stamp);
        }
    }

    /// Queues a redraw for the output showing the MRU window switcher.
    pub fn queue_redraw_mru_output(&mut self) {
        if let Some(output) = self.ui.mru.output().cloned() {
            self.queue_redraw(&output);
        }
    }
}
