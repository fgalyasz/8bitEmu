---
title: Loading sound and turbo load
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: 32
---

# PRD: Loading sound and turbo load

Hobby/solo. The loading beeps can be switched off. While they are off, a load runs ahead of the display.

## 1. Vision

Sound → Loading sound is checked by default, and F4 toggles it. Checked, the ear plays at normal speed and the beeps are heard. Unchecked, the loader still sees the same pulses, but each refresh runs extra frames and those beeps are dropped. Game audio returns when the tape is no longer playing.

## 2. User journeys

- **UJ-1. Hear it.** Anna leaves Loading sound checked. A tape loads in real time and the border tone comes through the speaker.
- **UJ-2. Turbo.** She unchecks it, or presses F4. The copyright screen stays in real time. Once the loader is reading the ear, the tape runs many frames per refresh with no loading beep. When the tape ends, the game is heard again.

## 3. Features

#### FR-1: Loading sound

The item is a check in the Sound menu, checked at start. F4 toggles it on platforms without a menu bar. Unchecking it does not change pulse widths.

**Consequences:**
- With the sound on, a polling machine that has latched the tape reports a turbo budget of 0.
- With the sound off, the same machine reports a budget of 32.
- A halted machine with the sound off still reports 0.

#### FR-2: Turbo while the loader is reading

While the sound is off and the tape is latched and still playing, each refresh after the load prompt runs up to 32 extra frames and discards their audio. The prompt itself stays one frame per refresh. Audio is kept when the sound is on, and when the tape is not loading.

**Consequences:**
- For the prompt's frames, a quiet presenter and a heard presenter stay on the same tape edge.
- The next refresh moves the quiet presenter further ahead.
- The sound is heard for a halted tape, and not heard once that polling machine has latched.

## 4. Non-goals

A memory trap that pokes the block in, contention, and saving the check across launches.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The budget, prompt, and audio assertions above pass.
