# 2D Canvas — Planning Update Masterplan

> **Purpose**: Single source of truth for what needs to happen before Phase 1.5.3 begins.
> **Created**: TEAM_009 after comprehensive contradiction sweep and USER alignment.

---

## Executive Summary

The 2D Canvas refactor is a **breaking change** that removes workspaces entirely. This is NOT an incremental migration with feature flags — it's a hard cutover.

### Key Principles (from USER)

1. **No Animation Implementation** — Animations already exist in main branch. We must NOT REGRESS them.
2. **Break Early, Fix Properly** — Let compiler show all call sites, fix them directly.
3. **Test Before Breaking** — Port tests BEFORE Phase 1.5.3, not after.
4. **Long Way = Right Way** — Take the correct approach, never shortcuts.

---

## Pre-Phase 1.5.3 Checklist

Before any team starts Phase 1.5.3, these must be complete:

### 1. Animation Regression Audit ✅ Required
**Document**: [`phases/animation-regression-checklist.md`](phases/animation-regression-checklist.md)

- [ ] Audit all animations in `scrolling.rs` and `workspace.rs`
- [ ] Document trigger conditions for each animation
- [ ] Create verification checklist for post-refactor testing
- [ ] **Goal**: Ensure refactored code triggers SAME animations in SAME scenarios

### 2. Testing Strategy ✅ Required
**Location**: Added to [`phases/phase-1.5-integration.md`](phases/phase-1.5-integration.md)

- [ ] Port existing ScrollingSpace tests to Row/Canvas2D
- [ ] Write new tests for 2D-specific behavior
- [ ] All tests must pass BEFORE Phase 1.5.3 begins

### 3. Workspace Removal Checklist ✅ Required
**Document**: [`phases/phase-1.5.3-removal-checklist.md`](phases/phase-1.5.3-removal-checklist.md)

- [ ] Detailed list of everything to remove
- [ ] Correct removal order (dependents before dependencies)
- [ ] Config handling: Remove entirely (breaking change)
- [ ] IPC handling: Remove endpoints entirely

---

## Phase Order (Confirmed)

```
Phase 1.5.3 → Phase 1.5.4 → Phase 2 → Phase 3 → Phase 4 → Phase 5
   │              │            │          │          │         │
   │              │            │          │          │         └─ IPC, Docs, Overview replacement
   │              │            │          │          └─ Navigation + Polish
   │              │            │          └─ Camera System (zoom)
   │              │            └─ Row Spanning
   │              └─ Monitor Integration (wire Canvas2D)
   └─ Replace Workspace (BREAKING CHANGE)
```

### Rationale
- Row spanning is architecturally simpler than camera
- Camera requires row spanning to work properly (zoom to fit spanning window)
- Animation regression handled within each phase (parallel checklist)

---

## Animation Strategy

### The Misunderstanding (Corrected)

❌ **Wrong**: "We need to implement animations"
✅ **Correct**: "Animations already exist. We must NOT REGRESS them."

### How Golden Tests Work (Positions)
```
ScrollingSpace.snapshot() → positions, indices, view_offset
Row.snapshot()            → positions, indices, view_offset
                          ↓
                    Compare: Must be identical
```

### How Animation Regression Works (Behavior)
```
Trigger: User presses Mod+R (cycle preset width)
         ↓
ScrollingSpace: Calls animate_view_offset_to_column()
Row:            Must ALSO call animate_view_offset_to_column()
                          ↓
                    Verify: Same animation triggered
```

### What We're Testing
1. **Position Regression** — Golden tests (58 tests, already working)
2. **Animation Trigger Regression** — New checklist (manual + automated)

---

## Testing Strategy

### Before Phase 1.5.3
1. Port ScrollingSpace unit tests to Row
2. Port Workspace unit tests to Canvas2D
3. Verify all 251+ tests pass
4. Verify all 58 golden tests pass

### During Phase 1.5.3
1. Run tests after each removal step
2. Fix failures immediately (don't accumulate)
3. Use animation checklist for manual verification

### After Phase 1.5.4
1. Remove ScrollingSpace tests (dead code)
2. Remove Workspace tests (dead code)
3. Keep golden tests (reference implementation)

---

## Workspace Removal Strategy

### What Gets Removed

| Category | Items |
|----------|-------|
| **Monitor fields** | `workspaces`, `active_workspace_idx`, `workspace_switch_*` |
| **Keybinds** | `Mod+1/2/3` workspace switching (disable, repurpose later) |
| **Overview** | Entire overview mode (broken until Phase 5) |
| **Hot corner** | Top-right corner trigger |
| **Config** | `workspaces { }` block (breaking change) |
| **IPC** | `niri msg workspaces`, related endpoints |
| **Animations** | Workspace switch animations |

### Removal Order
1. Remove consumers first (keybinds, hot corner, IPC)
2. Remove overview mode
3. Remove Monitor workspace fields
4. Remove config parsing
5. Remove Workspace struct (if not used elsewhere)

---

## Documents to Create/Update

### Create New
1. ✅ `phases/animation-regression-checklist.md` — Audit of all animations
2. ✅ `phases/phase-1.5.3-removal-checklist.md` — Detailed removal steps

### Update Existing
1. ✅ `phases/phase-1.5-integration.md` — Add testing requirements
2. ✅ `README.md` — Link to new documents
3. ✅ `ai-teams-rules.md` — Add lessons learned
4. ⏳ `phases/phase-5-integration.md` — Rewrite after Phase 1.5.4

---

## Success Criteria for Phase 1.5.3

- [ ] All workspace-related code removed
- [ ] All tests still pass (251+)
- [ ] All golden tests still pass (58)
- [ ] Animation regression checklist verified
- [ ] Config with `workspaces { }` produces clear error
- [ ] `niri msg workspaces` returns error
- [ ] Hot corner does nothing
- [ ] Overview mode removed (broken state OK)
- [ ] `Mod+1/2/3` disabled (no action)

---

## Lessons Learned (Added to Rules)

### From TEAM_009 Contradiction Sweep

1. **Animation ≠ Implementation** — Refactoring means preserving behavior, not adding features
2. **Test Before Breaking** — Port tests before removing code, not after
3. **Document Removal Scope** — Create detailed checklist before large removals
4. **Parallel Checklists** — Animation regression runs alongside all phases
5. **Replan When Needed** — Don't drift, update the plan when gaps are found

---

## Quick Reference

| Document | Purpose |
|----------|---------|
| This file | Masterplan overview |
| `animation-regression-checklist.md` | Audit of animations to preserve |
| `phase-1.5.3-removal-checklist.md` | What to remove and in what order |
| `phase-1.5-integration.md` | Current phase with testing requirements |
| `TEAM_009_contradiction_sweep.md` | Original sweep with USER answers |
| `TEAM_009_followup_planning_gaps.md` | Follow-up questions with USER answers |

---

*Created by TEAM_009 — Planning Update Masterplan*
