# Phase 2: Row System Implementation

> **Status**: ⏳ PENDING (after Phase 1)
> **Goal**: Implement complete row lifecycle and naming system

---

## Related TODOs from Codebase

These TODOs will be resolved by this phase:
- `TODO(TEAM_018): Implement proper duplicate name checking` (mod.rs:4635)
- `TODO(TEAM_025): Implement proper row removal` (canvas/mod.rs:312)
- `TODO(TEAM_024): Implement row removal in canvas` (monitor/config.rs:28)
- `TODO(TEAM_022): Implement previous row tracking` (monitor/navigation.rs:59,67)
- `TODO(TEAM_018): Implement previous row tracking` (canvas/navigation.rs:331)

---

## Overview

Based on USER decisions:
- Any row can be named, including row 0
- Names are unique per output, case-insensitive
- Row 0 is special (origin, always exists)
- Empty unnamed rows are deleted, named rows persist
- Rows can be created explicitly or when windows are added

---

## Step 2.1: Row Naming

### Tasks

- [ ] **2.1.1**: Implement `Row::set_name(Option<String>)`
- [ ] **2.1.2**: Implement `Row::name() -> Option<&str>`
- [ ] **2.1.3**: Validate name uniqueness per output (case-insensitive)
- [ ] **2.1.4**: Allow naming row 0
- [ ] **2.1.5**: Add `set-row-name` action/keybind

### Implementation

```rust
impl Row {
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
    
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Canvas2D {
    pub fn set_row_name(&mut self, row_idx: i32, name: Option<String>) -> Result<(), NameError> {
        // Check uniqueness (case-insensitive)
        if let Some(ref n) = name {
            for (idx, row) in &self.rows {
                if *idx != row_idx {
                    if row.name().is_some_and(|existing| existing.eq_ignore_ascii_case(n)) {
                        return Err(NameError::Duplicate);
                    }
                }
            }
        }
        
        if let Some(row) = self.rows.get_mut(&row_idx) {
            row.set_name(name);
        }
        Ok(())
    }
}
```

---

## Step 2.2: Row Lifecycle

### Creation

Rows are created when:
- A window is added to a non-existent row index
- Explicitly via config (`row "name" { }`)
- Explicitly via keybind/IPC

### Deletion

Rows are deleted when:
- They become empty AND are unnamed AND are not row 0

### Tasks

- [ ] **2.2.1**: Implement `Canvas2D::ensure_row(row_idx) -> &mut Row`
- [ ] **2.2.2**: Implement `Canvas2D::cleanup_empty_rows()` with proper rules
- [ ] **2.2.3**: Ensure row 0 always exists
- [ ] **2.2.4**: Add `create-row` action for explicit creation

### Implementation

```rust
impl Canvas2D {
    pub fn cleanup_empty_rows(&mut self) {
        self.rows.retain(|&idx, row| {
            // Keep row 0 (origin)
            idx == 0 
            // Keep named rows
            || row.name().is_some() 
            // Keep non-empty rows
            || !row.is_empty()
        });
    }
}
```

---

## Step 2.3: Row IDs (Global Counter)

### Tasks

- [ ] **2.3.1**: Use `Layout::row_id_counter` for global unique IDs
- [ ] **2.3.2**: Pass ID generator to Canvas2D when creating rows
- [ ] **2.3.3**: Remove per-canvas ID counter
- [ ] **2.3.4**: IDs are not persisted across sessions

### Implementation

```rust
impl Layout {
    fn next_row_id(&mut self) -> RowId {
        self.row_id_counter += 1;
        RowId(self.row_id_counter)
    }
}
```

---

## Step 2.4: Active Row

### Behavior

- Active row follows focus (row containing focused window)
- Can be explicitly set by user
- Default is row 0 when output is added
- Windows open on active row unless `open-on-row` specifies otherwise

### Tasks

- [ ] **2.4.1**: Track `active_row_idx` in Canvas2D
- [ ] **2.4.2**: Update active row when focus changes
- [ ] **2.4.3**: Add `focus-row` action for explicit row switching
- [ ] **2.4.4**: Windows without `open-on-row` go to active row

---

## Step 2.5: Window Placement

### Rules

1. `open-on-row "name"` → find or create row with that name
2. `open-on-output "name"` without `open-on-row` → active row on that output
3. No targeting → active row on active output

### Tasks

- [ ] **2.5.1**: Implement `open-on-row` window rule
- [ ] **2.5.2**: Create row if `open-on-row` specifies non-existent name
- [ ] **2.5.3**: Update window placement logic

---

## Verification

```bash
cargo test
cargo insta test

# Manual tests:
# - Create named row via config
# - Windows open on correct row
# - Empty unnamed rows are cleaned up
# - Named rows persist when empty
# - Row 0 cannot be deleted
```

---

## Success Criteria

- [ ] Rows can be named (any row, including 0)
- [ ] Names are unique per output
- [ ] Row 0 always exists
- [ ] Empty unnamed rows are cleaned up
- [ ] Named rows persist when empty
- [ ] `open-on-row` creates row if needed
- [ ] Active row follows focus

---

*Phase 2 - Row System*
