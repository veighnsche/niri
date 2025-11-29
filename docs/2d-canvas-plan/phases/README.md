# niri/mod.rs Refactor — Master Plan v2

> **Status**: 🔴 **IN PROGRESS**  
> **Goal**: Reduce `mod.rs` from 3554 LOC to <700 LOC  
> **Current**: 3554 LOC | **Target**: ~600 LOC

---

## 🚨 The Problem

Despite having 13 extracted modules, `mod.rs` is still **3554 lines**!

```
Current State:
├── mod.rs         3554 LOC  ← THIS IS THE PROBLEM
├── init.rs         518 LOC  ✓
├── hit_test.rs     428 LOC  ✓
├── render.rs       415 LOC  ✓
├── screenshot.rs   369 LOC  ✓
├── lock.rs         291 LOC  ✓
├── screencast.rs   290 LOC  ✓
├── output.rs       287 LOC  ✓
├── types.rs        260 LOC  ✓
├── frame_callbacks 252 LOC  ✓
├── screencopy.rs   210 LOC  ✓
├── pointer.rs      193 LOC  ✓
├── rules.rs         77 LOC  ✓
└── mru.rs           60 LOC  ✓
                   ─────────
                   7204 LOC total
```

---

## Root Cause Analysis

### Why is mod.rs still huge?

Looking at what's left, we have **TWO categories** of functions:

#### 1. `impl State` methods (~1700 LOC)
These need access to BOTH `backend` AND `niri`:
- Config handling (~350 lines)
- Cursor/focus movement (~300 lines)  
- Keyboard focus (~260 lines)
- DBus message handlers (~200 lines)
- Screenshot UI (~100 lines)

#### 2. `impl Niri` methods (~1300 LOC)
These only need `&mut self` on Niri:
- Output management (~300 lines) - blocked by `OutputState` type
- Render functions (~400 lines) - blocked by `OutputRenderElements` type
- Surface scanout (~230 lines)
- Misc utilities (~100 lines)

#### 3. Type Definitions (~500 LOC)
- `Niri` struct (~237 lines) - stays here
- `OutputState` struct (~52 lines) - **BLOCKS output extraction**
- `State` struct (~5 lines) - stays here
- `OutputRenderElements` macro (~25 lines) - **BLOCKS render extraction**
- Helper types and functions

---

## The Strategy: Unblock → Extract

### Key Insight
Two types are blocking major extractions:
1. **`OutputState`** - Move to `types.rs` → Unblocks output.rs expansion
2. **`OutputRenderElements`** - Move to `render.rs` → Unblocks render() extraction

### Phase Plan

| Phase | Description | Lines Moved | Blocker Resolved |
|-------|-------------|-------------|------------------|
| **P1** | Move OutputState to types.rs | ~52 | Unblocks output.rs |
| **P2** | Move output mgmt to output.rs | ~300 | - |
| **P3** | Move OutputRenderElements to render.rs | ~25 | Unblocks render.rs |
| **P4** | Move render functions to render.rs | ~400 | - |
| **P5** | Create cursor.rs for State cursor methods | ~200 | - |
| **P6** | Create focus.rs for keyboard focus | ~260 | - |
| **P7** | Create config.rs for reload_config | ~350 | - |
| **P8** | Create dbus.rs for message handlers | ~200 | - |
| **P9** | Cleanup: move small utilities | ~100 | - |

**Expected Result**: mod.rs ~600 LOC (structs + core methods only)

---

## Detailed Phases

### [Phase P1: OutputState → types.rs](phase-P1-output-state.md)
- Move `OutputState` struct to `types.rs`
- Move `CLEAR_COLOR_LOCKED` constant
- Update imports throughout codebase
- **Risk**: Low (pure data movement)
- **Time**: ~20 minutes

### [Phase P2: Output Management → output.rs](phase-P2-output-mgmt.md)  
- Move `reposition_outputs()` (~120 lines)
- Move `add_output()` (~95 lines)
- Move `remove_output()` (~85 lines)
- **Risk**: Medium (complex dependencies)
- **Time**: ~45 minutes

