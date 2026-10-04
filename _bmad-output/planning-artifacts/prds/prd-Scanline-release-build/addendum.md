# Addendum — Release build

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`uname -s` maps Darwin to macos, Linux to linux, and MINGW, MSYS, or CYGWIN to windows. `uname -m` maps arm64 and aarch64 to aarch64, and x86_64 and amd64 to x86_64. The cargo artifact is `target/release/scanline-app`, plus `.exe` on Windows. The shipped file drops the `-app` suffix.

## FR-2

Source `"$HOME/.cargo/env"` when that file exists. Run `cargo build --release -p scanline-app` from the repo root. Copy the artifact into the dist folder. Honor `CARGO_TARGET_DIR` when it is set.
