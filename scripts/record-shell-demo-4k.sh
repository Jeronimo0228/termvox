#!/usr/bin/env bash
# Record termvox shell --demo --demo-auto with real mic bar + fake agent TUI, render 4K MP4.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CAST="docs/assets/termvox-shell-demo.cast"
GIF="docs/assets/termvox-shell-demo.gif"
OUT="docs/assets/termvox-demo-shell-4k.mp4"
LINKEDIN="docs/assets/termvox-demo-4k-linkedin.mp4"
AGG="${ROOT}/.cache/bin/agg"
AGENT="${1:-opencode}"

mkdir -p .cache/bin

if [[ ! -x "$AGG" ]]; then
  echo "Downloading agg (asciinema gif generator)..."
  curl -sL -o "$AGG" "https://github.com/asciinema/agg/releases/download/v1.9.0/agg-x86_64-unknown-linux-gnu"
  chmod +x "$AGG"
fi

command -v ffmpeg >/dev/null || { echo "ffmpeg required"; exit 1; }
command -v asciinema >/dev/null || { echo "asciinema required (pip install asciinema)"; exit 1; }

echo "Building termvox..."
cargo build --release -p termvox --locked -q
export TERMVOX_DEMO_SCRIPT="$ROOT/scripts/demo-agent-tui.sh"
BIN="$ROOT/target/release/termvox"

echo "Recording shell demo (agent=$AGENT)..."
asciinema rec -y --overwrite -i 2 --cols 120 --rows 42 -e TERM=xterm-256color \
  -c "timeout 45 $BIN shell --demo --demo-auto --agent $AGENT --fresh" "$CAST"

echo "Rendering GIF..."
"$AGG" --font-size 32 --line-height 1.4 --theme dracula "$CAST" "$GIF"

echo "Encoding 4K MP4..."
ffmpeg -y -loglevel error -i "$GIF" \
  -vf "scale=3840:2160:flags=lanczos,fps=30" \
  -c:v libx264 -crf 17 -pix_fmt yuv420p -movflags +faststart "$OUT"

echo "LinkedIn stretch (~55s)..."
ffmpeg -y -loglevel error -i "$OUT" \
  -filter:v "setpts=3.05*PTS" -r 30 \
  -c:v libx264 -crf 17 -pix_fmt yuv420p -movflags +faststart "$LINKEDIN"

ls -lh "$OUT" "$LINKEDIN"
ffprobe -v error -show_entries stream=width,height -show_entries format=duration -of default=nw=1 "$OUT" "$LINKEDIN"

echo "Done. Attach $LINKEDIN to LinkedIn."