### [Phase P3: OutputRenderElements → render.rs](phase-P3-render-types.md)
- Move `niri_render_elements!` macro invocation
- Move `OutputRenderElements` type
- Move `scale_relocate_crop` helper
- **Risk**: Medium (macro complexity)
- **Time**: ~30 minutes

### [Phase P4: Render Functions → render.rs](phase-P4-render-impl.md)
- Move `render()` (~250 lines)
- Move `render_layer()` (~25 lines)  
- Move `redraw()` (~130 lines)
- Move `queue_redraw*` functions
- **Risk**: Medium (many dependencies)
- **Time**: ~1 hour

### [Phase P5: Create cursor.rs](phase-P5-cursor.md)
- Create new `cursor.rs` module
- Move `move_cursor()` and related State methods
- Move `move_cursor_to_rect()`, `move_cursor_to_focused_tile()`
- Move `maybe_warp_cursor_to_focus*()` methods
- **Risk**: Medium (State access patterns)
- **Time**: ~45 minutes

### [Phase P6: Create focus.rs](phase-P6-focus.md)
- Create new `focus.rs` module
- Move `update_keyboard_focus()` (~260 lines - HUGE)
- Move `focus_default_monitor()`, `focus_window()`
- Move `refresh_popup_grab()`
- **Risk**: High (complex focus logic)
- **Time**: ~1 hour

### [Phase P7: Create config.rs](phase-P7-config.md)
- Create new `config.rs` module
- Move `reload_config()` (~300 lines - HUGE)
- Move `reload_output_config()` (~100 lines)
- Move keyboard config methods
- **Risk**: High (many config interactions)
- **Time**: ~1.5 hours

### [Phase P8: Create dbus.rs](phase-P8-dbus.md)
- Create new `dbus.rs` module
- Move `on_pw_msg()`, `on_screen_cast_msg()`
- Move `on_screen_shot_msg()`, `on_introspect_msg()`
- Move `on_login1_msg()`, `on_locale1_msg()`
- **Risk**: Low (isolated handlers)
- **Time**: ~30 minutes

### [Phase P9: Final Cleanup](phase-P9-cleanup.md)
- Move remaining small utilities
- Clean up unused imports
- Verify all modules <500 LOC
- Update documentation
- **Risk**: Low
- **Time**: ~30 minutes

---

## Success Criteria

### Phase Complete ✓
- [ ] mod.rs < 700 LOC
- [ ] Each module < 500 LOC
- [ ] `cargo check` passes
- [ ] All 270 tests pass
- [ ] No circular dependencies

### Final Architecture
```
src/niri/
├── mod.rs (~600)        # Niri + State structs, core initialization
├── types.rs (~350)      # All data types including OutputState
├── output.rs (~600)     # All output management
├── render.rs (~650)     # All rendering including OutputRenderElements
├── cursor.rs (~250)     # Cursor movement (State methods)
├── focus.rs (~300)      # Keyboard focus (State methods)
├── config.rs (~450)     # Config reload (State methods)
├── dbus.rs (~250)       # DBus handlers (State methods)
├── hit_test.rs (~430)   # Hit testing (unchanged)
├── lock.rs (~290)       # Session lock (unchanged)
├── screenshot.rs (~400) # Screenshots (unchanged)
├── screencopy.rs (~210) # Screencopy (unchanged)
├── screencast.rs (~320) # Screencast (unchanged)
├── frame_callbacks (~250) # Frame callbacks (unchanged)
├── pointer.rs (~200)    # Pointer constraints (unchanged)
├── rules.rs (~80)       # Window rules (unchanged)
├── mru.rs (~60)         # MRU switcher (unchanged)
└── init.rs (~520)       # Niri::new (unchanged)
```

---

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo check` | Verify compilation |
| `cargo test` | Run all tests |
| `wc -l src/niri/*.rs` | Check line counts |
| `grep -n "^impl " src/niri/mod.rs` | Find impl blocks |

---

## Team Registration

Before starting a phase:
1. Check `.teams/` for latest team number
2. Create your team file: `.teams/TEAM_XXX_phase_name.md`
3. Follow the phase instructions
4. Update this README with progress
