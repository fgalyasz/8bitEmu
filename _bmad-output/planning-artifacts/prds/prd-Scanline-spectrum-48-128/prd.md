---
title: Spectrum 48K and 128K
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/11
---

# PRD: Spectrum 48K and 128K

Hobby/solo. One user-visible goal: boot a 48K or basic 128K Spectrum from a ROM file the user supplies, and run an SNA snapshot.

## 1. Vision

The display pipe and the small Z80 subset already draw a test picture. This increment is the machine. The copyrighted ROM stays out of the binary. The user points Scanline at a file.

## 2. User journeys

- **UJ-1. Boot.** Anna runs Scanline with `--rom` pointing at a 16K or 32K image. The Spectrum starts at address 0. With a 128K image she passes `--model 128` and sees the 128K memory map.
- **UJ-2. Play.** She passes `--sna` as well, or instead. The snapshot's RAM, registers, and border appear. Keys reach the Spectrum matrix. F1, F2, and F3 still switch looks. Escape quits. F12 resets.
- **UJ-3. No file.** With no arguments the previous ROM-free picture still runs.

## 3. Features

#### FR-1: Z80

The core executes the documented unprefixed, `CB`, `ED`, `DD`, and `FD` set, including `IX`/`IY` displacement and the half-registers. Flag bits `S Z H PV N C` follow the instruction. Bits 3 and 5 stay clear. An undefined `ED` opcode is an 8-cycle NOP.

**Consequences:**
- `DAA` turns `0x15+0x27` into `0x42`, and `0x90+0x90` into `0x80` with carry set.
- `LDIR` copies the requested bytes and leaves `BC` at 0.
- `LD IX, nn` then `INC IX` changes `IX`.
- `RLC` through `SRL`, `BIT`, `RES`, and `SET` update the operand and the documented flags.

#### FR-2: Frame

A booted machine runs 69888 T-states per source frame. If interrupts are enabled, and `EI` has retired, the frame begins with a maskable interrupt. Mode 0 and 1 jump to `0x0038`. Mode 2 reads the vector `I * 256 + 0xFF`. `HALT` waits inside the frame and wakes on that interrupt. `EI` enables interrupts after the next instruction.

**Consequences:**
- A ROM that does `EI` / `HALT`, with `RET` replaced by a store at `0x0038`, does not store on frame 1 and does store on frame 2.
- The ROM-free picture still advances one ink step per source frame.

#### FR-3: Keyboard

Port `0xFE` reads the eight half-rows. A pressed key clears its bit. No pressed key reads `0xFF`. Host keys map to the matrix. Left Shift is Caps Shift. Right Shift and Left Control are Symbol Shift. Digit keys belong to the Spectrum. Looks move to F1, F2, and F3.

**Consequences:**
- Row 3 bit 0 held down, `IN` from `0xF7FE`, returns `0xFE`.
- Releasing the key returns `0xFF`.

#### FR-4: 48K and 128K memory

`0x0000`–`0x3FFF` is ROM. `0x4000` is RAM bank 5, `0x8000` is bank 2, `0xC000` is the paged bank. On a 128K machine, `OUT` to port `0x7FFD` sets the paged bank (bits 0–2), the screen bank (bit 3: bank 5 or 7), the ROM half (bit 4), and the lock (bit 5). A locked port ignores later writes. A 48K machine ignores `0x7FFD`. The picture is read from the selected screen bank.

**Consequences:**
- Writing `0xC000`, paging bank 1, and writing `0xC000` again leaves the two values in different banks.
- Bit 5 set, a later page write does not change the bank.
- Screen bit 3 selects bank 7 for the displayed bitmap.

#### FR-5: Images

`--rom` loads 16384 bytes on 48K, or 16384 or 32768 bytes on 128K. A 16K image in 128K mode is used for both ROM halves. `--sna` loads a 49179-byte 48K snapshot or a 131103-byte 128K snapshot. Any other length is an error that names the kind and the length. Reset (F12) returns to address 0 without clearing RAM. No file keeps the ROM-free picture.

**Consequences:**
- A crafted 48K SNA restores `A`, pops `PC` from `SP`, and sets the border.
- A crafted 128K SNA takes `PC` from the trailer and places the extra banks around banks 5, 2, and the paged bank.
- A 10-byte ROM string contains `rom` and `10`.

#### FR-6: Sound

Port `0xFE` bit 4 is the beeper. Ports `0xFFFD` and `0xBFFD` are the AY on a 128K machine. Each booted frame fills a 48 kHz buffer from those sources. The window plays it when the host audio device opens. If the device does not open, the picture still runs.

**Consequences:**
- A frame that toggles the beeper contains both signs in the buffer.
- An AY register written through `0xFFFD` / `0xBFFD` reads back.

## 4. Non-goals

Tape and TZX, memory contention, the floating bus, Kempston, the +2A/+3 paging scheme, TR-DOS, flag bits 3 and 5, and a ROM inside the repository.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The DAA, paging, interrupt, keyboard, SNA, and beeper assertions above pass.
