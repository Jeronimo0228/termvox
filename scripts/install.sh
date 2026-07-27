#!/usr/bin/env bash
set -euo pipefail

REPO="${TERMVOX_INSTALL_REPO:-Jeronimo0228/termvox}"
VERSION="${TERMVOX_VERSION:-latest}"
PREFIX="${TERMVOX_INSTALL_PREFIX:-$HOME/.local}"
INSTALL_SOURCE="${TERMVOX_INSTALL_SOURCE:-0}"

detect_target() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)
  case "${os}-${arch}" in
    Linux-x86_64) echo "x86_64-unknown-linux-gnu" ;;
    Linux-aarch64 | Linux-arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin-x86_64) echo "x86_64-apple-darwin" ;;
    Darwin-arm64) echo "aarch64-apple-darwin" ;;
    MINGW*-x86_64 | MSYS*-x86_64 | CYGWIN*-x86_64) echo "x86_64-pc-windows-msvc" ;;
    *)
      echo "unsupported platform: ${os}-${arch}" >&2
      return 1
      ;;
  esac
}

echo "TermVox installer"
echo "================="

mkdir -p "$PREFIX/bin"

install_from_release() {
  local target asset ext tmp
  target="$(detect_target)" || return 1
  if [[ "$VERSION" == "latest" ]]; then
    asset=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/${VERSION}" \
      | grep -oE "termvox-v[^\"]+-${target}\\.(tar\\.gz|zip)" | head -1) || return 1
  else
    asset="termvox-v${VERSION#v}-${target}"
    if [[ "$target" == *windows* ]]; then
      asset="${asset}.zip"
    else
      asset="${asset}.tar.gz"
    fi
  fi
  [[ -n "$asset" ]] || return 1
  ext="${asset##*.}"
  tmp="$(mktemp -d)"
  cleanup() { rm -rf "$tmp"; }
  trap cleanup EXIT
  echo "Downloading $asset ..."
  curl -fsSL -o "$tmp/$asset" \
    "https://github.com/${REPO}/releases/${VERSION}/download/${asset}"
  if curl -fsSL -o "$tmp/$asset.sha256" \
    "https://github.com/${REPO}/releases/${VERSION}/download/${asset}.sha256" 2>/dev/null; then
    (cd "$tmp" && sha256sum -c "$asset.sha256")
  fi
  if [[ "$ext" == "gz" ]]; then
    tar -xzf "$tmp/$asset" -C "$tmp"
    install -m 0755 "$tmp/termvox" "$PREFIX/bin/termvox"
  else
    unzip -q "$tmp/$asset" -d "$tmp"
    install -m 0755 "$tmp/termvox.exe" "$PREFIX/bin/termvox.exe"
  fi
  return 0
}

install_from_source() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust (cargo) is required when no release binary is available."
    echo "Install from https://rustup.rs or set TERMVOX_VERSION to a published release."
    exit 1
  fi
  if [[ "$(uname -s)" == "Linux" ]] && ! pkg-config --exists alsa 2>/dev/null; then
    echo "Tip: install ALSA headers, e.g. sudo dnf install alsa-lib-devel"
  fi
  local branch="${TERMVOX_INSTALL_BRANCH:-main}"
  local tmp
  tmp="$(mktemp -d)"
  cleanup() { rm -rf "$tmp"; }
  trap cleanup EXIT
  git clone --depth 1 --branch "$branch" "https://github.com/${REPO}.git" "$tmp/termvox"
  cargo install --path "$tmp/termvox/crates/termvox-cli" --force --root "$PREFIX"
}

if [[ "$INSTALL_SOURCE" == "1" ]]; then
  install_from_source
elif ! install_from_release; then
  echo "No pre-built release found for this platform; building from source..."
  install_from_source
fi

export PATH="$PREFIX/bin:$PATH"
termvox models install default
termvox init --preset cursor --force 2>/dev/null || termvox init --force

echo
echo "Installed TermVox to $PREFIX/bin"
echo "Quick start:"
echo "  termvox daemon start --background"
echo "  # focus Cursor Agent, then Alt+Space or: termvox talk"
echo
echo "Optional: install the VS Code / Cursor extension from extensions/vscode-termvox/"
