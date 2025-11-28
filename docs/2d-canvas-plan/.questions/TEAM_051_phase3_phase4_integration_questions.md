# TEAM_051: Phase 3 Monitor & Phase 4 Layout Mod Integration — MASTERPLAN

> **Status**: ✅ RESOLVED — Comprehensive design decisions made  
> **Based on**: Full codebase analysis including Canvas2D, Row, Monitor, IPC, and gesture systems  
> **Outcome**: This document is now the authoritative reference for Phase 3/4 integration

---

# EXECUTIVE SUMMARY

The Canvas2D migration is fundamentally about replacing a **1D workspace list** with an **infinite 2D surface**. The key insight from analyzing the codebase is:

1. **Row = New Workspace**: Each Row already has a unique `WorkspaceId` (stored in `Row.workspace_id`)
2. **Canvas2D = Workspace Container**: Canvas2D manages rows like Monitor managed workspaces
3. **External Compatibility**: IPC and protocols can continue using `WorkspaceId` — just backed by Row data

**The answer to most questions is: Use the existing `WorkspaceId` system backed by Row data.**

---

# PHASE 3: MONITOR LAYER ANSWERS

## 1. Insert Hint Rendering (TEAM_022) - render.rs

### Answer: **Option A — Convert via existing Row.id() method**

**Rationale**: Row already stores a unique `WorkspaceId`:
```rust
// From src/layout/row/mod.rs:230-232
pub fn id(&self) -> crate::layout::workspace_types::WorkspaceId {
    self.workspace_id
}
```

**Implementation**:
```rust
// In hit_test.rs — this is already correct!
// Line 122: return (InsertWorkspace::Existing(ws.id()), geo);
// Row.id() returns the stored WorkspaceId
```

**Key Files**:
- `src/layout/monitor/types.rs`: Keep `InsertWorkspace` unchanged — it already works with `WorkspaceId`
- `src/layout/monitor/hit_test.rs`: Already calls `ws.id()` on Row (line 122, 138)
- `src/layout/canvas/operations.rs`: `ensure_row()` generates unique `WorkspaceId` for new rows (line 27-28)

**No changes needed** — the current implementation is correct.

---

## 2. Gesture Support (TEAM_024) - gestures.rs

### Answer: **Gestures work with row indices internally, expose WorkspaceId externally**

**Current State Analysis**:
The gesture system in `gestures.rs` already uses row indices internally:
```rust
// From gestures.rs:83 - uses canvas.rows().count() for bounds
let (min, max) = gesture.min_max(self.canvas.rows().count());

// From gestures.rs:116 - uses row_count for gesture bounds
let row_count = self.canvas.rows().count();
```

**Implementation Strategy**:

1. **Keep gesture internals as-is** — they work with row counts/indices
2. **For the TODO at line 142-147**: When we need to track "previous workspace" for back-and-forth:
   ```rust
   // SOLUTION: Use Canvas2D's built-in previous_row_idx tracking
   // From canvas/mod.rs:82
   pub(crate) previous_row_idx: i32,
   
   // The gesture end should call:
   self.previous_row_idx = self.active_row_idx;  // Already done in focus_row()
   ```

3. **For active row setting (line 147)**: 
   ```rust
   // After gesture ends, call focus_row which handles everything
   self.canvas.focus_row(new_idx as i32);
   ```

**Implementation**:
```rust
// In workspace_switch_gesture_end(), replace:
// TODO: TEAM_024: Get workspace ID from canvas row
// TODO: TEAM_024: Set active workspace index in canvas

// With:
if current_active_idx != new_idx {
    // Canvas2D.focus_row() already tracks previous_row_idx
}
// Update active row via Canvas2D.focus_row() which animates camera
self.canvas.focus_row(new_idx as i32);
```

---

## 3. Row Removal in Canvas (TEAM_024) - config.rs

### Answer: **Use Canvas2D.remove_row() with row 0 protection**

**Current State**:
- `Canvas2D::remove_row()` already exists (mod.rs line 329-355)
- It already handles active row adjustment

**Rules for Row Removal**:
1. ✅ **Row 0 cannot be removed** — it's the origin (per TEAM_042 decision Q2c)
2. ✅ **Named rows persist** — `cleanup_empty_rows()` keeps them (operations.rs line 45)
3. ✅ **Active row adjustment** — already implemented in `remove_row()`

**Implementation for config.rs line 28**:
```rust
// Replace the TODO with:
if self.canvas.rows().count() > 1 && self.active_workspace_idx() != 0 {
    // Only remove row 0 if it's empty and we're not on it
    if let Some(row) = self.canvas.row(0) {
        if row.is_empty() {
            self.canvas.remove_row(0);
        }
    }
}
```

**Important**: Before removing any row, check:
```rust
fn can_remove_row(&self, idx: i32) -> bool {
    // Cannot remove row 0 (origin)
    if idx == 0 { return false; }
    // Cannot remove named rows
    if let Some(row) = self.canvas.row(idx) {
        if row.name().is_some() { return false; }
    }
    // Must have at least one row remaining
    self.canvas.rows().count() > 1
}
```

