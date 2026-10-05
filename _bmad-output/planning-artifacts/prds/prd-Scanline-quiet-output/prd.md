---
title: Quiet output
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 54
---

# PRD: Quiet output

Hobby/solo. A still screen keeps clicking.

## 1. Vision

When the picture is quiet, the speaker is quiet. A brief gap in the sample queue does not turn into a tick.

## 2. User journeys

- **UJ-1. Still screen.** Anna sits on a game's score table. She hears silence, not a knock every half second.

## 3. Features

#### FR-1: A gap repeats the last sample

The sound device keeps the previous sample when the queue is empty. One displayed second of the machine is 48000 samples, so steady playback does not fall behind the device.

**Consequences:**
- After `0.5` and `-0.5`, further output stays at `-0.5`.
- A halted frame yields 960 samples, each `-0.2`.

## 4. Non-goals

A different sample rate than 48000. Changing the beeper's idle level.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The assertions above pass.
