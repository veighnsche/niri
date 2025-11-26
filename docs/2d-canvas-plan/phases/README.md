# Canvas2D Refactor Phases

> **Status**: IN PROGRESS - Phase 1.5.3
> **Goal**: Replace workspace-based layout with 2D canvas + rows

## Current Architecture (Legacy)
- **Workspaces**: 1D list of workspaces, each with scrolling columns
- **Overview Mode**: Special mode to view all workspaces
- **Navigation**: Workspace switching (up/down) + column navigation (left/right)

## Target Architecture (Canvas2D)
- **Canvas**: 2D grid of rows × columns
- **Rows**: Replace workspaces, can be named
- **Camera**: 2D viewport that can pan/zoom instead of overview mode
- **Navigation**: Direct 2D navigation (up/down/left/right)

---

## Phase Status

| Phase | Status | Description |
|-------|--------|-------------|
| [Phase 0](phase-0-preparation.md) | ✅ COMPLETE | Golden test infrastructure |
| [Phase 1](phase-1-row-and-canvas.md) | ✅ COMPLETE | Row and Canvas foundation |
| **Phase 1.5** | 🔄 **IN PROGRESS** | **Workspace → Row migration** |
| [Phase 2](phase-2-row-spanning.md) | ⏳ PENDING | Row spanning support |
| [Phase 3](phase-3-camera.md) | ⏳ PENDING | 2D camera system |
| [Phase 4](phase-4-navigation.md) | ⏳ PENDING | 2D navigation |
| [Phase 5](phase-5-integration.md) | ⏳ PENDING | Final integration |

---

## Phase 1.5: Workspace → Row Migration

**Current Status**: 🔄 **MISPLANNED - NEEDS REALIGNMENT**

### What Was Supposed to Happen:
1. **Part 1**: Monitor methods migration ✅
2. **Part 2**: Workspace → Row transformation (rename + implement) ❌
3. **Part 3**: Overview mode removal ✅
4. **Part 4**: Remove workspace fields ⏳
5. **Part 5**: Remove workspace config/IPC ⏳

### What Actually Happened:
- **Part 2**: Only did renaming, not implementation ❌
- **Part 3**: Overview removal completed ✅
- **Current**: "Row" methods still call workspace code ❌

### Remaining Work:
1. **Actually implement row-based navigation** (not just rename)
2. **Remove all workspace code** (files, fields, methods)
3. **Remove workspace config parsing**
4. **Remove workspace IPC endpoints**

---

## Active Phase Files

- [**phase-1.5.3-actual-row-implementation.md**](phase-1.5.3-actual-row-implementation.md) - **NEW**: Implement real row navigation
- [**phase-1.5.3-part4-remove-workspace-fields.md**](phase-1.5.3-part4-remove-workspace-fields.md) - Remove workspace structs/fields
- [**phase-1.5.3-part5-config-and-ipc.md**](phase-1.5.3-part5-config-and-ipc.md) - Remove workspace config/IPC
- [**phase-1.5.3-removal-checklist.md**](phase-1.5.3-removal-checklist.md) - Verification checklist

---

## Archived Phase Files

See `archive/` folder for old/misleading phase documentation that was:
- Overly fragmented (too many sub-parts)
- Misleading (renaming vs implementation)
- Outdated (superseded by actual progress)

**Key Lesson**: Future phases should be **implementation-focused**, not just **renaming-focused**.
