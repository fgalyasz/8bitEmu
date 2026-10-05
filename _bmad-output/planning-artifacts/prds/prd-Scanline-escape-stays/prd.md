---
title: Escape stays
status: draft
created: 2026-10-05
updated: 2026-10-05
parent_issue: 44
---

# PRD: Escape stays

Hobby/solo. Escape is too easy to hit, so it no longer closes Scanline.

## 1. Vision

A stray Escape leaves the Spectrum running. The window still closes from its close button or the Scanline menu.

## 2. User journeys

- **UJ-1. Stay open.** Anna hits Escape while a game is loading. The window stays. She closes it from the menu or the close button when she is done.

## 3. Features

#### FR-1: Escape is not quit

No keyboard key closes the app. Escape does nothing to the host window. The close button and the Scanline menu still quit.

**Consequences:**
- Escape is not a quit key.
- The close request still leaves the process.

## 4. Non-goals

A new quit shortcut, and mapping Escape to Spectrum BREAK.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The Escape assertion above passes.
