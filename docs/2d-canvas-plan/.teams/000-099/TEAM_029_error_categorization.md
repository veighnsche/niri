# TEAM_029: Error Categorization for Batch Fixing

## Status: ✅ COMPLETED

## Team Assignment
- **Team Number**: 029
- **Task**: Categorize all 142 compilation errors for structured batch fixing
- **Previous Team**: TEAM_028 (Implemented Row navigation methods)

## Summary
Categorized all 142 compilation errors into 11 distinct categories that can be batch-fixed by future teams. Updated TODO.md with structured error list and accurate line numbers from actual cargo check output.

## Key Deliverables
1. **TODO.md** updated with comprehensive "COMPILATION ERRORS — BATCH FIX GUIDE" section
2. **11 error categories** with fix patterns, exact line numbers, and difficulty ratings
3. **Recommended fix order** for future teams to tackle errors efficiently

## Error Analysis Summary

| Category | Count | Batch-Fixable? | Estimated Effort |
|----------|-------|----------------|------------------|
| E0308 Type Mismatches | 39 | ✅ Yes | Medium |
| E0026 NoOutputs Pattern | 16 | ✅ Yes - single pattern | Easy |
| E0615 Method Calls | 14 | ✅ Yes - add `()` | Easy |
| E0609 Field Access | 11 | ✅ Yes - `workspaces` → `canvas` | Easy |
| E0599 Missing Methods | ~15 | ⚠️ Partial | Medium-Hard |
| E0027 Missing Canvas Field | 7 | ✅ Yes - add `canvas` to pattern | Easy |
| E0277 Trait Bounds | 7 | ⚠️ Depends | Medium |
| E0061 Argument Count | ~8 | ⚠️ Manual | Medium |
| E0282 Type Annotations | 2 | ⚠️ Manual | Easy |
| E0432 Imports | 2 | ✅ Yes | Easy |
| E0499/E0596 Borrow | 2 | ⚠️ Refactor needed | Hard |

## Files Affected

| File | Error Count | Primary Issues |
|------|-------------|----------------|
| src/layout/mod.rs | 125 | All categories |
| src/layout/row/mod.rs | 26 | Type mismatches, borrow |
| src/layout/monitor/mod.rs | 9 | Missing methods |
| src/layout/canvas/operations.rs | 9 | Type mismatches |
| src/input/mod.rs | 8 | Field access, types |
| src/layout/workspace_types.rs | 7 | Type mismatches |

## Handoff
- [x] Code compiles (`cargo check`) — N/A (analysis only)
- [x] Team file complete
- [x] TODO.md updated with batch-fix instructions

## Handoff Notes for TEAM_030

### Recommended First Steps:
1. **Start with Category 1** (E0026/E0027) — All 23 errors are the same pattern: change `workspaces` to `canvas` in `MonitorSet::NoOutputs` patterns
2. **Then Category 2** (E0615) — Add `()` to `active_workspace_idx` calls (note: some are assignments needing setter)
3. **Then Category 3** (E0609) — Change `mon.workspaces` to `mon.canvas.workspaces()`

### Batch Fix Commands:
```bash
# Verify current error count
cargo check 2>&1 | grep -E "^error\[" | wc -l

# Find all NoOutputs patterns
grep -rn "NoOutputs.*workspaces" src/layout/mod.rs

# Find all active_workspace_idx field accesses
grep -rn "active_workspace_idx[^(]" src/layout/mod.rs
```

### Files to Focus On:
- **src/layout/mod.rs** — 125 error locations (vast majority)
- **src/layout/monitor/mod.rs** — 9 error locations (missing methods)
- **src/layout/row/mod.rs** — 26 error locations (type mismatches)

---
*TEAM_029 completed error categorization — 142 errors organized into 11 batch-fixable categories*
