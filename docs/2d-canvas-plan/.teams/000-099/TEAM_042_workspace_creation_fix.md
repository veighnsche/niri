# TEAM_042: Architecture Questionnaire & Phase Reorganization

## Status: ✅ COMPLETE

## Tasks Completed

### 1. Debugging Test Hang ✅
- Fixed named workspaces not created at startup
- Fixed empty named rows being deleted
- Fixed `resolve_add_window_target` stub
- Identified architectural mismatch (old workspace vs new row semantics)

### 2. Architecture Questionnaire ✅
Created comprehensive questionnaire in `.questions/TEAM_042_workspace_vs_canvas_architecture.md`:
- Q1: Row naming (any row, unique per output)
- Q2: Row lifecycle (empty unnamed deleted, named persist)
- Q3: Active row (follows focus, default row 0)
- Q4: Window placement (`open-on-row` replaces `open-on-workspace`)
- Q5: Row IDs (global counter, not persisted)
- Q6: Config syntax (remove `workspace` immediately)
- Q7: Camera bookmarks (saved positions)
- Q8: Legacy tests (update to row semantics)

### 3. Phase Reorganization ✅
Archived old phases and created new ones:
- Phase 1: Config Migration (workspace → row)
- Phase 2: Row System (lifecycle, naming)
- Phase 3: Row Spanning
- Phase 4: Camera System
- Phase 5: Camera Bookmarks
- Phase 6: Navigation & Polish

### 4. TODO.md Overhaul ✅
- Extracted all 84 TODOs from codebase
- Organized by file and priority
- Linked TODOs to phases
- Added blocking animation bug section
- Archived completed work

## Key Decisions Made (by USER)
- Remove `workspace` syntax immediately (no deprecation)
- Row 0 is origin, always exists, can be named
- Rows indexed ..., -2, -1, 0, 1, 2, ... (can go negative)
- Window's row determined by top-left corner
- `Mod+1/2/3` for camera bookmarks, not rows

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) - 67 failures (animation bug)
- [x] Team file complete
- [x] TODO.md updated
- [x] Phases reorganized
- [x] Questionnaire resolved
