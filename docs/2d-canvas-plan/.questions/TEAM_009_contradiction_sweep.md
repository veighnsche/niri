# TEAM_009: Contradiction Sweep & Clarification Questionnaire

> **Purpose**: Document all contradictions, drift, and ambiguities found across documentation and code.
> **Created**: After Phase 1.5.2 completion

---

## CRITICAL CONTRADICTIONS

### 1. Feature Flag vs Breaking Change ⚠️ MAJOR

**The Problem**: Multiple documents still reference feature flags, but the Key Decisions clearly state workspaces are REMOVED (not feature-flagged).

| Location | Says | Should Say |
|----------|------|------------|
| `ai-teams-rules.md:240` | "then feature flag" | "then replace Workspace" |
| `ai-teams-rules.md:255` | "Feature flag integration is significant work" | Remove or update |
| `phase-1.5-integration.md:61` | "Feature flag — `canvas-2d` feature not created" | ✅ Already updated |
| `phase-5-integration.md:47-53` | `#[cfg(feature = "canvas-2d")]` throughout | Remove all cfg attributes |
| `phase-5-integration.md:433` | "Consider removing feature flag" | No feature flag exists |

**Source of Truth** (from `ai-teams-rules.md:243`):
> "Workspaces **removed** — one infinite canvas per output"

**Source of Truth** (from `2D_CANVAS_QUESTIONNAIRE.md:192,199`):
> "No, one giant 2D canvas replaces workspaces"
> "I want to completely eliminate the concept of having multiple workspaces per monitor"

**Question for USER**: Can you confirm this is correct? No feature flag, just complete workspace removal on this branch?

AFFIRMATIVE

---

### 2. Phase Duplication: Replace Workspaces Appears Twice

- **Phase 1.5.3**: "Replace Workspace with Canvas2D ⚠️ BREAKING CHANGE"
- **Phase 5.1**: "Replace Workspaces"

**Question for USER**: Should Phase 5.1 be removed/renamed? It seems like Phase 1.5.3 already covers workspace replacement. Perhaps Phase 5 should focus on:
- IPC updates
- Overview replacement  
- Documentation
- (NOT workspace replacement again)

Yeah let's break code early. so that we can fix all the call sites immediately and whatever is the best choice to take the long way of doing things.. the correct way... so What to do in 5.1... that is too far in the future... appearantly we can't even look that far correctly... just remove the current 5.1 or check it off with a note that it has been done early... we don't need a replacement... because we might want to do it in a different way... 

---

### 3. Code Comment Ambiguity

**File**: `src/layout/canvas/mod.rs:4`
```rust
//! It replaces `Workspace` for 2D mode.
```

This says "for 2D mode" which implies a mode switch, not a full replacement.

**Question for USER**: Should this say "It replaces `Workspace` entirely" instead?

AFFIRMATIVE

---

## DOCUMENTATION DRIFT

### 4. Outdated Status in ai-teams-rules.md

**Line 238-240**:
```
**Phase**: 1.5 IN PROGRESS (Integration)  
**Completed**: Phase 0, Phase 1 Core, **Phase 1.5.1 (Row complete)**  
**Next Step**: Phase 1.5.2 (Canvas2D), then feature flag
```

**Should be**:
```
**Phase**: 1.5 IN PROGRESS (Integration)  
**Completed**: Phase 0, Phase 1 Core, Phase 1.5.1 (Row), **Phase 1.5.2 (Canvas2D + FloatingSpace)**  
**Next Step**: Phase 1.5.3 (Replace Workspace in Monitor)
```

**Note**: I cannot edit this file (protected path). USER needs to update manually.

Will do manually, after you have fully update that file finally

---

### 5. TODO.md Outdated Status

**Line 6**: "Phase 1.5.2 in progress"

**Should be**: "Phase 1.5.2 complete, Phase 1.5.3 next"

yes these are details

---

### 6. Phase 5 Needs Complete Rewrite

The entire `phase-5-integration.md` is written assuming feature flags exist. Key issues:

