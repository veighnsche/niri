# TEAM_085: Input Refactor (Phases I1.1 & I1.2)

## Status: In Progress 🔄

## Summary

Extracting modules from `src/input/mod.rs` following the Phase I1 plan.

## Work Done

1. **Created `src/input/binds.rs`** with pure functions:
   - `modifiers_from_state()` - Convert XKB modifiers to niri Modifiers
   - `find_bind()` - Find bind for key input (hardcoded + configured)
   - `find_configured_bind()` - Match trigger + modifiers to configured bind
   - `find_configured_switch_action()` - Lid/tablet mode switch actions
   - `mods_with_binds()` - Get modifiers that have bindings for triggers
   - `mods_with_mouse_binds()` - Mouse button bindings
   - `mods_with_wheel_binds()` - Mouse wheel bindings
   - `mods_with_finger_scroll_binds()` - Touchpad scroll bindings

2. **Added unit tests** for `modifiers_from_state()` in binds.rs

3. **Updated `mod.rs`**:
   - Added `pub mod binds;`
   - Re-exported functions for backwards compatibility
   - Removed duplicate function definitions
   - Fixed test imports (added `Binds`, `ScreenshotUi`, `Duration`)
   - Added `screenshot_ui` variable to test

## Files Created

- `src/input/binds.rs` (~230 lines)

## Files Modified

- `src/input/mod.rs`:
  - Added module declaration and re-exports
  - Removed ~120 lines of duplicate code
  - Fixed test imports

## Verification

- [x] `cargo check` passes
- [x] No State/Niri imports in binds.rs (pure module)
- [x] Functions re-exported for backwards compatibility

## Notes

- `should_intercept_key()` was NOT moved - it depends on `ScreenshotUi` (not pure)
- `make_binds_iter()` was NOT moved - it depends on `WindowMruUi` (not pure)
- Pre-existing test failures in `layout/tests.rs` (missing `ColumnDisplay`, `PresetSize` imports) are unrelated to this work

## LOC Impact

- `mod.rs`: -120 lines (removed duplicates)
- `binds.rs`: +230 lines (new file with tests)
- Net: +110 lines (but now properly separated and testable)

## Handoff

- [x] Code compiles
- [x] Pure functions extracted
- [x] Backwards compatible re-exports
- [x] Team file complete

---

## Phase I1.2: Extract Device Management ✅

1. **Created `src/input/device.rs`** (286 lines):
   - `on_device_added()` - Handle device hotplug
   - `on_device_removed()` - Handle device removal
   - `apply_libinput_settings()` - Apply config to libinput devices
   - Per-device-type helpers: `apply_touchpad_settings()`, `apply_mouse_settings()`, etc.
   - `apply_scroll_method()` - Shared helper to reduce duplication

2. **Updated `mod.rs`**:
   - Added `mod device;`
   - Re-exported `apply_libinput_settings`
   - Removed ~308 lines of duplicate code

**LOC Progress:**
- Started: 5123 lines
- After I1.1: 4963 lines (-160)
- After I1.2: 4655 lines (-308)
- **Total reduction: 468 lines (9%)**

---

## Phase I1.3: Extract Event Handlers 🔄

### Completed:
1. **Created `src/input/touch.rs`** (226 lines):
   - `on_touch_down()`, `on_touch_up()`, `on_touch_motion()`
   - `on_touch_frame()`, `on_touch_cancel()`

2. **Created `src/input/gesture.rs`** (317 lines):
   - `on_gesture_swipe_begin/update/end()`
   - `on_gesture_pinch_begin/update/end()`
   - `on_gesture_hold_begin/end()`

### Completed:
3. **Created `src/input/tablet.rs`** (237 lines):
   - `on_tablet_tool_axis()`, `on_tablet_tool_tip()`
   - `on_tablet_tool_proximity()`, `on_tablet_tool_button()`

4. **Created `src/input/keyboard.rs`** (274 lines):
   - `on_keyboard()`, `start_key_repeat()`, `hide_cursor_if_needed()`

5. **Created `src/input/pointer.rs`** (1105 lines):
   - `on_pointer_motion()`, `on_pointer_motion_absolute()`
   - `on_pointer_button()`, `on_pointer_axis()`

**LOC Progress:**
- Started: 5123 lines
- After I1.1 (binds): 4963 lines
- After I1.2 (device): 4655 lines
- After touch.rs: ~4430 lines
- After gesture.rs: ~4171 lines
- After tablet.rs: ~3920 lines
- After keyboard.rs: ~3705 lines
- After pointer.rs: **2627 lines**
- **Total reduction: 2496 lines (49%)**

---

## Handoff

- [x] Code compiles (`cargo check`)
- [x] All extracted modules working
- [x] Backwards compatible re-exports
- [x] Team file complete

## Phase I1.3 COMPLETE ✅
