---
title: Save and size from the menu
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/27
---

# PRD: Save and size from the menu

Hobby/solo. The picture size and the saves live on function keys. The menus should do the same jobs.

## 1. Vision

View sets the picture to fit, 125%, 150%, 175%, or 200%. File saves the running machine as a snapshot, and saves a standard-speed recording as a TAP or a TZX. The dialog asks for the path.

## 2. User journeys

- **UJ-1. Size.** Anna opens View and chooses 200%. The window becomes 576 by 448. Fit returns the window to 960 by 720 and the picture fills it. F5 through F8 still set the same four sizes. Dragging the window still returns to fit.
- **UJ-2. Save.** She chooses File → Save Snapshot… and picks a `.sna`. File → Save Tape as TAP… and Save Tape as TZX… write the standard-speed recording. With nothing recorded, those two items say so and do not open a dialog.

## 3. Features

#### FR-1: Picture size

The View menu has Fit, 125%, 150%, 175%, and 200%. The four percentages use the same sizes as F5 through F8. Fit clears the percentage and requests a 960 by 720 window.

**Consequences:**
- The menu id `size-200` is 200%. `size-125` is 125%. `size-fit` is fit.
- An unknown id is not a size.

#### FR-2: Save snapshot and tape

File has Save Snapshot…, Save Tape as TAP…, and Save Tape as TZX…. Command-S on macOS and Ctrl-S elsewhere open the snapshot dialog. Command-Shift-T and Command-Shift-Z (Ctrl on the other platforms) open the TAP and TZX dialogs. A missing extension is added. Cancel writes nothing. TAP and TZX are the recorded standard-speed blocks. An empty recording is refused. A TZX file is a version-1 standard-speed tape that this player can open.

**Consequences:**
- `save-sna`, `save-tap`, and `save-tzx` select those three saves. Any other id does not.
- `notes` saved as a snapshot becomes `notes.sna`. `game.TAP` kept as a tape stays `game.TAP`.
- Command-S is a snapshot. Command-Shift-T is a TAP. Command-Shift-Z is a TZX. A plain S is not a save.
- A finished standard-speed recording is a TAP whose payload is the recorded byte, and a TZX beginning `ZXTape!` that `open_tape` accepts. An empty recording is neither.

## 4. Non-goals

Exporting the tape that is currently playing, turbo or custom TZX blocks, a screenshot, C64 images, and a menu bar on Linux. Linux keeps the same shortcuts.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The size, save-id, extension, shortcut, and tape-image assertions above pass.
