---
title: C64 sprites
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 93
---

# PRD: C64 sprites

Hobby/solo. Draw the eight VIC-II hardware sprites over the text screen so games that use sprites show figures, not only the character grid.

## 1. Vision

Enabled sprites at `$D015` appear at their X/Y positions with their color and 24×21 pattern from the sprite pointer block. Hires sprites first; multicolor and expand follow if they stay small.

## 2. User journeys

- **UJ-1.** Anna soft-loads a PRG that enables sprite 0; she sees the sprite pattern on the blue screen.

## 3. Features

#### FR-1: Sprite enable, position, color

`$D015` enables sprites 0–7. X from `$D000`+MSB `$D010`, Y from `$D001`…. Color from `$D027`–`$D02E`. Pattern via pointers at screen+`$3F8` in the VIC bank (CIA2 PRA).

**Consequences:**
- A test that fills a sprite block, sets pointer/position/enable, paints a non-background pixel inside the sprite box.

#### FR-2: Hires blit over text

Each set bit in the 24×21 pattern draws the sprite color over the content bitmap. Screen coordinates use the usual 24/50 origin offsets into the 320×200 area.

**Consequences:**
- Transparent (cleared) sprite bits leave the underlying text/background.

#### FR-3: X/Y expand and multicolor

`$D01D` / `$D017` double width/height. `$D01C` multicolor uses `$D025`/`$D026` and two-bit pairs.

**Consequences:**
- Unit tests cover one expanded and one multicolor case.

## 4. Non-goals

Sprite-sprite collision registers, sprite-background collision, priority vs background accuracy, bitmap modes, badlines, cycle-exact fetch.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Hires sprite paint test passes.
