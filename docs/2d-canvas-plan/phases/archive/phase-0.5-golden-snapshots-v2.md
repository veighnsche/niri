# Phase 0.5: Golden Snapshot Infrastructure (Revised)

> **Problem**: The original Phase 0.5 created snapshots from "current code" without proving they match the original main branch behavior. We need verifiable golden testing.

## Design Goals

1. **Provable correctness**: Golden snapshots MUST come from main branch code
2. **Side-by-side comparison**: Both golden and refactored code run, outputs compared
3. **Reproducible**: Anyone can regenerate golden snapshots from main branch

> Note: TEAM_010 removed chmod 444 protection — git doesn't preserve file permissions,
> so this forced manual workarounds. The tests themselves are the protection.

---

## Architecture

```
src/layout/
├── golden/                      # READ-ONLY, chmod 555 directory
│   ├── mod.rs                   # Module that re-exports golden types
│   ├── scrolling.rs             # Original ScrollingSpace (renamed, imports fixed)
│   ├── column.rs                # Original Column code
│   └── snapshot.rs              # Snapshot impl for golden types
│
├── snapshot.rs                  # Snapshot types (shared by both)
│
├── tests/
│   ├── golden.rs                # Test harness
│   └── snapshots/               # READ-ONLY after generation
│       ├── golden/              # Snapshots from golden code
│       └── refactored/          # Snapshots from refactored code (must match)
```

## How It Works

```
┌─────────────────┐     ┌──────────────────┐
│  Golden Code    │     │  Refactored Code │
│  (from main)    │     │  (current)       │
└────────┬────────┘     └────────┬─────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌──────────────────┐
│ golden.snapshot()│     │ current.snapshot()│
└────────┬────────┘     └────────┬─────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌──────────────────┐
│ Golden YAML     │ === │ Refactored YAML  │
│ (locked)        │     │ (must match)     │
└─────────────────┘     └──────────────────┘
```

---

## Implementation Steps

### Step 0.5.1: Extract and Adapt Golden Code (2-3 hours)

**Goal**: Make `scrolling_original.rs` compile as `golden::ScrollingSpace`

1. Create `src/layout/golden/scrolling.rs` from main branch
2. Fix imports: `super::` → `crate::layout::`
3. Rename struct: `ScrollingSpace` → `GoldenScrollingSpace` (avoid conflicts)
4. Add `snapshot()` method that returns `ScrollingSnapshot`
5. Make entire `golden/` directory read-only

**Verification**: `cargo check` passes with golden module

### Step 0.5.2: Create Golden Test Harness (1-2 hours)

**Goal**: Test harness that runs BOTH golden and refactored code

```rust
// src/layout/tests/golden.rs

/// Runs the same operations on both golden and refactored code,
/// asserts their snapshots are identical.
fn compare_golden_and_refactored(ops: &[Op]) {
    let golden_snapshot = run_golden_ops(ops);
    let refactored_snapshot = run_refactored_ops(ops);
    
    assert_eq!(golden_snapshot, refactored_snapshot, 
        "Refactored code produced different output than golden code");
}
```

**Verification**: Tests explicitly compare both implementations

### Step 0.5.3: Generate Locked Golden Snapshots (1 hour)

**Goal**: Create read-only baseline snapshots from golden code

1. Run golden code to generate `tests/snapshots/golden/*.snap`
2. Lock all snapshot files: `chmod 444`
3. Lock snapshot directory: `chmod 555`
4. Add git pre-commit hook to verify permissions

**Verification**: `ls -la` shows read-only permissions

### Step 0.5.4: Implement Comparison Tests (2-3 hours)

**Goal**: 58+ tests that compare golden vs refactored

Each test:
1. Runs operations on `GoldenScrollingSpace`
2. Runs same operations on `ScrollingSpace`
3. Compares snapshots
4. Fails if they differ

**Verification**: All 58 tests pass

### Step 0.5.5: CI Integration (1 hour)

**Goal**: CI fails if golden files are modified or snapshots differ

1. Add CI step to verify golden file permissions
2. Add CI step to verify snapshot file permissions
3. Fail build if any golden/snapshot file was modified

**Verification**: CI pipeline includes permission checks

---

## File Permissions

| Path | Permission | Reason |
|------|------------|--------|
| `src/layout/golden/` | 555 | Directory read-only |
| `src/layout/golden/*.rs` | 444 | Source files read-only |
| `tests/snapshots/golden/` | 555 | Directory read-only |
| `tests/snapshots/golden/*.snap` | 444 | Snapshots read-only |

---

## Challenges

### Challenge 1: Golden Code Dependencies

The original `scrolling.rs` uses types from other modules. Options:

**A. Copy all dependencies** (chosen)
- Copy `Column`, `Tile`, etc. into golden module
- More code, but fully isolated

**B. Share types via trait**
- Define `LayoutElement` trait, both implement it
- Cleaner, but requires careful interface design

**C. Use git worktree**
- Keep main branch in separate directory
- Run tests against both
- Cleanest, but complex CI setup

### Challenge 2: Test Infrastructure

The existing `check_ops()` function uses current types. Need:
- `check_golden_ops()` for golden code
- `check_refactored_ops()` for current code
- Both return comparable `Snapshot` types

### Challenge 3: Keeping Golden Code Frozen

Once golden code is set up:
- NEVER modify it
- If main branch changes, we need to decide: update golden or keep old baseline?
- Recommendation: Golden code represents the "last known good" state

---

## Success Criteria

1. [x] Golden source code extracted from main branch
2. [x] Verification script created (`scripts/verify-golden.sh`)
3. [x] 91 golden tests pass (was 58, expanded by later teams)
4. [x] Documentation explains the system

> Note: chmod 444 criteria removed by TEAM_010 — git doesn't preserve permissions.

## Current Implementation (Pragmatic Approach)

Instead of making golden code compile separately (complex), we use:

1. **Locked snapshots**: YAML files created from original code, now read-only
2. **Source reference**: `scrolling_original.rs` for human reference
3. **Verification script**: Checks permissions and runs tests
4. **Git-based verification**: Can regenerate from commit `75d5e3b0`

This approach is simpler and still provides:
- Tamper-proof snapshots (read-only)
- Verifiable source (git commit)
- Automated checks (script + CI)

---

## Estimated Time

| Step | Time |
|------|------|
| 0.5.1: Extract golden code | 2-3 hours |
| 0.5.2: Test harness | 1-2 hours |
| 0.5.3: Lock snapshots | 1 hour |
| 0.5.4: Comparison tests | 2-3 hours |
| 0.5.5: CI integration | 1 hour |
| **Total** | **7-10 hours** |

---

## Next Steps After Completion

Once golden testing is robust:
1. Phase 0.2: AnimatedValue (DONE by TEAM_005)
2. Phase 1: Row + Canvas2D
3. Every refactor MUST pass golden tests before merge
