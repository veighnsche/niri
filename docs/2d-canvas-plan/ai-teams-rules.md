# AI Team Rules — 2D Canvas Refactor

> **READ THIS FIRST. FOLLOW STRICTLY.**

---

## Rule 0: Quality Over Speed

**Always take the correct approach, never the quick shortcut.**

- If the plan recommends Option B (clean slate), do Option B
- If a proper refactor requires more work, do the work
- Never choose "faster to implement" over "architecturally correct"
- Wrappers and indirection layers are technical debt — avoid them
- Future teams will inherit your decisions — leave them clean code

**Good > Quick. Always.**

---

## Rule 1: Canonical Source

`/home/vince/Projects/niri/docs/2d-canvas-plan/` is the **ONLY** source of truth.
- All plans live here
- All team logs live here
- Edit nowhere else for planning/coordination

---

## Rule 2: Team Registration

Every new AI conversation = new team number.

### Your Team Number
- Check `.teams/` folder for highest existing `TEAM_XXX`
- Your number = highest + 1
- **Once assigned, it's yours forever in this conversation**

### Your Team File
Create: `.teams/TEAM_XXX_three_word_summary.md`

### Code Comments
When you modify code, add your team number:
```rust
// TEAM_XXX: Brief explanation of change
```

---

## Rule 3: Before You Start

1. **Read** `docs/2d-canvas-plan/README.md`
2. **Read** the current phase file in `phases/`
3. **Check** `.teams/` for recent team files
4. **Check** `.questions/` for any unanswered questions
5. **Claim** your team number
6. **Create** your team file
7. **Run** `cargo xtask test-all golden` — golden tests must pass BEFORE you start
8. **Then** start coding

---

## Rule 4: Golden Snapshot Testing

**Purpose**: Prevent behavioral regressions. Refactored code must produce identical layout positions as original main branch code.

### 🚫 NEVER RUN `cargo insta accept` ON GOLDEN TESTS 🚫

Golden snapshots come from the **`golden-snapshots` branch**, NOT this branch.
If tests fail, your code is wrong — fix it, don't accept new snapshots.

### Provenance

| Item | Location |
|------|----------|
| **Source branch** | `golden-snapshots` (derived from `main`) |
| **Sync command** | `cargo xtask golden-sync pull` |
| **Generate command** | `cargo xtask golden-sync generate` |
| **xtask source** | `xtask/src/golden_sync/mod.rs` |

### If You Need to Change Snapshot Behavior

1. Make the change on `golden-snapshots` branch FIRST
2. Run `cargo xtask golden-sync pull` to sync to this branch
3. Never modify `.snap` files directly on refactor branches

### ⚠️ MANDATORY: If You Touch Layout Logic

**Before ANY refactor that touches layout files, you MUST:**
1. Run `cargo xtask test-all golden` — verify tests pass BEFORE your changes
2. Make your changes
3. Run `cargo xtask test-all golden` — verify they STILL pass
4. If tests fail → you introduced a regression, fix it

### Key Files
- `src/layout/snapshot.rs` — Snapshot types (positions, indices, etc.)
- `src/layout/tests/golden.rs` — Snapshot comparison tests
- `src/layout/tests/snapshots/*.snap` — Baseline snapshots (from `golden-snapshots` branch)
- `xtask/src/test_all/mod.rs` — Test runner with golden verification
- `xtask/src/golden_sync/mod.rs` — Golden sync tool

### What Snapshots Capture
- Column X positions and widths
- Tile bounds (x, y, w, h)
- Active indices (column, tile)
- View offset (camera position)
- Working area

### Workflow
1. Golden code and refactored code both implement `snapshot()` → same types
2. Tests compare outputs using `insta` (`cargo test --lib golden`)
3. If snapshots differ → regression found, fix before proceeding
4. Use `cargo insta review` to inspect diffs — **NEVER accept on golden tests**
5. Run `cargo xtask golden-sync clean` to remove `.snap.new` files

### Adding New Features?
- New features won't have golden tests (original code didn't have them)
- But existing behavior MUST still match golden snapshots
- To add new golden tests: add them on `golden-snapshots` branch, then sync

