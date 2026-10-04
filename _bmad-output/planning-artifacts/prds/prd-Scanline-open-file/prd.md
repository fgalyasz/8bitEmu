---
title: Open a file from the menu
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/22
---

# PRD: Open a file from the menu

Hobby/solo. One user-visible goal: choose the ROM, snapshot, or tape from a menu instead of typing a path.

## 1. Vision

The command line still accepts `--rom`, `--sna`, `--tap`, and `--tzx`. The window also has a File menu. Open… shows the system file dialog. The extension decides what the file is. No ROM is bundled.

## 2. User journeys

- **UJ-1. Choose a tape.** Anna is in the window. She chooses File → Open… and picks a `.tzx` or `.tap`. A booted machine resets and types `LOAD ""`.
- **UJ-2. Choose a ROM or snapshot.** She picks a `.rom` or `.sna` the same way. A 32K ROM or a 128K snapshot selects the 128K machine. A 16K ROM keeps the machine she already started.

## 3. Features

#### FR-1: Open…

The menu item is Open… in the File menu. Command-O on macOS and Ctrl-O on the other platforms open the same dialog. The dialog accepts `.rom`, `.sna`, `.tap`, and `.tzx`, in any letter case. Cancel leaves the machine alone. Any other extension is refused and names the path. A tape opened on a booted machine rewinds that machine so the load prompt runs again.

**Consequences:**
- `Bosconian '87 (Europe).tzx` and `GAME.TAP` are tapes. `48.rom` is a ROM. `snap.sna` is a snapshot.
- `notes.txt` and a name with no extension are refused.
- A 32768-byte ROM selects the 128K machine. A 16384-byte ROM keeps the current machine.
- A 131103-byte snapshot selects the 128K machine. A 49179-byte snapshot selects the 48K machine.

## 4. Non-goals

Drag and drop, a recent-files list, opening several files at once, a tape browser, C64 images, and a ROM in the repository.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The extension and machine-size assertions above pass.
