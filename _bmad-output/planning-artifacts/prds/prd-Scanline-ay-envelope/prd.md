---
title: AY envelope
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 52
---

# PRD: AY envelope

Hobby/solo. The 128K menu step is one click. After a game starts, that same click keeps repeating.

## 1. Vision

Stepping in the 128K menu makes one short click, then silence. Once a game leaves the envelope running, that click repeats by itself. It should finish and stay quiet until the shape is written again.

## 2. User journeys

- **UJ-1. One step, one click.** Anna loads a 128K game. She no longer hears the menu up/down click knocking on through the load and the game.

## 3. Features

#### FR-1: The envelope follows its shape

Register 13 chooses the shape. Continue, attack, alternate, and hold follow the chip. A one-shot decay starts loud and then stays silent. A repeating shape keeps going. Writing register 13 restarts the envelope. Until that write, an envelope-controlled channel stays silent.

**Consequences:**
- Shape 0 is loud at the write, silent after sixteen steps, and still silent after sixteen more.
- Writing shape 0 again makes it loud at once.
- Shape 8 is loud again after sixteen steps.
- Shape 11 stays loud after the first cycle. Shape 10 is quiet at the turn and loud one step later.
- With no shape write, sixteen steps stay silent.

## 4. Non-goals

A different noise generator. Envelope shapes on a chip other than the AY-3-8910.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The assertions above pass.
