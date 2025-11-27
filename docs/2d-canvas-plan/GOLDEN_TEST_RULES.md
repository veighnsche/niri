# Golden Test Rules: CRITICAL GUIDELINES

## ⚠️ **READ THIS FIRST: SACRED RULES**

These rules were learned the hard way by TEAM_018. **NEVER VIOLATE THEM.**

---

## Rule 0: Golden Tests Are Sacred

**Golden snapshots capture MAIN BRANCH behavior - not your refactor's behavior.**

- Golden tests prevent regressions
- They represent the COMMITTED behavior from the main branch
- Your refactor MUST match this behavior exactly
- If tests fail, YOUR CODE IS WRONG - not the tests

---

## Rule 1: NEVER Accept Golden Snapshot Changes

**FORBIDDEN COMMANDS:**
```bash
# NEVER run these commands:
cargo insta accept
cargo insta review  # (and accepting changes)
cargo insta test --accept
```

**WHY?** 
- `.snap.new` files contain YOUR (potentially broken) code's output
- Accepting them OVERWRITES the golden baseline
- This defeats the entire purpose of regression testing

**INSTEAD:**
```bash
# If golden tests fail:
cargo test --lib golden  # See what's different
# Fix your code until tests pass
# NEVER change the tests
```

---

## Rule 2: NEVER Remove Workspace-Related Golden Tests

**Workspace tests MUST continue working with the workspace system.**

- DO NOT remove tests that use workspace functionality
- DO NOT "update" tests to use canvas instead of workspace
- The workspace system must remain intact for regression testing

**WHY?**
- Golden tests verify EXISTING behavior works
- Removing them hides potential regressions
- Future users depend on this behavior staying consistent

---

## Rule 3: Golden Files Are Read-Only (chmod 444)

**Golden snapshot files are protected for a reason:**
```bash
# Files are chmod 444 (read-only) to prevent accidental changes
ls -la src/layout/tests/snapshots/
# -r--r--r-- 1 user user 1234 date snapshot.snap
```

**⚠️ CRITICAL: NEVER use sudo for permission fixes!**
- If you mess up permissions, use `git checkout HEAD -- src/layout/tests/snapshots/`
- NEVER run `sudo chmod` or `sudo chown` on repo files
- This creates ownership issues that break the entire repo

**IF you need to restore golden snapshots:**
```bash
# Use the proper tool, not manual changes
cargo xtask golden-sync pull
# OR restore from git if permissions are messed up
git checkout HEAD -- src/layout/tests/snapshots/
chmod 444 src/layout/tests/snapshots/*.snap
```

---

## Rule 4: Canvas Migration Must Be Surgical

**CORRECT APPROACH for workspace → canvas migration:**

### ✅ **WHAT YOU CAN DO:**
1. **Implement canvas methods** - build the canvas system functionality
2. **Add canvas features** - new capabilities in the canvas system
3. **Prepare for migration** - make canvas ready for future integration
4. **Test canvas separately** - create separate tests for canvas functionality

### ❌ **WHAT YOU CANNOT DO:**
1. **Wholesale migration** - replace workspace with canvas everywhere
2. **Change fundamental behavior** - alter how existing operations work
3. **Break golden tests** - any change that makes golden tests fail
4. **Remove workspace code** - delete workspace methods still used by tests

### 🎯 **SURGICAL MIGRATION PROCESS:**
1. **Identify specific operation** to migrate (e.g., `focus_row_up`)
2. **Verify canvas implementation** produces IDENTICAL results
3. **Add feature flag** or conditional to use canvas for that operation
4. **Test BOTH systems** work identically
5. **GRADUALLY expand** - one operation at a time
6. **If ANY golden test fails** - revert immediately

---

## Rule 5: If Golden Tests Fail, Fix Your Code

**DEBUGGING PROCESS:**
```bash
# 1. Run golden tests to see failures
cargo xtask test-all golden

# 2. Examine the differences (don't accept them!)
# The .snap.new files show what your code produces vs expected

# 3. Fix your implementation to match expected behavior
# NOT the other way around!

# 4. Clean up any .snap.new files
cargo xtask test-all clean

# 5. Verify tests pass
cargo xtask test-all golden
```

**COMMON FAILURE CAUSES:**
- Changed method signatures
- Altered data structures  
- Modified timing or ordering
- Different default values
- Changed error handling

---

## Rule 6: Use Golden Sync Tools Properly

**PROPER WORKFLOW:**
```bash
# Run golden verification
cargo xtask test-all golden

# Check test artifact status
cargo xtask test-all status

# Pull fresh snapshots (if corrupted)
cargo xtask golden-sync pull

# Clean up .snap.new files (after fixing code)
cargo xtask test-all clean

# NEVER: Modify snapshot files manually
# NEVER: Accept snapshot changes without approval
```

---

## Rule 7: Document Intentional Behavior Changes

**If you MUST change behavior (rare):**

1. **Create NEW golden tests** for the new behavior
2. **Keep OLD golden tests** for backward compatibility
3. **Document the change** thoroughly
4. **Get explicit approval** from maintainers
5. **Use feature flags** to opt-in to new behavior

---

## TEAM_018's Mistakes: Learn From Them

### ❌ **WHAT TEAM_018 DID WRONG:**
1. **Accepted golden snapshot change** with `cargo insta review`
2. **Removed workspace-related golden test** (`golden_w3_focus_window_or_row_down`)
3. **Wholesale migrated workspace → canvas** breaking all golden tests
4. **Tried to change tests instead of fixing code**

### ✅ **HOW TEAM_018 FIXED IT:**
1. **Reverted all changes** with `git checkout HEAD`
2. **Restored golden snapshots** with `cargo xtask golden-sync pull`
3. **Kept workspace system intact** for golden tests
4. **Implemented canvas methods** without forcing migration

### 📚 **LESSONS LEARNED:**
- Canvas system can be fully implemented WITHOUT breaking workspace
- Migration must be GRADUAL and SURGICAL
- Golden tests protect users from regressions
- "Quality over speed" means preserving existing behavior

---

## Quick Reference

| Task | Approach | Status |
|------|----------|---------|
| Implement canvas features | ✅ Build canvas system | SAFE |
| Add new canvas methods | ✅ Extend canvas API | SAFE |
| Test canvas functionality | ✅ Create canvas tests | SAFE |
| Migrate workspace → canvas | ❌ Wholesale replacement | FORBIDDEN |
| Change golden test behavior | ❌ Accept snapshot changes | FORBIDDEN |
| Remove workspace tests | ❌ Delete failing tests | FORBIDDEN |

---

## Emergency Recovery

**If you messed up golden tests:**
```bash
# 1. Revert your changes
git checkout HEAD -- src/layout/

# 2. Restore golden snapshots (if permissions are OK)
cargo xtask golden-sync pull

# 3. Fix permissions if you used sudo (TEAM_018's mistake!)
git checkout HEAD -- src/layout/tests/snapshots/
chmod 444 src/layout/tests/snapshots/*.snap

# 4. Start over with surgical approach
```

**⚠️ NEVER use sudo for permission fixes!**
- Using `sudo chmod` or `sudo chown` creates ownership issues
- This can break the entire repository permissions
- Always use `git checkout` to restore files with correct permissions

---

## Remember

**Golden tests exist to PROTECT USERS from regressions.**

Your convenience ≠ user stability.

**When in doubt: preserve existing behavior.**

---

*Last updated: TEAM_036 - Migrated verify-golden.sh to cargo xtask test-all golden*