---

# PHASE 4: LAYOUT MOD LAYER ANSWERS

## 4. Workspace Config Removal (TEAM_020) - mod.rs

### Answer: **Replace workspace config with row config, keep global canvas options**

**Config Migration Table**:

| Old Workspace Config | New Location | Reason |
|---------------------|--------------|--------|
| `workspace.name` | `Row.name` | Per-row property ✅ Already exists |
| `workspace.open_on_output` | `Canvas2D.output` | Per-canvas property ✅ Already exists |
| `empty_workspace_above_first` | `empty_row_above_first` | Rename only, keep behavior |
| `workspace_switch` animation | `row_switch` animation | Rename only |

**Fields to Remove Completely**:
- `workspace.idx` — Rows use i32 indices naturally
- `workspace.is_active` — Use `canvas.active_row_idx == row.row_index()`
- `workspace.is_focused` — Derive from monitor focus state

**Decision**: No backward compatibility — per TEAM_042 Q6c, remove immediately.

---

## 5. Window Operations Implementation - mod.rs

### Answer: **Delegate to Canvas2D methods; Canvas2D is the primary API**

**Delegation Pattern**:
```rust
// Layout delegates to Monitor, Monitor delegates to Canvas2D
// From mod.rs, window operations should follow this pattern:

// BEFORE (workspace-based):
fn add_window(&mut self, ...) {
    let workspace = self.active_workspace_mut()?;
    workspace.add_tile(...);
}

// AFTER (canvas-based):
fn add_window(&mut self, ...) {
    self.canvas.add_tile(...);  // Canvas2D handles row selection
}
```

**Canvas2D Already Has**:
- `add_tile()` — Adds to active row
- `find_window()`, `find_window_mut()` — Cross-row search
- `windows()`, `windows_mut()` — Iteration across all rows
- `active_window()`, `active_window_mut()` — Properly handles floating vs tiled

**Key Principle**: Canvas2D owns the complexity of multi-row operations.

---

## 6. API Compatibility & Migration

### Answer: **Maintain WorkspaceId externally, use row indices internally**

**IPC Compatibility Strategy**:

1. **Keep `niri-ipc::Workspace` struct unchanged**:
   ```rust
   pub struct Workspace {
       pub id: u64,      // From Row.id().0
       pub idx: u8,      // From row_index as u8
       pub name: Option<String>,  // From row.name()
       pub output: Option<String>,
       // ... rest unchanged
   }
   ```

2. **Row → Workspace conversion** (for IPC):
   ```rust
   fn row_to_workspace(row: &Row, output_name: &str) -> niri_ipc::Workspace {
       niri_ipc::Workspace {
           id: row.id().0,
           idx: row.row_index() as u8,
           name: row.name().map(|s| s.to_string()),
           output: Some(output_name.to_string()),
           // ... compute is_active, is_focused, etc from context
       }
   }
   ```

3. **Timeline**:
   - **Now**: IPC continues using "workspace" terminology, backed by Row data
   - **Phase 6**: Introduce "row" IPC APIs, deprecate "workspace" APIs
   - **Later Release**: Remove workspace APIs entirely

**The ext_workspace protocol can continue unchanged** — it uses u32 IDs which we provide from WorkspaceId.

---

# ARCHITECTURE ANSWERS

## 7. Data Model Consistency

### Answer: **Hybrid approach — i32 indices internally, WorkspaceId for external identity**

**Internal (Canvas2D, Row, Monitor)**:
- Use `i32` for row indices — natural for BTreeMap, allows negative rows
- Use `usize` for column indices — always positive
- Use `WorkspaceId` when communicating with external systems

**External (IPC, Protocols, Window Rules)**:
- Use `WorkspaceId` for stable identity
- Use "row name" for user-facing references

**Type Mapping**:
```rust
// Canvas2D internal:
rows: BTreeMap<i32, Row<W>>  // Key = row index
active_row_idx: i32

// Row internal:
row_index: i32
workspace_id: WorkspaceId  // Stable identity

// IPC/Protocol:
workspace.id: u64  // = Row.workspace_id.0
workspace.idx: u8  // = Row.row_index as u8 (for compatibility)
```

---

## 8. Performance Considerations

### Answer: **Canvas2D is more efficient for 2D operations, no significant overhead**

**Analysis**:

| Operation | Workspace (Old) | Canvas2D (New) | Impact |
|-----------|-----------------|----------------|--------|
| Find window | O(w) per workspace, O(n×w) total | O(n) across all rows | Same |
| Active workspace | O(1) direct access | O(1) via BTreeMap | Same |
| Render | Per-workspace culling | Per-row culling | Same |
| Row switch | N/A | O(log n) BTreeMap | New capability |

**Key Insight**: The BTreeMap for rows adds O(log n) lookup, but n is typically small (1-10 rows). The overhead is negligible.

