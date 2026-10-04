---
title: Keep the tape playing through a load
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/30
---

# PRD: Keep the tape playing through a load

Hobby/solo. `LOAD ""` is typed, but the pilot never becomes a program.

## 1. Vision

The tape stays parked until the loader is reading the ear. Once that happens, it keeps playing through the leader wait and the pause between blocks. The last data pulse ends before the pause, so the checksum byte is the byte that was recorded.

## 2. User journeys

- **UJ-1. Autostart.** Anna opens a tape on a booted 48K machine. After the copyright screen the line is `LOAD ""`. The border stripes, and the header name is accepted.
- **UJ-2. Typed load.** She types the LOAD keyword herself. The pilot is still at the start, and it keeps running after Enter.

## 3. Features

#### FR-1: Stay playing after the loader is seen

Ordinary keyboard scanning does not move the tape. A frame that reads port `0xFE` at least 64 times latches playback. Later frames keep advancing the tape even when they do not read the port. Reset and a newly opened tape clear the latch.

**Consequences:**
- Three frames of a halted machine leave the tape on its first edge.
- One polling frame leaves it there. The next frame moves it.
- The frame after that, which does not read the port, moves it further.

#### FR-2: The pause is a new edge

The silence after a block starts at the opposite level from the last data pulse. That pulse therefore ends on time.

**Consequences:**
- For a one-byte block of `0x00`, the ear level one cycle after the last data pulse differs from the level during that pulse.

## 4. Non-goals

Contention, the floating bus, flag bits 3 and 5, and a fast-loader trap.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The latch and pause-edge assertions above pass.
