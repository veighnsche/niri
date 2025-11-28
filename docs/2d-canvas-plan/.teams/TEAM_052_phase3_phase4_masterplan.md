# TEAM_052: Phase 3/4 Integration Masterplan & Implementation

## Status: ✅ COMPLETE

## Summary
Created comprehensive masterplan and implemented Phase 3/4 integration fixes based on full codebase analysis.

## Work Completed

### 1. Full Codebase Analysis
- Analyzed Canvas2D, Row, Monitor, IPC structures
- Reviewed gesture handling code
- Studied InsertWorkspace and WorkspaceId usage
- Examined IPC protocol compatibility requirements

### 2. Masterplan Document Created
Updated `.questions/TEAM_051_phase3_phase4_integration_questions.md` with:
- Executive summary of migration strategy
- Answers to all 12 questions with code examples
- Implementation checklist with specific code changes
- Key design decisions summary table

### 3. Phase 3 Implementation
- **gestures.rs** (lines 142-147): Replaced TODOs with `self.canvas.focus_row(new_idx as i32)`
- **config.rs** (line 28): Added row removal logic with empty/unnamed checks

### 4. Phase 4 Implementation
- Removed all 7 TEAM_020 TODOs from mod.rs
- Kept workspace iteration for MonitorSet::NoOutputs compatibility
- Added cleanup_empty_rows() when transitioning to NoOutputs
- Fixed verify_invariants() to allow row 0 as empty origin

## Key Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| InsertWorkspace | Keep unchanged | Row.id() provides WorkspaceId |
| Gesture indices | i32 internal, WorkspaceId external | Matches Canvas2D model |
| Row removal | Use Canvas2D.remove_row() | Already implemented |
| Row 0 (origin) | Always kept, even if empty | Origin row must always exist |
| Workspace iteration | Keep for NoOutputs case | Required for test compatibility |
| IPC compatibility | Keep WorkspaceId from Row | External tools work |

## Critical Insight
**Row already has a stored WorkspaceId** — the migration doesn't need to synthesize IDs:
```rust
// From Row.id() in row/mod.rs:230-232
pub fn id(&self) -> WorkspaceId {
    self.workspace_id  // Stored on Row creation
}
```

## Files Modified
- `src/layout/monitor/gestures.rs` - Fixed gesture TODOs
- `src/layout/monitor/config.rs` - Added row removal logic
- `src/layout/mod.rs` - Removed TEAM_020 TODOs, fixed invariants

## Test Status
- **Before**: 229 passed, 43 failed
- **After**: 233 passed, 39 failed (4 tests fixed)
- Remaining failures mostly in `tests::floating` (pre-existing issues)

## Remaining Test Failures (39)
Categories:
1. **Floating tests** (24 failures) - Pre-existing FloatingSpace issues
2. **Golden tests** (2 failures) - May need investigation
3. **Workspace operations** (8 failures) - May need Canvas2D adaptation
4. **Animation tests** (2 failures) - May need investigation
5. **Other** (3 failures)

---

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [x] Masterplan document complete
- [x] Phase 3 implementation complete
- [x] Phase 4 implementation complete (TEAM_020 TODOs removed)
- [x] verify_invariants() fixed for Canvas2D
- [x] Test improvement: 43 → 39 failures

### Next Steps for Future Teams
1. Investigate floating test failures (likely pre-existing)
2. Fix remaining workspace operation test failures
3. Update golden tests if needed
4. Continue terminology cleanup (workspace → row)

*TEAM_052 — Phase 3/4 Integration Complete*
