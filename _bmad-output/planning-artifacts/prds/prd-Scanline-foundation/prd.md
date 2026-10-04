---
title: Scanline foundation
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: https://github.com/fgalyasz/Scanline/issues/1
---
# PRD: Scanline foundation

Hobby/solo. One user-visible goal: open Scanline on Windows, macOS, or Linux and see a stable, sharp Spectrum-style test pattern, with three looks, and no ROM.

## 1. Vision

Scanline is the TenPrint player for the Spectrum 48K, the basic 128K, and the C64. The picture is the product. This increment is the present pipe those cores will feed. It does not emulate a CPU yet.

## 2. User journeys

- **UJ-1. First window.** Anna opens Scanline. A bordered test pattern appears immediately. Attribute blocks, one red scanline, a border stripe, a checker, a flickering block, and a moving sprite are visible. No ROM prompt.
- **UJ-2. Sharp by default.** The pixels stay on whole physical pixels. Resizing the window letterboxes. It does not bilinear-blur the image.
- **UJ-3. Three looks.** Keys 1, 2, and 3 switch Sharp, Soft edge, and Temporal color. The title shows the look. Soft edge eases a luminance step. Temporal color mixes two-frame flicker and a stable checker in linear light, and leaves real motion on the current frame.
- **UJ-4. Steady refresh.** On a 60 Hz panel the pattern advances five source frames for every six display refreshes, in a fixed order. The repeated refresh shows the same frame again.

## 3. Features

#### FR-1: One window

Scanline opens one window and quits when the window closes or the user presses Escape. The title is the active look. Nothing runs after quit.

**Consequences:**
- Close and Escape both leave the process.
- The title is `Scanline — Sharp`, `Scanline — Soft edge`, or `Scanline — Temporal color`.

#### FR-2: Structured frame

The core builds a presented frame from a bitmap, per-scanline attributes, a per-row border, and a separate sprite layer. Palette index 0 is black and visible. Sprite value 255 is transparent.

**Consequences:**
- A wrong bitmap, attribute, border, or sprite length is an error that names the plane and both lengths.
- A palette index above 15 is an error that names the index.
- Width must be a non-zero multiple of 8.
- The border color of each presented row is kept, including rows that are only border.
- A sprite may start off the frame; visible pixels still land, transparent pixels do not.

#### FR-3: Sharp present

The largest integer scale that fits the window is used. The scaled rectangle is centered. Sampling is nearest-neighbor, in physical pixels. Sharp colors are the palette bytes themselves.

**Consequences:**
- A destination smaller than the source still uses scale 1.
- A zero size returns no scale.
- A ULA red ink pixel is exactly `215, 0, 0`.
- The C64 palette is the VICE Colodore 3.6.1 table. The ULA palette is `0x00` / `0xD7` / `0xFF`.

#### FR-4: Soft edge and temporal color

Soft edge moves a pixel 25% of the way toward its right-hand neighbor in linear light when their luminance differs. Temporal color blends a pixel with the previous frame when the value returned after two frames, and with its right-hand neighbor when a stable period-2 checker stays inside the row. Other changes stay on the current frame. The sprite replaces the background after that blend.

**Consequences:**
- Linear half of black and white is 188, not 128.
- Keys 1, 2, and 3 select the looks. Any other digit does not.
- A mismatched history length is an error.

#### FR-5: Frame clock and mailbox

Spectrum time is `15625/312` Hz. C64 PAL time is `13684/273` Hz. Completed audio frames are an integer division of sample count by that rate. The mailbox keeps only the latest value. Display tick `n` on a 60 Hz panel shows source frame `n * 5 / 6`. Variable refresh shows tick `n`.

**Consequences:**
- At 48 kHz, Spectrum frame 1 completes at sample 959, not 958.
- At 48 kHz, C64 PAL frame 1 completes at sample 958, not 957.
- A zero sample rate is an error.
- Two publishes then one take returns the second value only.
- The app uses the 60 Hz pace. The first two display ticks share one source frame.

#### FR-6: ROM-free test pattern

The pattern is a Spectrum-sized frame: 256×192 content, 16-pixel border, ULA palette, one moving sprite, a red border stripe, a per-scanline red content row, a flicker block, and a checker.

**Consequences:**
- `demo_frame` needs no ROM and no CPU.
- Sprite x is `(tick % 120) * 2`.
- The window shows this pattern.

## 4. Non-goals

- Z80, 6510, VIC-II, SID, AY, 128K paging, tape, disk, and ROM loading.
- +2A, +3, and the Amiga.
- Fullscreen, installers, DMG, and tenprintsoftware.com.
- A neural upscaler. The same source pixel stays the same color every time it is shown.

## 5. Success metrics

- **SM-1**: `cargo test` passes. New core lines are at least 95% covered.
- **SM-C1**: Palette bytes, linear mixes, scale, cadence, mailbox, compose errors, sprite clipping, and the 6:5 hold are asserted without sleeps.
