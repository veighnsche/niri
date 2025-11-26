# Phase 1.5.3 Part 3: Overview Removal Guide

> **Status**: IN PROGRESS (TEAM_014)
> **Goal**: Completely remove overview mode from niri

## What Was Done

TEAM_014 has disabled overview mode by:
1. Making all overview methods return no-ops or constants
2. Adding `DEPRECATED(overview)` tags to all affected code
3. Keeping stub methods/fields for compilation compatibility

## How to Find All Deprecated Code

```bash
# Find all overview-related deprecated code
grep -rn "DEPRECATED(overview)" src/

# Find remaining overview references (should only be stubs)
grep -rn "overview" src/layout/ src/input/ | grep -v "DEPRECATED"
```

---

## Step-by-Step Removal Guide

### Step 1: Remove Overview Config (niri-config)

**Files to modify:**
- `niri-config/src/lib.rs` — Remove `Overview` struct and `overview` field from `Config`
- `niri-config/src/overview.rs` — Delete entire file (if exists)

**After removal:**
- Update `Options::from_config` in `src/layout/mod.rs` to not reference `config.overview`
- Remove `overview` field from `Options` struct

### Step 2: Remove Overview Actions

**Files to modify:**
- `niri-config/src/actions.rs` — Remove `ToggleOverview`, `OpenOverview`, `CloseOverview` actions

**After removal:**
- Remove the no-op match arm in `src/input/mod.rs` (search for `Action::ToggleOverview`)

### Step 3: Remove Overview Gestures

**Files to modify:**
- `src/input/mod.rs`:
  - Remove 4-finger gesture handling (line ~3488)
  - Remove `uninverted_delta_y` variable (line ~3525)
  - Remove `was_inside_hot_corner` variables (lines ~2205, ~2425)
  - Remove `is_overview_open` variable in tablet handler (line ~3343)

### Step 4: Remove Hot Corner Overview Trigger

**Files to modify:**
- `src/input/mod.rs`:
  - Simplify hot corner handling (currently just sets `pointer_inside_hot_corner`)
  - Consider removing hot corner entirely or repurposing it

**Related fields to remove:**
- `Niri::pointer_inside_hot_corner` field

### Step 5: Remove TouchOverviewGrab

**Files to delete:**
- `src/input/touch_overview_grab.rs` — Delete entire file

**Files to modify:**
- `src/input/mod.rs`:
  - Remove `mod touch_overview_grab;`
  - Remove `use touch_overview_grab::TouchOverviewGrab;`
  - Remove code that creates `TouchOverviewGrab` (line ~3897-3931)

### Step 6: Remove Overview from Layout

**Files to modify:**
- `src/layout/mod.rs`:
  - Remove `is_overview_open()` method (returns `false`)
  - Remove `overview_zoom()` method (returns `1.0`)
  - Remove `compute_overview_zoom()` function
  - Remove `overview` field from `Options` struct

### Step 7: Remove Overview from Monitor

**Files to modify:**
- `src/layout/monitor/mod.rs`:
  - Already removed `overview_open` and `overview_progress` fields ✓
  
- `src/layout/monitor/workspace_compat.rs`:
  - Remove `overview_zoom()` method (returns `1.0`)
  
- `src/layout/monitor/gestures.rs`:
  - Remove `dnd_scroll_gesture_begin()` early return (line ~165-167)
  - Clean up `is_clamped: true` comment
  
- `src/layout/monitor/render.rs`:
  - Remove `render_workspace_shadows()` method (returns empty iterator)
  
- `src/layout/monitor/hit_test.rs`:
  - Clean up DEPRECATED comments

### Step 8: Remove Workspace Shadows

**Files to modify:**
- `src/layout/workspace.rs`:
  - Remove `shadow` field from `Workspace` struct
  - Remove `compute_workspace_shadow_config()` function
  - Remove `render_shadow()` method
  - Remove `Shadow` import and usage

**Files to modify in niri-config:**
- Remove `WorkspaceShadow` struct

### Step 9: Remove Overview from IPC

**Files to modify:**
- `src/ipc/server.rs`:
  - Remove overview event stream handling (line ~782-788)
  
- `niri-ipc/src/lib.rs`:
  - Remove overview-related IPC types

### Step 10: Remove Overview Tests

**Files to modify:**
- `src/layout/tests.rs`:
  - Remove `Op::OverviewGestureBegin`, `Op::OverviewGestureUpdate`, `Op::OverviewGestureEnd` variants
  - Remove `Op::ToggleOverview` variant
  - Remove any tests that specifically test overview behavior

### Step 11: Clean Up Remaining References

**Run these commands to find stragglers:**
```bash
# Find all remaining overview references
grep -rn "overview" src/ niri-config/ niri-ipc/ --include="*.rs"

# Find all DEPRECATED(overview) tags
grep -rn "DEPRECATED(overview)" src/
```

---

## Verification Checklist

After each step:
- [ ] `cargo check` passes
- [ ] `cargo test --lib` passes
- [ ] `cargo insta test` passes (golden tests)

After all steps:
- [ ] No `DEPRECATED(overview)` comments remain
- [ ] No `overview` references remain (except maybe docs)
- [ ] Hot corner either removed or repurposed
- [ ] 4-finger gesture either removed or repurposed
- [ ] All overview-related config options removed

---

## Files Summary

### Files to DELETE entirely:
- `src/input/touch_overview_grab.rs`

### Files with MAJOR changes:
- `src/layout/mod.rs` — Remove overview methods and Options field
- `src/layout/workspace.rs` — Remove shadow field and methods
- `src/input/mod.rs` — Remove gesture handlers and hot corner
- `niri-config/src/lib.rs` — Remove Overview config

### Files with MINOR changes (cleanup):
- `src/layout/monitor/*.rs` — Remove DEPRECATED comments
- `src/layout/tests.rs` — Remove Op variants
- `src/ipc/server.rs` — Remove overview event stream

---

## Notes for Future Teams

1. **Don't rush** — Remove one component at a time and verify compilation
2. **Run tests after each step** — Golden tests will catch layout regressions
3. **Check IPC compatibility** — External tools may depend on overview events
4. **Update documentation** — Remove overview from wiki pages
5. **Config migration** — Users with overview config will get warnings/errors

## Related Documents

- `docs/2d-canvas-plan/phases/phase-1.5.3-part3-remove-overview.md` — Original plan
- `docs/2d-canvas-plan/README.md` — Overall 2D canvas plan
