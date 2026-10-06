---
title: C64 SBC charset
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 77
---

# PRD: C64 SBC charset

Hobby/solo. Fix: Open ROMs boot text was shifted one character (`READY.` → `QDE@X.`) because 6510 `SBC` inverted carry before the ADC path.

## 1. Vision

Screen codes from PETSCII conversion (`SEC` / `SBC #$40`) must match stock C64 results so the boot banner is readable.

## 2. User journeys

- **UJ-1.** Anna boots `--machine c64` with Open ROMs and reads the banner and `READY.` normally.

## 3. Features

#### FR-1: Binary SBC

`SBC` in binary mode is `ADC` with the ones-complement operand and the existing carry (borrow inverse). Carry is not flipped before that add.

**Consequences:**
- `SEC` / `LDA #$41` / `SBC #$40` leaves `A = $01`.
- After Open ROMs cold start, screen RAM contains screen codes for `READY.` (`18,5,1,4,25,46`) on some row.

## 4. Non-goals

Reworking BCD edge cases beyond the complement-ADC path; stock Commodore ROM redistribution.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The SBC and READY screen-code assertions pass.
