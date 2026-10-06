# Addendum — Rename to 8bitEmu

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

Replace user-facing `Scanline` with `8bitEmu`. Default paths `scanline.sna` / `.tap` / `.tzx` become `8bitemu.*`. Eprintln prefix `scanline:` becomes `8bitemu:`.

## FR-2

Move `crates/scanline-core` → `crates/eightbit-emu-core` and `crates/scanline-app` → `crates/eightbit-emu-app`. Update workspace members, imports (`eightbit_emu_core`, `eightbit_emu_app`), release scripts, PDLC helpers, and `gh repo rename 8bitEmu`.
