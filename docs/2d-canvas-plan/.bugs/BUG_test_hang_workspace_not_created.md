# BUG_test_hang_workspace_not_created

## Symptom
`cargo test` hangs forever on `window_opening::target_output_and_workspaces` tests. The test shows:
- Window goes to wrong output (`headless-1` instead of `headless-2`)
- Window goes to unnamed workspace instead of `ws-2`
- Initial configure has `0 × 0` bounds instead of proper values

## Reproduction
Run `cargo test` - the `target_output_and_workspaces` test hangs.

The test config specifies:
```
workspace "ws-2" {
    open-on-output "headless-2"
}

window-rule {
    exclude title="parent"
    open-on-workspace "ws-2"
}
```

But the workspace `ws-2` is never created, so the window falls back to the active monitor.

## Root Cause
**FOUND**: Named workspaces from config are only created in `State::reload_config()`, but `Server::new()` (used by test fixture) only calls `State::new()` which doesn't create named workspaces.

The flow is:
1. `Server::new(config)` → `State::new(config, ...)` → `Niri::new(config, ...)` → `Layout::new(clock, config)`
2. `Layout::new` creates a default canvas but does NOT call `ensure_named_workspace` for workspaces in config
3. When window opens with `open-on-workspace "ws-2"`, `monitor_for_workspace("ws-2")` returns `None`
4. Window falls back to active monitor (headless-1)
5. Test expects headless-2, assertion fails
6. Test hangs because parallel rayon tests keep running

## Hypothesis Log
| Chase ID | Team | Hypothesis | Result | Notes |
|----------|------|------------|--------|-------|
| 001 | TEAM_042 | Named workspaces not created at startup | ROOT CAUSE | See Root Cause section |

## Current Status
INVESTIGATING

## Fix
Add workspace creation to `Layout::with_options_and_workspaces()` or ensure `Server::new()` calls the equivalent of workspace creation from config.

The fix should iterate over `config.workspaces` and call `ensure_named_workspace` for each.
