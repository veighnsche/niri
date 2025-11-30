# TEAM_071: Phase P1 - Extract ProtocolStates Container

> **Status**: 🔄 In Progress  
> **Date**: 2025-11-29  
> **Phase**: P1 - Extract ProtocolStates Container  
> **Time Estimate**: ~30 minutes  
> **Risk Level**: 🟢 Low (pure mechanical grouping)

---

## Goal

Group the 25+ Smithay protocol state fields from `Niri` into a single `ProtocolStates` container struct.

This is the safest first step: pure mechanical refactoring with no behavioral changes.

---

## Current State Analysis

I need to examine the current `src/niri/mod.rs` to understand:
- The exact protocol state fields present
- Their initialization patterns in `Niri::new`
- Their access patterns throughout the codebase

---

## Work Plan

Following the phase document exactly:

### Unit 1: Create protocols.rs
- Create `src/niri/protocols.rs` with ProtocolStates struct
- Add all necessary imports
- Define struct with all protocol fields (shortened names)
- Add empty impl block

### Unit 2: Add Module Declaration
- Update `src/niri/mod.rs` to include protocols module
- Add pub use declaration

### Unit 3: Move Fields to ProtocolStates
- Cut protocol fields from Niri struct
- Paste into ProtocolStates struct
- Rename fields (remove `_state` suffix)

### Unit 4: Update All Access Patterns
- Search and replace usage patterns
- `self.compositor_state` → `self.protocols.compositor`
- `niri.xdg_shell_state` → `niri.protocols.xdg_shell`

### Unit 5: Move Initialization to ProtocolStates::new
- Extract protocol initialization from Niri::new
- Move to ProtocolStates::new method
- Update Niri::new to use ProtocolStates::new

---

## Verification

- [ ] `cargo check` passes
- [ ] `cargo test` passes (270 tests)
- [ ] All 25+ protocol fields moved from Niri
- [ ] Niri has single `protocols: ProtocolStates` field
- [ ] All access patterns updated

---

## Handoff

- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Team file complete
- [ ] Ready for Phase P2: OutputSubsystem
