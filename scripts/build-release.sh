#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

platform_of() {
  case "$1" in
    Darwin) echo macos ;;
    Linux) echo linux ;;
    MINGW*|MSYS*|CYGWIN*) echo windows ;;
    *) return 1 ;;
  esac
}

arch_of() {
  case "$1" in
    arm64|aarch64) echo aarch64 ;;
    x86_64|amd64) echo x86_64 ;;
    *) echo "$1" ;;
  esac
}

dist_folder() {
  echo "dist/scanline-$1-$2"
}

cargo_binary() {
  if [[ "$1" == windows ]]; then
    echo scanline-app.exe
    return
  fi
  echo scanline-app
}

shipped_binary() {
  if [[ "$1" == windows ]]; then
    echo scanline.exe
    return
  fi
  echo scanline
}

target_dir() {
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    echo "$CARGO_TARGET_DIR"
    return
  fi
  echo "$root/target"
}

die() {
  echo "scanline: $*" >&2
  exit 1
}

load_cargo() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  command -v cargo >/dev/null || die "cargo is not installed"
}

build_release() {
  (cd "$root" && cargo build --release -p scanline-app)
}

publish() {
  local source="$1"
  local folder="$2"
  local dest="$3"
  mkdir -p "$root/$folder"
  cp "$(target_dir)/release/$source" "$root/$folder/$dest"
  chmod +x "$root/$folder/$dest"
}

main() {
  local platform arch folder source dest
  platform="$(platform_of "$(uname -s)")" || die "unsupported system: $(uname -s)"
  arch="$(arch_of "$(uname -m)")"
  folder="$(dist_folder "$platform" "$arch")"
  source="$(cargo_binary "$platform")"
  dest="$(shipped_binary "$platform")"
  load_cargo
  build_release
  publish "$source" "$folder" "$dest"
  echo "$folder/$dest"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
