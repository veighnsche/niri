# TEAM_020: Workspace File Cleanup

## Status: STARTING

## Team Assignment
- **Team Number**: 020
- **Task**: Complete removal of workspace compatibility files and remaining workspace method calls
- **Previous Team**: TEAM_019 (Workspace compatibility removal - partial progress)

## Problem Statement

TEAM_019 made significant progress on workspace → canvas migration but the legacy workspace files still exist:

- `src/layout/monitor/workspace_compat.rs` (272 lines) - Should be deleted
- `src/layout/monitor/workspace_ops.rs` (486 lines) - Should be deleted  
- `src/layout/workspace.rs` (1998 lines) - The massive workspace struct - Should be deleted

These files contain legacy code that is no longer needed if Canvas2D has successfully replaced workspace functionality.

## Current State Analysis

### From TEAM_019 Progress
- ✅ Golden tests pass (84/84)
- ✅ Canvas2D workspace replacement methods implemented
- ✅ Some workspace method calls migrated to canvas
- ✅ Some workspace golden tests removed

### ❌ What Remains
- Legacy workspace compatibility files still exist
- Unknown number of workspace method calls still remaining in layout layer
- Workspace struct may still be referenced/used
- Need to verify complete migration is possible

## Implementation Plan

### Step 1: Survey Remaining Workspace Usage
- [ ] Search for all remaining references to workspace methods/types
- [ ] Identify which files still import/use workspace code
- [ ] Map remaining workspace → canvas migrations needed

### Step 2: Complete Any Missing Canvas Methods
- [ ] Check if any workspace functionality is missing from Canvas2D
- [ ] Implement any missing canvas methods
- [ ] Ensure canvas can fully replace workspace functionality

### Step 3: Migrate Remaining Workspace Method Calls
- [ ] Systematically replace workspace method calls with canvas equivalents
- [ ] Update imports to use canvas instead of workspace
- [ ] Fix compilation errors as they arise

### Step 4: Remove Legacy Workspace Files
- [ ] Delete `src/layout/monitor/workspace_compat.rs`
- [ ] Delete `src/layout/monitor/workspace_ops.rs`
- [ ] Delete `src/layout/workspace.rs`

### Step 5: Cleanup and Verification
- [ ] Remove any remaining workspace-related imports
- [ ] Fix compilation errors
- [ ] Verify golden tests still pass (CRITICAL)
- [ ] Update any documentation/comments

## Critical Constraints

### 🚨 GOLDEN TEST RULES
- **NEVER accept golden snapshot changes** - they represent MAIN BRANCH behavior
- **If golden tests fail, fix YOUR CODE** - not the tests
- **Run `./scripts/verify-golden.sh` before AND after changes**

### 🎯 Migration Strategy
- **Preserve existing behavior** - architectural cleanup only
- **Complete removal** - all workspace code should be gone
- **Canvas as sole system** - Canvas2D provides all layout functionality

## Success Criteria

- [ ] All workspace compatibility files deleted
- [ ] No workspace references remaining in layout layer
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] **Golden tests pass (`cargo insta test`)** - CRITICAL
- [ ] Canvas2D is the sole layout system

## Handoff

This phase completes the workspace → Canvas2D migration by removing all legacy workspace code. If successful, the workspace system will be completely eliminated and Canvas2D will be the only layout system.

**Next priority**: Full Canvas2D feature parity and optimization.

---

## Progress Log

### Initial Assessment Complete
- [x] Golden tests verified (84/84 pass)
- [x] Team file created
- [x] Legacy workspace files identified
- [x] Cleanup strategy planned

### Current State Analysis
After surveying the codebase, I found that **workspace usage is still extensive**:

#### Major Workspace Usage Areas:
1. **Layout Core (`layout/mod.rs`)**: 100+ workspace method calls
   - `workspaces()` / `workspaces_mut()` - workspace iteration
   - `active_workspace()` / `active_workspace_ref()` - active workspace access
   - Monitor workspace management in layout operations

