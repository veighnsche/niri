# Canvas2D Refactor Phases

> **Status**: 🔄 **IN PROGRESS - Phase 1**
> **Goal**: Replace workspace-based layout with 2D canvas + rows

## Target Architecture (Canvas2D)

- **Canvas**: 2D infinite surface per output
- **Rows**: Horizontal strips indexed by integers (..., -2, -1, 0, 1, 2, ...)
- **Row 0**: Origin point, always exists
- **Camera**: 2D viewport with (x, y, zoom) for navigation
- **Bookmarks**: Saved camera positions (replaces workspace switching)

---

## Phase Status

| Phase | Status | Description |
|-------|--------|-------------|
| [**Phase 1**](phase-1-config-migration.md) | 🔄 **CURRENT** | Config migration (workspace → row) |
| [Phase 2](phase-2-row-system.md) | ⏳ PENDING | Row lifecycle and naming |
| [Phase 3](phase-3-row-spanning.md) | ⏳ PENDING | Windows spanning multiple rows |
| [Phase 4](phase-4-camera-system.md) | ⏳ PENDING | Camera with zoom |
| [Phase 5](phase-5-camera-bookmarks.md) | ⏳ PENDING | Camera bookmarks (Mod+1/2/3) |
| [Phase 6](phase-6-navigation-polish.md) | ⏳ PENDING | Geometric navigation & polish |

---

## Key Decisions (from TEAM_042 Questionnaire)

### Row Naming
- Any row can be named, including row 0
- Names unique per output, case-insensitive

### Row Lifecycle
- Created when window added OR explicitly requested
- Empty unnamed rows deleted, named rows persist
- Row 0 always exists (origin)

### Active Row
- Follows focus, can be explicitly set
- Default is row 0 when output added
- Windows open on active row unless `open-on-row` specified

### Config Syntax
- `workspace` → `row` (remove immediately, no deprecation)
- `open-on-workspace` → `open-on-row`

### Camera Bookmarks
- Saved (x, y, zoom) positions
- Can optionally reference named rows
- `Mod+1/2/3` jump to bookmarks

---

## Archived Phases

See `archive/` folder for completed phase documentation:
- Phase 0: Golden test infrastructure ✅
- Phase 1 (old): Row and Canvas foundation ✅
- Phase 1.5: Workspace → Canvas2D migration ✅
- Phase 6 (old): Workspace cleanup ✅

---

## Quick Reference

| Old Concept | New Concept |
|-------------|-------------|
| Workspace | Row |
| `workspace "name" {}` | `row "name" {}` |
| `open-on-workspace` | `open-on-row` |
| `Mod+1/2/3` (workspace) | `Mod+1/2/3` (bookmark) |
| Overview mode | Camera zoom out |
| Workspace switching | Row navigation + bookmarks |
