# Golden Snapshots

## ⚠️ AI AGENTS: READ THIS FIRST ⚠️

These snapshots come from the `golden-snapshots` branch, NOT this branch.
**NEVER run `cargo insta accept` on golden tests.**

If you need to update snapshot behavior:
1. The change MUST be made on the `golden-snapshots` branch FIRST
2. Then run `cargo xtask golden-sync pull` to sync to this branch
3. Never modify `.snap` files directly on refactor branches

## Provenance

- **Source branch**: `golden-snapshots` (derived from `main`)
- **Sync command**: `cargo xtask golden-sync pull`
- **Generate command**: `cargo xtask golden-sync generate`

## If Tests Fail

1. **DO NOT** run `cargo insta accept` — this corrupts the baseline
2. **FIX YOUR CODE** — your refactor changed behavior
3. Run `cargo insta review` to see what changed
4. Run `cargo xtask golden-sync clean` to remove `.snap.new` files

## Commands

```bash
cargo xtask golden-sync status   # Check sync status
cargo xtask golden-sync pull     # Pull from golden-snapshots branch
cargo xtask golden-sync clean    # Remove .snap.new files
cargo test --lib golden          # Run golden tests
```

## Documentation

- **Rules**: `docs/2d-canvas-plan/ai-teams-rules.md` (Rule 4)
- **xtask source**: `xtask/src/golden_sync/mod.rs`

---

*TEAM_004: Initial. TEAM_010: Added provenance and AI agent instructions.*
