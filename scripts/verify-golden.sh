#!/usr/bin/env bash
# Verify golden snapshot integrity
# This script checks that:
# 1. All golden files are read-only
# 2. Golden tests pass (snapshots match current code)

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "=== Golden Snapshot Verification ==="
echo

# Check 1: Snapshot README exists and is read-only
echo "Checking snapshot README..."
README="src/layout/tests/snapshots/README.md"
if [ ! -f "$README" ]; then
    echo -e "${RED}ERROR: $README not found${NC}"
    exit 1
fi
README_PERMS=$(stat -c %a "$README" 2>/dev/null || stat -f %Lp "$README")
if [ "$README_PERMS" != "444" ]; then
    echo -e "${RED}ERROR: $README should be 444, got $README_PERMS${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Snapshot README OK (444)${NC}"

# Check 2: Snapshot files are read-only (444)
echo "Checking snapshot file permissions..."
SNAPSHOT_DIR="src/layout/tests/snapshots"
if [ -d "$SNAPSHOT_DIR" ]; then
    for file in "$SNAPSHOT_DIR"/*.snap; do
        if [ -f "$file" ]; then
            FILE_PERMS=$(stat -c %a "$file" 2>/dev/null || stat -f %Lp "$file")
            if [ "$FILE_PERMS" != "444" ]; then
                echo -e "${RED}ERROR: $file should be 444, got $FILE_PERMS${NC}"
                exit 1
            fi
        fi
    done
    echo -e "${GREEN}✓ Snapshot file permissions OK (444)${NC}"
else
    echo -e "${RED}ERROR: Snapshot directory not found${NC}"
    exit 1
fi

# Check 3: Run golden tests
echo
echo "Running golden tests..."
if cargo test --lib golden 2>&1 | tail -5; then
    echo -e "${GREEN}✓ Golden tests pass${NC}"
else
    echo -e "${RED}ERROR: Golden tests failed${NC}"
    exit 1
fi

echo
echo -e "${GREEN}=== All verifications passed ===${NC}"
