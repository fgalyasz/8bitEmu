---
title: C64 READY boot
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 73
---

# PRD: C64 READY boot

Hobby/solo. One user-visible goal: after `--machine c64`, the Open ROMs cold start finishes and the character screen shows the boot banner (not an empty blue field).

## 1. Vision

The first C64 slice painted VIC colours but stuck in a `$D012` raster wait. This increment advances the raster line and CIA-1 Timer A so the KERNAL can leave IOINIT, clear the screen, and hand off to BASIC. SID and sprites stay out.

## 2. User journeys

- **UJ-1. Boot.** Anna runs `--machine c64` with Open ROMs in `roms/`. Within a short time she sees white text on a blue screen (Open ROMs colour scheme), including a boot banner / READY-style prompt.
- **UJ-2. Spectrum stays.** Spectrum launch is unchanged.

## 3. Features

#### FR-1: VIC raster

`$D012` (and `$D011` bit 7) follow the PAL beam as the CPU runs. A busy-wait that compares `$D012` to itself eventually observes a change.

**Consequences:**
- Two successive reads of `$D012` after enough cycles differ when the beam moves.
- The Open ROMs raster sync loop at cold start exits.

#### FR-2: CIA-1 Timer A IRQ

CIA-1 Timer A counts CPU cycles when started. Underflow sets ICR bit 0, and with the mask enabled raises IRQ. Reading `$DC0D` reports and clears the condition. The machine no longer forces a fake IRQ every frame.

**Consequences:**
- After the KERNAL programs Timer A, IRQs occur without a manual `trigger_irq` each frame.
- Reading `$DC0D` after an underflow returns a value with bit 0 set, then clears.

#### FR-3: Visible boot text

After enough source frames with Open ROMs, screen memory is not a field of spaces: at least one boot banner character is present, and the painted frame shows a non-background ink pixel in the text area.

**Consequences:**
- A test loading `roms/c64-*.rom` finds a non-space, non-zero screen code in `$0400`–`$07E7` within 300 frames.
- The top-left glyph cell is not entirely background colour in the painted frame.

## 4. Non-goals

Full CIA B / TOD, SID, sprites, badlines, cycle-exact VIC, tape/PRG, and Commodore stock ROM dumps in git.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The raster, ICR, and boot-text assertions above pass.
