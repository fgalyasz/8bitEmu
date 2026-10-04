---
title: Release build
status: done
created: 2026-10-04
updated: 2026-10-04
parent_issue: 38
---

# PRD: Release build

Hobby/solo. One script writes a release executable for the computer it runs on.

## 1. Vision

`./scripts/build-release.sh` builds Scanline in release mode and copies the executable into `dist/`. On macOS and Linux the file is `scanline`. On Windows it is `scanline.exe`. The folder name says which system and CPU it is for.

## 2. User journeys

- **UJ-1. Build here.** Anna runs the script on her Mac, a Windows machine, or a Linux machine. It prints the path of that machine's executable.
- **UJ-2. Wrong system.** The script stops, with the system name, when it does not recognize the host.

## 3. Features

#### FR-1: Output name

macOS goes to `dist/scanline-macos-<arch>/scanline`. Linux goes to `dist/scanline-linux-<arch>/scanline`. Windows goes to `dist/scanline-windows-<arch>/scanline.exe`. `arm64` is written as `aarch64`.

**Consequences:**
- Darwin on arm64 names `dist/scanline-macos-aarch64/scanline` and reads `scanline-app`.
- Linux on x86_64 names `dist/scanline-linux-x86_64/scanline`.
- A Windows host names `dist/scanline-windows-x86_64/scanline.exe` and reads `scanline-app.exe`.
- An unknown system name is rejected.

#### FR-2: Host build

The script builds the `scanline-app` release for the host and copies that binary to the path from FR-1. It does not download a ROM.

**Consequences:**
- After a successful run, the printed path is an executable file.

## 4. Non-goals

A DMG or installer, a ROM inside the folder, and building the other two systems from the one you are on.

## 5. Success metrics

- **SM-1**: `cargo test` passes, and `scripts/build-release.test.sh` passes.
- **SM-C1**: The name assertions above pass, and a host run leaves the executable in `dist/`.
