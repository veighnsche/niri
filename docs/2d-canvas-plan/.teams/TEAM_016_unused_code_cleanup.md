# TEAM_016: Unused Code Cleanup

## Status: COMPLETED

## Team Assignment
- **Team Number**: 016
- **Task**: Clean up unused variables and dead code after overview removal
- **Previous Team**: TEAM_015 (Overview Removal Complete)

## Starting Point
- Overview mode completely removed by TEAM_015
- All 284 tests pass
- All 91 golden tests pass
- Code compiles with warnings about unused variables and dead code

## Work Plan

Clean up remaining compiler warnings after overview removal:

1. Fix unused variables in `src/input/mod.rs`:
   - `was_inside_hot_corner` (2 occurrences)
   - `is_overview_open` 
   - `uninverted_delta_y`
   - `pos_within_output`, `output`
   - `matched_narrow`
   - `ws_id`, `window`
   - `repeat` (unnecessary mut)

2. Remove unused functions:
   - `grab_allows_hot_corner` in src/input/mod.rs
   - `animate_view_offset` and `animate_view_offset_to_column_centered` in src/layout/row/view_offset.rs

3. Clean up unused test fields in src/layout/tests.rs:
   - `ws_name` in SetRowName/UnsetRowName
   - `delta`, `timestamp` in OverviewGestureUpdate

## Progress

### Step 1: Fix unused variables in input/mod.rs ✅
- [x] Fixed `was_inside_hot_corner` variables (prefixed with underscore)
- [x] Fixed `is_overview_open` variable (prefixed with underscore)
- [x] Fixed `uninverted_delta_y` variable (prefixed with underscore)
- [x] Fixed other unused variables in pointer handling
- [x] Removed unnecessary `mut` from `repeat` variable
- [x] Removed unused `PointerGrab` import

### Step 2: Remove unused functions ✅
- [x] Removed `grab_allows_hot_corner` function (26 lines)
- [x] Removed unused `animate_view_offset` method
- [x] Removed unused `animate_view_offset_to_column_centered` method

### Step 3: Clean up test code ✅
- [x] Prefixed unused fields in test Op variants with underscore
- [x] Fixed all pattern matching references to renamed fields
- [x] Updated all test construction code to use new field names

## Changes Made

### Files Modified
- `src/input/mod.rs`: Fixed unused variables, removed unused function and import
- `src/layout/row/view_offset.rs`: Removed 2 unused animation methods
- `src/layout/tests.rs`: Fixed unused field warnings in test Op variants

### Files Deleted
- No files deleted, only unused code removed

## Handoff
- [x] Code compiles (`cargo check`) - no warnings
- [x] Tests pass (`cargo test --lib`) - 284 tests passed
- [x] Golden tests pass (`cargo test layout::tests::golden`) - 91 tests passed
- [x] Team file complete

## Summary

**TEAM_016 successfully completed the cleanup of all unused code and compiler warnings after the overview removal by TEAM_015.** All cleanup tasks have been completed:

1. ✅ Fixed all unused variables in src/input/mod.rs (8 variables)
2. ✅ Removed unused functions (grab_allows_hot_corner, animate_view_offset methods)
3. ✅ Cleaned up unused test fields in src/layout/tests.rs
4. ✅ Removed unused imports

**Verification Results:**
- Code compiles cleanly with no warnings or errors
- All 284 unit tests pass
- All 91 golden snapshot tests pass (no layout regressions)
- All compiler warnings and dead code eliminated

The niri codebase is now in a clean, warning-free state after the complete removal of overview mode.

## Notes for Next Team

The overview removal and cleanup is now fully complete. The codebase compiles cleanly with no warnings and all tests pass. Ready for the next phase of the 2D canvas refactor.
