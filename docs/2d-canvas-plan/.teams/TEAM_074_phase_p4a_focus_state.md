# TEAM_074: Phase P4a Focus State Container Implementation

## Status: Starting Work

### Objective:
Implement Phase P4a: Extract Focus State Container from niri.rs refactor plan.

### Current State:
- Phase P1 (ProtocolStates) completed by TEAM_071
- Phase P2 (OutputSubsystem) completed by TEAM_072
- Phase P3 (CursorSubsystem) completed by TEAM_073
- Starting Phase P4a implementation

### Work Units:
1. ⏳ Create focus.rs with FocusState
2. ⏳ Add to subsystems/mod.rs
3. ⏳ Move Fields from Niri
4. ⏳ Update Access Patterns

### Files to Modify:
- `src/niri/subsystems/focus.rs` (new)
- `src/niri/subsystems/mod.rs` (add focus module)
- `src/niri/mod.rs` (remove fields, add subsystem)
- `src/niri/init.rs` (update initialization)
- Various files (update access patterns)

### Fields to Extract:
- `keyboard_focus`
- `layer_shell_on_demand_focus`
- `idle_inhibiting_surfaces`
- `keyboard_shortcuts_inhibiting_surfaces`

### Progress:
- Starting implementation

### Handoff:
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Team file complete
