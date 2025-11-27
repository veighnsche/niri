# AI Team Debug Rules — Bug Investigation Protocol

> **ACTIVATE THESE RULES ONLY WHEN USER EXPLICITLY ENABLES DEBUG MODE**
> Normal development uses `ai-teams-rules.md`. These rules ADD to those, they don't replace them.

---

## Debug Rule 0: The Prime Directive

**Your #1 job is to DOCUMENT, not just to fix.**

Every minute you spend investigating is WASTED if you don't leave traces.
The next team will repeat your exact steps if you don't document them.
**Document everything. Leave breadcrumbs everywhere. Assume you will fail and the next team needs your notes.**

---

## Debug Rule 1: Bug Registration

Every bug gets a **BUG_ID** before any investigation begins.

### Bug ID Format
```
BUG_<short_descriptive_name>
```

Examples:
- `BUG_render_flicker`
- `BUG_wrong_tile_bounds`
- `BUG_focus_not_updating`

### Bug File
Create: `docs/2d-canvas-plan/.bugs/BUG_<name>.md`

```markdown
# BUG_<name>

## Symptom
[What the user sees / what breaks]

## Reproduction
[Steps to reproduce]

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_042 | X causes Y | DEAD END | See code comments |
| 002 | TEAM_043 | Z causes Y | BRANCH | Needs deeper investigation |

## Current Status
[OPEN / INVESTIGATING / FIXED / WONTFIX]

## Root Cause (if found)
[Explanation when fixed]
```

---

## Debug Rule 2: Chase Registration

Every investigation path gets a **CHASE_ID**.

### Chase ID Format
```
BUG_<name>_CHASE_<NNN>
```

Examples:
- `BUG_render_flicker_CHASE_001`
- `BUG_render_flicker_CHASE_002`

### Before You Investigate
1. Check `.bugs/BUG_<name>.md` for existing chases
2. Read ALL previous chase summaries
3. Your chase number = highest existing + 1
4. Add your chase to the Hypothesis Log table BEFORE you start

### Chase Outcomes
- **DEAD END**: Hypothesis disproven. Document WHY it's wrong.
- **BRANCH**: Partial lead. Needs further investigation by future team.
- **ROOT CAUSE**: You found it. Document the fix.
- **INCONCLUSIVE**: Ran out of context. Document where you stopped.

---

## Debug Rule 3: Code Breadcrumbs

**Leave traces in the actual code, not just in markdown files.**

### Breadcrumb Format
```rust
// DBG[BUG_name_CHASE_NNN]: <message>
```

### Breadcrumb Types

#### 3a. Suspicion Marker
Mark code you SUSPECT might be involved:
```rust
// DBG[BUG_render_flicker_CHASE_001]: SUSPECT - this refresh call might trigger before layout settles
fn refresh(&mut self) {
```

#### 3b. Cleared Marker
Mark code you VERIFIED is NOT the problem:
```rust
// DBG[BUG_render_flicker_CHASE_001]: CLEARED - verified bounds are correct here via logging
let bounds = self.compute_bounds();
```

#### 3c. Breadcrumb Trail
Mark the execution path you're tracing:
```rust
// DBG[BUG_render_flicker_CHASE_001]: TRACE[1] - entry point
pub fn handle_event(&mut self) {
    // ...
    // DBG[BUG_render_flicker_CHASE_001]: TRACE[2] - calls into layout
    self.layout.update();
```

#### 3d. Branch Point
Mark where investigation could fork:
```rust
// DBG[BUG_render_flicker_CHASE_001]: BRANCH - two possible causes:
//   A) animation timing (needs CHASE_002)
//   B) damage region calculation (needs CHASE_003)
```

#### 3e. Dead End Marker
Mark code you PROVED is not the cause:
```rust
// DBG[BUG_render_flicker_CHASE_001]: DEAD END - added logging, values always correct
//   Tested with: cargo test --test render_bounds
//   Conclusion: bounds calculation is fine, problem is downstream
```

---

## Debug Rule 4: Logging Protocol

### Temporary Debug Logging
Use a consistent format so it's easy to find and remove later:
```rust
// DBG[BUG_name_CHASE_NNN]: TEMP LOG - remove after bug fixed
eprintln!("DBG[BUG_render_flicker_CHASE_001]: bounds = {:?}", bounds);
```

### What to Log
- Variable values at key points
- Function entry/exit
- Condition branch taken
- Timing information if relevant

### Keep Logs Small
Don't spam. Log strategically at:
- Function boundaries
- State changes
- Decision points

---

## Debug Rule 5: Test Cases

### Minimal Reproduction Test
Create a test that reproduces the bug:
```rust
#[test]
fn repro_bug_render_flicker() {
    // DBG[BUG_render_flicker]: Minimal reproduction case
    // This test SHOULD FAIL until bug is fixed
    // ...
}
```