| Section | Issue |
|---------|-------|
| Step 5.1 Target State | Uses `#[cfg(feature = "canvas-2d")]` and `#[cfg(not(...))]` |
| Step 5.3 IPC | Uses `#[cfg(feature = "canvas-2d")]` |
| Post-Launch | Says "Consider removing feature flag" |

**Recommendation**: Rewrite Phase 5 to assume workspaces are already gone (Phase 1.5.3-4 does the replacement).

The recommendation is correct. but we don't need to invent new tasks that might drift again later

---

## UNANSWERED QUESTIONS FROM TEAM_008

### Q1: Row Module Testing
> "Should we add unit tests for Row before proceeding?"

**Status**: Not answered. Row still has no dedicated unit tests.

**Question for USER**: Is this acceptable? Current test coverage:
- 251 integration tests (test ScrollingSpace, not Row)
- 58 golden tests (test ScrollingSpace, compare to Row output)
- Row-specific tests: 0

We definitely need good testing.. but not duplicated testing and we need to look forward on what to test. if we need testing.. please make a new file in phases to include writing tests.

---

### Q2: Canvas2D Module Structure
> "Should Canvas2D be split into submodules like Row was?"

**Status**: ✅ RESOLVED by TEAM_008 refactoring. Canvas2D is now:
```
canvas/
├── mod.rs        (243 lines)
├── navigation.rs (91 lines)
├── operations.rs (103 lines)
├── render.rs     (85 lines)
└── floating.rs   (142 lines)
```

Already done... sent a gift to TEAM_008

---

### Q3: Row vs ScrollingSpace API Parity
> "Are animation config differences intentional simplifications?"

**Status**: Not answered.

**Question for USER**: Row methods use `self.options.animations.*` directly instead of taking `anim_config: Option<Animation>` parameters. Is this:
- A) Intentional simplification (keep it)
- B) Should match ScrollingSpace for future compatibility

Noooooooo there is NO simplification intention for animations... I feel like animations are being treated as an afterthought.. I don't see any animations in the planning for the refactor. and that actually bothers me.. I think the root of the lack of animation planning is because we don't know how to snapshot animations... This is a major problem and gap in refactoring that needs to be addressed individually.. but this must be at least documented

---

### Q4: FloatingSpace Integration Priority
> "Is FloatingSpace critical for the MVP?"

**Status**: ✅ RESOLVED. TEAM_009 integrated FloatingSpace into Canvas2D.

Thanks TEAM_009, please send them a gift

---

### Q5: Animation TODOs (TEAM_006)
> "Is column animation during add/remove important for Phase 1.5?"

**Status**: Not answered. These TODOs still exist:
- `row/operations/add.rs:157`: Animate movement of other columns
- `row/operations/move_col.rs:48`: Animate column movement

**Question for USER**: Should these be:
- A) Done now (before Monitor integration)
- B) Deferred to Phase 4 (Navigation + Polish)
- C) Low priority, defer indefinitely

- D) Make it an emergency that we've skipped animation planning.... <-

---

## CODE TODOs FOUND

### Active TODOs in canvas/

| File | Line | TODO |
|------|------|------|
| `render.rs:25` | TODO(TEAM_007) | Apply camera offset to render elements (Phase 3) |
| `floating.rs:126` | TODO(TEAM_009) | Add close animation for tiled windows in rows |
| `navigation.rs:79` | TODO(TEAM_007) | Add vertical_view_movement config (Phase 3) |

### Active TODOs in row/

| File | Line | TODO |
|------|------|------|
| `operations/add.rs:157` | TODO(TEAM_006) | Animate movement of other columns |
| `operations/move_col.rs:48` | TODO(TEAM_006) | Animate column movement |

### FIXMEs (lower priority)

| File | Line | FIXME |
|------|------|-------|
| `row/resize.rs:111` | FIXME | Smarter height distribution |
| `row/view_offset.rs:235` | FIXME | Compute and use current velocity |
| `row/operations/remove.rs:54` | FIXME | Tiles can move by X too |
| `row/operations/remove.rs:204` | FIXME | Preserve activate_prev_column_on_removal |

