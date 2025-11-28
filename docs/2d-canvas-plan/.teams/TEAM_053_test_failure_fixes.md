# TEAM_053: Systematic Test Failure Fixes

**Team Number**: 053  
**Focus**: Fix remaining 39 failing tests systematically  
**Started**: 2025-11-28  

## Current Status

- **Build**: ✅ Compiles  
- **Tests**: 233 passed, 39 failed (85.7% pass rate)  
- **Golden Tests**: ✅ 88/88 pass  
- **Progress**: Improved from 55 failed → 39 failed since TEAM_046  

## Test Failure Analysis

### Initial Test Run
```
test result: FAILED. 233 passed; 39 failed; 0 ignored; 0 measured; 0 filtered out
```

### Notable Snapshot Failures
- `unfullscreen_to_same_size_floating` (line 1051)
- `unmaximize_to_same_size_windowed_fullscreen_floating` (line 1171)  
- `unmaximize_to_same_size_same_bounds_floating` (line 1279)

These suggest floating window fullscreen/maximize state issues.

## Strategy

1. **Run detailed test analysis** to categorize all 39 failures
2. **Focus on floating window issues** (appear to be major category)
3. **Fix animation system regressions** if still present
4. **Address fullscreen/maximize state handling**
5. **Iterate until 100% pass rate**

## Next Steps

1. Run full test with detailed output to see all failure categories
2. Group failures by type (floating, animation, fullscreen, etc.)
3. Tackle highest-impact category first
4. Document fixes and run golden tests after each major change

## Investigation Update

### Status: Investigation Complete - Found Root Cause

### Root Cause Analysis:
I investigated the floating fullscreen test failure (`unfullscreen_to_same_size_floating`) and discovered a **fundamental Canvas2D integration issue**:

**Problem**: The test expects the window to be fullscreen (1920×1080) at line 1051, but it's already Normal (936×1048) when the floating toggle runs.

**Root Cause**: The XDG shell fullscreen handlers aren't being triggered during `f.double_roundtrip(id)`, meaning `window.set_fullscreen(None)` never actually sets the window to fullscreen in the Canvas2D layout system.

### Investigation Findings:
1. **Test Sequence**:
   - Line 1047: `window.set_fullscreen(None)` - client-side XDG call
   - Line 1048: `f.double_roundtrip(id)` - should trigger XDG handlers but doesn't
   - Line 1057: `toggle_window_floating(None)` - my toggle runs with window already Normal

2. **XDG Handler Integration**:
   - `src/handlers/xdg_shell.rs:644` properly calls `self.niri.layout.set_fullscreen(&window, true)`
   - `Canvas2D::set_fullscreen()` is implemented and delegates to rows correctly
   - But debug logging shows neither Canvas2D::set_fullscreen nor Canvas2D::toggle_fullscreen are called during the test

3. **Fullscreen Test Status**:
   - 31 fullscreen tests pass, 10 fail - indicates Canvas2D fullscreen integration partially works
   - The issue is specifically with test fixture XDG handler integration, not the core fullscreen system

### Implemented Solutions (Ready for When Integration is Fixed):
I implemented fullscreen size preservation logic that should work once the XDG handler integration is fixed:

1. **Row::get_fullscreen_size_for_window()** - Public method to safely query fullscreen size
2. **Row::update_window() fullscreen preservation** - Captures fullscreen size during unfullscreen transitions
3. **Canvas2D::toggle_floating_window_by_id()** - Enhanced to preserve fullscreen dimensions

### Next Steps Required:
This is a **blocking Canvas2D integration issue** that requires:
- Investigating why XDG shell handlers don't trigger during test `double_roundtrip`
- Fixing the test fixture to properly integrate with Canvas2D fullscreen handling
- OR determining if this is intentional test behavior that needs different handling

### Impact:
- **Blocks**: All floating fullscreen tests and potentially other XDG-based fullscreen functionality
- **Scope**: 10 failing fullscreen tests, 22 failing floating tests
- **Priority**: High - fundamental integration issue

### Recommendation:
Document this as a known Canvas2D integration issue and focus on other test categories (workspace/move, animation, window opening) that don't depend on XDG fullscreen integration.

---
*Updated: TEAM_053 investigation complete - identified blocking Canvas2D integration issue*

## Handoff Checklist

- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`) 
- [ ] Golden tests pass (`cargo insta test`)
- [ ] Team file complete with all changes documented
