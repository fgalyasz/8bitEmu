# Addendum — Launcher shell

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1 / FR-5 — LaunchMode

`parse_mode(args) -> LaunchMode::{Launcher, Boot(Launch)}`. Boot when any of `--machine`, `--rom`, `--sna`, `--tap`, `--tzx`, `--prg`, `--kernal`, `--basic`, `--chargen` appears. `--model` alone does not force Boot.

## FR-2 / FR-3 — AppConfig

Fields: `machine`, `model_128`, `spectrum_rom`, `kernal`, `basic`, `chargen`. Persist under platform config dir. Defaults: Spectrum 48K + `roms/spectrum-48.rom` + `roms/c64-{kernal,basic,chargen}.rom`.

## FR-4 — App mode

`App` holds `Idle | Running`. Idle clears surface; Start builds Session from config via existing `presenter_from` path.
