---
title: Real time
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: 40
---

# PRD: Real time

Hobby/solo. The machine runs at Spectrum speed, and the sounds that belong to that speed are heard.

## 1. Vision

A faster panel does not run the Spectrum faster. The 128K chip plays at its real pitch. With Loading sound on, a tape is heard.

## 2. User journeys

- **UJ-1. Steady speed.** Anna plays on a 120 Hz display. The game moves at the same speed as on a 60 Hz display.
- **UJ-2. Hear the load.** Loading sound is on. The pilot tone is audible. Turning it off stays silent and runs ahead.

## 3. Features

#### FR-1: Wall clock

Display steps follow the clock, sixty per second. A redraw that is not yet due does not run the machine. A stall catches up at most four steps, then drops the rest.

**Consequences:**
- A lateness shorter than one step runs 0 steps.
- A lateness of one step runs 1.
- A lateness of five steps, with a cap of four, runs 4.

#### FR-2: AY pitch

Tone and noise count 16 chip clocks per period unit. The envelope counts 256. The clock fed in is half the CPU clock.

**Consequences:**
- Tone period 4 is silent after 63 clocks and audible on the 64th.
- Noise period 1 is silent after 15 clocks and audible on the 16th.
- Envelope period 1 does not rise at 16 clocks, and has risen by 256.

#### FR-3: Loading beep

While Loading sound is on and the tape is moving, the ear level is mixed into the audio. While it is off, that level is not added. Each queued sample is written to every output channel.

**Consequences:**
- Sound on, after the tape starts, the buffer has a sample above 0 and a sample below −0.2.
- Sound off, the same run stays on one value.
- A stereo buffer of four slots, fed 0.5 then −0.5, is 0.5, 0.5, −0.5, −0.5.

## 4. Non-goals

A 128K line length, and playing the beep while the load is running ahead.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The pace, pitch, ear, and channel assertions above pass.
