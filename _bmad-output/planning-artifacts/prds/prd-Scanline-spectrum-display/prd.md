---
title: Spectrum 48K display
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/6
---

# PRD: Spectrum 48K display

Hobby/solo. One user-visible goal: open Scanline and see a picture a Z80 wrote into the Spectrum display file. No ROM.

## 1. Vision

The present pipe already scales and colors a frame. This increment feeds it from a 48K memory map. A small program we own draws the picture. The copyrighted Spectrum ROM stays out of the binary.

## 2. User journeys

- **UJ-1. Machine picture.** Anna opens Scanline. The border is blue. Three bright horizontal lines cross the screen thirds. The first eight pixels of the top line step through the ink colors. No ROM prompt.
- **UJ-2. Same window.** Keys 1, 2, and 3 still switch looks. Escape still quits. A 60 Hz panel still holds one source frame across the repeated refresh.

## 3. Features

#### FR-1: 48K memory

Addresses `0x0000`–`0x3FFF` read `0xFF` and ignore writes. Addresses `0x4000`–`0xFFFF` are RAM. The built-in program is copied to `0x8000`.

**Consequences:**
- A write below `0x4000` does not change the next read.
- A write at `0x4000` and at `0xFFFF` reads back.

#### FR-2: Display file

Bitmap bytes use the Spectrum address formula. Attribute bytes start at `0x5800`, one byte per character cell, repeated for the eight scanlines of that cell. The core linearizes both into the existing frame.

**Consequences:**
- Pixel column 0 of rows 0, 1, 8, and 64 is at `0x4000`, `0x4100`, `0x4020`, and `0x4800`.
- The attribute for pixel `(0, 0)` is at `0x5800`. The attribute for pixel `(0, 64)` is at `0x5900`.
- A set bit uses ink. A clear bit uses paper. Bright adds 8 to the palette index.

#### FR-3: Z80 subset

The core runs the unprefixed loads, ALU, increments, relative jumps, absolute jumps, `DJNZ`, `IN A,(n)`, `OUT (n),A`, and `HALT` that the program and the tests use. `HALT` ends the frame. The next frame resumes at the following instruction. Any other opcode stops the frame with an error that names the byte. A frame that does not halt inside 100000 steps stops with a step-limit error.

**Consequences:**
- Documented flag bits `S Z H PV N C` follow the instruction. Flag bits 3 and 5 stay clear.
- `CP` does not change `A`. `INC`/`DEC` of an 8-bit register keep `C`.
- Opcode `0xCB` is an error whose text contains `0xcb`.
- `JR $` does not spin forever; the error text contains `step`.

#### FR-4: Border port

`OUT (254),A` stores bits 0–2 as the border color. Every presented border row uses that color. `IN A,(n)` returns `0xFF` and does not change flags.

**Consequences:**
- After the built-in program's first frame the border index is 1.
- An `OUT` to any other port leaves the border unchanged.

#### FR-5: Window

The presenter shows one new machine frame per new source index. Looks, integer scale, and the 6:5 cadence stay as in the foundation PRD.

**Consequences:**
- The first two 60 Hz refreshes show the same ink on pixel `(0, 0)`. The third refresh shows the next ink.
- Pixel `(8, 0)`, pixel `(0, 64)`, and pixel `(0, 128)` are bright white (index 15). Pixel `(8, 1)` is bright black (index 8). Pixel `(0, 1)` is black (index 0), because that cell's attribute no longer has the bright bit.

## 4. Non-goals

Keyboard, tape, beeper, contention, interrupts, `IX`/`IY`, `CB`/`ED` prefixes, a ROM file, 128K paging, and the C64.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The address assertions and the first two machine frames match the consequences above.
