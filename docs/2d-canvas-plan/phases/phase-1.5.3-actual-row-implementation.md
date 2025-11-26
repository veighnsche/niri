# Phase 1.5.3: Actual Row Implementation

> **Status**: ⏳ PENDING
> **Prerequisite**: Overview removal complete (TEAM_015)
> **Critical**: This is the REAL work that Part 2 was supposed to do

---

## ⚠️ **CRITICAL: GOLDEN TEST RULES**

**LEARNED FROM TEAM_018's MISTAKES - READ BEFORE STARTING**

🚨 **NEVER accept golden snapshot changes** - they represent MAIN BRANCH behavior  
🚨 **NEVER remove workspace-related golden tests** - they must continue working  
🚨 **If golden tests fail, fix YOUR CODE** - not the tests  

**CORRECT APPROACH**: Implement canvas methods WITHOUT breaking workspace system  
**FORBIDDEN**: Wholesale workspace → canvas migration that changes behavior  

**FULL GUIDELINES**: See [../GOLDEN_TEST_RULES.md](../GOLDEN_TEST_RULES.md)

*TEAM_018 violated these rules and had to revert all work. Don't make their mistake.*

---

## Problem Statement

**Part 2 was supposed to implement row-based navigation, but only did renaming.**

Current state after "rename-only" approach:
```rust
// This is what we have NOW (broken):
pub fn focus_row_up(&mut self) {
    monitor.switch_workspace_up();  // Still calls workspace code!
}

// This is what we NEED:
pub fn focus_row_up(&mut self) {
    canvas.focus_row_up();  // Call 2D canvas navigation!
}
```

---

## Implementation Plan

### Step 1: Implement Row Navigation in Canvas

**Files to modify:**
- `src/layout/canvas.rs` - Add row navigation methods
- `src/layout/mod.rs` - Fix layout methods to call canvas instead of workspace

**Canvas methods to implement:**
```rust
impl Canvas {
    pub fn focus_row_up(&mut self) -> bool
    pub fn focus_row_down(&mut self) -> bool
    pub fn move_window_to_row_up(&mut self) -> bool  
    pub fn move_window_to_row_down(&mut self) -> bool
    pub fn move_column_to_row_up(&mut self) -> bool
    pub fn move_column_to_row_down(&mut self) -> bool
    pub fn move_row_up(&mut self) -> bool
    pub fn move_row_down(&mut self) -> bool
    pub fn set_row_name(&mut self, name: Option<String>)
    pub fn unset_row_name(&mut self)
}
```

### Step 2: Fix Layout Method Implementation

**Files to modify:**
- `src/layout/mod.rs` - Replace workspace calls with canvas calls

**Before:**
```rust
pub fn focus_row_up(&mut self) {
    monitor.switch_workspace_up();  // ❌ Workspace code
}
```

**After:**
```rust
pub fn focus_row_up(&mut self) {
    self.canvas.focus_row_up();  // ✅ Canvas code
}
```

### Step 3: Remove Monitor Workspace Dependencies

**Files to modify:**
- `src/layout/monitor/mod.rs` - Remove workspace methods
- `src/layout/monitor/workspace_ops.rs` - Delete entire file
- `src/layout/monitor/navigation.rs` - Delete entire file

### Step 4: Update Tests

**Files to modify:**
- `src/layout/tests.rs` - Fix tests to use canvas navigation
- Remove workspace-specific test scenarios

---

## Detailed Implementation

### Canvas Row Navigation Logic

The canvas should treat rows as the Y-axis in the 2D grid:

```rust
impl Canvas {
    pub fn focus_row_up(&mut self) -> bool {
        if self.camera.row > 0 {
            self.camera.row -= 1;
            self.focus_active_tile_in_view();
            true
        } else {
            false
        }
    }
    
    pub fn focus_row_down(&mut self) -> bool {
        if self.camera.row < self.rows.len().saturating_sub(1) {
            self.camera.row += 1;
            self.focus_active_tile_in_view();
            true
        } else {
            false
        }
    }
}
```

### Window Movement Between Rows

```rust
impl Canvas {
    pub fn move_window_to_row_up(&mut self) -> bool {
        let Some(active_tile) = self.active_tile() else { return false };
        let current_row = self.tile_row(&active_tile)?;
        
        if current_row == 0 { return false; }
        
        // Remove from current row
        self.remove_tile_from_row(&active_tile, current_row);
        
        // Add to row above
        self.add_tile_to_row(active_tile, current_row - 1);
        
        // Focus the moved window
        self.focus_tile(&active_tile);
        true
    }
}
```

### Row Naming

```rust
impl Canvas {
    pub fn set_row_name(&mut self, name: Option<String>) {
        let row_idx = self.camera.row;
        if let Some(row) = self.rows.get_mut(row_idx) {
            row.name = name;
        }
    }
}
```

---

## Verification

### Compilation Tests
```bash
cargo check                    # Should compile
cargo test --lib              # All 284 tests pass
cargo insta test              # All golden tests pass
```

### Functional Tests
- Row navigation works (up/down)
- Window movement between rows works
- Row naming works
- No workspace code remaining

### Integration Tests
- All keybindings work
- IPC commands work
- Config parsing works

---

## Handoff Criteria

- [ ] All layout methods call canvas instead of workspace
- [ ] Canvas implements full row navigation
- [ ] All workspace files deleted
- [ ] All tests pass
- [ ] No workspace references remaining

---

## Notes

This is the **critical missing piece** that makes the Canvas2D refactor actually work.

Without this step, we're just calling workspace code with row names - no architectural change has actually happened.

**This is the difference between:**
- ❌ "Rename workspace to row" (what Part 2 did)
- ✅ "Replace workspace with canvas rows" (what this phase does)
