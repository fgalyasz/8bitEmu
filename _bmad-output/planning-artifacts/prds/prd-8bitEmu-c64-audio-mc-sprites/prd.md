---
title: C64 audio out and multicolor sprite latch
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: pending
---

# PRD: C64 audio out and multicolor sprite latch

Hobby/solo. Fix: C64 SID samples never reached the speaker, and multiplexed multicolor sprites used end-of-frame color registers (Monty left figure black instead of gray).

## 1. Vision

C64 audio always plays through the window speaker. Latched sprite draws keep `$D025`/`$D026` and the sprite’s own color from the raster Y hit.

## 2. User journeys

- **UJ-1.** Anna hears Monty title music when MUSIC ON is shown.
- **UJ-2.** The left Monty sprite is gray, not a black silhouette.

## 3. Features

#### FR-1: C64 hears audio

`Presenter::hears_loading` is true on C64 so `play_tick` pushes SID samples every refresh.

**Consequences:**
- Unit or app-level: C64 host reports hears_loading; speaker path receives non-empty buffers while SID is gated.

#### FR-2: Latch multicolor colors

`SpriteDraw` stores `mc1`, `mc2`, and uses latched `color` for pair code `10`. Paint does not read live `$D025`–`$D027` for latched draws.

**Consequences:**
- Multiplex test: change `$D025` after latch; paint still uses the latched gray/shared colors.

## 4. Non-goals

SID filter, digi samples, cycle-exact sprite DMA.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Multicolor latch color test passes.
