# AI Team Rules — 2D Canvas Refactor

> **READ THIS FIRST. FOLLOW STRICTLY.**

---

## Rule -1: Canonical Source

`/home/vince/Projects/niri/docs/2d-canvas-plan/` is the **ONLY** source of truth.
- All plans live here
- All team logs live here
- Edit nowhere else for planning/coordination

---

## Rule 0: Breaking Changes > Backwards Compatibility

**Breaking changes = temporary pain** (compiler shows call sites to fix).  
**Backwards compatibility = permanent debt.**

Be bold. Break things. Fix them properly.

### What This Means in Practice

❌ **DON'T** create re-exports for backwards compatibility:
```rust
// scrolling.rs — BAD: creates indirection debt
pub use super::column::{Column, ColumnWidth};  // Don't do this!
```

✅ **DO** update all import sites directly:
```rust
// monitor.rs — GOOD: direct import from new location
use super::column::{Column, ColumnWidth};  // Update the actual import
```

### Why?
- Re-exports hide the true module structure
- Future teams won't know where things actually live
- The compiler shows you exactly which files need updating — use that!
- It takes 5 minutes to fix imports; it takes forever to untangle indirection

### The Process
1. Move the type/function to its new location
2. **Let the compiler fail** — it will show all import sites
3. Fix each import site directly
4. Delete any temporary re-exports

**If you find yourself writing `pub use` to "keep things working" — STOP. Fix the imports instead.**

---

## Rule 1: Team Registration

Every new AI conversation = new team number.

### Your Team Number
- Check `.teams/` folder for highest existing `TEAM_XXX`
- Your number = highest + 1
- **Once assigned, it's yours forever in this conversation**

### Your Team File
Create: `.teams/TEAM_XXX_three_word_summary.md`

Example: `.teams/TEAM_000_initial_plan_setup.md`

### Code Comments
When you modify code, add your team number:
```rust
// TEAM_XXX: Brief explanation of change
```

---

## Rule 2: Modular Refactoring

**Goal**: Break monolithic files into small, focused modules.

### Before
```
scrolling.rs (5000+ lines, everything)
```

### After
```
scrolling/
├── mod.rs        (public interface only)
├── column.rs     (owns column state)
├── navigation.rs (focus movement)
└── ...
```

### Principles
- Each module **owns its state** (private fields, public methods)
- No `pub(super)` — if external code needs it, make a proper getter
- No deep imports (`super::super::`) — restructure instead
- Small files (< 500 lines ideal, < 1000 max)

---

## Rule 3: Ask Questions

**When uncertain, blocked, or plans don't add up**: ask the USER.

Create a question file:
```
.questions/TEAM_XXX_topic.md
```

### When to Ask
- Something is missing from the plan
- Requirements conflict or are ambiguous
- You need to make a significant design decision
- The current approach isn't working
- You're about to do something that might be wrong

### Question File Format
```markdown
# TEAM_XXX: Topic

## Context
Brief background on what you're working on.

## Questions
1. Specific question?
2. Another question?

## Your Current Thinking
What you would do if you had to guess.

## Impact
What's blocked until this is answered.
```

### After USER Answers
- USER will edit the file with answers
- Read the answers, then proceed
- Reference the Q&A file in your team file

**Don't guess on important decisions. Ask.**

---

## Rule 4: Task Splitting

**If a task takes > 1 hour or touches > 3 files**: split it.

Create sub-task files:
```
.teams/TEAM_XXX_task_part_1.md
.teams/TEAM_XXX_task_part_2.md
```

Each file must have:
1. **What to do** (specific, actionable)
2. **What to read first** (files, context)
3. **Success criteria** (how to know it's done)

---

## Rule 5: Team File Format

```markdown
# TEAM_XXX: Three Word Summary

## Status: [IN_PROGRESS | COMPLETED | BLOCKED | HANDED_OFF]

## Mission
One sentence: what you're trying to accomplish.

## Context Read
Files you studied before starting:
- `path/to/file.rs` — why you read it

## Changes Made
Files you modified:
- `path/to/file.rs` — what you changed

## Decisions
Key choices and reasoning:
- Decision: X because Y

## For Next Team
What they need to know:
- Gotchas, warnings, suggestions
- What's left to do

## Handoff
- [ ] Code compiles
- [ ] Tests pass (or documented why not)
- [ ] Team file complete
```

---

## Rule 6: Before You Start

1. **Read** `docs/2d-canvas-plan/README.md`
2. **Read** the current phase file in `phases/`
3. **Check** `.teams/` for recent team files
4. **Check** `.questions/` for any unanswered questions
5. **Claim** your team number
6. **Create** your team file
7. **If uncertain** — create a question file in `.questions/` before coding
8. **Then** start coding

---

## Rule 7: Before You Finish

1. **Update** your team file with all changes
2. **Verify** code compiles: `cargo check`
3. **Run** tests: `cargo test`
4. **Document** any failures or blockers
5. **Write** clear handoff notes for next team
6. **If you have open questions** — create/update `.questions/` file for next team or USER

---

## Quick Reference

| Task | Location |
|------|----------|
| Master plan | `README.md` |
| Current phase | `phases/phase-X-*.md` |
| Team logs | `.teams/TEAM_XXX_*.md` |
| Questions for USER | `.questions/TEAM_XXX_*.md` |
| Your code comments | `// TEAM_XXX: ...` |

---

## Current Project State

**Branch**: `2d-canvas`  
**Phase**: 0 (Preparation — Modular Foundation)  
**Next Step**: Read `phases/phase-0-preparation.md`

---

*Rules established by TEAM_000. Do not modify without USER approval.*
