# TEAM_093 — Phase 04: Move connector_connected to DeviceManager

> **Started**: Nov 30, 2025
> **Phase**: 04-connector-connected.md
> **Status**: 🔄 IN PROGRESS

## Mission

Implement Phase 04 of the TTY refactor: move the `connector_connected` method from `Tty` to `DeviceManager`. This is the largest single method to move (~347 lines) and handles creating a DRM surface when a monitor is connected.

---

## Implementation Plan

### Step 1: Add required imports to devices.rs
### Step 2: Move connector_connected method body to DeviceManager
### Step 3: Transform method signature and return type
### Step 4: Create delegation wrapper in Tty
### Step 5: Update method calls and verification

---

## Progress

- [x] Team registration
- [ ] Add imports to devices.rs
- [ ] Move connector_connected method to DeviceManager  
- [ ] Transform method signature and return type
- [ ] Create Tty delegation wrapper
- [ ] Verify compilation

---

## Technical Notes

**Source**: `src/backend/tty/mod.rs` lines 1011-1357  
**Target**: `src/backend/tty/devices.rs` DeviceManager impl  
**LOC**: ~347 lines to move

Key transformations needed:
- Add `config` and `debug_tint` parameters
- Apply `self.devices.*` → `self.*` transformations
- Replace `self.config.borrow()` with `config.borrow()`
- Replace `self.render.debug_tint()` with `debug_tint` param

Note: `connector_disconnected` is already in DeviceManager.
