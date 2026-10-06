#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC1091
source "$root/scripts/build-release.sh"

failures=0

check() {
  local label="$1"
  local actual="$2"
  local expected="$3"
  if [[ "$actual" == "$expected" ]]; then
    return
  fi
  echo "fail $label: got $actual want $expected" >&2
  failures=$((failures + 1))
}

rejects() {
  local label="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "fail $label: should reject" >&2
    failures=$((failures + 1))
  fi
}

check macos-folder "$(dist_folder "$(platform_of Darwin)" "$(arch_of arm64)")" \
  "dist/eightbit-emu-macos-aarch64"
check macos-cargo "$(cargo_binary macos)" eightbit-emu
check macos-ship "$(shipped_binary macos)" eightbit-emu
check linux-folder "$(dist_folder "$(platform_of Linux)" "$(arch_of x86_64)")" \
  "dist/eightbit-emu-linux-x86_64"
check linux-ship "$(shipped_binary linux)" eightbit-emu
check windows-folder "$(dist_folder "$(platform_of MINGW64_NT-10.0)" "$(arch_of amd64)")" \
  "dist/eightbit-emu-windows-x86_64"
check windows-cargo "$(cargo_binary windows)" eightbit-emu.exe
check windows-ship "$(shipped_binary windows)" eightbit-emu.exe
check aarch64 "$(arch_of aarch64)" aarch64
rejects unknown platform_of Haiku

if [[ "$failures" -ne 0 ]]; then
  echo "$failures failed" >&2
  exit 1
fi
echo "ok"