Are these TODO and FIXMEs still relevant? and are they documented in the global todo list??? please add them there!!!

---

## NEW QUESTIONS FOR CLARIFICATION

### Q6: Camera Offset in Render
The README keyboard shortcuts show:
- `Mod+Scroll` → Zoom in/out
- Camera position follows focus

But `canvas/render.rs` has a TODO to "Apply camera offset to render elements."

**Question for USER**: Is camera offset needed for Phase 1.5, or is it truly Phase 3 work? Currently:
- `camera_x` and `camera_y` fields exist in Canvas2D
- `camera_y` animates when changing rows
- But camera offset is NOT applied to render elements

- This is a really hard question for me right now... Let's think about it:... let's document this gap of information... We need a special files for unanswered questions... NO NO NO our current setup is good enough.... I can just ask you for unansered questions.. no need to organize it.

---

### Q7: Mod+1/2/3 Repurposing Timeline

`ai-teams-rules.md:245` says:
> "`Mod+1/2/3` repurposed for camera bookmarks"

But this requires:
1. Removing workspace switching (Phase 1.5.3)
2. Implementing camera bookmarks (Phase 5.4)

**Question for USER**: Should `Mod+1/2/3` be:
- A) Disabled entirely after Phase 1.5.3 (until bookmarks implemented)
- B) Temporarily repurposed for `focus-row N` (as Phase 5 suggests)
- C) Something else?

A Disable entirely... because new shortkeys should be discovered during daily use...

---

### Q8: Overview Mode

The questionnaire (line 117) says:
> "Niri already has an overview mode... I want to completely replace that with this 2D thing"

**Question for USER**: When workspaces are removed in Phase 1.5.3:
- A) Overview mode breaks (acceptable, will fix in Phase 5.2)
- B) Overview mode should still work (show zoomed-out canvas)
- C) Overview mode should be removed entirely in Phase 1.5.3

C - We should remove it entirely... and we have to take the long way to fully replace it... so we need to plan it correctly.. we need to be able to test even with the broken code during replacement. --- this also needs to be planned... please make a list of things that need to be planned. in a follow up questionaire.

---

### Q9: Monitor.workspaces Removal Scope

Phase 1.5.3 says "Remove `workspaces: Vec<Workspace<W>>` from Monitor"

**Question for USER**: What about:
- `active_workspace_idx` — also remove?
- `workspace_switch_*` methods — also remove?
- Workspace-related animations — also remove?
- Workspace-related IPC — return error? empty?

Yes All remove... we don't have workspaces anymore... it's going to be a hard sweep... also NO MORE hot corner in the top right... that also activates the overview... remove that too. Yeah look for this specifiaclly

---

### Q10: Row Span vs Camera Zoom Priority

The questionnaire priorities:
1. Zoom out to see more windows
2. 2D navigation
3. Per-window size control
4. Origin-based RTL/LTR behavior
5. Smooth animations

Row spanning is Phase 2, Camera zoom is Phase 3.

**Question for USER**: Given zoom is priority #1, should Phase 3 (Camera) come before Phase 2 (Row Spanning)?

This question should be answered by doing forward looking plannen what is best to implement in which order... I don't want features to work in order.. I want the perfect implementation in the perfect order.. do it the long way

---

## RECOMMENDED ACTIONS

### Immediate (USER Must Do)

1. **Update `ai-teams-rules.md`** — I cannot edit this protected file
   - Line 240: Change "then feature flag" to "then replace Workspace"
   - Line 255: Remove or update feature flag reference
   - Lines 238-240: Update current status

   Yes please update the ai-teams-rules.md in the editable version one last time.. then I will copy it again.

2. **Answer questions above** — Especially Q1, Q3, Q5, Q6, Q7, Q8, Q9, Q10

### For Next Team (TEAM_010)

1. **Rewrite `phase-5-integration.md`** — Remove all feature flag references
2. **Update `TODO.md`** — Mark Phase 1.5.2 complete
3. **Begin Phase 1.5.3** — Replace Workspace in Monitor

---

*Created by TEAM_009 after completing Phase 1.5.2 FloatingSpace integration*
