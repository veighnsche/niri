# Phase 5: Camera Bookmarks

> **Status**: ⏳ PENDING (after Phase 4)
> **Goal**: Save and restore camera positions (x, y, zoom)

---

## Overview

Based on USER decisions:
- Camera bookmarks are saved (x, y, zoom) positions
- Bookmarks can optionally reference a named row
- `Mod+1/2/3` jump to bookmarks (more powerful than just rows)
- This replaces the old workspace switching keybinds

---

## Core Concept

A bookmark saves the camera state:
```rust
pub struct CameraBookmark {
    /// Saved camera position
    x: f64,
    y: f64,
    zoom: f64,
    
    /// Optional: reference to a named row
    /// If set, jumping to bookmark also ensures this row exists
    row_name: Option<String>,
    
    /// User-defined name for the bookmark
    name: Option<String>,
}
```

---

## Step 5.1: Bookmark Storage

### Tasks

- [ ] **5.1.1**: Add `bookmarks: Vec<CameraBookmark>` to Canvas2D
- [ ] **5.1.2**: Implement `save_bookmark(slot: usize)`
- [ ] **5.1.3**: Implement `jump_to_bookmark(slot: usize)`
- [ ] **5.1.4**: Support up to 10 bookmarks (slots 0-9)

### Implementation

```rust
impl Canvas2D {
    pub fn save_bookmark(&mut self, slot: usize) {
        let bookmark = CameraBookmark {
            x: self.camera.x(),
            y: self.camera.y(),
            zoom: self.camera.zoom(),
            row_name: self.active_row().and_then(|r| r.name().map(String::from)),
            name: None,
        };
        
        if slot >= self.bookmarks.len() {
            self.bookmarks.resize(slot + 1, CameraBookmark::default());
        }
        self.bookmarks[slot] = bookmark;
    }
    
    pub fn jump_to_bookmark(&mut self, slot: usize) -> bool {
        let Some(bookmark) = self.bookmarks.get(slot).cloned() else {
            return false;
        };
        
        // If bookmark references a named row, ensure it exists
        if let Some(ref name) = bookmark.row_name {
            self.ensure_named_row(name);
        }
        
        self.camera.animate_to(
            bookmark.x,
            bookmark.y,
            bookmark.zoom,
            &self.options.animations.camera,
        );
        true
    }
}
```

---

## Step 5.2: Keybinds

### Actions

| Action | Effect |
|--------|--------|
| `jump-to-bookmark N` | Jump to bookmark slot N |
| `save-bookmark N` | Save current camera to slot N |
| `delete-bookmark N` | Delete bookmark in slot N |

### Default Keybinds

```kdl
binds {
    Mod+1 { jump-to-bookmark 1; }
    Mod+2 { jump-to-bookmark 2; }
    Mod+3 { jump-to-bookmark 3; }
    // ... up to 9
    
    Mod+Shift+1 { save-bookmark 1; }
    Mod+Shift+2 { save-bookmark 2; }
    Mod+Shift+3 { save-bookmark 3; }
    // ... up to 9
}
```

### Tasks

- [ ] **5.2.1**: Add `jump-to-bookmark` action
- [ ] **5.2.2**: Add `save-bookmark` action
- [ ] **5.2.3**: Add `delete-bookmark` action
- [ ] **5.2.4**: Update default-config.kdl with keybinds

---

## Step 5.3: Row Reference

### Behavior

When a bookmark references a named row:
1. If row exists → jump to bookmark position
2. If row doesn't exist → create it, then jump

This allows bookmarks to "remember" a row even if it was deleted.

### Tasks

- [ ] **5.3.1**: Store row name in bookmark when saving
- [ ] **5.3.2**: Ensure row exists when jumping
- [ ] **5.3.3**: Handle case where row was renamed

---

## Step 5.4: Config Persistence (Optional)

### Tasks

- [ ] **5.4.1**: Consider persisting bookmarks to config/state file
- [ ] **5.4.2**: Load bookmarks on startup
- [ ] **5.4.3**: Save bookmarks on change

**Note**: Per USER decision, row IDs are not persisted. Bookmarks could be persisted separately if desired.

---

## Step 5.5: IPC

### New Commands

| Command | Description |
|---------|-------------|
| `niri msg bookmarks` | List all bookmarks |
| `niri msg jump-to-bookmark N` | Jump to bookmark N |
| `niri msg save-bookmark N` | Save current position to bookmark N |

### Tasks

- [ ] **5.5.1**: Add bookmark IPC commands
- [ ] **5.5.2**: Return bookmark info (position, row name)

---

## Success Criteria

- [ ] Can save camera position to bookmark
- [ ] Can jump to saved bookmark
- [ ] `Mod+1/2/3` work for bookmarks
- [ ] Bookmarks can reference named rows
- [ ] Jumping to bookmark with row reference creates row if needed
- [ ] Smooth animation when jumping

---

*Phase 5 - Camera Bookmarks*
