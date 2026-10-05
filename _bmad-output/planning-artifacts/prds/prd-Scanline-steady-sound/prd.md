---
title: Steady sound
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 58
---

# PRD: Steady sound

Hobby/solo. A held note wobbles, as if the vibrato were too deep.

## 1. Vision

A sustained AY note holds its pitch. A gap between picture frames does not stretch the waveform.

## 2. User journeys

- **UJ-1. A held note.** Anna listens to a 128K tune. The notes stay level instead of swelling and dipping several times a second.

## 3. Features

#### FR-1: The speaker waits for a lead

Playback starts once 4096 samples are queued. Until then the output follows the first queued sample and does not consume the queue. After that, an empty queue still repeats the last sample.

**Consequences:**
- With 4095 samples of `-0.2` queued, a read returns `-0.2` and leaves the queue untouched.
- The next queued sample starts playback, and that read consumes one sample.

## 4. Non-goals

Changing the envelope speed. Changing the tone pitch.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The assertions above pass.
