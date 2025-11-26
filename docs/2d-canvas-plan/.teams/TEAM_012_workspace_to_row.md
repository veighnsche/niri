# TEAM_012: Workspace → Row Implementation

## Status: IN PROGRESS

## Team Assignment
- **Team Number**: 012
- **Task**: Implement Phase 1.5.3 Part 2 (Workspace → Row transformation)
- **Previous Team**: TEAM_011 (Planning complete)

## Starting Point
Per TEAM_011 handoff:
- Planning complete with detailed phase documents (Parts 2A-2E)
- Golden tests pass (91 tests)
- 284 tests pass total

## Work Plan

Execute in alphabetical order per plan:
1. Part 2A (Config) — Replace Action variants in niri-config + niri-ipc ✅
2. Part 2B (Input) — Replace handlers in src/input/mod.rs ✅
3. Part 2C (Layout) — Replace methods in src/layout/mod.rs (PENDING)
4. Part 2D (Monitor) — Refactor into modules + row navigation (PENDING)
5. Part 2E (Tests) — Transform/remove workspace tests (PENDING)

## Call Site Analysis Documents

Created comprehensive call site analysis for each part:
- `phases/phase-1.5.3-part2a-callsites.md` - Config/IPC analysis ✅
- `phases/phase-1.5.3-part2b-callsites.md` - Input handlers analysis ✅
- `phases/phase-1.5.3-part2c-callsites.md` - Layout methods analysis (PENDING)
- `phases/phase-1.5.3-part2d-callsites.md` - Monitor methods analysis (PENDING)
- `phases/phase-1.5.3-part2e-callsites.md` - Tests analysis (PENDING)

## Scope Summary

| Part | File | Workspace References | Status |
|------|------|---------------------|--------|
| 2A | niri-ipc/src/lib.rs | ~30 action variants | ✅ DONE |
| 2A | niri-config/src/binds.rs | ~30 action variants | ✅ DONE |
| 2B | src/input/mod.rs | ~25 action handlers | ✅ DONE |
| 2B | src/ui/hotkey_overlay.rs | ~12 references | ✅ DONE |
| 2C | src/layout/mod.rs | ~600 references | PENDING |
| 2D | src/layout/monitor.rs | ~490 references | PENDING |
| 2E | src/layout/tests.rs | ~143 references | PENDING |

## Progress

### Part 2A: Config Actions ✅ COMPLETE
- [x] Replace in niri-config/src/binds.rs
  - Replaced all Workspace action variants with Row equivalents
  - Removed actions that reference WorkspaceReference (FocusWorkspace, MoveWindowToWorkspace, etc.)
  - Kept WorkspaceReference type for internal use (will be removed in later phase)
- [x] Replace in niri-ipc/src/lib.rs
  - Replaced all Workspace action variants with Row equivalents
  - Removed WorkspaceReferenceArg type and its FromStr impl
- [x] Update src/ui/hotkey_overlay.rs
  - Updated action_name() to use Row terminology
  - Updated important_actions() to use Row actions
- [x] Created call site analysis: `phase-1.5.3-part2a-callsites.md`

### Part 2B: Input Handlers ✅ COMPLETE
- [x] Replace handlers in src/input/mod.rs
  - Replaced all Action::FocusWorkspace* with Action::FocusRow*
  - Replaced all Action::MoveWindowToWorkspace* with Action::MoveWindowToRow*
  - Replaced all Action::MoveColumnToWorkspace* with Action::MoveColumnToRow*
  - Replaced all Action::MoveWorkspace* with Action::MoveRow*
  - Replaced Action::SetWorkspaceName/UnsetWorkspaceName with SetRowName/UnsetRowName
  - Layout method calls still use old names (will be updated in Part 2C)
- [x] Created call site analysis: `phase-1.5.3-part2b-callsites.md`

### Part 2C: Layout Methods (PENDING)
See `phases/phase-1.5.3-part2c-callsites.md` for complete analysis.
- [ ] Rename ~25 public methods
- [ ] Update ~50+ call sites in 5 files
- [ ] Remove WorkspaceReference-dependent methods

### Part 2D: Monitor Refactor (PENDING)
See `phases/phase-1.5.3-part2d-callsites.md` for complete analysis.
- [ ] Rename ~15 public methods
- [ ] Update ~30+ call sites in 3 files

### Part 2E: Tests (PENDING)
See `phases/phase-1.5.3-part2e-callsites.md` for complete analysis.
- [ ] Rename ~25 Op enum variants
- [ ] Update ~30+ match arms
- [ ] Rename ~8 test functions

## Changes Made

### niri-ipc/src/lib.rs
- `FocusWindowOrWorkspaceDown/Up` → `FocusWindowOrRowDown/Up`
- `MoveWindowDownOrToWorkspaceDown/Up` → `MoveWindowDownOrToRowDown/Up`
- `FocusWorkspaceDown/Up` → `FocusRowDown/Up`
- `FocusWorkspace(ref)` → REMOVED
- `FocusWorkspacePrevious` → `FocusPreviousPosition`
- `MoveWindowToWorkspaceDown/Up` → `MoveWindowToRowDown/Up`
- `MoveWindowToWorkspace` → REMOVED
- `MoveColumnToWorkspaceDown/Up` → `MoveColumnToRowDown/Up`
- `MoveColumnToWorkspace` → REMOVED
- `MoveWorkspaceDown/Up` → `MoveRowDown/Up`
- `MoveWorkspaceToIndex` → `MoveRowToIndex`
- `SetWorkspaceName` → `SetRowName`
- `UnsetWorkspaceName` → `UnsetRowName`
- `MoveWorkspaceToMonitor*` → `MoveRowToMonitor*`
- Removed `WorkspaceReferenceArg` type

### niri-config/src/binds.rs
- Same action renames as niri-ipc
- Kept `WorkspaceReference` for internal use (marked with TEAM_012 comment)
- Updated `From<niri_ipc::Action>` implementation

### src/input/mod.rs
- Updated all action handlers to use new Row action names
- Layout method calls preserved (will be renamed in Part 2C)

### src/ui/hotkey_overlay.rs
- Updated action_name() display strings
- Updated important_actions() to use Row actions

## Handoff
- [x] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`./scripts/verify-golden.sh`)
- [ ] Team file complete
