# TEAM_089 — TTY Refactor Audit & Fix

> **Started**: Nov 30, 2025

## Mission

Audit the claimed work on phases T1.4b, T1.5, T1.6, T1.7 and complete the missing implementations.

---

## Audit Results: What Was Claimed vs What Was Done

### Phase T1.4a: DeviceManager Struct & Accessors
**Status**: ✅ **COMPLETE** — Struct exists with all accessors

### Phases T1.4b, T1.5, T1.6, T1.7
**Status**: ❌ **NOT DONE** — Only stubs created, no actual code moved

**mod.rs is still 2594 lines** — should be ~600 lines after all phases complete.

---

## New Phase Structure

Archived old phases to `phases/archive/` and created 13 focused phase files:

| # | Phase | Target | LOC | Status |
|---|-------|--------|-----|--------|
| 01 | [device_added](../phases/01-device-added.md) | devices.rs | ~196 | ⏳ |
| 02 | [device_changed](../phases/02-device-changed.md) | devices.rs | ~163 | ⏳ |
| 03 | [device_removed](../phases/03-device-removed.md) | devices.rs | ~108 | ⏳ |
| 04 | [connector_connected](../phases/04-connector-connected.md) | devices.rs | ~347 | ⏳ |
| 05 | [render](../phases/05-render.md) | render.rs | ~181 | ⏳ |
| 06 | [on_vblank](../phases/06-on-vblank.md) | render.rs | ~188 | ⏳ |
| 07 | [estimated_vblank_timer](../phases/07-estimated-vblank-timer.md) | render.rs | ~77 | ⏳ |
| 08 | [refresh_ipc_outputs](../phases/08-refresh-ipc-outputs.md) | outputs.rs | ~110 | ⏳ |
| 09 | [gamma_control](../phases/09-gamma-control.md) | outputs.rs | ~43 | ⏳ |
| 10 | [set_monitors_active](../phases/10-set-monitors-active.md) | outputs.rs | ~18 | ⏳ |
| 11 | [vrr_control](../phases/11-vrr-control.md) | outputs.rs | ~33 | ⏳ |
| 12 | [on_output_config_changed](../phases/12-on-output-config-changed.md) | outputs.rs | ~192 | ⏳ |
| 13 | [final_cleanup](../phases/13-final-cleanup.md) | all | - | ⏳ |

**Total**: ~1656 LOC to move

---

## Execution Groups

### Group A: Device Lifecycle (01-04)
Must be done in order. Move device management methods to `devices.rs`.

### Group B: Render Pipeline (05-07)
Can be done after Group A. Move render methods to `render.rs`.

### Group C: Output Management (08-12)
Can be done after Group A. Move output methods to `outputs.rs`.

### Final: Cleanup (13)
Remove dead code, update documentation.

---

## Progress

- [x] Audit complete
- [x] New phase files created
- [ ] Group A: Device lifecycle (01-04)
- [ ] Group B: Render pipeline (05-07)
- [ ] Group C: Output management (08-12)
- [ ] Final cleanup (13)

---

## Handoff

- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] mod.rs < 700 LOC
- [ ] Team file complete