**Optimization Already In Place**:
- `workspaces_with_render_geo()` filters rows outside viewport (render.rs line 126)
- Row geometry is cached in `ColumnData` (row/mod.rs line 68-73)

---

## 9. Testing Strategy

### Answer: **Gradual replacement with golden snapshot validation**

**Strategy**:
1. ✅ **Golden tests** — 88/88 passing, validate behavioral equivalence
2. ✅ **Compile-time verification** — Rust's type system catches API mismatches
3. 🔄 **Integration tests** — 55 failing tests need Canvas2D adaptation

**Approach**:
- Fix failing tests by updating them to use Canvas2D semantics
- Do NOT create parallel implementations — that's technical debt
- Golden tests are the safety net — if they pass, behavior is preserved

---

# IMPLEMENTATION STRATEGY ANSWERS

## 10. Phase Completion Criteria

### Answer: **Clearly defined criteria per phase**

**Phase 3 Monitor Layer — COMPLETE when**:
- [ ] All Monitor methods use Canvas2D instead of workspace list
- [ ] Insert hint rendering works with Row.id()
- [ ] Gestures navigate rows correctly
- [ ] Row removal in config.rs implemented
- [ ] 0 compilation errors in `src/layout/monitor/`

**Phase 4 Layout Mod Layer — COMPLETE when**:
- [ ] All workspace references removed from mod.rs
- [ ] Window operations delegate to Canvas2D
- [ ] Configuration uses row terminology
- [ ] All 268 tests pass (current: 213 pass, 55 fail)
- [ ] Golden tests still pass (88/88)

**Migration COMPLETE when**:
- [ ] `workspace_types.rs` only contains WorkspaceId for compatibility
- [ ] No TODO markers referencing workspace migration
- [ ] IPC returns valid data from Canvas2D/Row

---

## 11. Rollback Strategy

### Answer: **No rollback — forward-only migration**

**Rationale**:
- Old workspace code was **already deleted** (TEAM_021)
- Canvas2D is already the sole layout system
- Maintaining parallel systems creates more bugs than it solves

**If Issues Arise**:
1. Use `git revert` to individual TEAM commits if needed
2. Golden tests catch behavioral regressions
3. Compilation errors are immediate feedback

---

## 12. Documentation Requirements

### Answer: **Update wiki and config docs progressively**

**From TODO.md, these docs need updates**:
- [x] `Overview.md` — Overview removed
- [x] `Workspaces-legacy.md` — Archived
- [ ] `Configuration:-Named-Workspaces.md` → rename to `Configuration:-Named-Rows.md`
- [ ] `Configuration:-Key-Bindings.md` — Update actions to row-based
- [ ] `Configuration:-Gestures.md` — Update for row navigation
- [ ] `IPC.md` — Document that "workspace" is now backed by Row

---

# IMPLEMENTATION CHECKLIST

## Immediate Actions (Phase 3)

### gestures.rs TODOs (lines 142-147)
```rust
// Replace:
// TODO: TEAM_024: Get workspace ID from canvas row
// self.previous_workspace_id = Some(self.canvas.workspaces()[self.active_workspace_idx()].id());

// With:
// previous_row_idx is already tracked by Canvas2D.focus_row()

// Replace:
// TODO: TEAM_024: Set active workspace index in canvas
// self.active_workspace_idx = new_idx;

// With:
self.canvas.focus_row(new_idx as i32);
```

### config.rs TODO (line 28)
```rust
// Replace:
// TODO: TEAM_024: Implement row removal in canvas
// self.canvas.remove_row(0);

// With:
if let Some(row) = self.canvas.row(0) {
    if row.is_empty() && row.name().is_none() {
        self.canvas.remove_row(0);
    }
}
```

### render.rs TODO (line 45)
```rust
// The insert hint rendering already works correctly
// Row.id() returns WorkspaceId which InsertWorkspace::Existing expects
// No changes needed
```

## Phase 4 Cleanup

### mod.rs Workspace References
Each TEAM_020 TODO should be resolved by:
1. Replacing `workspace` variable names with `row`
2. Using `canvas.workspaces()` which iterates rows
3. Using `row.id()` when WorkspaceId is needed

---

# KEY DESIGN DECISIONS SUMMARY

| Decision | Choice | Rationale |
|----------|--------|-----------|
| InsertWorkspace | Keep as-is | Row.id() provides WorkspaceId |
| Gesture indices | i32 internally, WorkspaceId externally | Matches Canvas2D model |
| Row removal | Use Canvas2D.remove_row() | Already implemented correctly |
| Workspace config | Remove, no compatibility | Per TEAM_042 Q6c |
| Window operations | Delegate to Canvas2D | Single source of truth |
| IPC compatibility | Keep WorkspaceId, populate from Row | External tools continue working |
| Testing | Golden tests + gradual fix | Behavioral equivalence guaranteed |
| Rollback | None, forward-only | Old code deleted, no parallel path |

---

*TEAM_051 — Phase 3/4 Integration Masterplan Complete*
