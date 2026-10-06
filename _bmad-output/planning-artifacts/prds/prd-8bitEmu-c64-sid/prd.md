---
title: C64 SID sound
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 88
---

# PRD: C64 SID sound

Hobby/solo. Play audible tones from the SID register file so music and SFX in soft-loaded software can be heard.

## 1. Vision

When a C64 program writes `$D400`–`$D418`, the speaker mixes three voices (triangle / saw / pulse / noise) with ADSR and master volume. Filter and digi tricks wait for a later pass.

## 2. User journeys

- **UJ-1.** Anna loads a PRG that pokes a tone into SID and hears a steady note from the window speaker.

## 3. Features

#### FR-1: SID register I/O

`$D400`–`$D7FF` map onto a 32-byte SID register file (mirrored). Writes update voice and volume state; reads return the last written values.

**Consequences:**
- Unit tests poke `$D400` and read back.

#### FR-2: Three voices and envelopes

Each voice has frequency, pulse width, control waveform bits, and ADSR. Gate starts attack; release follows gate clear. Master volume is `$D418` low nibble.

**Consequences:**
- A gated triangle at a mid frequency with volume > 0 yields a non-silent sample stream after ticks.
- Volume 0 yields silence.

#### FR-3: Frame audio path

C64 `take_audio` returns 48 kHz mono samples accumulated during the frame, same rate as the Spectrum path.

**Consequences:**
- The app speaker already consumes `Presenter::take_audio`; no new UI.

## 4. Non-goals

Analog filter, resonance, sync/ring accuracy, combined waveform quirks, digi samples via `$D418`, stereo, cycle-exact bus timing.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Tone and silence unit tests pass.
