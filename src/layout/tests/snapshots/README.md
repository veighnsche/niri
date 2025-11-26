# Golden Snapshots — READ BEFORE MODIFYING

## ⚠️ STOP — DO NOT MODIFY THESE FILES

These `.snap` files are **locked baselines** (chmod 444) created from the original
`main` branch code. They represent the "golden standard" that refactored code
must match.

## Source

- **Commit**: `75d5e3b0`
- **Date**: 2024-11-26
- **Original code**: `git show 75d5e3b0:src/layout/scrolling.rs`

## If Tests Fail

1. **DO NOT** run `cargo insta accept`
2. **DO NOT** modify snapshot files
3. **FIX YOUR CODE** — your refactor changed behavior
4. Run `cargo insta review` to see what changed
5. Compare with original: `git show 75d5e3b0:src/layout/scrolling.rs`

## If You MUST Update Snapshots

Only with **explicit USER approval**:

```bash
# 1. Get permission from USER first!
# 2. Unlock the file
chmod 644 src/layout/tests/snapshots/THE_FILE.snap

# 3. Update
cargo insta accept

# 4. Re-lock
chmod 444 src/layout/tests/snapshots/THE_FILE.snap
```

## Verification

Run before any layout refactor:
```bash
./scripts/verify-golden.sh
```

## Documentation

- **Rules**: `docs/2d-canvas-plan/ai-teams-rules.md` (Rule 4)
- **Phase plan**: `docs/2d-canvas-plan/phases/phase-0.5-golden-snapshots-v2.md`

---

*Created by TEAM_004. 58 snapshots covering Groups A-W.*
