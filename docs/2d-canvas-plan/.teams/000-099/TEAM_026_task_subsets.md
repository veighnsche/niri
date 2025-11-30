# TEAM_026: Subset Checklist

## Objective
Break the remaining workspace → Canvas2D migration work into tractable subsets so we can tackle `cargo check` errors incrementally.

## Subsets
- [ ] Tuple/iterator cleanup
  - Ensure every use of `canvas.workspaces()`/`workspaces_mut()` destructures `(row_idx, row)` correctly and avoids assuming `Workspace` types.
  - Replace `.workspaces()` loops with `canvas.rows()` where only row references are needed.
- [ ] Row method implementations
  - Fill in the stubbed `Row` helpers (`has_window`, `has_windows_or_name`, `activate_window`, `resolve_scrolling_width`, etc.) so existing callers compile.
  - Add any Canvas2D facade methods that still expect the old `Workspace` API (e.g., `rows_mut()` helpers, `remove_row`).
- [ ] Field access / method signature migration
  - Remove all remaining `.workspaces` field accesses from `Monitor`/`Layout` and switch to `canvas`/`row` accessors.
  - Adjust call sites that still expect `Workspace`-style arguments (e.g., `has_window(&W)` vs `has_window(&W::Id)` differences).
- [ ] Config/input/IPC clean-up
  - Finish disabling or rewriting workspace-specific keybinds, IPC commands, and config parsing noted in Phase 1.5.3 steps so no workspace APIs remain.
- [ ] Documentation/logging updates
  - Update `.teams/TEAM_026_*` logs and phase files to reflect subset progress.
  - Note any blockers or TODO(TEAM_026) markers added during implementation.

## Notes
- Golden tests were verified before starting; continue to run them when modifying layout logic.
- Each checked subset should leave `cargo check` closer to success without requiring an all-at-once approach.

*Created by TEAM_026*
