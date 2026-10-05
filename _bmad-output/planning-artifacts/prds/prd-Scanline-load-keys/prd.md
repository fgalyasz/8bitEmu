---
title: Load keys
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 48
---

# PRD: Load keys

Hobby/solo. 128K should not type `LOAD ""`, and a tape must not freeze while the loader is still reading.

## 1. Vision

A 128K machine starts Tape Loader with Enter. A multi-block tape keeps going when the loader asks for the next bytes.

## 2. User journeys

- **UJ-1. 128K menu.** Anna opens a tape on a 128K machine. The menu stays put, then Tape Loader starts. She does not see `LOAD ""` typed, and she does not hear a run of key clicks.
- **UJ-2. One load, many blocks.** She opens Auf Wiedersehen Monty. After the title, the later blocks keep loading instead of sitting on a stopped tape.

## 3. Features

#### FR-1: Enter on 128K

A 48K prompt still types the LOAD keyword, two quotes, and Enter. A 128K prompt waits until the menu is up, then presses Enter once. It never presses J.

**Consequences:**
- The 48K prompt's first key is J, and a later frame holds Symbol Shift with P.
- Every key in the 128K prompt is Enter.

#### FR-2: Continue after a stop

A pause of zero still stops the tape. If the loader is still reading on the next frame, the tape continues. If the loader goes quiet, the tape waits and continues when the next load reads the ear.

**Consequences:**
- The frame that hits the stop does not continue.
- The next frame, if the loader is still reading, continues.
- A quiet frame after the stop waits until the loader reads again.

## 4. Non-goals

Choosing 128 BASIC, and skipping a real gap between two separate loads.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The prompt and stop assertions above pass.
