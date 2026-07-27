#!/usr/bin/env bash
set -euo pipefail

REPO="${TERMVOX_INSTALL_REPO:-https://github.com/Jeronimo0228/termvox.git}"
BRANCH="${TERMVOX_INSTALL_BRANCH:-main}"
PREFIX="${TERMVOX_INSTALL_PREFIX:-$HOME/.local}"

echo "TermVox installer"
echo "================="

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust (cargo) is required. Install from https://rustup.rs and retry."
  exit 1
fi

if [[ "$(uname -s)" == "Linux" ]] && ! pkg-config --exists alsa 2>/dev/null; then
  echo "Tip: install ALSA headers for audio capture, e.g.:"
  echo "  Fedora: sudo dnf install alsa-lib-devel"
  echo "  Debian/Ubuntu: sudo apt install libasound2-dev"
fi

TMPDIR="$(mktemp -d)"
cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

git clone --depth 1 --branch "$BRANCH" "$REPO" "$TMPDIR/termvox"
cargo install --path "$TMPDIR/termvox/crates/termvox-cli" --force --root "$PREFIX"

export PATH="$PREFIX/bin:$PATH"
termvox models install default
termvox init --preset cursor --force || termvox init --force

echo
echo "Installed TermVox to $PREFIX/bin/termvox"
echo "Quick start:"
echo "  termvox daemon start --background"
echo "  # focus Cursor Agent, then press ALT+SPACE or run: termvox talk"
