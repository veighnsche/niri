# TEAM_009: Follow-up Questionnaire — Planning Gaps

> **Purpose**: Address the major planning gaps identified in the contradiction sweep.
> **Priority**: These need answers before Phase 1.5.3 begins.

---

## GAP 1: Animation Planning ⚠️ EMERGENCY

**The Problem**: Animations are being treated as an afterthought. There's no dedicated phase or plan for animation work, yet animations are core to niri's UX.

### Current State
- Golden tests snapshot **positions**, not **animations**
- We cannot verify animation correctness during refactoring
- Multiple animation TODOs are scattered and untracked
- Row methods simplified animation config parameters (intentionally or not?)

### Animation TODOs Found

| Location | Description |
|----------|-------------|
| `row/operations/add.rs:157` | Animate movement of other columns |
| `row/operations/move_col.rs:48` | Animate column movement |
| `canvas/floating.rs:126` | Add close animation for tiled windows |
| `canvas/render.rs:25` | Apply camera offset (affects animation rendering) |
| `canvas/navigation.rs:79` | Add vertical_view_movement config |

### Questions

**Q1.1**: How do we test animations during refactoring?
- A) Visual inspection only (current approach)
- B) Create animation snapshot tests (capture keyframes?) <-
- C) Record videos and compare <-
- D) Something else?

**Q1.2**: Should there be a dedicated "Animation Phase" or "Animation Audit"?
- A) Yes, add Phase 1.6 or Phase 4.5 for animation work
- B) No, handle animations within existing phases <-
- C) Create a parallel animation checklist that runs alongside all phases <-

**Q1.3**: What animations are critical for MVP (Phase 1.5.3)?
- Window open/close animations
- Focus change animations
- View offset (camera follow) animations
- Column add/remove animations
- Row change animations
- Other: All animations should ALREADY EXIST in the current main branch.. WE JUST NEED TO MAKE SURE THAT WE DON'T REGRESS!!!!! WE DON'T IMPLEMENT THE ANIMATIONS!!!!!! BIG MISUNDERSTANDING ON YOUR PART!!

---

## GAP 2: Testing Strategy

**The Problem**: Row and Canvas2D have no dedicated unit tests. We rely on:
- 251 integration tests (test ScrollingSpace)
- 58 golden tests (compare ScrollingSpace vs Row output)

### Questions

**Q2.1**: When should Row/Canvas2D tests be written?
- A) Before Phase 1.5.3 (block on tests) <-
- B) During Phase 1.5.3 (test as we integrate)
- C) After Phase 1.5.4 (test the integrated system)
- D) Create a dedicated testing phase

**Q2.2**: What should the test strategy be?
- A) Port existing ScrollingSpace tests to Row/Canvas2D <-
- B) Write new tests focused on 2D-specific behavior <-
- C) Both
- D) Focus on integration tests, skip unit tests

**Q2.3**: Should we create a `phases/phase-testing.md` file?
- A) Yes, dedicated testing phase document
- B) No, add testing sections to existing phase docs <-

---

## GAP 3: Workspace Removal Scope

**The Problem**: Phase 1.5.3 says "Remove workspaces" but the full scope is unclear.

### Things to Remove (Confirmed by USER)

| Item | Location | Status |
|------|----------|--------|
| `workspaces: Vec<Workspace<W>>` | Monitor | To remove |
| `active_workspace_idx` | Monitor | To remove |
| `workspace_switch_*` methods | Monitor | To remove |
| Workspace-related animations | Monitor | To remove |
| Workspace-related IPC | niri-ipc | To remove |
| Overview mode | Multiple | To remove |
| Hot corner (top right) | Input handling | To remove |
| `Mod+1/2/3` workspace switching | Keybinds | To disable |

### Questions

**Q3.1**: Should we create a detailed removal checklist?
- A) Yes, create `phases/phase-1.5.3-removal-checklist.md` <-
- B) No, just work through it during implementation

**Q3.2**: What about workspace-related config options?
```kdl
// Current config has:
workspaces {
    // ...
}
```
- A) Remove entirely (breaking config change) <-
- B) Keep but ignore (warn user)
- C) Error on startup if present <-

**Q3.3**: What about `niri msg workspaces` and related IPC?
- A) Return error "Workspaces not supported in 2D mode"
- B) Return empty response
- C) Remove the IPC endpoints entirely <-

---

## GAP 4: Overview Replacement

**The Problem**: Overview mode is being removed, but needs a replacement.

### Current Overview
- Triggered by keybind or hot corner
- Zooms out to show all workspaces
- Click to switch workspace