---

## Rule 5: Breaking Changes > Backwards Compatibility

**Breaking changes = temporary pain** (compiler shows call sites to fix).  
**Backwards compatibility = permanent debt.**

Be bold. Break things. Fix them properly.

### The Process
1. Move the type/function to its new location
2. **Let the compiler fail** — it will show all import sites
3. Fix each import site directly
4. Delete any temporary re-exports

**If you find yourself writing `pub use` to "keep things working" — STOP. Fix the imports instead.**

---

## Rule 6: No Dead Code

**Always remove dead code and unused files.**

- Delete unused functions, structs, modules
- Delete files that aren't wired up to anything
- Delete commented-out code blocks
- If code exists "for reference only" — delete it, use git history instead

**The codebase should only contain code that compiles and runs.**

---

## Rule 7: Modular Refactoring

**Goal**: Break monolithic files into small, focused modules.

### Principles
- Each module **owns its state** (private fields, public methods)
- No `pub(super)` — if external code needs it, make a proper getter
- No deep imports (`super::super::`) — restructure instead
- Small files (< 500 lines ideal, < 1000 max)

---

## Rule 8: Ask Questions

**When uncertain, blocked, or plans don't add up**: ask the USER.

Create a question file: `.questions/TEAM_XXX_topic.md`

**Don't guess on important decisions. Ask.**

---

## Rule 9: Maximize Context Window

**Do as much work as possible within your context window.**

- Don't stop after one task if you can continue
- You already have the context loaded — use it
- Next team will have to re-gather all context from scratch
- Only stop when: context runs out, blocked, or need USER input

### Task Splitting
If a single task takes > 1 hour or touches > 3 files: split it into sub-task files in `.teams/`

---

## Rule 10: Before You Finish

1. **Update** your team file with all changes
2. **Verify** code compiles: `cargo check`
3. **Run** tests: `cargo test`
4. **Run** golden tests: `cargo xtask test-all golden` — if touching layout logic
5. **Document** any failures or blockers
6. **Write** clear handoff notes for next team

