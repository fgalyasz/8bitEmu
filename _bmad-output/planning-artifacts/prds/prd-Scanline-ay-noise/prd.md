---
title: AY noise
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 46
---

# PRD: AY noise

Hobby/solo. The AY-3-8910 noise output is a pitched tone. It should be noise.

## 1. Vision

A channel with the noise mixer on hisses. It does not play a musical note.

## 2. User journeys

- **UJ-1. Noise, not a note.** Anna plays a 128K tune that uses the noise channel. She hears noise, not a sine-like tone.

## 3. Features

#### FR-1: Shift-register noise

The noise source is the chip's 17-bit shift register, clocked once per noise period. The new bit is bit 0 XOR bit 3. A channel output is the tone gate AND the noise gate. A disabled input stays high. Period 0 counts as 1.

**Consequences:**
- From the seed, one noise period of 1 leaves the output low, and the next period leaves it low.
- Tone and noise both enabled, with the tone high and the noise bit low, stay silent.
- With the tone disabled, the noise bit is heard.

## 4. Non-goals

A different envelope shape, and a separate noise generator per channel.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The three assertions above pass.
