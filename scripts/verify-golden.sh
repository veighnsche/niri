#!/usr/bin/env bash
# =============================================================================
# Golden Snapshot Verification
# =============================================================================
#
# ⚠️ AI AGENTS: READ THIS ⚠️
#
# Golden snapshots come from the `golden-snapshots` branch, NOT this branch.
# NEVER run `cargo insta accept` on golden tests!
#
# If tests fail:
#   1. FIX YOUR CODE — your refactor changed behavior
#   2. Run `cargo xtask golden-sync clean` to remove .snap.new files
#   3. DO NOT accept .snap.new files — they contain your broken output
#
# If you need to change snapshot behavior:
#   1. Make the change on `golden-snapshots` branch FIRST
#   2. Run `cargo xtask golden-sync pull` to sync here
#   3. See: xtask/src/golden_sync/mod.rs
#
# =============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "=== Golden Snapshot Verification ==="
echo

# Check: Snapshot directory exists
SNAPSHOT_DIR="src/layout/tests/snapshots"
if [ ! -d "$SNAPSHOT_DIR" ]; then
    echo -e "${RED}ERROR: Snapshot directory not found: $SNAPSHOT_DIR${NC}"
    echo
    echo "Run: cargo xtask golden-sync pull"
    exit 1
fi
echo -e "${GREEN}✓ Snapshot directory exists${NC}"

# Check for .snap.new files (regression evidence)
NEW_FILES=$(find "$SNAPSHOT_DIR" -name "*.snap.new" 2>/dev/null | wc -l)
if [ "$NEW_FILES" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $NEW_FILES .snap.new files (regression evidence)${NC}"
    echo "  Run: cargo xtask golden-sync clean"
fi

# Run golden tests
echo
echo "Running golden tests..."
if cargo test --lib golden 2>&1 | tail -5; then
    echo -e "${GREEN}✓ Golden tests pass${NC}"
else
    echo -e "${RED}ERROR: Golden tests failed${NC}"
    echo
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  DO NOT run 'cargo insta accept' — this corrupts the baseline!${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    echo
    echo "Your refactor changed behavior. To fix:"
    echo "  1. Run 'cargo insta review' to see what changed"
    echo "  2. Fix your code until tests pass"
    echo "  3. Run 'cargo xtask golden-sync clean' to remove .snap.new files"
    echo
    echo "Golden snapshots come from: golden-snapshots branch"
    echo "Sync command: cargo xtask golden-sync pull"
    exit 1
fi

echo
echo -e "${GREEN}=== All verifications passed ===${NC}"
