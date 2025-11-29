# Phase P6: Extract UiOverlays

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low (minimal coupling)  
> **Prerequisite**: Phase P5 complete  
> **Creates**: `UiOverlays` struct

---

## Goal

Group UI overlay state (screenshot UI, hotkey overlay, exit dialog, MRU switcher) into a dedicated `UiOverlays` container that:
- **Owns** all UI overlay state
- **Provides** unified `is_any_open()` checks
- **Simplifies** focus priority logic

---

## Why This Is Low Risk

The UI overlays:
- Have minimal coupling to other state
- Are mostly independent of each other
- Have simple open/close lifecycle
- Already have their own types (`ScreenshotUi`, `HotkeyOverlay`, etc.)

---

## Current State Analysis

### Fields to Move from Niri

```rust
// UI overlays (mod.rs lines ~404-409)
pub screenshot_ui: ScreenshotUi,
pub config_error_notification: ConfigErrorNotification,
pub hotkey_overlay: HotkeyOverlay,
pub exit_confirm_dialog: ExitConfirmDialog,
pub window_mru_ui: WindowMruUi,
pub pending_mru_commit: Option<PendingMruCommit>,

// Related state
pub pick_window: Option<async_channel::Sender<Option<MappedId>>>,
pub pick_color: Option<async_channel::Sender<Option<niri_ipc::PickedColor>>>,
```

---

## Target Architecture

### New File: `src/niri/subsystems/ui.rs`

```rust
//! UI overlays subsystem.
//!
//! Groups all modal UI elements: screenshot UI, hotkey overlay,
//! exit confirm dialog, MRU window switcher, and config error notification.

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
    pub fn new(config: &niri_config::Config) -> Self {
        Self {
            screenshot: ScreenshotUi::new(),
            config_error: ConfigErrorNotification::new(),
            hotkey: HotkeyOverlay::new(config),
            exit_dialog: ExitConfirmDialog::new(),
            mru: WindowMruUi::new(),
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
    pub fn toggle_hotkey_overlay(&mut self) {
        self.hotkey.toggle();
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
```

---

## Work Units

### Unit 1: Add UiOverlays to subsystems/mod.rs

```rust
mod cursor;
mod focus;
mod outputs;
mod streaming;
mod ui;

pub use cursor::CursorSubsystem;
pub use focus::{FocusModel, FocusContext};
pub use outputs::OutputSubsystem;
pub use streaming::StreamingSubsystem;
pub use ui::UiOverlays;
```

---

### Unit 2: Create UiOverlays Struct

Create `src/niri/subsystems/ui.rs` with:
1. Struct definition (fields can be public since they're already public types)
2. Constructor
3. Convenience query methods

**Verify**: `cargo check`

---

### Unit 3: Move Fields from Niri

1. Remove UI overlay fields from `Niri` struct
2. Add `pub ui: UiOverlays` field
3. Update `Niri::new` to create `UiOverlays`

**Verify**: `cargo check` (will fail - access patterns need updating)

---

### Unit 4: Update Access Patterns

```rust
// Before
self.screenshot_ui.is_open()
self.exit_confirm_dialog.open()
self.window_mru_ui.close()
self.config_error_notification.show()

// After
self.ui.screenshot.is_open()
self.ui.exit_dialog.open()
self.ui.mru.close()
self.ui.config_error.show()

// Or use convenience methods
self.ui.is_screenshot_open()
self.ui.open_exit_dialog()
self.ui.close_mru()
self.ui.show_config_error()
```

---

### Unit 5: Simplify Focus Context

The `FocusContext` from Phase P4 becomes cleaner:

```rust
// Before
let ctx = FocusContext {
    exit_dialog_open: self.niri.exit_confirm_dialog.is_open(),
    screenshot_ui_open: self.niri.screenshot_ui.is_open(),
    mru_ui_open: self.niri.window_mru_ui.is_open(),
    // ...
};

// After
let ctx = FocusContext {
    exit_dialog_open: self.niri.ui.is_exit_dialog_open(),
    screenshot_ui_open: self.niri.ui.is_screenshot_open(),
    mru_ui_open: self.niri.ui.is_mru_open(),
    // ...
};
```

---

## Verification Checklist

- [ ] `UiOverlays` struct exists
- [ ] All UI overlay fields removed from `Niri`
- [ ] `Niri.ui: UiOverlays` field added
- [ ] Convenience query methods work
- [ ] All access patterns updated
- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/ui.rs` | +200 lines (new) |
| `src/niri/subsystems/mod.rs` | +3 lines |
| `src/niri/mod.rs` | -10 lines (fields), +2 lines (field) |
| Various files | Updated access patterns |

---

## Benefits Achieved

1. **Unified queries**: `is_any_modal_open()` in one place
2. **Reduced Niri complexity**: 8 fewer fields
3. **Logical grouping**: All UI overlays together
4. **Simpler focus context**: Clear boolean queries

---

## Next Phase

After completing this phase, proceed to [Phase P7: ConfigManager](phase-P7-config-manager.md).
