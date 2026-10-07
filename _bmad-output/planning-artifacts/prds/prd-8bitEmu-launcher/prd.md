---
title: Launcher shell
status: done
created: 2026-10-07
updated: 2026-10-07
parent_issue: 105
---

# PRD: Launcher shell

Hobby/solo. Open one empty shared window at startup with a menu to pick the machine and ROM paths; Start boots that machine in the same window. CLI boot args keep direct launch.

## 1. Vision

A common shell so Spectrum 48K/128K and C64 share one entry point. Settings (machine + ROM locations) persist. Later stories hang games/BASIC/open flows off this window.

## 2. User journeys

- **UJ-1.** Ferenc double-clicks the app (no args). An empty window opens. He picks Machine → C64, sets ROM paths under Settings, then Start, and sees the C64 boot screen in that window.
- **UJ-2.** Ferenc runs `eightbit-emu --prg game.prg`. The launcher is skipped; the C64 session boots as today.

## 3. Features

#### FR-1: Empty startup window

With no CLI boot arguments the app opens one empty window (no machine ticking).

**Consequences:**
- `parse_mode` / empty args select launcher mode.
- Idle redraw clears a dark frame without advancing a Presenter.

#### FR-2: Machine menu

A Machine menu chooses Spectrum 48K, Spectrum 128K, or C64 (check items). The choice is saved.

**Consequences:**
- Selecting an item updates `AppConfig` and persists.
- Check state reflects the saved machine/model.

#### FR-3: ROM path settings

Settings menu items pick Spectrum ROM and C64 kernal/basic/chargen paths via file dialogs. Paths persist.

**Consequences:**
- Round-trip config file test for the four path keys.
- Defaults match current repo relative paths (`roms/spectrum-48.rom`, `roms/c64-*.rom`).

#### FR-4: Start in the same window

Machine → Start (or equivalent) builds a Session from config and attaches the Presenter in the same window.

**Consequences:**
- After Start, the machine paints and accepts input as in a CLI boot of that machine.
- Missing ROMs surface a clear error and leave the shell idle.

#### FR-5: CLI boot bypass

Any of `--machine`, `--rom`, `--sna`, `--tap`, `--tzx`, `--prg`, `--kernal`, `--basic`, `--chargen` boots directly without the launcher.

**Consequences:**
- Existing launch integration tests still pass; empty-args test expects launcher mode.

## 4. Non-goals

Separate settings dialog UI, D64, launcher Open-game flow, multiple windows, imgui panels.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Empty args → launcher mode; boot args → Boot(Launch).
- **SM-C2**: Config load/save round-trip for machine and ROM paths.
