---
title: Clear spent tape
status: done
created: 2026-10-07
updated: 2026-10-07
parent_issue: 111
---

# PRD: Clear spent tape

Hobby/solo. Fix: after a `.tap` / `.tzx` load has progressed, Reset must not rewind the tape and re-run LOAD. Clear the tape slot once its contents are no longer needed.

Follows user-first shell §0: Reset does what the user expects (clean reboot after a load), not a surprise second load.

## Features

#### FR-1: Spent tape clears

When the tape is no longer at the start and is not playing (finished or stopped past the start), clear `player` so the slot is empty.

**Consequences:**
- Unit test: advance past the end → `has_tape()` is false.

#### FR-2: Reset does not reload a used tape

Reset keeps a tape only while it is still at the start (retry before load). If the tape has progressed, eject it and do not arm a fresh LOAD prompt from that image.

**Consequences:**
- Test: load tape, advance edges, reset → no tape; `arm_prompt` does not queue LOAD.
- Test: load tape, reset while still at start → tape remains for one LOAD attempt.
