# TEAM_042: Canvas2D and Row Architecture Questions

> **Status**: ✅ RESOLVED - All decisions made
> **Context**: Defining the Canvas2D/Row system semantics (replacing old workspace concepts)
> **Outcome**: New phase files created based on USER decisions

---

## Canvas2D System Overview

Each output has a **Canvas2D** - an infinite 2D surface containing:
- **Rows**: Horizontal strips indexed by integers (..., -2, -1, 0, 1, 2, ...)
- **Origin**: Row 0 is the origin point
- **Active Row**: The currently focused row
- **Columns**: Within each row, windows are organized in columns

Navigation:
- `Mod+Up/Down`: Move focus between rows
- `Mod+Shift+Up/Down`: Move window/column to another row
- `Mod+Left/Right`: Move focus between columns within a row

---

## Questions

### Q1: Row Naming

Rows can optionally have names (for user reference and targeting).

**Questions**:
- **Q1a)** Can any row be named, including row 0? Yes, any row can be named
- **Q1b)** Can multiple rows have the same name? No, names should be unique per output
- **Q1c)** Are row names case-sensitive? No, case-insensitive (current behavior)
- **Q1d)** What characters are allowed in row names? Any string (let users decide)

**My recommendation**: 
- Q1a: Yes, any row can be named
- Q1b: No, names should be unique per output
- Q1c: No, case-insensitive (current behavior)
- Q1d: Any string (let users decide)

---

### Q2: Row Lifecycle

**Questions**:
- **Q2a)** When is a row created?
  - A) Only when a window is added to it
  - B) When explicitly requested (config, keybind, IPC)
  - C) Both <-

- **Q2b)** When is a row deleted?
  - A) When it becomes empty (no windows)
  - B) Only when explicitly requested
  - C) Empty rows are deleted, except named rows and row 0 <-

- **Q2c)** Is row 0 special?
  - A) Yes - always exists, cannot be deleted, is the default <- Yes but the reason is because it is the row that is on the 0,0 point of the canvas... also also... in our canvas we can also go negative, meaning that we can also go UP! rather then only going down...
  - B) No - row 0 is just like any other row

**My recommendation**: 
- Q2a: C (both)
- Q2b: C (empty unnamed rows are cleaned up, named rows persist)
- Q2c: A (row 0 is the origin, always exists)

---

### Q3: Active Row

**Questions**:
- **Q3a)** What determines the active row on an output?
  - A) The row containing the focused window
  - B) Explicitly set by user (persists even when empty)
  - C) Both (follows focus, but can be explicitly set) <- Yes but there is a difference... because in our Canvas2D we can zoom out and we can see multiple rows as the same time... we control the scaling of each window at the same time.... that is a feature that is completely new for Canvas2D... just keep that in mind... we need to zoom out gradually... and also if you look in the README.md of the 2d canvas plan... you can see that windows also can span across multiple rows.... so that is ALSO a completely new thing...

- **Q3b)** What is the default active row when an output is added?
  - A) Row 0 <-
  - B) The first named row (if any), otherwise row 0
  - C) Configurable per-output

- **Q3c)** When a window opens on an output without specifying a row, where does it go?
  - A) The active row <- ... right... so if a window SPANS across multiple rows... the 0,0 point (or the top left corner of the window) decided which row it is on... 
  - B) Row 0
  - C) The row containing the focused window on that output

**My recommendation**: 
- Q3a: C (follows focus, can be explicitly set)
- Q3b: A (row 0)
- Q3c: A (active row)

---

### Q4: Window Placement Rules

Window rules can specify where windows open.

**Questions**:
- **Q4a)** What targeting options should exist?
  - `open-on-output "name"` - target a specific output
  - `open-on-row "name"` - target a named row (NEW, replaces `open-on-workspace`)
  - `open-on-row-index N` - target row by index (NEW)
  - Others?

  `open-on-output` and `open-on-row` (deprecate `open-on-workspace`)

- **Q4b)** What happens if `open-on-row` specifies a non-existent row name?
  - A) Create the row with that name <-
  - B) Fall back to active row
  - C) Error/warning

- **Q4c)** Should `open-on-output` without `open-on-row` go to:
  - A) The active row on that output <-
  - B) Row 0 on that output
  - C) The first named row on that output (if any)

