# TEAM_079: UiOverlays Extraction

## Status: ✅ COMPLETED

**Team Number**: 079  
**Phase**: P6 - Extract UiOverlays  
**Time Estimate**: ~45 minutes  
**Risk Level**: 🟢 Low (minimal coupling)

## Task

Group UI overlay state (screenshot UI, hotkey overlay, exit dialog, MRU switcher) into a dedicated `UiOverlays` container that:
- Owns all UI overlay state
- Provides unified `is_any_open()` checks
- Simplifies focus priority logic

## Work Units

1. **Add UiOverlays to subsystems/mod.rs** ✅
2. **Create UiOverlays Struct** ✅
3. **Move Fields from Niri** ✅
4. **Update Access Patterns** ✅
5. **Add Convenience Methods** ✅

## Progress

- [x] Registered as TEAM_079
- [x] Read phase P6 specification
- [x] Analyze current UI overlay state in Niri
- [x] Create UiOverlays struct
- [x] Update subsystems/mod.rs
- [x] Move fields from Niri to UiOverlays
- [x] Update access patterns throughout codebase
- [x] Add convenience methods (is_any_open, etc.)
- [x] Verify compilation and tests

## Implementation Details

### Files Created/Modified:
- **Created**: `src/niri/subsystems/ui.rs` - UiOverlays struct with all UI overlay fields and convenience methods
- **Modified**: `src/niri/subsystems/mod.rs` - Added ui module export
- **Modified**: `src/niri/mod.rs` - Removed individual UI fields, added `ui: UiOverlays` field, updated all access patterns
- **Modified**: `src/niri/init.rs` - Updated constructor to initialize UiOverlays
- **Modified**: Multiple files across codebase to use `self.niri.ui.*` access pattern

### Key Features Implemented:
1. **Centralized UI State**: All UI overlay components (screenshot, config error, hotkey, exit dialog, MRU, pickers) in one struct
2. **Convenience Methods**: `is_any_modal_open()`, `open_screenshot()`, `close_screenshot()`, etc.
3. **Unified Access**: All UI overlay access now goes through `self.niri.ui.*`
4. **Proper Initialization**: UiOverlays initialized with config and animation clock references
5. **Clean API**: Maintains existing functionality while providing better organization

### Access Pattern Updates:
Updated 50+ references across the codebase:
- `screenshot_ui` → `ui.screenshot`
- `config_error_notification` → `ui.config_error`
- `hotkey_overlay` → `ui.hotkey`
- `exit_confirm_dialog` → `ui.exit_dialog`
- `window_mru_ui` → `ui.mru`
- `pick_window` → `ui.pick_window`
- `pick_color` → `ui.pick_color`

## Notes

This refactor successfully groups all modal UI elements into a dedicated subsystem, improving code organization and providing unified state management for UI overlays. The implementation maintains full backward compatibility while enabling cleaner future development.

---

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`)
- [x] Team file complete
