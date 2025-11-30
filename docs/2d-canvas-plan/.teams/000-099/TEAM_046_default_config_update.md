# TEAM_046 — Default Config Update

## Scope
- Update `resources/default-config.kdl` to match the 2D Canvas row-based layout.
- Remove or rewrite outdated workspace/overview-related bindings and comments.
- Keep the config valid for the current `niri-config` parser (no legacy workspace actions).

## Notes
- Overview actions were removed from IPC; default bind for `Mod+O` is now unbound.
- Alternative workspace navigation binds are now documented using row-based actions.
- Mouse wheel scroll comments now describe scrolling through rows instead of workspaces.

## Handoff
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Golden tests pass (`cargo insta test`) — N/A for config-only change
- [ ] Team file complete (this file)
