---
title: C64 sprite multiplex latch
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 101
---

# PRD: C64 sprite multiplex latch

Hobby/solo. Fix title-screen sprites that reuse VIC slots mid-frame (Auf Wiedersehen Monty bottom figures show as white snow).

## 1. Vision

When the raster reaches a sprite’s Y, latch that sprite’s X, pointer, color, and expand/multicolor flags for painting. End-of-frame register values no longer redraw every sprite with the wrong pattern.

## 2. User journeys

- **UJ-1.** Anna sees Monty (or similar) characters animate correctly instead of snow under the title.

## 3. Features

#### FR-1: Raster Y latch list

On each new raster line, for each enabled sprite whose Y equals the line, append a draw record (position, data base, colors, flags). Clear the list when the raster wraps.

**Consequences:**
- Unit test: change pointer after the latch line; paint uses the latched pattern.

#### FR-2: Paint from latches

`sprites::paint` draws the latch list. If the list is empty (poke-and-paint tests), fall back to live registers.

**Consequences:**
- Existing sprite unit tests still pass.

## 4. Non-goals

Cycle-exact sprite DMA, sprite collisions, priority vs background.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Multiplex latch unit test passes.
