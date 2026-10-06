---
title: Rename to 8bitEmu
status: draft
created: 2026-10-06
updated: 2026-10-06
parent_issue: 66
---

# PRD: Rename to 8bitEmu

Hobby/solo. The app is a usable 8-bit emulator. Scanline named a CRT look, not the product.

## 1. Vision

The product is **8bitEmu**. It hosts Spectrum today, C64 next, and can grow to other 8-bit machines. Rust packages use `eightbit-emu-*` because a crate name cannot start with a digit.

## 2. User journeys

- **UJ-1. Open the app.** Anna sees 8bitEmu in the window title and the macOS app menu.
- **UJ-2. Build and run.** `cargo run -p eightbit-emu-app` starts the same player.

## 3. Features

#### FR-1: Product name

User-visible text says 8bitEmu. Default save names use `8bitemu` as the stem. Log lines use the `8bitemu:` prefix.

**Consequences:**
- Look titles begin with `8bitEmu —`.
- F11 writes `8bitemu.sna`.

#### FR-2: Package and repository names

Workspace crates are `eightbit-emu-core` and `eightbit-emu-app`. The release binary is `eightbit-emu`. The GitHub repository is `8bitEmu`. New PRD folders use `prd-8bitEmu-<slug>`.

**Consequences:**
- `cargo test -p eightbit-emu-core` runs the core suite.
- Historical `prd-Scanline-*` folders stay as shipped history.

## 4. Non-goals

Renaming CRT “scanline” mechanics in code comments or test names that mean a display row. Website and DMG until a public increment.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The assertions above pass.
