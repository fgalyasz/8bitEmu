---
title: Smooth turbo
status: draft
created: 2026-10-04
updated: 2026-10-04
parent_issue: 42
---

# PRD: Smooth turbo

Hobby/solo. Turbo load is a faster clock that still draws every refresh.

## 1. Vision

With Loading sound off, the Spectrum clock runs ahead, and the border keeps moving. The picture is not a slideshow of one frame per second.

## 2. User journeys

- **UJ-1. Fast and continuous.** Anna turns the loading sound off. The stripes race, and a new picture arrives on every refresh until the tape ends.

## 3. Features

#### FR-1: One frame at a time

A turbo step runs one machine frame, and only while the loader is reading and the load prompt is finished. The window repeats that step for a short slice of each refresh, then draws the latest frame. The sound stays off.

**Consequences:**
- While the prompt is still typing, a rush leaves the tape edge unchanged.
- After the prompt, one rush moves a quiet tape past a heard step.
- With the sound on, rush does nothing.
- The turbo budget is 1 while the loader is reading, and 0 when the sound is on or the tape is idle.

## 4. Non-goals

A fixed count of skipped frames, and hearing the beep during turbo.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The prompt, rush, and budget assertions above pass.
