# TEAM_060: Ext-Row Protocol Design for Canvas2D

**Date**: Nov 28, 2025  
**Focus**: Complete redesign of ext-workspace protocol for Canvas2D architecture  
**Status**: 🔄 Design Phase

## Overview

The ext-workspace protocol was designed for discrete workspace containers. Canvas2D has:
- **Continuous infinite canvas** - no discrete containers
- **Camera-based navigation** - pan/zoom instead of switching
- **Rows as layout strips** - horizontal organization within canvas
- **Camera bookmarks** - saved positions replacing workspace switching

## New Protocol: ext-row-v1

### Core Concepts

1. **Rows**: Horizontal layout strips within the canvas (not containers)
2. **Camera**: Viewport with position (x, y) and zoom level
3. **Bookmarks**: Saved camera positions for quick navigation
4. **Canvas**: Infinite 2D space containing all rows

### Protocol Differences

| Old Concept (ext-workspace) | New Concept (ext-row) |
|----------------------------|----------------------|
| Discrete workspaces | Continuous rows |
| Workspace switching | Camera panning/zooming |
| Workspace activation | Row focus within viewport |
| Workspace groups | Outputs (still relevant) |
| Workspace coordinates | Camera coordinates + row indices |

## Protocol Specification

### Interface Names

```rust
// New protocol interfaces
ext_row_manager_v1    // replaces ext_workspace_manager_v1
ext_row_handle_v1     // replaces ext_workspace_handle_v1  
ext_row_group_v1      // replaces ext_workspace_group_handle_v1
```

### Data Structures

#### Row Handle
```rust
ext_row_handle_v1 {
    // Row identification
    id: string (optional name, or numeric representation)
    index: uint (position in canvas)
    
    // Row state
    active: bool (row has focused window)
    focused: bool (row is currently focused)
    urgent: bool (row has urgent window)
    
    // Row geometry (relative to canvas)
    x: fixed
    y: fixed  
    width: fixed
    height: fixed
    
    // Row content info
    window_count: uint
    active_window_id: object_id (optional)
}
```

#### Camera State
```rust
ext_row_manager_v1 {
    // Current camera view
    camera_x: fixed
    camera_y: fixed
    camera_zoom: fixed
    
    // Canvas bounds (logical, infinite)
    canvas_width: fixed (effectively infinite)
    canvas_height: fixed (effectively infinite)
    
    // Visible rows in current viewport
    visible_rows: array<ext_row_handle_v1>
    
    // Camera bookmarks
    bookmarks: array<bookmark_info>
}

bookmark_info {
    id: uint
    name: string (optional)
    x: fixed
    y: fixed
    zoom: fixed
}
```

#### Row Group (Output)
```rust
ext_row_group_v1 {
    // Output identification
    output: output
    
    // Output's canvas view
    camera_x: fixed
    camera_y: fixed
    camera_zoom: fixed
    
    // Rows intersecting this output's view
    visible_rows: array<ext_row_handle_v1>
}
```

### Events

#### Row Events
```rust
// Row lifecycle
row_created(row_handle)
row_removed(row_handle)

// Row state changes  
row_activated(row_id, focused)
row_deactivated(row_id)
row_urgency_changed(row_id, urgent)
row_active_window_changed(row_id, window_id)

// Row geometry changes
row_geometry_changed(row_id, x, y, width, height)
row_content_changed(row_id, window_count)
```

#### Camera Events
```rust
// Camera movement
camera_moved(x, y, zoom)
camera_bookmark_created(bookmark_info)
camera_bookmark_removed(bookmark_id)
camera_bookmark_updated(bookmark_info)
```

#### Canvas Events
```rust
// Canvas-wide changes
canvas_layout_changed()
rows_reordered()
```

### Requests

#### Row Operations
```rust
// Row focus and navigation
focus_row(row_id)
focus_row_at(x, y)  // focus row at canvas coordinates

// Row modification (if supported)
set_row_name(row_id, name)
```

#### Camera Operations
```rust
// Camera control
set_camera_position(x, y, zoom)
move_camera_by(delta_x, delta_y, delta_zoom)
center_camera_on_row(row_id)

// Bookmark management
create_bookmark(name, x, y, zoom)
remove_bookmark(bookmark_id)
goto_bookmark(bookmark_id)
```

#### Navigation
```rust
// High-level navigation
focus_row_up()
focus_row_down()
focus_row_left()
focus_row_right()
```

## Implementation Strategy

### Phase 1: Protocol Definition
1. Create new protocol XML files
2. Generate Rust bindings with wayland-scanner
3. Define new Rust trait interfaces

### Phase 2: State Management
1. Replace `ExtWorkspaceManagerState` with `ExtRowManagerState`
2. Implement camera state tracking
3. Implement bookmark management
4. Map internal Row state to protocol Row handles

### Phase 3: Event Generation
1. Replace workspace events with row events
2. Add camera movement events
3. Add bookmark events
4. Handle viewport culling for visible rows

### Phase 4: Request Handling
1. Implement row focus requests
2. Implement camera control requests
3. Implement bookmark operations
4. Map to internal Canvas2D operations

## Compatibility Considerations

### Breaking Changes
This is a complete protocol redesign, so:
- **Client compatibility**: Will be broken
- **Protocol version**: New major version (v1)
- **Interface names**: Completely different
- **Event types**: Fundamentally different

### Migration Path
1. **Dual protocol support**: Keep old protocol during transition
2. **Capability advertising**: Clients can detect new protocol
3. **Graceful fallback**: Old clients use old protocol, new clients use new one
4. **Documentation**: Clear migration guide for client developers

## Technical Challenges

### Viewport Culling
- Only send visible rows to clients
- Handle dynamic visibility as camera moves
- Balance between accuracy and performance

### Camera Coordinate System
- Map between logical and physical coordinates
- Handle zoom level transformations
- Coordinate with output scaling

### Bookmark Persistence
- Decide if bookmarks are session-only or persistent
- Handle bookmark conflicts and naming
- Integrate with configuration system

### Performance Optimization
- Minimize event traffic during camera movement
- Batch geometry updates
- Efficient row lookup by coordinates

## Files to Create/Modify

### New Files
```
src/protocols/ext_row/
├── ext_row_v1.xml          # Protocol specification
├── manager.rs              # Row manager implementation
├── row_handle.rs           # Row handle implementation  
├── row_group.rs            # Row group (output) implementation
├── state.rs                # Protocol state management
└── lib.rs                  # Public interface
```

### Modified Files
```
src/niri.rs                 # Replace ext_workspace with ext_row
src/handlers/mod.rs         # Update protocol handlers
src/layout/                 # Add camera/bookmark event generation
```

### Removed Files (eventually)
```
src/protocols/ext_workspace.rs  # Keep during transition period
```

## Next Steps

1. **Protocol Review**: Review and finalize protocol design
2. **XML Specification**: Write official protocol XML
3. **Rust Implementation**: Start with basic state management
4. **Testing**: Create test clients for new protocol
5. **Documentation**: Write migration guide for client developers

## Design Questions

1. **Bookmark Scope**: Should bookmarks be per-output or global?
2. **Row Identification**: Use numeric indices or string names?
3. **Camera Events**: How granular should camera movement events be?
4. **Backward Compatibility**: How long to maintain old protocol?

This redesign provides a clean foundation that properly represents Canvas2D's capabilities while maintaining a similar interface pattern for client developers.
