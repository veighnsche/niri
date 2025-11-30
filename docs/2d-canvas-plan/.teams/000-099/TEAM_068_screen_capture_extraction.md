# TEAM_068: niri.rs Modular Refactor (Phase A6 + A7 + A8)

> **Created**: Nov 29, 2025
> **Status**: ✅ COMPLETE
> **Phases**: A6 (Screen Capture) + A7 (Pointer/Rules) + A8 (Init)
> **Target**: Extract screencopy.rs, screencast.rs, pointer.rs, rules.rs, init.rs from niri/mod.rs

---

## Goal

Complete Phase A6 by extracting:
- `src/niri/screencopy.rs` — zwlr_screencopy protocol methods ✅
- `src/niri/screencast.rs` — PipeWire screencasting (feature-gated) ✅

**Note**: `screenshot.rs` was already completed by a previous team.

Complete Phase A7 by extracting:
- `src/niri/pointer.rs` — Pointer constraints, inactivity, focus-follows-mouse ✅
- `src/niri/rules.rs` — Window and layer rule recomputation ✅

Complete Phase A8 by extracting:
- `src/niri/init.rs` — Niri::new() constructor ✅

---

## Work Units (Phase A6)

### Unit 1: Create screencopy.rs ✅
Methods extracted (~200 LOC):
- `render_for_screencopy_with_damage()`
- `render_for_screencopy_without_damage()`
- `render_for_screencopy_internal()` (static method)
- `remove_screencopy_output()`

### Unit 2: Create screencast.rs ✅
Methods extracted (~220 LOC, all `#[cfg(feature = "xdp-gnome-screencast")]`):
- `render_for_screen_cast()`
- `render_windows_for_screen_cast()`
- `stop_cast()`
- `stop_casts_for_target()` (both feature-gated versions)

### Unit 3: Update mod.rs ✅
- Added `mod screencopy;` and `mod screencast;`
- Removed extracted methods (~360 LOC removed from mod.rs)

### Unit 4: Verify ✅
- `cargo check` passes
- `cargo test` passes (270 tests)

---

## Work Units (Phase A7)

### Unit 5: Create pointer.rs ✅
Methods extracted (~193 LOC):
- `maybe_activate_pointer_constraint()`
- `focus_layer_surface_if_on_demand()`
- `handle_focus_follows_mouse()`
- `reset_pointer_inactivity_timer()`
- `notify_activity()`

### Unit 6: Create rules.rs ✅
Methods extracted (~77 LOC):
- `recompute_window_rules()`
- `recompute_layer_rules()`

---

## Work Units (Phase A8)

### Unit 7: Create init.rs ✅
Methods extracted (~518 LOC):
- `Niri::new()` — Complete constructor with all protocol state initialization

---

## Progress

**Phase A6:**
- [x] Create screencopy.rs
- [x] Create screencast.rs
- [x] Update mod.rs module declarations
- [x] Remove methods from mod.rs

**Phase A7:**
- [x] Create pointer.rs
- [x] Create rules.rs
- [x] Update mod.rs module declarations
- [x] Remove methods from mod.rs

**Phase A8:**
- [x] Create init.rs
- [x] Update mod.rs module declarations
- [x] Remove Niri::new() from mod.rs

**Verification:**
- [x] `cargo check` passes
- [x] `cargo test` passes (270 tests)

---

## Technical Notes

1. **Feature gating**: Screencast methods use `#[cfg(feature = "xdp-gnome-screencast")]` directly on each method to maintain proper visibility from mod.rs

2. **Visibility**: Methods that are only called internally use `pub(super)`, while `stop_casts_for_target` is `pub` as it's called from outside the module

3. **Module structure**: Both files follow the existing pattern of `impl Niri` blocks in submodules

---

## Files Changed

| File | Change |
|------|--------|
| `src/niri/mod.rs` | Added module declarations, reduced from ~6600 to ~3695 LOC |
| `src/niri/screencopy.rs` | **NEW** - ~210 LOC |
| `src/niri/screencast.rs` | **NEW** - ~222 LOC |
| `src/niri/pointer.rs` | **NEW** - ~193 LOC |
| `src/niri/rules.rs` | **NEW** - ~77 LOC |
| `src/niri/init.rs` | **NEW** - ~518 LOC |

---

## Handoff Checklist

- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`) - 270 tests
- [x] Team file complete

---

## Next Phase

**Phases A6, A7, and A8 completed by TEAM_068.**

## 🎉 Part A (Modularization) Complete!

The niri.rs modular refactor is now complete. mod.rs reduced from ~6600 LOC to ~3695 LOC.

**Part B (Features) is now unblocked:**
- [Phase B1](../phases/phase-B1-camera-zoom.md) — Camera zoom system
- [Phase B2](../phases/phase-B2-camera-bookmarks.md) — Camera bookmarks
- [Phase B3](../phases/phase-B3-ipc-migration.md) — IPC/Protocol migration
- [Phase B4](../phases/phase-B4-row-spanning.md) — Row spanning
