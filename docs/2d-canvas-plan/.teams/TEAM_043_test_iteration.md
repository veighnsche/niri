# TEAM_043 — Test Iteration Until All Pass

## Mission
Run all tests (including golden) iteratively, fixing code until 100% pass rate.

## Status: IN PROGRESS

## Progress
- **Start**: 201 passed, 67 failed (75%)
- **Current**: 210 passed, 58 failed (78%)
- **Golden Tests**: ✅ 84/84 pass (maintained)

## Fixes Applied

### 1. Fixed refresh not calling Row::refresh() ✅
- **File**: `src/layout/mod.rs` (lines 4890-4908)
- **Issue**: Windows weren't receiving configure events with Activated state
- **Fix**: Added explicit `row.refresh()` calls in `MonitorSet::Normal` branch

### 2. Fixed width parameter ignored in Monitor::add_window() ✅
- **File**: `src/layout/monitor/mod.rs` (lines 360-384)
- **Issue**: Width was hardcoded to `Proportion(1.0)` instead of using passed parameter
- **Fix**: Use provided width, fallback to `Proportion(0.5)`

### 3. Added floating space refresh ✅
- **File**: `src/layout/mod.rs` (lines 4899-4901, 4922-4923)
- **Issue**: Floating windows weren't being refreshed
- **Fix**: Added `canvas.floating.refresh()` calls in both Normal and NoOutputs branches

### 4. Fixed set_column_width for floating windows ✅
- **File**: `src/layout/mod.rs` (lines 3127-3146)
- **Issue**: `set_column_width` only operated on rows, not floating windows
- **Fix**: Check `floating_is_active` and route to `FloatingSpace::set_window_width`

### 5. Fixed floating set_window_width/height ✅
- **File**: `src/layout/floating.rs` (lines 782-784, 831-833)
- **Issue**: Used `expected_size()` which returns niri's requested size, not client's committed size
- **Fix**: Use `size()` to preserve client's current dimensions

### 6. Fixed set_column_width not updating ColumnData ✅
- **File**: `src/layout/row/mod.rs` (lines 1686-1691, 1724-1728)
- **Issue**: Column width changes weren't updating cached `data[].width`
- **Fix**: Added `self.data[col_idx].update(col)` after width changes

## Remaining Issues

### Animation Tests (~10 failing)
- X positions are off by -100 (view offset issue)
- Example: Expected `x:0, x:100` but getting `x:-100, x:0`
- Root cause: View offset calculation in `animate_view_offset_to_column`

### Floating Tests (~25 failing)
- Size preservation issues when toggling floating
- Some tests still getting wrong height (100 vs 200)

### Fullscreen Tests (~5 failing)
- View offset preservation issues

### Window Opening Tests (~10 failing)
- Workspace targeting issues

### Interactive Move Tests (~8 failing)
- Various move/resize issues

## Files Modified
- `src/layout/mod.rs`
- `src/layout/monitor/mod.rs`
- `src/layout/floating.rs`
- `src/layout/row/mod.rs`
- `docs/2d-canvas-plan/TODO.md`

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) - 210/268 passing (78%)
- [x] Golden tests pass (`cargo insta test`) - 84/84 passing
- [x] Team file complete

## Next Steps for TEAM_044
1. Fix animation test view offset issues
2. Fix remaining floating window size preservation
3. Fix fullscreen view offset preservation
4. Fix window opening workspace targeting
