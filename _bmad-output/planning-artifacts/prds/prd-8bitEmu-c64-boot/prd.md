---
title: C64 boot
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 68
---

# PRD: C64 boot

Hobby/solo. One user-visible goal: boot a Commodore 64 from user-supplied Open ROMs (or equivalent) and see a character screen with a working keyboard.

## 1. Vision

Spectrum already runs. This increment adds a first C64 machine: 6510, banked 64K map, VIC-II text mode, CIA-1 keyboard. SID, sprites, tape, and cycle-exact VIC stay out. ROMs stay out of the repository; the user points at `c64-*.rom` files documented under `roms/`.

## 2. User journeys

- **UJ-1. Boot.** Anna runs 8bitEmu with `--machine c64` and three ROM paths (or the default `roms/c64-kernal.rom`, `roms/c64-basic.rom`, `roms/c64-chargen.rom`). The machine starts at the KERNAL reset vector. The window shows the C64 character screen with the Colodore palette.
- **UJ-2. Type.** Letter keys, digits, Return, and Shift reach the CIA-1 matrix. F1–F3 still switch looks. F12 resets. Escape does nothing.
- **UJ-3. Spectrum stays.** With no `--machine c64` the Spectrum path is unchanged, including the ROM-free demo picture.

## 3. Features

#### FR-1: 6510

The core executes the documented 6502/6510 set used by the KERNAL and BASIC, including decimal mode and the on-chip I/O port at `$00`/`$01`. An illegal opcode is a hard error that names the opcode.

**Consequences:**
- `LDA #$01` / `STA $0400` leaves `$01` at screen memory.
- `SED` then `ADC` with `$09+$01` yields `$10` in decimal mode.
- Port `$01` default after reset maps BASIC, KERNAL, and I/O as on a stock C64.

#### FR-2: Memory map

`$0000`–`$FFFF` is 64K RAM under the ROMs. `$A000`–`$BFFF` is BASIC when LORAM is set. `$E000`–`$FFFF` is KERNAL when HIRAM is set. `$D000`–`$DFFF` is I/O when CHAREN maps I/O; character ROM is readable when CHAREN maps characters. Color RAM sits at `$D800`–`$DBFF`.

**Consequences:**
- Loading the three 8K/8K/4K images places them in the BASIC, KERNAL, and chargen slots.
- A wrong length is `ImageLength` naming `kernal`, `basic`, or `chargen`.

#### FR-3: VIC-II text mode

Each source frame runs 19656 CPU cycles (PAL). The picture is 320×200 characters from `$0400` and color RAM, with border and background from `$D020` / `$D021`. The frame uses `PaletteKind::C64`. Soft edge and temporal looks still apply.

**Consequences:**
- Writing `'A'` (PETSCII screen code) at `$0400` with color `$01` shows a white glyph in the top-left cell after a frame.
- Border color `$D020 = 6` paints a blue border index.

#### FR-4: Keyboard

CIA-1 ports `$DC00` / `$DC01` read the 8×8 matrix. Host keys map to that matrix. No pressed key leaves the port open (`$FF`).

**Consequences:**
- Holding A clears the matching bit on a CIA read.
- Releasing restores `$FF` when no other key is held.

#### FR-5: Launch

`--machine c64` selects the C64. `--kernal`, `--basic`, and `--chargen` set paths. If omitted, `roms/c64-kernal.rom`, `roms/c64-basic.rom`, and `roms/c64-chargen.rom` relative to the working directory are used. Missing or wrong-sized files are clear errors. Spectrum flags stay valid only for Spectrum.

**Consequences:**
- `--machine c64` without files loads the three default paths when present.
- `--machine spectrum` (default) ignores C64 defaults.

## 4. Non-goals

SID audio, sprites, raster IRQs beyond the frame timer, VIC badlines/contention, CIA timers beyond the keyboard ports, IEC/disk, tape/PRG/CRT, REU, illegal NOP grids, and bundling ROM images in git.

## 5. Success metrics

- **SM-1**: `cargo test` passes; new C64 core modules stay at least 95% line coverage.
- **SM-C1**: The CPU, banking, glyph, border, keyboard, and image-length assertions above pass.
