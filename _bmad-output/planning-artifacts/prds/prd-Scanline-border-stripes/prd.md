---
title: Border stripes
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: 34
---

# PRD: Border stripes

Hobby/solo. The border color follows the beam, so a load is striped.

## 1. Vision

On a real Spectrum the border port is sampled while the picture is drawn. A loader that changes the port during the frame paints horizontal stripes. Scanline does the same for the border it shows.

## 2. User journeys

- **UJ-1. Loading stripes.** Anna loads a tape with the sound on. The border is bands of color, not one flat color for the whole frame.
- **UJ-2. A steady border.** A program that sets the border once, before the beam arrives, still fills the border with that one color.

## 3. Features

#### FR-1: Scanline border

Each visible row keeps the border color in effect when the beam reaches that row. A later write does not repaint rows the beam has already passed.

**Consequences:**
- A write of color 1, a delay into the picture, then a write of color 2, leaves the top border row 1 and the bottom border row 2.
- A single write of color 5 before the picture leaves every border row 5.

## 4. Non-goals

A color change inside one row, 128K line timing, and contention.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The two border assertions above pass.
