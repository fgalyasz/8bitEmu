# Addendum — Scanline foundation

Mechanism for implementers. FR IDs refer to `prd.md`.

## Layout

- `crates/scanline-core` — clocks, scale, cadence, mailbox, palettes, compose, shade, demo pattern, presenter. No window crate.
- `crates/scanline-app` — winit window, wgpu present, WGSL shader.
- Tests cover core behavior and one offscreen GPU compare. No sleeps.

## Clocks

Spectrum: 3_500_000 / 69_888 = 15_625 / 312. C64 PAL: 985_248 / 19_656 = 13_684 / 273. `frames_from_samples` is integer division. A later audio callback will pass the sample count; this increment does not open a device.

## Present

`Presenter` uses `PresentPace::Fixed60Hz`. Source index `n * 5 / 6`. The mailbox type exists for the later core handoff. The demo presenter stores the latest source frame itself and repeats it on the held refresh.

## Picture

`compose` expands the border, then stamps sprites. `shade_image` is the color spec. The shader repeats it.

GPU readback of the sharp demo frame matches `shade_image` exactly. Soft and temporal mixes use the same linear formula; a later panel may differ by one 8-bit level because the shader is f32.

Sprite index 255 is transparent. Index 0 is black and covers the background. The sprite is applied after the temporal blend, so a moving sprite does not smear.

## Keys

Digit 1 Sharp, 2 Soft edge, 3 Temporal. Escape quits. The surface is FIFO (vsync). The viewport is the centered integer rectangle in physical pixels.
