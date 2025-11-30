# TEAM_105: Window Activation Bug Fixes

## Summary
Fixed multiple bugs related to window activation via click:
1. Floating windows not being found during activation
2. Wrong row key being used for tiled window activation  
3. Off-by-one error causing wrong column to be activated

## Bugs Fixed

### Bug 1: Floating Windows Not Found
**Symptom:** Clicking on a floating window did not activate it.

**Root Cause:** `Layout::activate_window()` in `focus.rs` only searched through rows, never checking the floating space stored at `mon.canvas.floating`.

**Fix:** Added checks for floating windows first in both `activate_window()` and `activate_window_without_raising()`.

### Bug 2: Wrong Row Key Used
**Symptom:** Clicking on tiled windows did nothing.

**Root Cause:** Code used `.enumerate()` on `rows()` and captured the enumerate index (0, 1, 2...) instead of the actual BTreeMap row key. Then passed wrong index to `row_mut()`.

**Fix:** Removed `.enumerate()` and directly captured the `row_key` from the `(row_key, row)` tuples.

### Bug 3: Off-by-One Column Activation (Main Bug)
**Symptom:** Clicking on window N activated window N-1 (e.g., clicking second window activated first).

**Root Cause:** In `Canvas2D::focus_row()`, the code called `row.focus_column(col_idx)` which expects a **1-based** index (for external API compatibility), but passed a **0-based** `active_column_idx`. This caused `focus_column()` to subtract 1, activating the wrong column.

**Fix:** Changed to use `row.focus_column_idx(col_idx)` which expects 0-based indices.

## Files Modified

### src/layout/layout_impl/focus.rs
- Added floating window checks to `activate_window()` and `activate_window_without_raising()`
- Fixed row key usage (removed `.enumerate()`, use actual BTreeMap key)
- Set `floating_is_active` appropriately when activating tiled vs floating windows

### src/layout/canvas/navigation.rs
- Changed `focus_row()` to use `focus_column_idx()` (0-based) instead of `focus_column()` (1-based)

### src/layout/row/navigation.rs
- Made `focus_column_idx()` `pub(crate)` so Canvas can call it

### src/layout/tests.rs
- Added regression test `activate_window_activates_correct_column`

## Test Added

```rust
#[test]
fn activate_window_activates_correct_column() {
    // Creates 3 windows, verifies activating each one
    // activates the correct column (0, 1, or 2)
}
```

## Verification

```bash
cargo test --lib activate_window_activates_correct_column
# Result: ok. 1 passed

cargo check
# Result: Finished successfully
```

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test --lib activate_window_activates_correct_column`)
- [x] Debug logging removed
- [x] Team file complete
