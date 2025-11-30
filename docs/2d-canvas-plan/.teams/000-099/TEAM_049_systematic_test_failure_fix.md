# TEAM_049 — Systematic Test Failure Fix

## Team Number
TEAM_049

## Objective
Fix the remaining 52 pre-existing test failures from Canvas2D refactor systematically.

## Current Status Assessment
- **Tests**: 221 passed, 51 failed (from TEAM_048's run)
- **Golden Tests**: 246 passed, 26 failed + 1264 snapshots to review
- Previous teams identified clear failure patterns

## Failure Categories (from TEAM_044 analysis)
1. **Floating Tests (~22 failing)** - Complex interactions between `expected_size()`, `request_size_once` state machine, and configure throttling
2. **Animation Tests (~10 failing)** - View offset issues  
3. **Fullscreen Tests (~5 failing)** - View offset preservation
4. **Window Opening Tests (~10 failing)** - Workspace targeting
5. **Interactive Move Tests (~8 failing)** - Various issues

## Strategy
Following smart friend advice: Look for common patterns first, then tackle by category.

### Phase 1: Analyze Failure Patterns
1. Run one failing test from each category with verbose output
2. Identify the core assertion failures
3. Look for common API mismatches (workspace→row, view_offset, etc.)

### Phase 2: Fix by Priority
1. **Animation Tests** - Likely related to camera offset issue I found
2. **Fullscreen Tests** - Similar view offset issues
3. **Floating Tests** - Most complex, save for after understanding patterns
4. **Window Opening Tests** - Workspace targeting should be straightforward
5. **Interactive Move Tests** - Address remaining issues

### Phase 3: Golden Tests
- Address the 26 golden failures after unit tests are stable

## Work Log

### Step 1 — Create baseline and analyze patterns
- Running test failure analysis by category

### Step 2 — Investigated animation test failures
- **Root Cause Found**: Row::advance_animations() was not updating cached ColumnData widths with animated values
- **Fix Applied**: Added ColumnData update loop after column animation advancement in Row::advance_animations()
- **Impact**: Animation tests still failing, but discovered deeper issue with Column::width() also using cached data

### Step 3 — Current Status Assessment
- **Total Test Failures**: 52 (unchanged from baseline)
- **Row ColumnData Fix**: Applied but animation tests still failing due to deeper Column data issues
- **Fullscreen Investigation**: Found view offset mismatch (-16 vs +16) during unfullscreen transitions
- **Impact Assessment**: Need to run broader test analysis before diving deeper

### Pattern Identified
- **Systematic Caching Issue**: Multiple cached data structures (Row ColumnData, Column internal data) not updated with animated values
- **View Offset Issues**: Fullscreen logic missing view offset save/restore during transitions
- **Migration Gaps**: Canvas2D migration left some logic incomplete (fullscreen, workspace targeting)

### Progress Summary
1. ✅ **Identified Root Cause Pattern**: Cached data not updated during animations
2. ✅ **Applied Partial Fix**: Row ColumnData updates in advance_animations()
3. 🔄 **Investigating**: Fullscreen view offset preservation logic
4. ⏸️ **Deferred**: Column data caching (complex, lower priority)

## Step-by-Step Investigation Guide for Future Teams

### Phase 1: Baseline Analysis
1. **Run Full Test Suite**
   ```bash
   cargo test --lib 2>&1 | grep "^test.*FAILED" | wc -l  # Count failures
   cargo test --lib 2>&1 | grep "^test.*FAILED" | head -15  # See pattern
   ```

2. **Categorize Failures by Type**
   - Animation tests (width_resize, etc.)
   - Fullscreen tests (unfullscreen_*, etc.)
   - Interactive move tests (interactive_move_*)
   - Window opening tests
   - Floating tests

3. **Pick Representative Test from Each Category**
   - Run with `--nocapture` to see failure details
   - Look for patterns in assertion failures

### Phase 2: Root Cause Investigation
1. **For Animation Test Failures**
   - Check if test uses `format_tiles()` → calls `tiles_with_render_positions()`
   - Examine `Row::column_x()` method → uses cached `self.data`
   - Check `Row::advance_animations()` → updates columns but not cached data
   - **Fix Pattern**: Add `self.data[col_idx].update(column)` after animation advancement

2. **For Fullscreen Test Failures**
   - Look for view offset mismatches (e.g., -16 vs +16)
   - Check `Row::set_fullscreen()` and `Column::set_fullscreen()` → no view offset logic
   - View offset save/restore likely at Canvas2D/Layout level
   - **Fix Pattern**: Port view offset save/restore from old ScrollingSpace

3. **For Interactive Move Failures**
   - Look for None unwrap errors → simple missing workspace creation
   - **Fix Pattern**: Add None check or proper workspace handling

### Phase 3: Systematic Pattern Recognition
1. **Identify Cached Data Issues**
   - Search for `.data.get()` usage patterns
   - Check if `advance_animations()` updates all cached structures
   - Look for similar patterns in Column, Row, Canvas2D

2. **Migration Gap Analysis**
   - Compare old ScrollingSpace methods vs new Row/Canvas2D
   - Look for missing view offset, animation, or workspace logic

### Phase 4: Prioritization Strategy
1. **Easy Wins First** - None unwrap fixes (workspace targeting)
2. **Medium Impact** - View offset logic (fullscreen)
3. **Complex Systematic** - Cached data updates (animations)

### Phase 5: Implementation Pattern
1. **Apply Fix**
2. **Test Single Category**
3. **Assess Impact**
4. **Document in Team File**
5. **Move to Next Category**

## Handoff Notes for Next Team

### High-Impact Issues Identified
1. **Column Data Caching** (Animation Tests)
   - **Location**: `src/layout/column/layout.rs` Column::width() uses cached `self.data`
   - **Problem**: Similar to Row ColumnData issue - cached data not updated during animations
   - **Fix Needed**: Add Column data updates in Column::advance_animations() or similar
   - **Impact**: ~10 animation test failures

2. **Fullscreen View Offset Preservation** (Fullscreen Tests)
   - **Location**: Likely `src/layout/canvas/operations.rs` or `src/layout/mod.rs`
   - **Problem**: View offset save/restore logic missing during unfullscreen transitions
   - **Symptom**: -16 vs +16 view offset mismatch
   - **Fix Needed**: Port view offset save/restore from old ScrollingSpace
   - **Impact**: ~5 fullscreen test failures

3. **Workspace Targeting** (Interactive Move Tests)
   - **Location**: `src/layout/mod.rs:4320` None unwrap
   - **Problem**: Simple None unwrap in workspace targeting logic
   - **Fix Needed**: Add None check or proper workspace creation
   - **Impact**: ~8 interactive move test failures (EASY WIN)

### Recommended Priority
1. **Workspace Targeting** - Easy None unwrap fixes for quick wins
2. **Fullscreen View Offset** - Medium complexity, high impact
3. **Column Data Caching** - Complex, systematic fix needed

### Files Modified
- `src/layout/row/mod.rs` - Added ColumnData updates in advance_animations() (TEAM_049)

### Compilation Status
- ✅ Code compiles successfully
