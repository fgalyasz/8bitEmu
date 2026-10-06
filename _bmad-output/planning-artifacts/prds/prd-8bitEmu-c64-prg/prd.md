---
title: C64 PRG load
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 83
---

# PRD: C64 PRG load

Hobby/solo. Soft-load a Commodore `.prg` into a booted C64 so BASIC or machine-code payloads run without a disk/tape drive.

## 1. Vision

Anna opens a `.prg` she owns. Bytes land at the file’s load address. A BASIC program at `$0801` links and auto-`RUN`s. Other load addresses jump to the start. CLI `--prg` boots C64 and loads the same way.

## 2. User journeys

- **UJ-1.** Anna runs `--machine c64 --prg game.prg` and sees the program start after READY.
- **UJ-2.** On a booted C64, File → Open… chooses `demo.prg` and the payload runs.

## 3. Features

#### FR-1: PRG parse and RAM write

Two-byte little-endian load address, then payload. Reject files shorter than three bytes and payloads that wrap past `$FFFF`.

**Consequences:**
- Core exposes `load_prg` on the C64 machine / presenter.
- Unit tests cover happy path, short file, and overflow.

#### FR-2: BASIC link and autostart

When the load address is `$0801`, update VARTAB / ARYTAB / STREND (and `$AE`/`$AF`) to the first free byte after the payload, then queue `RUN` + RETURN in the KERNAL keyboard buffer. Other addresses set the CPU PC to the load address.

**Consequences:**
- A tiny BASIC stub PRG changes screen RAM after autostart frames.
- A non-`$0801` PRG with a known store executes from the load address.

#### FR-3: CLI and File → Open

`--prg path` selects the C64 machine (default ROM paths apply) and loads after boot. Open dialog accepts `.prg` and only applies it on a C64 host.

**Consequences:**
- Launch parse stores `prg`; unknown Spectrum session refuses `.prg` with a clear message.

## 4. Non-goals

D64/T64, IEC bus, tape for C64, CRT cartridges, authentic LOAD timing, stock ROM redistribution.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: Parse / link / overflow unit tests pass.
- **SM-C2**: `--prg` is accepted and implies C64.
