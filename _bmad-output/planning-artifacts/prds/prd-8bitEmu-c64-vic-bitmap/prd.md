---
title: C64 VIC bitmap and raster IRQ
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 97
---

# PRD: C64 VIC bitmap and raster IRQ

Hobby/solo. Fix games that switch to bitmap mode and wait on the raster IRQ (e.g. Auf Wiedersehen Monty shows PETSCII garbage and freezes).

## 1. Vision

When `$D011` bit 5 is set, the window paints the 8K bitmap in the VIC bank instead of ROM text. When `$D01A` enables the raster interrupt, crossing `$D012`/`$D011.7` raises IRQ so the game’s main loop continues.

## 2. User journeys

- **UJ-1.** Anna soft-loads a bitmap title screen and sees real graphics, not letter soup.
- **UJ-2.** A game that enables the raster IRQ keeps animating instead of hanging.

## 3. Features

#### FR-1: Hires and multicolor bitmap paint

`$D011` BMM selects bitmap. Bitmap base from `$D018` bit 3 and CIA2 bank; screen matrix from `$D018` high nibble. `$D016` MCM selects multicolor bitmap (2-bit pairs, background + screen + color RAM).

**Consequences:**
- Unit test: fill bitmap/screen, set BMM, paint shows ink from screen nybble.

#### FR-2: Raster compare IRQ

Compare line from `$D012` + `$D011` bit 7. On line entry, set `$D019` bit 0; if `$D01A` bit 0 is set, assert the IRQ line (with CIA). Writing 1s to `$D019` clears those flags.

**Consequences:**
- Test: arm compare, advance past the line, CPU takes IRQ / `$D019` shows pending.

#### FR-3: Text mode RAM charset (small)

When not in bitmap mode, glyph bytes come from VIC bank + charset offset (`$D018` bits 1–3), except the chargen ROM window when the offset selects `$1000`/`$1800` in bank 0/2.

**Consequences:**
- Soft-loaded custom fonts show correctly; ROM font still works for `$D018=$14`.

## 4. Non-goals

Badlines, cycle-exact fetches, ECM mode, sprite priority vs bitmap accuracy, D64/IEC.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Bitmap and raster IRQ unit tests pass.