### 2D Replacement Vision
- Triggered by same keybind (no hot corner)
- Zooms out to show entire canvas
- Click to focus any tile
- Stays interactive while zoomed

### Questions

**Q4.1**: When should overview replacement be implemented?
- A) Phase 1.5.3 (remove old, add new together)
- B) Phase 3 (Camera System) — requires zoom
- C) Phase 5 (Integration) — as originally planned <-
- D) Create dedicated phase for overview replacement

**Q4.2**: Can we have a "broken" state where overview doesn't exist?
- A) Yes, acceptable during development <-
- B) No, must have some overview functionality at all times

**Q4.3**: What's the minimum viable overview?
- A) Just zoom out to fit all windows (no interaction)
- B) Zoom out + click to focus <-
- C) Full interactive overview with smooth zoom

---

## GAP 5: Phase Order Optimization

**The Problem**: Current phase order may not be optimal.

### Current Order
1. Phase 1.5.3-4: Replace Workspace, Monitor integration
2. Phase 2: Row Spanning
3. Phase 3: Camera System (zoom)
4. Phase 4: Navigation + Polish
5. Phase 5: Integration (IPC, docs)

### User Priorities (from questionnaire)
1. **Zoom out to see more windows** ← Phase 3
2. 2D navigation ← Phase 1.5 + 4
3. Per-window size control ← Phase 2?
4. Origin-based RTL/LTR behavior ← Phase 4
5. Smooth animations ← ???

### Questions

**Q5.1**: Should Camera (Phase 3) come before Row Spanning (Phase 2)?
- A) Yes, zoom is priority #1
- B) No, row spanning is architecturally simpler <-
- C) Do them in parallel
- D) Merge them into one phase

**Q5.2**: Should there be an Animation phase?
- A) Yes, add Phase 1.6 (Animation Audit) before Phase 2
- B) Yes, add Phase 4.5 (Animation Polish) after Phase 4
- C) No, handle animations within each phase <-

**Q5.3**: What's the ideal phase order?
```
Current:  1.5.3 → 1.5.4 → 2 → 3 → 4 → 5
Option A: 1.5.3 → 1.5.4 → 3 → 2 → 4 → 5  (Camera before Row Span)
Option B: 1.5.3 → 1.5.4 → 1.6 → 2 → 3 → 4 → 5  (Add Animation Audit)
Option C: 1.5.3 → 1.5.4 → 3 → 2 → 4 → 1.6 → 5  (Animation at end)
Option D: Something else? <- Please think about this extensively what the best order is... we need to refactor the plan because clearly a lot of gaps that need to be filled is changing the the entire plan
```

---

## GAP 6: Things That Need Planning Documents

Based on the sweep, these items need dedicated planning:

| Item | Suggested Document |
|------|-------------------|
| Testing strategy | `phases/phase-testing.md` |
| Animation audit | `phases/phase-animation-audit.md` |
| Workspace removal checklist | `phases/phase-1.5.3-removal-checklist.md` |
| Overview replacement | Already in Phase 5, but needs detail |

### Questions

**Q6.1**: Which of these should be created now?
- [2] Testing strategy
- [1] Animation audit
- [3] Workspace removal checklist
- [ ] None, just proceed

**Q6.2**: Should phase-5-integration.md be rewritten now or later?
- A) Now (before Phase 1.5.3)
- B) After Phase 1.5.4 (when we know what's left) <-
- C) Just before Phase 5 starts

---

## Summary: Decisions Needed

| # | Question | Options |
|---|----------|---------|
| Q1.1 | How to test animations? | A/B/C/D |
| Q1.2 | Dedicated animation phase? | A/B/C |
| Q1.3 | Critical MVP animations? | List |
| Q2.1 | When to write tests? | A/B/C/D |
| Q2.2 | Test strategy? | A/B/C/D |
| Q2.3 | Create testing phase doc? | A/B |
| Q3.1 | Create removal checklist? | A/B |
| Q3.2 | Workspace config handling? | A/B/C |
| Q3.3 | Workspace IPC handling? | A/B/C |
| Q4.1 | When to implement overview? | A/B/C/D |
| Q4.2 | Allow broken overview state? | A/B |
| Q4.3 | Minimum viable overview? | A/B/C |
| Q5.1 | Camera before Row Span? | A/B/C/D |
| Q5.2 | Add Animation phase? | A/B/C |
| Q5.3 | Ideal phase order? | A/B/C/D |
| Q6.1 | Which docs to create now? | Checkboxes |
| Q6.2 | When to rewrite Phase 5? | A/B/C |

---

*Created by TEAM_009 — Alignment Bot Mode*
