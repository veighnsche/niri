# TEAM_052: Phase 3/4 Integration Masterplan

## Status: ✅ COMPLETE

## Summary
Created comprehensive masterplan answering all Phase 3/4 integration questions based on full codebase analysis.

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

## Key Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| InsertWorkspace | Keep unchanged | Row.id() provides WorkspaceId |
| Gesture indices | i32 internal, WorkspaceId external | Matches Canvas2D model |
| Row removal | Use Canvas2D.remove_row() | Already implemented |
| Workspace config | Remove, no compatibility | Per TEAM_042 decision |
| Window operations | Delegate to Canvas2D | Single source of truth |
| IPC compatibility | Keep WorkspaceId from Row | External tools work |
| Rollback | None, forward-only | Old code already deleted |

## Critical Insight
**Row already has a stored WorkspaceId** — the migration doesn't need to synthesize IDs:
```rust
// From Row.id() in row/mod.rs:230-232
pub fn id(&self) -> WorkspaceId {
    self.workspace_id  // Stored on Row creation
}
```

This means:
- InsertWorkspace::Existing(ws.id()) already works
- IPC can populate Workspace.id from Row.id().0
- No conversion layer needed between Row and workspace systems

## Files Created/Modified
- `.questions/TEAM_051_phase3_phase4_integration_questions.md` - Complete rewrite with masterplan

## Handoff

### Immediate Actions for Next Team
1. **gestures.rs** (lines 142-147): Replace TODOs with `self.canvas.focus_row(new_idx as i32)`
2. **config.rs** (line 28): Replace TODO with row removal logic (check is_empty && no name)
3. **render.rs** (line 45): No changes needed — Row.id() already provides WorkspaceId

### Phase 4 Tasks
- Replace all TEAM_020 TODOs in mod.rs
- Update variable names from `workspace` to `row`
- Ensure all tests pass (current: 213/268)

---

## Handoff Checklist
- [x] Code compiles (`cargo check`)
- [x] Masterplan document complete
- [x] Team file complete
- [x] Clear next steps documented

*TEAM_052 — Codebase analysis and masterplan complete*
