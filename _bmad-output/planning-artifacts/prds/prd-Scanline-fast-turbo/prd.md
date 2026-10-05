---
title: Fast turbo
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 50
---

# PRD: Fast turbo

Hobby/solo. With the loading sound off, a tape should finish in a few seconds, and the loading picture should stay on screen.

## 1. Vision

Anna turns the loading sound off. The tape runs ahead, the loading artwork appears and remains visible, and the game is ready in a few seconds.

## 2. User journeys

- **UJ-1. A short load with the picture.** Anna loads a long 128K tape with the loading sound off. She sees the loading picture, and the tape finishes in a few seconds.

## 3. Features

#### FR-1: Paint the load, skip the hidden work

While the loading sound is off and the loader is reading, turbo runs CPU frames without queuing audio and without building a picture for each one. At the end of each rush slice the latest picture is painted, so the loading artwork stays on screen.

**Consequences:**
- A hidden rush leaves the audio buffer empty.
- The picture painted after those frames is the full 288×224 frame.
- A quiet load still runs ahead of a load with the sound on.
- Turbo does not run during the key prompt, or while the loading sound is on.

## 4. Non-goals

A faster load while the loading beep is on. A shorter key prompt. Border stripes on the frames that are not painted.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scanline-core` line coverage stays at least 95%.
- **SM-C1**: The four assertions above pass.