**My recommendation**: 
- Q4a: `open-on-output` and `open-on-row` (deprecate `open-on-workspace`)
- Q4b: A (create the row)
- Q4c: A (active row)

---

### Q5: Row IDs (Internal)

Rows need unique identifiers for IPC and internal tracking.

**Questions**:
- **Q5a)** How should row IDs be structured?
  - A) Global counter (unique across all outputs) <-
  - B) Per-output counter (unique within output)
  - C) Composite (output_id, row_index)

- **Q5b)** Should row IDs be stable across sessions?
  - A) Yes (persist in config/state)
  - B) No (regenerated each session) <-

**My recommendation**: 
- Q5a: A (global counter for simplicity)
- Q5b: B (no persistence needed yet, can add later)

---

### Q6: Config Syntax

**Questions**:
- **Q6a)** Should we keep the `workspace` config block or replace it?
  ```kdl
  # OLD (workspace)
  workspace "browser" {
      open-on-output "eDP-1"
  }
  
  # NEW (row)
  row "browser" {
      open-on-output "eDP-1"
  }
  ```
  Replace with `row` block

- **Q6b)** Should we keep `open-on-workspace` or replace with `open-on-row`? Replace with `open-on-row`

- **Q6c)** Deprecation timeline?
  - A) Remove workspace syntax immediately <-
  - B) Keep as alias, emit warning
  - C) Keep both indefinitely

**My recommendation**: 
- Q6a: Replace with `row` block
- Q6b: Replace with `open-on-row`
- Q6c: B (keep as alias with warning for one release cycle)

---

### Q7: Camera Bookmarks (Future)

The plan mentions camera bookmarks replacing workspace switching.

**Questions**:
- **Q7a)** What is a camera bookmark?
  - A) A saved (x, y, zoom) position <- Yes this... so in our Canvas2D thingy... we need to keep track of that
  - B) A saved (row_index, column_index) position
  - C) A saved (row_name) reference

- **Q7b)** How do bookmarks relate to named rows?
  - A) Bookmarks are separate from rows
  - B) Named rows automatically create bookmarks
  - C) Bookmarks can optionally reference a named row <-

- **Q7c)** Should `Mod+1/2/3` jump to bookmarks or rows? Bookmarks (more powerful than just rows)

**My recommendation**: 
- Q7a: A (camera position, more flexible)
- Q7b: C (optional reference)
- Q7c: Bookmarks (more powerful than just rows)

---

### Q8: Legacy Test Handling

The test `target_output_and_workspaces` uses old workspace semantics.

**Questions**:
- **Q8a)** Should we:
  - A) Delete the test (workspace behavior is deprecated)
  - B) Update the test to use row semantics <-
  - C) Keep test for backwards compatibility layer

**My recommendation**: B (update to row semantics)

---

## Summary

| Question | Topic | My Recommendation |
|----------|-------|-------------------|
| Q1 | Row naming | Any row can be named, unique per output |
| Q2 | Row lifecycle | Empty unnamed rows cleaned up, named rows persist |
| Q3 | Active row | Follows focus, default is row 0 |
| Q4 | Window placement | `open-on-row` replaces `open-on-workspace` |
| Q5 | Row IDs | Global counter, not persisted |
| Q6 | Config syntax | Replace `workspace` with `row`, deprecation warning |
| Q7 | Camera bookmarks | Saved positions, separate from rows |
| Q8 | Legacy tests | Update to row semantics |

---

## ✅ Decisions Applied

All USER decisions have been incorporated into new phase files:

| Phase | File | Description |
|-------|------|-------------|
| 1 | `phase-1-config-migration.md` | Replace workspace → row config syntax |
| 2 | `phase-2-row-system.md` | Row lifecycle, naming, IDs |
| 3 | `phase-3-row-spanning.md` | Windows spanning multiple rows |
| 4 | `phase-4-camera-system.md` | Camera with (x, y, zoom) |
| 5 | `phase-5-camera-bookmarks.md` | Saved camera positions |
| 6 | `phase-6-navigation-polish.md` | Geometric navigation, polish |

Old phase files archived to `phases/archive/`.

---

*TEAM_042 - Questionnaire resolved, phases created*
