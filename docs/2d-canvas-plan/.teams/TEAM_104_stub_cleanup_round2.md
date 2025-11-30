# TEAM_104: Stub Cleanup Round 2

## Status: COMPLETED ✅

## Objective
Fix remaining stub implementations and clean up dead code identified after TEAM_103's initial cleanup.

## Fixes Applied

### 1. `Row::refresh()` - extra_size calculation
**Location**: `src/layout/row/mod.rs:330`
**Problem**: Hardcoded `Size::new(0.0, 0.0)` instead of calculating tab indicator size
**Fix**: Use `col.extra_size()` which properly calculates tab indicator size in tabbed mode

### 2. `Row::activate_window_without_raising()`
**Location**: `src/layout/row/navigation.rs:35-50`
**Problem**: Stub returning `false` without activating the window
**Fix**: Implemented proper activation - for tiled rows this is equivalent to `activate_window()` since rows don't have stacking order

### 3. `Row::descendants_added()`
**Location**: `src/layout/row/mod.rs:1163-1166`
**Problem**: Stub returning `false` always
**Fix**: Returns `true` if window exists in any column (for tiled windows, no stacking order changes needed unlike floating)

### 4. Dead Code Removed

| Method | Location | Reason |
|--------|----------|--------|
| `Row::current_output()` | `row/mod.rs` | Never called - rows get output from monitor/canvas |
| `Row::make_tile()` | `row/mod.rs` | Never called - tile creation via Canvas2D::make_tile() |
| `Monitor::previous_workspace_idx()` | `monitor/mod.rs` | Never called - Canvas2D uses camera bookmarks |
| `Output` import | `row/mod.rs:53` | Unused after removing current_output() |

### 5. Comment Cleanup

| Method | Change |
|--------|--------|
| `Row::update_layout_config()` | Clarified as intentional no-op (rows inherit from monitor) |
| `Row::has_windows_or_name()` | Removed "Stub implementation" - it's actually implemented |
| `Row::active_window_mut()` | Removed "Stub implementation" - it's actually implemented |
| `Row::active_tile_visual_rectangle()` | Removed "Stub implementation" - it's actually implemented |

## Files Modified

- `src/layout/row/mod.rs` - extra_size fix, dead code removal
- `src/layout/row/navigation.rs` - activate_window_without_raising implementation
- `src/layout/row/state.rs` - comment cleanup
- `src/layout/monitor/mod.rs` - dead code removal

## Test Results

- **All 278 tests pass** ✅
- **No warnings** (fixed unused import)

## Handoff Checklist

- [x] Code compiles (`cargo check`)
- [x] Tests pass (`cargo test`)
- [x] Team file complete
- [x] No remaining stub implementations in layout code

## Code Comments Added

All changes marked with `// TEAM_104:` where applicable.