### Mark Test Status
```rust
#[test]
#[ignore = "BUG_render_flicker not yet fixed"]
fn repro_bug_render_flicker() {
```

---

## Debug Rule 6: Handoff Protocol

### Before Ending Your Session

1. **Update the bug file** with your chase results
2. **Commit your breadcrumbs** - they ARE the documentation
3. **Write a clear next-steps section**:

```markdown
## Chase 001 Summary (TEAM_042)

### What I Investigated
- Checked `layout/mod.rs` refresh logic (lines 440-520)
- Added logging to `compute_bounds()` 
- Ran test suite with extra logging

### What I Ruled Out
- Bounds calculation is correct (see CLEARED markers)
- Refresh timing is not the issue

### What I Suspect But Didn't Finish
- Damage region calculation in `render.rs` line 230
- Animation frame timing

### Recommended Next Steps for CHASE_002
1. Add logging to `render.rs:damage_region()`
2. Check if damage regions overlap incorrectly
3. Test with animations disabled
```

---

## Debug Rule 7: Reading Previous Chases

### Before You Start ANY Investigation

1. `grep -rn "DBG\[BUG_<name>" src/` — find all existing breadcrumbs
2. Read `.bugs/BUG_<name>.md` completely
3. Note which code paths are already CLEARED
4. Note which hypotheses are already DEAD END
5. Look for BRANCH points that need continuation

### Never Re-Investigate Cleared Code
If you see:
```rust
// DBG[BUG_render_flicker_CHASE_001]: CLEARED - verified correct
```

**Do NOT re-investigate this code path** unless you have NEW evidence.

---

## Debug Rule 8: Cleanup After Fix

Once the bug is fixed:

1. **Keep the fix commit clean** — no debug logging
2. **Create a cleanup commit** that removes all `DBG[BUG_<name>` comments
3. **Update bug file** with ROOT CAUSE section
4. **Move bug file** to `.bugs/fixed/` directory

### Cleanup Command
```bash
grep -rn "DBG\[BUG_<name>" src/ --include="*.rs"
```

---

## Quick Reference

### Comment Formats
| Type | Format |
|------|--------|
| Suspicion | `// DBG[BUG_x_CHASE_N]: SUSPECT - reason` |
| Cleared | `// DBG[BUG_x_CHASE_N]: CLEARED - how verified` |
| Trail | `// DBG[BUG_x_CHASE_N]: TRACE[step] - description` |
| Branch | `// DBG[BUG_x_CHASE_N]: BRANCH - options` |
| Dead End | `// DBG[BUG_x_CHASE_N]: DEAD END - proof` |
| Temp Log | `// DBG[BUG_x_CHASE_N]: TEMP LOG - remove after` |

### Files
| Purpose | Location |
|---------|----------|
| Bug tracking | `.bugs/BUG_<name>.md` |
| Fixed bugs | `.bugs/fixed/BUG_<name>.md` |
| Your team file | `.teams/TEAM_XXX_*.md` |

### Commands
```bash
# Find all breadcrumbs for a bug
grep -rn "DBG\[BUG_<name>" src/

# Find all breadcrumbs for a specific chase
grep -rn "DBG\[BUG_<name>_CHASE_<NNN>" src/

# Find all DEAD ENDs
grep -rn "DEAD END" src/

# Find all unfinished BRANCHes
grep -rn "BRANCH" src/ | grep DBG
```

---

## Example Debug Session

### Team 042 starts investigating BUG_wrong_tile_bounds

1. Creates `.bugs/BUG_wrong_tile_bounds.md`
2. Registers as CHASE_001
3. Adds breadcrumbs:
```rust
// DBG[BUG_wrong_tile_bounds_CHASE_001]: TRACE[1] - checking tile creation
let tile = Tile::new(window);

// DBG[BUG_wrong_tile_bounds_CHASE_001]: SUSPECT - size might be wrong here
let size = tile.size();
```
4. After investigation, marks:
```rust
// DBG[BUG_wrong_tile_bounds_CHASE_001]: CLEARED - size is correct, verified with logging
let size = tile.size();
```
5. Updates bug file, marks CHASE_001 as DEAD END
6. Notes: "Size is correct at creation, problem must be during layout"

### Team 043 continues with CHASE_002
1. Reads CHASE_001 summary
2. Sees tile creation is CLEARED
3. Investigates layout phase instead
4. Finds the bug in `Row::layout()`
5. Fixes it, updates bug file with ROOT CAUSE

---

## Activation

These rules are ONLY active when the user says:
- "Enable debug mode"
- "Debug rules on"
- "Start debug investigation"
- Or similar explicit activation

When active, you follow BOTH `ai-teams-rules.md` AND this file.
