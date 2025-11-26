# Snapshot Component Audit

> **Purpose**: Critical review for next team to verify golden testing integrity.
> **Status**: 🔴 **CRITICAL ISSUE** — Golden snapshots may NOT be from golden branch!

---

## 🚨 CRITICAL QUESTIONS FOR NEXT TEAM TO VERIFY

### Question 1: Were the golden snapshot files generated from the GOLDEN BRANCH (main)?

**Finding:**
- Snapshot files first appear in commit `75d5e3b0`
- Commit `75d5e3b0` is on branch `2d-canvas` (our refactor branch)
- Commit `75d5e3b0` is NOT on `main` branch
- The merge-base with main is `54c7fdcd`

**⚠️ PROBLEM:** The snapshot files were created ON THE REFACTOR BRANCH, not from the golden (main) branch!

```bash
# Verify this:
git branch -a --contains 75d5e3b0   # Shows: 2d-canvas (NOT main!)
git merge-base main 75d5e3b0        # Shows: 54c7fdcd
```

### Question 2: Are the golden snapshots compared against our refactored implementation?

**Current Setup:**
```
src/layout/tests/golden.rs:27
    fn snapshot(ops) -> ScrollingSnapshot {
        check_ops(ops).active_workspace().unwrap().scrolling().snapshot()
    }
```

This calls `snapshot()` on **CURRENT branch code** — which is the refactored code.

**⚠️ PROBLEM:** We are comparing refactored code against refactored code!

---

## 🔴 ROOT CAUSE

The golden testing setup is **BROKEN**:

1. Snapshots were generated from commit `75d5e3b0` on `2d-canvas` branch
2. Tests run against current `2d-canvas` branch code
3. **Both are the same refactored code!**

This means:
- ❌ We are NOT comparing against main branch (golden) code
- ❌ The snapshots do NOT represent golden behavior
- ❌ Tests will pass even if refactor breaks things

---

## ✅ WHAT SHOULD HAPPEN

### Correct Golden Testing Flow:
```
1. Checkout main branch (golden)
2. Run tests → Generate snapshots from golden production code
3. Lock snapshots (chmod 444)
4. Checkout refactor branch
5. Run tests → Compare refactored code output against golden snapshots
6. If mismatch → Refactor broke something!
```

### Current (BROKEN) Flow:
```
1. On refactor branch
2. Generate snapshots from refactored code
3. Run tests against refactored code
4. Always passes! (comparing same code to itself)
```

---

## 📋 ACTION ITEMS FOR NEXT TEAM

### Priority 1: Verify Golden Snapshot Origin
```bash
# Check which branch created the snapshots
git log --oneline src/layout/tests/snapshots/ | head -5

# Check if that commit is on main
git branch -a --contains <commit>
```

### Priority 2: Regenerate Snapshots from Main (if needed)
```bash
# Checkout main branch
git checkout main

# Run tests to generate golden snapshots
cargo insta test --accept

# Copy snapshots somewhere safe
cp -r src/layout/tests/snapshots/ /tmp/golden-snapshots/

# Return to refactor branch
git checkout 2d-canvas

# Replace snapshots with golden ones
cp -r /tmp/golden-snapshots/* src/layout/tests/snapshots/

# Lock them
chmod 444 src/layout/tests/snapshots/*.snap

# Run tests - NOW they compare refactored vs golden!
cargo test --lib golden
```

### Priority 3: Document the Process
Update `scripts/verify-golden.sh` to enforce this workflow.

---

## 📁 Files to Review

| File | What to Check |
|------|---------------|
| `src/layout/tests/snapshots/*.snap` | Were these generated from main? |
| `src/layout/tests/golden.rs` | Does `snapshot()` call the right code? |
| `scripts/verify-golden.sh` | Does it enforce golden branch origin? |

---

## Snapshot Infrastructure Components (for reference)

### Snapshot Types — `src/layout/snapshot.rs`
- `ScrollingSnapshot` — Layout state
- `ColumnSnapshot` — Column state  
- `TileSnapshot` — Tile state
- `AnimationTimelineSnapshot` — Animation state (TEAM_010 addition)

### Snapshot Generation — Various files
- `ScrollingSpace::snapshot()` — `src/layout/scrolling.rs`
- `Column::snapshot()` — `src/layout/column/mod.rs`

### Test Infrastructure — `src/layout/tests/golden.rs`
- 91 test cases
- Uses `insta::assert_yaml_snapshot!`

---

*Audit by TEAM_010 — CRITICAL REVIEW NEEDED*
