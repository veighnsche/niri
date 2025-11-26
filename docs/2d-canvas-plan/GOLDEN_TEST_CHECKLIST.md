# Golden Test Compliance Checklist

**Use this checklist BEFORE starting any work that might affect golden tests.**

---

## 🚨 Pre-Work Checklist

### Before You Start:
- [ ] I have read [GOLDEN_TEST_RULES.md](GOLDEN_TEST_RULES.md)
- [ ] I understand that golden tests represent MAIN BRANCH behavior
- [ ] I will NEVER accept golden snapshot changes
- [ ] I will keep workspace system intact for golden tests
- [ ] I will fix MY CODE if golden tests fail, not change tests

### Risk Assessment:
- [ ] My changes will NOT affect existing workspace behavior
- [ ] My changes will NOT break any golden tests
- [ ] I am implementing NEW features, not changing existing ones
- [ ] If migrating workspace → canvas, I will do it surgically

---

## 🔍 Implementation Checklist

### During Development:
- [ ] I run `cargo test --lib golden` regularly to check for regressions
- [ ] If any golden test fails, I STOP and fix the issue immediately
- [ ] I NEVER run `cargo insta accept` or similar commands
- [ ] I keep workspace methods working exactly as before

### Canvas Migration (if applicable):
- [ ] I implement canvas methods WITHOUT removing workspace methods
- [ ] I verify canvas produces IDENTICAL results to workspace
- [ ] I use feature flags or conditionals for gradual migration
- [ ] I test BOTH systems work identically before proceeding

---

## ✅ Verification Checklist

### Before Finishing:
- [ ] `cargo check` passes without errors
- [ ] `cargo test --lib golden` passes ALL tests (90/90)
- [ ] No `.snap.new` files exist in `src/layout/tests/snapshots/`
- [ ] I have NOT accepted any golden snapshot changes
- [ ] Workspace system still works exactly as before
- [ ] Canvas system is implemented but not forced on existing code

### Emergency Recovery (if needed):
- [ ] I can revert changes with `git checkout HEAD -- src/layout/`
- [ ] I can restore golden snapshots with `cargo xtask golden-sync pull`
- [ ] I will NEVER use `sudo chmod` or `sudo chown` on repo files
- [ ] I use `git checkout HEAD -- src/layout/tests/snapshots/` for permission issues
- [ ] I know to ask for help if I'm unsure about approach

---

## 🚫 Forbidden Actions Checklist

**I have NOT done any of these:**
- [ ] ❌ Run `cargo insta accept` or accepted snapshot changes
- [ ] ❌ Removed workspace-related golden tests
- [ ] ❌ Modified golden snapshot files manually
- [ ] ❌ Used `sudo chmod` or `sudo chown` on repo files
- [ ] ❌ Changed fundamental behavior that breaks golden tests
- [ ] ❌ Deleted workspace methods still used by tests
- [ ] ❌ Forced canvas migration that changes existing behavior

---

## 📞 Getting Help

**If you're unsure:**
1. **STOP** - Don't proceed if you have doubts
2. **READ** - Review [GOLDEN_TEST_RULES.md](GOLDEN_TEST_RULES.md)
3. **ASK** - Create a question file in `.questions/TEAM_XXX_topic.md`
4. **REVERT** - When in doubt, revert and start over

**Remember**: It's better to ask for help than to break golden tests!

---

## 🎯 Success Criteria

**You're successful if:**
- ✅ All golden tests pass (90/90)
- ✅ Your new features work without breaking existing behavior
- ✅ Workspace system remains intact
- ✅ Canvas system is implemented but not forced
- ✅ No golden snapshot files were modified

---

*Last updated: TEAM_018 - Learn from our mistakes.*
