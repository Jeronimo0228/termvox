#!/usr/bin/env bash
# Render docs/assets/termvox-demo-4k.mp4 from the VHS tape (3840×2160 @ 60fps).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TAPE="docs/assets/termvox-demo.tape"
OUT="docs/assets/termvox-demo-4k.mp4"
TERMVOX_BIN="${ROOT}/target/release/termvox"

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "error: ffmpeg is required (install via your package manager)" >&2
  exit 1
fi

if ! command -v vhs >/dev/null 2>&1; then
  if ! command -v go >/dev/null 2>&1; then
    echo "error: vhs not found and go is not installed to fetch it" >&2
    exit 1
  fi
  echo "Installing vhs via go install..."
  go install github.com/charmbracelet/vhs@latest
  export PATH="${PATH}:${HOME}/go/bin"
fi

if [[ ! -x "$TERMVOX_BIN" ]]; then
  echo "Building termvox (release)..."
  cargo build --release -p termvox
fi

export PATH="${ROOT}/target/release:${PATH}"

if [[ ! -f "$TAPE" ]]; then
  echo "error: missing tape file: $TAPE" >&2
  exit 1
fi

echo "Rendering 4K demo with vhs..."
vhs "$TAPE"

if [[ -f "$OUT" ]]; then
  echo "Created $OUT ($(du -h "$OUT" | cut -f1))"
  if command -v ffprobe >/dev/null 2>&1; then
    ffprobe -v error -select_streams v:0 \
      -show_entries stream=width,height,r_frame_rate,codec_name \
      -of default=noprint_wrappers=1 "$OUT"
  fi
else
  echo "error: expected output not found: $OUT" >&2
  exit 1
fi
