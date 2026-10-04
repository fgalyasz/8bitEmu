---
title: Type LOAD and hear the tape
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/25
---

# PRD: Type LOAD and hear the tape

Hobby/solo. The tape command never reaches BASIC, and a typed LOAD does not hear the tape.

## 1. Vision

At the start of a BASIC line the Spectrum is in keyword mode. LOAD is the J key, not the letters L, O, A, and D. The tape stays parked until the loader is actually reading the ear, so the pilot is still there when LOAD starts.

## 2. User journeys

- **UJ-1. Autostart.** Anna opens a tape on a booted 48K machine. After the copyright screen, the line reads `LOAD ""` and Enter. The border stripes show that the loader hears the tape from its first pilot.
- **UJ-2. Typed load.** She types the LOAD keyword herself. The tape has not been consumed while she was looking at the copyright screen.

## 3. Features

#### FR-1: LOAD keyword

The prompt waits until the editor is ready, then holds J, then Symbol Shift with P twice, then Enter. J is row 6 mask `0x08`.

**Consequences:**
- The first held key is row 6 mask `0x08`.
- A later frame still holds Symbol Shift (row 7 mask `0x02`) together with P (row 5 mask `0x01`).

#### FR-2: The tape waits

Ordinary keyboard scanning does not move the tape. Once a frame reads port `0xFE` often enough to be a loader, the following frames play the ear pulses. Playback is still the pulse stream, not a poked block.

**Consequences:**
- Three frames of a halted machine leave the tape on its first edge.
- Two frames of a port-`0xFE` polling loop move the tape off that edge.

## 4. Non-goals

A fast loader trap, contention, the floating bus, and flag bits 3 and 5.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The key and tape-edge assertions above pass.
