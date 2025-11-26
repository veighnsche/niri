# AI Team Rules — 2D Canvas Refactor

> **READ THIS FIRST. FOLLOW STRICTLY.**

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
7. **Run** `cargo insta test` — golden tests must pass BEFORE you start
8. **Then** start coding

---

## Rule 4: Golden Snapshot Testing

**Purpose**: Prevent behavioral regressions. Refactored code must produce identical layout positions as original main branch code.

### ⚠️ MANDATORY: If You Touch Layout Logic

**Before ANY refactor that touches these files, you MUST:**
1. Run `./scripts/verify-golden.sh` — verify permissions and tests pass BEFORE your changes
2. Make your changes
3. Run `cargo insta test` again — verify they STILL pass
4. If tests fail → you introduced a regression, fix it

### Key Files
- `src/layout/snapshot.rs` — Snapshot types (positions, indices, etc.)
- `src/layout/tests/golden.rs` — Snapshot comparison tests
- `src/layout/tests/snapshots/*.snap` — Locked baseline snapshots (chmod 444)
- `scripts/verify-golden.sh` — Verification script

> ⚠️ Snapshot files are chmod 444 (read-only). **NEVER modify them.**
> Source commit: `75d5e3b0` — use `git show 75d5e3b0:src/layout/scrolling.rs` to see original code

### What Snapshots Capture
- Column X positions and widths
- Tile bounds (x, y, w, h)
- Active indices (column, tile)
- View offset (camera position)
- Working area

### Workflow
1. Golden code and refactored code both implement `snapshot()` → same types
2. Tests compare outputs using `insta` (`cargo insta test`)
3. If snapshots differ → regression found, fix before proceeding
4. Use `cargo insta review` to inspect diffs (never accept without USER approval)

### Adding New Features?
- New features won't have golden tests (original code didn't have them)
- But existing behavior MUST still match golden snapshots
- Create new test scenarios in `tests/golden.rs` for new features

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
4. **Run** golden tests: `cargo insta test` — if touching layout logic
5. **Document** any failures or blockers
6. **Write** clear handoff notes for next team

### Team File Handoff Checklist
```markdown
## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo insta test`) — if touching layout logic
- [ ] Team file complete
```

---

## Quick Reference

| Task | Location |
|------|----------|
| Master plan | `README.md` |
| Current phase | `phases/phase-X-*.md` |
| Team logs | `.teams/TEAM_XXX_*.md` |
| Questions for USER | `.questions/TEAM_XXX_*.md` |
| Golden snapshots | `src/layout/tests/snapshots/*.snap` |
| Snapshot types | `src/layout/snapshot.rs` |
| Verification script | `scripts/verify-golden.sh` |
| Your code comments | `// TEAM_XXX: ...` |

---

## Current Project State

**Branch**: `2d-canvas`  
**Phase**: 0 COMPLETE → Next: Phase 1 (Row + Canvas2D)  
**Completed**: Phase 0.1, 0.2, 0.3, 0.5 (all Phase 0 steps)  
**Next Step**: `phases/phase-1-row-and-canvas.md`

**Key Decisions (TEAM_004)**:
- Workspaces **removed** — one infinite canvas per output
- `Mod+Up/Down` uses geometric navigation (crosses rows)
- `Mod+1/2/3` repurposed for camera bookmarks
- Always enabled (breaking change, no opt-in)

**TEAM_005 Additions**:
- `AnimatedValue` abstraction in `src/layout/animated_value/`
- `AnimatedPoint` ready for Camera (Phase 3)

---

*Rules established by TEAM_000. Updated by TEAM_005.*
