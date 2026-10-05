---
title: Turbo load by default
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 62
---

# PRD: Turbo load by default

Hobby/solo. A tape should load quickly unless the user asks to hear the beeps.

## 1. Vision

Opening a tape runs at turbo speed. Loading sound stays available for people who want the border tone.

## 2. User journeys

- **UJ-1. Open and play.** Anna opens a tape. It finishes in a few seconds. She can turn Loading sound on if she wants to hear the load.

## 3. Features

#### FR-1: Turbo is the default

A new session starts with Loading sound off. Sound → Loading sound starts unchecked. F4 still toggles it. Turning it on restores the real-time load and the beeps.

**Consequences:**
- A new machine reports Loading sound off.
- While the loader is reading and Loading sound is off, the turbo budget is 1.
- With Loading sound on, the turbo budget is 0.

## 4. Non-goals

Changing how turbo rushes, or removing Loading sound.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The assertions above pass.
