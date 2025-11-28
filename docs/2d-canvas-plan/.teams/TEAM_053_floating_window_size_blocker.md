# TEAM_053: Floating Window Size Preservation Blocker

## Status: BLOCKED - Requires smithay/viewport expertise

## Summary
Investigated the failing `unfocus_preserves_current_size` test. The test expects floating windows to preserve their 200×200 size after unfocus, but the window stays at 100×100.

## Root Cause Identified
The test uses `viewport.set_destination(200, 200)` to resize the window, but **smithay's `window.geometry().size` does not reflect viewport destination** - it only returns buffer geometry (100×100).

### Test Flow
1. Client calls `viewport.set_destination(200, 200)`
2. Client calls `ack_last_and_commit()`
3. `FloatingSpace::refresh()` runs
4. `refresh()` reads `tile.window().size()` → returns 100×100 (buffer size, not viewport destination)
5. `refresh()` sends configure with 100×100
6. Window stays 100×100 instead of 200×200

## Attempted Solutions
1. ✅ Added `floating_window_size` updates in `update_window()` - but `update_window()` never called during test
2. ✅ Modified `refresh()` to preserve `expected_size()` - returns wrong size
3. ✅ Captured `tile.window().size()` before mutable borrow - still 100×100
4. ✅ Skipped configure sends entirely - breaks test expectations
5. ❌ Read viewport destination directly - **couldn't find smithay API**

## Blocker
**Cannot find smithay API to read viewport destination from surface state.**

The fix requires either:
1. Finding smithay's API to read `wp_viewport.set_destination()` value
2. Understanding why `window.geometry().size` doesn't include viewport scaling
3. Test infrastructure change if viewport is supposed to update geometry

## Files Modified (Cleaned Up)
- `src/layout/floating.rs` - Reverted to original `refresh()` implementation
- `src/tests/floating.rs` - Removed debug logging
- `src/layout/mod.rs` - Removed debug logging

## Handoff
- [ ] Code compiles (`cargo check`) ✅
- [ ] Tests pass (`cargo test`) - This specific test still fails
- [ ] Team file complete ✅

## Next Steps for Future Team
1. Search smithay source/docs for viewport destination reading
2. Check if `Window::geometry()` should include viewport - may be smithay bug
3. Consider if test uses wrong resize mechanism (commit() vs ack_last_and_commit())
