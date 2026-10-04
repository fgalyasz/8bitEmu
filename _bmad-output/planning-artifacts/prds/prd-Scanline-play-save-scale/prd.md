---
title: Play, save, and scale
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/17
---

# PRD: Play, save, and scale

Hobby/solo. One user-visible goal: load a TAP or TZX, run the program, save work, use Kempston, and size the picture.

## 1. Vision

The 48K and basic 128K machine already boots a ROM or an SNA. This increment is how a game or a typed program gets in and back out, and how the picture is sized. The picture stays nearest-neighbor. No ROM is bundled.

## 2. User journeys

- **UJ-1. Play.** Anna passes `--rom` and `--tap` or `--tzx`. The machine types `LOAD ""`. The ROM loader reads the tape from the ear bit and the program runs.
- **UJ-2. Keep.** She writes BASIC or machine code. F11 writes `scanline.sna`. A standard-speed `SAVE` also grows `scanline.tap`.
- **UJ-3. Size and stick.** F5 through F8 set 125%, 150%, 175%, and 200%. Dragging the window fits the picture freely. Arrows and Right Alt are the Kempston joystick.

## 3. Features

#### FR-1: TAP and TZX

`--tap` reads a TAP image. `--tzx` reads a TZX 1.x image. Playback is a stream of ear pulses on port `0xFE` bit 6, so the ROM loader and a loader that samples the port both hear it. Supported TZX bodies are standard data, turbo, pure tone, pulse sequence, pure data, direct recording, pause, group, loop, stop-if-48K, signal level, text, archive, hardware, custom info, and the glue block. Any other id is an error that names the id. A truncated image is an error that names `tape` and the length. F9 resumes after a stop-the-tape pause.

**Consequences:**
- A one-byte TAP block's first pilot pulse is 2168 T-states, and the level flips at that boundary.
- A TZX standard-speed block carries the same bytes as the TAP payload.
- A file whose first block id is `0x19` names `0x19`.

#### FR-2: Start the load

With a tape and a booted ROM, the window types `LOAD ""` and Enter after one second. The keys are L, O, A, D, Symbol Shift with P twice, then Enter.

**Consequences:**
- The prompt's first key is row 6 mask `0x02`.
- The quote is Symbol Shift (row 7 mask `0x02`) together with P (row 5 mask `0x01`).

#### FR-3: Save

F11 writes a 48K or 128K SNA of the running machine to `scanline.sna` in the working directory. Loading that image restores A, the border, and the first RAM byte. MIC, port `0xFE` bit 3, is recorded. A standard-speed recording becomes TAP blocks in `scanline.tap`.

**Consequences:**
- A 48K snapshot round-trips A, the border, and the byte at `0x8000`.
- Pulses of a known standard-speed byte decode back to that byte.

#### FR-4: Kempston

Port `0x1F` is the Kempston joystick. Bit 0 is right, 1 left, 2 down, 3 up, 4 fire. A set bit is pressed. Nothing pressed reads `0x00`. Arrow keys steer. Right Alt is fire.

**Consequences:**
- With no stick, `IN` from `0x1F` is `0x00`.
- Right held sets bit 0. Fire sets bit 4.

#### FR-5: Picture size

The picture scales with nearest neighbor. Fit keeps the frame's aspect inside the window. F5 is 125%, F6 150%, F7 175%, F8 200% of the native frame, centered. A later drag returns to fit. The native frame is 288 by 224. 200% is 576 by 448. 125% is 360 by 280.

**Consequences:**
- 200% of 288 by 224 is 576 by 448.
- A wide window fit is as tall as the window and no wider than the aspect allows.

## 4. Non-goals

Contention, the floating bus, flag bits 3 and 5, CSW, generalized TZX data, a tape menu, +2A/+3, the C64, and a ROM in the repository.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The pilot, Kempston, snapshot, and scale assertions above pass.
