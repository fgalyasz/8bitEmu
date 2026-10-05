---
title: AY mix
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 60
---

# PRD: AY mix

Hobby/solo. Bosconian in Scanline does not sound like the same tune in Retro Virtual Machine.

## 1. Vision

An AY tune keeps the shape it has on the chip. Quiet notes stay quiet, and the envelope moves at the chip's rate.

## 2. User journeys

- **UJ-1. The title tune.** Anna plays Bosconian '87. The melody sits forward, and the accompaniment does not swell over it.

## 3. Features

#### FR-1: Volumes follow the measured steps

Each of the sixteen levels uses the measured AY curve, not an even ramp. A middle level is much quieter than full.

**Consequences:**
- A channel held at level 15 is louder than 0.3.
- A channel held at level 8, times four, is still quieter than level 15.

#### FR-2: The envelope steps every sixteen clocks

One envelope step takes sixteen chip clocks times the period. A full cycle of sixteen steps takes 256 clocks times the period.

**Consequences:**
- Period 1 is still silent after 8 clocks, and has risen by 16.

## 4. Non-goals

Matching the 128K crystal to the cent. Stereo placement.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The assertions above pass.
