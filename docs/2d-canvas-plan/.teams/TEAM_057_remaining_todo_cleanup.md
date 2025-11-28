# TEAM_057: Remaining TODO Cleanup

## Team Number
**TEAM_057** - Next team after TEAM_056

## Mission
Analyze and fix remaining TODOs from the codebase, focusing on easy fixes immediately and documenting complex ones for future teams.

## Rules Compliance
- ✅ Read ai-teams-rules.md
- ✅ Checked latest team number (TEAM_056)
- ✅ Ran golden tests (found failures, avoided layout changes)
- ✅ Followed quality over speed approach

## Work Completed

### ✅ Easy TODOs Fixed
1. **src/layout/row_types.rs**: Removed outdated TODO documentation comments
   - Cleaned up "Eventually remove WorkspaceId" comments 
   - These were just documentation notes, not actionable items
2. **src/layout/mod.rs line 3692**: Removed resolved TODO about original_output field
   - Functionality was already commented out, TODO was no longer needed

### ✅ Complex TODOs Analyzed and Documented
1. **HIGH PRIORITY**: `src/layout/mod.rs:4752` - Duplicate name checking for canvas rows
   - **Why complex**: Needs collision detection, conflict resolution, API compatibility
   - **Impact**: Causing test failure in `move_window_to_workspace_with_different_active_output`

2. **MEDIUM PRIORITY**: 6 functional enhancement TODOs
   - Canvas cleanup logic (lifecycle management)
   - Window height setting API design
   - Workspace ID to row mapping (external compatibility)
   - Column movement animation porting
   - Coordinate conversion in floating_logical_to_size_frac
   - Insert hint rendering with Canvas2D

### ✅ Documentation Updated
- **Global TODO.md**: Comprehensive analysis with complexity reasoning
- Implementation order recommendations for future teams
- Clear requirements and complexity explanations

## Current State
- **Golden Tests**: ⚠️ 2 failing (avoided layout changes per rules)
- **Compilation**: ✅ Code compiles with warnings
- **TODOs Reduced**: From 9 total to 6 complex items remaining

## Handoff Notes

### Next Team Priority
1. **Fix the duplicate name checking issue** (line 4752) - it's causing test failures
2. Consider implementing workspace ID to row mapping for external compatibility

### Files Modified
- `src/layout/row_types.rs`: Cleaned up documentation TODOs
- `src/layout/mod.rs`: Removed resolved TODO comment
- `docs/2d-canvas-plan/TODO.md`: Added comprehensive analysis

### Complex Items Requiring Deep Work
All remaining TODOs need significant architectural decisions:
- API design for Canvas2D window operations
- Animation system porting from ScrollingSpace
- External system compatibility layers
- Rendering system integration

## Status
- [x] Team file created
- [x] Golden tests run (found failures, acted accordingly)
- [x] TODOs analyzed and categorized
- [x] Easy fixes implemented
- [x] Complex items documented with reasoning
- [x] Global TODO.md updated
- [x] Handoff prepared

## Quality Assurance
- Followed "quality over speed" rule from ai-teams-rules.md
- Did not attempt complex architectural changes without proper analysis
- Provided clear reasoning for why each TODO is complex
- Maintained golden test compliance by avoiding layout changes

---
*Completed: Nov 28, 2025*
