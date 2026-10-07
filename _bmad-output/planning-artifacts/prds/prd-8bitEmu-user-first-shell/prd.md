---
title: User-first shell
status: done
created: 2026-10-07
updated: 2026-10-07
parent_issue: 108
---

# PRD: User-first shell

Hobby/solo. Make the launcher feel obvious: the app does what people expect with almost no setup. Config stays flexible for power users but never blocks the happy path.

## 0. Product principles (every later feature)

1. **User first** — Prefer the action the user meant over exposing knobs.
2. **Zero setup when possible** — Sensible defaults; discover ROMs; remember the last machine.
3. **One gesture** — Choosing a machine or opening a file should start play, not open a settings scavenger hunt.
4. **Fail helpful** — If something is missing, ask for that one thing (or a ROM folder), then continue.
5. **Power under the hood** — Explicit paths and CLI remain; they must not be required for normal use.

Future Spectrum/C64 stories must cite these principles in their PRD decision log.

## 1. Vision

Double-click → a working machine (last choice, or Spectrum 48K). Open a game → the right machine boots and loads it. ROM paths resolve from a folder of standard names; Settings is for exceptions.

## 2. User journeys

- **UJ-1.** Ferenc opens the app. The last machine boots immediately (first run: Spectrum 48K if its ROM is found).
- **UJ-2.** He picks Machine → Commodore 64. The C64 boots in the same window without a separate Start.
- **UJ-3.** From the empty or running shell he chooses File → Open… on a `.prg`. C64 boots (or switches) and soft-loads the program.
- **UJ-4.** ROMs live in `roms/` (or a chosen folder). He never fills four path fields unless he wants overrides.

## 3. Features

#### FR-1: Auto-start last machine

Launcher mode starts the configured machine as soon as the window opens when required ROMs resolve. Idle black screen only if start fails.

**Consequences:**
- `run_launcher` / first `open` attempts start.
- Unit/integration coverage: resolved defaults → launch paths set; missing ROM leaves idle.

#### FR-2: Machine choice starts

Selecting Spectrum 48K / 128K / C64 saves the choice and starts that machine immediately. Start remains as Restart.

**Consequences:**
- Menu handler calls start after apply.
- 128K resolves `spectrum-128.rom` by default; 48K resolves `spectrum-48.rom`.

#### FR-3: Open works anytime

File → Open… (and ⌘O) works in Idle and Running. File type picks the machine, boots if needed, then loads media (ROM/SNA/tape/PRG).

**Consequences:**
- Idle Open no longer no-ops.
- Opening a `.prg` on Spectrum switches to C64 with ROMs then loads PRG.
- Opening Spectrum media on C64 switches to Spectrum.

#### FR-4: Smart ROM discovery

Resolve ROMs from, in order: explicit override path if it exists; `rom_folder` + standard name; candidate folders (`./roms`, executable-adjacent `roms`, Application Support `roms`).

**Consequences:**
- Config gains optional `rom_folder`.
- Tests cover discovery order and 48 vs 128 filename choice.
- Settings: primary **ROM Folder…**; per-file picks remain as overrides.

#### FR-5: Principles recorded for the roadmap

This PRD’s §0 is the bar for later features (games open flow, D64, etc.).

**Consequences:**
- Decision log states the principle gate.
- No separate product doc required this increment.

## 4. Non-goals

Imgui settings panel, D64, multi-window, removing CLI bypass, bundling copyrighted ROMs.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Discovery picks 48/128 and C64 trio from a temp `roms` folder.
- **SM-C2**: Open-kind → machine mapping covered by unit tests.
- **SM-C3**: Auto-start / machine-start / idle-open behavior matches FR-1…FR-3 in code paths.
