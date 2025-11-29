//! UI overlays subsystem.
//!
//! Groups all modal UI elements: screenshot UI, hotkey overlay,
//! exit confirm dialog, MRU window switcher, and config error notification.

use std::rc::Rc;
use std::cell::RefCell;

use crate::ui::config_error_notification::ConfigErrorNotification;
use crate::ui::exit_confirm_dialog::ExitConfirmDialog;
use crate::ui::hotkey_overlay::HotkeyOverlay;
use crate::ui::mru::WindowMruUi;
use crate::ui::screenshot_ui::ScreenshotUi;
use crate::window::mapped::MappedId;

use super::super::types::PendingMruCommit;

/// UI overlays subsystem.
///
/// Groups all modal UI elements that can appear over the compositor,
/// including screenshot selection, hotkey overlay, exit confirmation,
/// and the window MRU switcher.
pub struct UiOverlays {
    /// Screenshot selection UI.
    pub screenshot: ScreenshotUi,
    
    /// Config error notification banner.
    pub config_error: ConfigErrorNotification,
    
    /// Hotkey overlay (shows available bindings).
    pub hotkey: HotkeyOverlay,
    
    /// Exit confirmation dialog.
    pub exit_dialog: ExitConfirmDialog,
    
    /// Window MRU (most recently used) switcher UI.
    pub mru: WindowMruUi,
    
    /// Pending MRU commit (for focus debouncing).
    pub pending_mru_commit: Option<PendingMruCommit>,
    
    /// Channel for window picker results.
    pub pick_window: Option<async_channel::Sender<Option<MappedId>>>,
    
    /// Channel for color picker results.
    pub pick_color: Option<async_channel::Sender<Option<niri_ipc::PickedColor>>>,
}

impl UiOverlays {
    /// Creates a new UI overlays container.
    pub fn new(config: &niri_config::Config, animation_clock: &crate::animation::Clock, config_rc: &Rc<RefCell<niri_config::Config>>) -> Self {
        Self {
            screenshot: ScreenshotUi::new(animation_clock.clone(), config_rc.clone()),
            config_error: ConfigErrorNotification::new(animation_clock.clone(), config_rc.clone()),
            hotkey: HotkeyOverlay::new(config_rc.clone(), config.clone()),
            exit_dialog: ExitConfirmDialog::new(animation_clock.clone(), config_rc.clone()),
            mru: WindowMruUi::new(config_rc.clone()),
            pending_mru_commit: None,
            pick_window: None,
            pick_color: None,
        }
    }
    
    // =========================================================================
    // Modal State Queries
    // =========================================================================
    
    /// Returns whether any modal UI is open.
    ///
    /// Modal UIs take keyboard focus and should block normal input.
    pub fn is_any_modal_open(&self) -> bool {
        self.screenshot.is_open()
            || self.exit_dialog.is_open()
            || self.mru.is_open()
    }
    
    /// Returns whether the screenshot UI is open.
    pub fn is_screenshot_open(&self) -> bool {
        self.screenshot.is_open()
    }
    
    /// Returns whether the exit dialog is open.
    pub fn is_exit_dialog_open(&self) -> bool {
        self.exit_dialog.is_open()
    }
    
    /// Returns whether the MRU switcher is open.
    pub fn is_mru_open(&self) -> bool {
        self.mru.is_open()
    }
    
    // =========================================================================
    // Screenshot UI
    // =========================================================================
    
    /// Opens the screenshot UI.
    pub fn open_screenshot(&mut self, show_pointer: bool) {
        self.screenshot.open(show_pointer);
    }
    
    /// Closes the screenshot UI.
    pub fn close_screenshot(&mut self) {
        self.screenshot.close();
    }
    
    // =========================================================================
    // Exit Dialog
    // =========================================================================
    
    /// Opens the exit confirmation dialog.
    pub fn open_exit_dialog(&mut self) {
        self.exit_dialog.open();
    }
    
    /// Closes the exit confirmation dialog.
    pub fn close_exit_dialog(&mut self) {
        self.exit_dialog.close();
    }
    
    // =========================================================================
    // MRU Switcher
    // =========================================================================
    
    /// Opens the MRU window switcher.
    pub fn open_mru(&mut self) {
        self.mru.open();
    }
    
    /// Closes the MRU window switcher.
    pub fn close_mru(&mut self) {
        self.mru.close();
        self.pending_mru_commit = None;
    }
    
    // =========================================================================
    // Config Error Notification
    // =========================================================================
    
    /// Shows the config error notification.
    pub fn show_config_error(&mut self) {
        self.config_error.show();
    }
    
    /// Hides the config error notification.
    pub fn hide_config_error(&mut self) {
        self.config_error.hide();
    }
    
    // =========================================================================
    // Hotkey Overlay
    // =========================================================================
    
    /// Shows the hotkey overlay.
    pub fn show_hotkey_overlay(&mut self) {
        self.hotkey.show();
    }
    
    /// Hides the hotkey overlay.
    pub fn hide_hotkey_overlay(&mut self) {
        self.hotkey.hide();
    }
    
    /// Toggles the hotkey overlay.
    pub fn toggle_hotkey_overlay(&mut self) -> bool {
        if self.hotkey.is_open() {
            self.hotkey.hide()
        } else {
            self.hotkey.show()
        }
    }
    
    // =========================================================================
    // Pickers
    // =========================================================================
    
    /// Returns whether window picking is active.
    pub fn is_picking_window(&self) -> bool {
        self.pick_window.is_some()
    }
    
    /// Returns whether color picking is active.
    pub fn is_picking_color(&self) -> bool {
        self.pick_color.is_some()
    }
}
