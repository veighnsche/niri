# TEAM_069: mod.rs Continuation Refactor + Master Plan v2

## Status: ✅ Master Plan Created

## Problem Statement
The `src/niri/mod.rs` was 3695 lines with 68 functions despite having 12 extracted modules.
The extracted modules are properly implemented (using `impl Niri` blocks), but the extraction was incomplete.

## Master Plan v2 Created

Created comprehensive 9-phase refactoring plan in `phases/`:

| Phase | Description | Lines | Status |
|-------|-------------|-------|--------|
| P1 | OutputState → types.rs | ~50 | ⏳ PENDING |
| P2 | Output mgmt → output.rs | ~300 | ⏳ PENDING |
| P3 | OutputRenderElements → render.rs | ~30 | ⏳ PENDING |
| P4 | Render functions → render.rs | ~400 | ⏳ PENDING |
| P5 | Create cursor.rs | ~200 | ⏳ PENDING |
| P6 | Create focus.rs | ~300 | ⏳ PENDING |
| P7 | Create config.rs | ~400 | ⏳ PENDING |
| P8 | Create dbus.rs | ~200 | ⏳ PENDING |
| P9 | Final cleanup | ~100 | ⏳ PENDING |

**Target**: mod.rs from 3554 → ~600 LOC

See `phases/README.md` for full details.

## Analysis

### Current State
| File | Lines | Functions |
|------|-------|-----------|
| mod.rs | 3695 | 68 |
| output.rs | 287 | 21 |
| render.rs | 415 | 9 |
| hit_test.rs | 428 | 7 |
| init.rs | 518 | 1 |
| lock.rs | 291 | 6 |
| pointer.rs | 193 | 5 |
| screenshot.rs | 325 | 4 |
| screencopy.rs | 210 | 4 |
| frame_callbacks.rs | 252 | 3 |
| screencast.rs | 222 | 2 |
| rules.rs | 77 | 2 |
| types.rs | 260 | 7 |

### Extraction Plan (by size impact)

#### 1. render.rs - Add ~500 lines
- `render()` (~250 lines) - main render function
- `render_layer()` (~25 lines)
- `redraw()` (~130 lines)
- `queue_redraw_all()` (~5 lines)
- `queue_redraw()` (~4 lines)
- `redraw_queued_outputs()` (~15 lines)

#### 2. New: config.rs - ~350 lines
- `reload_config()` (~300 lines from State)
- `set_xkb_file()` (~15 lines)
- `load_xkb_file()` (~10 lines)
- `set_xkb_config()` (~15 lines)

#### 3. New: focus.rs - ~300 lines
- `update_keyboard_focus()` (~250 lines from State)
- `focus_default_monitor()` (~20 lines from State)
- `focus_window()` (~15 lines from State)

#### 4. screencast.rs - Add ~200 lines
- `on_pw_msg()` (~20 lines)
- `redraw_cast()` (~70 lines)
- `set_dynamic_cast_target()` (~50 lines)
- `on_screen_cast_msg()` (~130 lines from State)
- `refresh_mapped_cast_window_rules()` (~15 lines)
- `refresh_mapped_cast_outputs()` (~45 lines)

#### 5. screenshot.rs - Add ~150 lines
- `screenshot_window()` (~50 lines)
- `open_screenshot_ui()` (~50 lines from State)
- `handle_pick_color()` (~15 lines from State)
- `confirm_screenshot()` (~25 lines from State)
- `on_screen_shot_msg()` + `handle_take_screenshot()` (from State)

#### 6. output.rs - Add ~220 lines
- `reposition_outputs()` (~120 lines)
- `add_output()` (~95 lines)
- `remove_output()` (~75 lines)
- `refresh_on_demand_vrr()` (~30 lines)

#### 7. New: cursor.rs - ~200 lines (cursor movement from State)
- `move_cursor()`
- `move_cursor_to_rect()`
- `move_cursor_to_focused_tile()`
- `maybe_warp_cursor_to_focus()`
- `maybe_warp_cursor_to_focus_centered()`
- `move_cursor_to_output()`
- `refresh_pointer_contents()`
- `update_pointer_contents()`

#### 8. New: mru.rs - ~80 lines
- `confirm_mru()` (from State)
- `close_mru()`
- `cancel_mru()`
- `mru_apply_keyboard_commit()`
- `queue_redraw_mru_output()`

### Expected Outcome
After extraction:
- mod.rs: ~1000 lines (core State struct, Niri struct, small utilities)
- 8+ new/expanded modules with focused responsibilities

## Progress

### Completed ✅
- [x] Analysis of 68 functions
- [x] Extraction plan created
- [x] Created mru.rs (60 lines) - MRU window switcher functions
- [x] Expanded screencast.rs (+68 lines) - refresh_mapped_cast_window_rules, refresh_mapped_cast_outputs
- [x] Expanded screenshot.rs (+43 lines) - screenshot_window

### Results
| File | Before | After | Change |
|------|--------|-------|--------|
| mod.rs | 3695 | 3554 | -141 |
| mru.rs | 0 | 60 | +60 |
| screencast.rs | 222 | 290 | +68 |
| screenshot.rs | 325 | 368 | +43 |

### Remaining Challenges
The remaining functions in mod.rs are tightly coupled with:
1. **OutputRenderElements type** - defined via macro in mod.rs, used by render()
2. **OutputState struct** - defined in mod.rs, used by output management
3. **State struct methods** - `impl State` methods need access to both backend and niri

### Recommendations for Future Teams
1. The render() function (~250 lines) could be moved if OutputRenderElements type is also moved
2. Output management functions could be moved if OutputState is extracted to types.rs
3. Consider creating a cursor.rs for cursor movement functions from State

## Handoff
- [x] Code compiles (`cargo check`)
- [x] All existing tests pass
- [x] Team file updated
- [ ] Further extraction blocked by type coupling
