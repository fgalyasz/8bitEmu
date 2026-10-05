---
title: AY OUTD
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 64
---

# PRD: AY OUTD

Hobby/solo. Saboteur II has AY music in another emulator and silence here.

## 1. Vision

A 128K tune that writes the chip with OUTD is heard.

## 2. User journeys

- **UJ-1. Saboteur II menu.** Anna loads the Europe / 128K tape. The menu tune plays.

## 3. Features

#### FR-1: Block I/O reaches the ports

OUTI, OUTD, OTIR, and OTDR write `(HL)` through port `BC` after `B` counts down. INI, IND, INIR, and INDR read the same way into `(HL)`.

**Consequences:**
- After OUTD writes volume 15 to AY register 8 with tone A open, the frame buffer has a sample above the quiet beeper floor.

## 4. Non-goals

Exact undocumented flag bits for block I/O.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The assertion above passes.
