---
title: AY pitch
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 56
---

# PRD: AY pitch

Hobby/solo. The music sits an octave below the real chip.

## 1. Vision

An AY note sounds at the pitch the chip's formula gives: the clock divided by sixteen times the period.

## 2. User journeys

- **UJ-1. The tune.** Anna plays a 128K game. The melody is in the same range as on a real Spectrum.

## 3. Features

#### FR-1: A tone cycle lasts sixteen times the period

The square wave flips once every eight times the tone period, so a full cycle is sixteen times the period. Noise and the envelope keep their present periods.

**Consequences:**
- With period 1, the tone is silent for 7 clocks, sounds on the 8th, and is silent again after 16 clocks.
- The envelope still takes 256 clocks for one step.

## 4. Non-goals

A different noise generator. Matching the 128K crystal to the cent.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The assertions above pass.