2. **Monitor System**: Still heavily workspace-based
   - `monitor/workspace_compat.rs` - 272 lines of compatibility methods
   - `monitor/workspace_ops.rs` - 486 lines of workspace operations
   - Monitor struct still contains workspace fields

3. **External Systems**: 
   - IPC server uses `active_workspace()` and workspace IDs
   - Tests use workspace methods extensively
   - Protocol handlers reference workspace types

#### Problem: Canvas2D is Not Ready for Complete Replacement
The Canvas2D system exists but lacks many critical methods:
- Missing workspace iteration equivalents
- Missing active workspace concept (uses active_row instead)
- Missing workspace ID system
- Missing many workspace operation methods

### Revised Strategy: Incremental Migration
Cannot delete workspace files yet - Canvas2D needs more implementation.

**New Plan**:
1. **Phase 1**: Implement missing Canvas2D methods to match workspace functionality
2. **Phase 2**: Systematically migrate workspace method calls to canvas calls  
3. **Phase 3**: Remove workspace compatibility files
4. **Phase 4**: Remove main workspace.rs file

### Phase 1 Progress: Canvas2D Methods Implementation
- [x] Added workspace replacement methods to Canvas2D:
  - [x] `has_windows()` / `has_windows_or_name()` - window existence checks
  - [x] `find_wl_surface()` / `find_wl_surface_mut()` - surface lookup
  - [x] `update_window()` - window state updates
  - [x] `activate_window()` / `activate_window_without_raising()` - window activation
  - [x] `set_fullscreen()` / `toggle_fullscreen()` - fullscreen control
  - [x] `set_maximized()` / `toggle_maximized()` - maximize control
  - [x] `has_urgent_window()` - urgency checks
  - [x] `scroll_amount_to_activate()` - scroll calculations
  - [x] `descendants_added()` - hierarchy checks
  - [x] `popup_target_rect()` - popup positioning
- [x] Code compiles successfully
- [x] Golden tests pass (84/84)

### Phase 2 Progress: Workspace Method Call Migration
- [x] Updated `workspace_compat.rs`:
  - [x] `windows()` - now uses canvas only
  - [x] `has_window()` - now uses canvas only  
  - [x] `active_window()` - now uses canvas only
  - [x] Added canvas iteration methods: `canvas_rows()`, `canvas_rows_mut()`, `canvas_find_window()`, `canvas_find_window_mut()`
- [x] Updated `workspace_ops.rs`:
  - [x] `clean_up_workspaces()` - now uses canvas cleanup
- [x] Updated `layout/mod.rs` core methods:
  - [x] `descendants_added()` - now uses canvas first, workspace fallback
  - [x] `popup_target_rect()` - now uses canvas first, workspace fallback
  - [x] `set_fullscreen()` - now uses canvas first, workspace fallback
  - [x] `toggle_fullscreen()` - now uses canvas first, workspace fallback
  - [x] `set_maximized()` - now uses canvas first, workspace fallback
  - [x] `toggle_maximized()` - now uses canvas first, workspace fallback
  - [x] `update_config()` - now uses canvas first, workspace fallback
- [x] Added `rows_mut()` method to Canvas2D for iteration support
- [x] Code compiles successfully
- [x] Golden tests pass (84/84)

### Migration Strategy Applied
Using **dual-path approach** for compatibility:
1. **Canvas-first**: Try canvas methods first
2. **Workspace fallback**: Keep workspace methods as backup
3. **Gradual transition**: Allows incremental migration without breaking existing functionality
4. **TODO markers**: Mark workspace sections for eventual removal

### Next Steps
1. Continue migrating more workspace method calls in layout/mod.rs
2. Focus on high-usage methods: workspaces(), workspaces_mut(), active_workspace()
3. Add canvas equivalents for workspace iteration patterns
4. Eventually remove workspace compatibility files
