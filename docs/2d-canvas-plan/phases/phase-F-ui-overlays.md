# Phase F: Extract UiOverlays

> **Status**: ⏳ PENDING  
> **Time Estimate**: ~45 minutes  
> **Risk Level**: 🟢 Low (minimal coupling)  
> **Prerequisite**: Phase E complete  
> **Creates**: `UiOverlays` struct

---

## Goal

Group UI overlay state into a `UiOverlays` container that:
- **Owns** all modal UI state
- **Provides** unified `is_any_open()` checks
- **Simplifies** focus priority logic

---

## Fields to Move from Niri

```rust
// UI overlays (mod.rs lines ~404-414)
pub screenshot_ui: ScreenshotUi,
pub config_error_notification: ConfigErrorNotification,
pub hotkey_overlay: HotkeyOverlay,
pub exit_confirm_dialog: ExitConfirmDialog,
pub window_mru_ui: WindowMruUi,
pub pending_mru_commit: Option<PendingMruCommit>,
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
pub struct UiOverlays {
    /// Screenshot selection UI.
    pub screenshot: ScreenshotUi,
    
    /// Config error notification banner.
    pub config_error: ConfigErrorNotification,
    
    /// Hotkey overlay.
    pub hotkey: HotkeyOverlay,
    
    /// Exit confirmation dialog.
    pub exit_dialog: ExitConfirmDialog,
    
    /// Window MRU switcher.
    pub mru: WindowMruUi,
    
    /// Pending MRU commit.
    pub pending_mru_commit: Option<PendingMruCommit>,
    
    /// Window picker channel.
    pub pick_window: Option<async_channel::Sender<Option<MappedId>>>,
    
    /// Color picker channel.
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
    pub fn is_any_modal_open(&self) -> bool {
        self.screenshot.is_open()
            || self.exit_dialog.is_open()
            || self.mru.is_open()
    }
    
    /// Returns whether screenshot UI is open.
    pub fn is_screenshot_open(&self) -> bool {
        self.screenshot.is_open()
    }
    
    /// Returns whether exit dialog is open.
    pub fn is_exit_dialog_open(&self) -> bool {
        self.exit_dialog.is_open()
    }
    
    /// Returns whether MRU switcher is open.
    pub fn is_mru_open(&self) -> bool {
        self.mru.is_open()
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

### Unit 1: Add to subsystems/mod.rs

### Unit 2: Create UiOverlays struct

### Unit 3: Move fields from Niri

### Unit 4: Update access patterns

```rust
// Before
self.screenshot_ui.is_open()
self.exit_confirm_dialog.open()

// After
self.ui.screenshot.is_open()
self.ui.exit_dialog.open()
// Or use convenience methods:
self.ui.is_screenshot_open()
```

### Unit 5: Verify

---

## Verification Checklist

- [ ] `UiOverlays` struct created
- [ ] All UI overlay fields removed from Niri
- [ ] `is_any_modal_open()` works
- [ ] `cargo check` passes
- [ ] `cargo test` passes

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/subsystems/ui.rs` | **NEW** ~150 lines |
| `src/niri/mod.rs` | -8 fields |

---

## Next Phase

After completing this phase, proceed to [Phase G: InputTracking](phase-G-input-tracking.md).
