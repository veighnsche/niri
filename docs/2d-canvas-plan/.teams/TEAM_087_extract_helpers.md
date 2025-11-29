# TEAM_087: Extract Helper Functions

## Status: ✅ Complete

## Task
Phase I1.5: Extract helper predicates and utilities into `src/input/helpers.rs`.

## Functions to Extract
From `mod.rs` (lines 2074-2225):
- `should_activate_monitors<I>` - Event predicate
- `should_hide_hotkey_overlay<I>` - Event predicate
- `should_hide_exit_confirm_dialog<I>` - Event predicate
- `should_notify_activity<I>` - Event predicate
- `should_reset_pointer_inactivity_timer<I>` - Event predicate
- `allowed_when_locked` - Action predicate
- `allowed_during_screenshot` - Action predicate
- `hardcoded_overview_bind` - Hardcoded binds

## Progress
- [x] Create `src/input/helpers.rs`
- [x] Move helper functions
- [x] Update `mod.rs` imports
- [x] Verify compilation
- [x] Clean up unused imports in mod.rs

## Handoff
- [x] Code compiles (`cargo check`) - 0 warnings
- [x] Tests pass (`cargo test`) - 278 tests passed
- [x] Team file complete
