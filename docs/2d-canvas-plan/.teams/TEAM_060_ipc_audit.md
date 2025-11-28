# TEAM_060: IPC Audit for Canvas2D Migration

**Date**: Nov 28, 2025  
**Focus**: Comprehensive IPC audit for workspace → row terminology migration  
**Status**: ✅ Audit Complete

## Overview
Performed a complete audit of IPC-related code to identify all workspace terminology that needs to be updated for the Canvas2D refactor.

## Key Findings

### 1. IPC Actions (niri-ipc/src/lib.rs)
**Status**: ✅ ALREADY MIGRATED

The IPC Action enum has already been updated to use row-based terminology:
- ✅ `FocusRowDown` / `FocusRowUp` (already migrated)
- ✅ `FocusWindowOrRowDown` / `FocusWindowOrRowUp` (already migrated)  
- ✅ `MoveWindowToRowDown` / `MoveWindowToRowUp` (already migrated)
- ✅ `MoveColumnToRowDown` / `MoveColumnToRowUp` (already migrated)
- ✅ `MoveRowDown` / `MoveRowUp` / `MoveRowToIndex` (already migrated)
- ✅ `SetRowName` / `UnsetRowName` (already migrated)
- ✅ `MoveRowToMonitor*` actions (already migrated)

**Missing Actions**: No workspace-related actions found that need migration.

### 2. IPC Events (niri-ipc/src/lib.rs)
**Status**: ⚠️ NEEDS MIGRATION

Several workspace-related events still exist and need to be updated:

#### Events to Rename:
- `WorkspacesChanged` → `RowsChanged`
- `WorkspaceUrgencyChanged` → `RowUrgencyChanged`  
- `WorkspaceActivated` → `RowActivated`
- `WorkspaceActiveWindowChanged` → `RowActiveWindowChanged`

#### Event Fields to Update:
- Event struct fields containing `workspace_id` → `row_id`
- Event descriptions mentioning "workspace" → "row"

### 3. IPC State Structures (niri-ipc/src/lib.rs)
**Status**: ⚠️ NEEDS MIGRATION

#### Structs to Rename:
- `Workspace` struct → `Row` struct
- `WorkspacesState` → `RowsState`

#### Fields to Rename:
- `workspaces: Vec<Workspace>` → `rows: Vec<Row>`
- `workspace_id: Option<u64>` → `row_id: Option<u64>`
- All struct field documentation mentioning "workspace"

### 4. IPC Requests (niri-ipc/src/lib.rs)
**Status**: ⚠️ NEEDS MIGRATION

#### Requests to Update:
- `Request::Workspaces` → `Request::Rows`
- `Response::Workspaces(Vec<Workspace>)` → `Response::Rows(Vec<Row>)`

### 5. Protocol Implementation (src/protocols/ext_workspace.rs)
**Status**: ⚠️ NEEDS MAJOR REDESIGN

This is the most complex part of the migration. The ext-workspace protocol is designed around the concept of discrete workspaces, but Canvas2D has:

- **No discrete workspaces** - only continuous rows
- **Camera bookmarks** replace workspace switching  
- **One infinite canvas per output** - not separate containers

#### Options for ext-workspace protocol:
1. **Maintain compatibility**: Map rows to workspace-like entities for external clients
2. **Protocol redesign**: Create new ext-row protocol (breaking change)
3. **Hybrid approach**: Keep workspace protocol but implement as row viewports

#### Current Issues:
- Protocol assumes discrete workspace containers
- Workspace coordinates system (X=0, Y=workspace_index) doesn't map to Canvas2D
- Workspace activation/deactivation concepts don't translate directly
- Workspace groups as outputs concept needs adaptation

### 6. Internal State Management (src/niri.rs)
**Status**: ⚠️ NEEDS MIGRATION

#### Methods to Update:
- `workspace_under()` → `row_under()`
- `workspace_under_cursor()` → `row_under_cursor()`
- `find_output_and_workspace_index()` → `find_output_and_row_index()`

#### Fields to Update:
- `workspace_reference: WorkspaceReference` → `row_reference: RowReference`

### 7. Input Handling (src/input/mod.rs)
**Status**: ✅ ALREADY MIGRATED

All input actions have been updated to use row-based terminology:
- ✅ All `FocusWorkspace*` → `FocusRow*` migrations complete
- ✅ All `MoveWindowToWorkspace*` → `MoveWindowToRow*` migrations complete  
- ✅ All `MoveColumnToWorkspace*` → `MoveColumnToRow*` migrations complete
- ✅ All `MoveWorkspace*` → `MoveRow*` migrations complete

## Migration Priority

### High Priority (Breaking Changes)
1. **IPC Events** - External clients depend on these
2. **IPC State Structures** - Breaking change to API
3. **IPC Requests** - Breaking change to API

### Medium Priority (Internal Consistency)  
1. **Internal State Management** - Method names and fields
2. **Protocol Implementation** - Complex redesign needed

### Low Priority (Documentation)
1. **Comments and Documentation** - Update all references

## Recommended Approach

### Phase 1: IPC API Migration
1. Rename all workspace-related structs, enums, and fields in niri-ipc
2. Update event and request types
3. Maintain backward compatibility through aliases during transition

### Phase 2: Protocol Redesign  
1. Decide on approach for ext-workspace protocol
2. Implement Canvas2D-compatible protocol
3. Update protocol handlers

### Phase 3: Internal Cleanup
1. Update remaining internal method names
2. Clean up any remaining workspace references
3. Update documentation

## Files Requiring Changes

### niri-ipc/src/lib.rs
- Events: `WorkspacesChanged`, `WorkspaceUrgencyChanged`, `WorkspaceActivated`, `WorkspaceActiveWindowChanged`
- Structs: `Workspace`, `WorkspacesState`  
- Requests: `Workspaces`
- Fields: `workspaces`, `workspace_id`

### src/protocols/ext_workspace.rs
- Entire protocol implementation needs redesign
- Workspace → Row mapping logic
- Coordinate system adaptation

### src/niri.rs
- Methods: `workspace_under()`, `workspace_under_cursor()`, `find_output_and_workspace_index()`
- Fields: `workspace_reference`

### src/layout/mod.rs
- `WorkspaceReference` enum → `RowReference`
- Related find methods

## Impact Assessment

### Breaking Changes: HIGH
- IPC API changes will affect all external clients
- Protocol changes may break client compatibility

### Development Effort: HIGH  
- Protocol redesign is complex
- Need to maintain backward compatibility during transition

### Risk Level: MEDIUM
- IPC changes are isolated to external interface
- Internal layout changes already complete

## Next Steps

1. **Decision needed**: Approach for ext-workspace protocol (compatibility vs redesign)
2. **Plan migration strategy**: Backward compatibility approach
3. **Implement Phase 1**: IPC API migration in niri-ipc
4. **Address protocol**: Complex redesign work
5. **Complete internal cleanup**: Remaining method names

This audit provides a complete roadmap for the IPC portion of the Canvas2D migration.