### Team File Handoff Checklist
```markdown
## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo xtask test-all golden`) — if touching layout logic
- [ ] Team file complete
```

---

## Rule 11: TODO Tracking

**All incomplete work MUST be clearly marked and tracked.**

### In Code
Use this exact format for searchability:
```rust
// TODO(TEAM_XXX): Brief description of what needs to be done
```

### Global TODO List
Before finishing, run: `grep -rn "TODO(TEAM" src/layout/`

Add any new TODOs to: `docs/2d-canvas-plan/TODO.md`

### TODO.md Format
```markdown
## src/layout/row/mod.rs
- [ ] TODO(TEAM_006): Port add_window from ScrollingSpace (line 440)
- [ ] TODO(TEAM_006): Port remove_window from ScrollingSpace (line 441)
```

**Future teams check TODO.md first** to see planned feature locations.

---

## Quick Reference

| Task | Location |
|------|----------|
| Master plan | `README.md` |
| Current phase | `phases/phase-X-*.md` |
| Team logs | `.teams/TEAM_XXX_*.md` |
| Questions for USER | `.questions/TEAM_XXX_*.md` |
| **Global TODOs** | `TODO.md` |
| **Golden snapshots** | `src/layout/tests/snapshots/*.snap` |
| **Golden source branch** | `golden-snapshots` (NEVER accept on other branches!) |
| **Golden sync tool** | `cargo xtask golden-sync` |
| **Golden verification** | `cargo xtask test-all golden` |
| **Run all tests** | `cargo xtask test-all run` |
| Your code comments | `// TEAM_XXX: ...` |

---

## Current Project State

**Branch**: `2d-canvas`  
**Phase**: 1.5 IN PROGRESS (Integration)  
**Completed**: Phase 0, Phase 1 Core, Phase 1.5.1 (Row), **Phase 1.5.2 (Canvas2D + FloatingSpace)**  
**Next Step**: Phase 1.5.3 (Replace Workspace in Monitor — BREAKING CHANGE)

**Key Decisions**:
- Workspaces **removed** — one infinite canvas per output
- `Mod+Up/Down` uses geometric navigation (crosses rows)
- `Mod+1/2/3` repurposed for camera bookmarks
- Row owns columns directly (Option B, not wrapper)

---

## Lessons Learned (from TEAM_000 → TEAM_008)

### 1. Phase Sizing
Large phases should be split. Phase 1 became Phase 1 + Phase 1.5 because:
- Creating modules is different from wiring them into the compositor
- Workspace replacement is significant work on its own
- Each phase should have a clear "done" state

### 2. Module Structure
Follow the Column module pattern:
```
module/
├── mod.rs          (~150-300 lines) - Core struct + re-exports
├── operations.rs   (~100-200 lines) - Add/remove/move
├── navigation.rs   (~50-100 lines)  - Focus methods
├── layout.rs       (~50-100 lines)  - Position queries
├── render.rs       (~150-200 lines) - Rendering
└── view_offset.rs  (if needed)      - Scroll/animation
```
Keep files < 500 lines (ideal), < 1000 lines (max).

### 3. Porting Strategy
When porting from `scrolling.rs`:
1. **Read the source** — understand what the method does
2. **Identify dependencies** — what other methods does it call?
3. **Port helpers first** — bottom-up, not top-down
4. **Test incrementally** — run `cargo check` after each method

### 4. Documentation Maintenance
- Update README.md progress tracking **during** work, not after
- Mark team files as COMPLETE when done
- Phase files should reflect actual module structure

### 5. Golden Tests
Always run `cargo xtask test-all golden` before AND after changes to layout logic.
The golden tests catch regressions that unit tests miss.

### 6. Refactor Large Files Properly (TEAM_008)
When a file exceeds 500 lines, **refactor it into submodules**:

```rust
// WRONG: "The 500-line guideline isn't always achievable"
// RIGHT: Split into submodules using the idiomatic Rust pattern

// Before: operations.rs (692 lines - TOO BIG!)
// After:
operations/
├── mod.rs      (22 lines)  - Re-exports
├── add.rs      (159 lines) - Add tile/column
├── remove.rs   (246 lines) - Remove tile/column
├── move_col.rs (50 lines)  - Move column left/right
└── consume.rs  (250 lines) - Consume/expel window
```

The pattern:
1. Create a directory with the module name
2. Create submodules for each logical grouping
3. Each submodule has its own `impl<W: LayoutElement> Row<W>` block
4. The `mod.rs` just declares the submodules — no re-exports needed for impl blocks

**Never change the rules because you couldn't follow them. Refactor properly.**

### 7. Ask Questions Early (TEAM_008)
Create `.questions/TEAM_XXX_*.md` files for:
- Architectural decisions that affect future phases
- API design choices (e.g., should Row match ScrollingSpace exactly?)
- Priority questions (e.g., is FloatingSpace critical for MVP?)

### 8. Animation = Preservation, Not Implementation (TEAM_009)
**Critical misunderstanding to avoid**: Animations already exist in the main branch. Refactoring means:
- ❌ NOT implementing new animations
- ✅ Ensuring existing animations still trigger in the same scenarios
- ✅ Using the animation regression checklist to verify no regressions

### 9. Test Before Breaking (TEAM_009)
When doing breaking changes:
1. Port existing tests to new code BEFORE removing old code
2. Verify tests pass on new code
3. THEN remove old code
4. Don't accumulate breakage — fix failures immediately

### 10. Document Removal Scope (TEAM_009)
Before large removals (like workspace elimination):
1. Create detailed removal checklist
2. Document correct removal order (dependents before dependencies)
3. Document what breaks and when it will be fixed
4. Reference: `phases/phase-1.5.3-removal-checklist.md`

### 11. Read the MASTERPLAN (TEAM_009)
Before starting any work:
1. Read `MASTERPLAN.md` first — it's the single source of truth
2. Check pre-phase requirements (testing, animation audit)
3. Don't skip blockers

---

*Rules established by TEAM_000. Updated by TEAM_009.*
