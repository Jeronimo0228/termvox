#!/usr/bin/env bash
# Prepare and optionally record a TermVox demo for LinkedIn / social.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:-}"

echo "== TermVox demo prep =="
echo

if ! command -v termvox >/dev/null 2>&1; then
  echo "termvox not on PATH. Install first:"
  echo "  npm install -g termvox"
  exit 1
fi

echo "Version: $(termvox --version)"
echo
echo "== doctor (summary) =="
termvox doctor 2>&1 | head -20
echo

echo "== mic smoke (2 s) =="
termvox test --seconds 2
echo

if [[ "$MODE" == "--asciinema" ]]; then
  if ! command -v asciinema >/dev/null 2>&1; then
    echo "asciinema not found. Install: pip install asciinema  OR  dnf install asciinema"
    exit 1
  fi
  CAST="termvox-demo-$(date +%Y%m%d).cast"
  echo "Recording asciinema cast to $CAST"
  echo "When the shell opens, run:"
  echo "  termvox doctor"
  echo "  termvox test --seconds 2"
  echo "  cd YOUR_PROJECT && termvox shell --agent cursor"
  echo "Then press F8, speak, confirm. Exit shell with Ctrl+\\, then exit the recording with Ctrl+D or type exit."
  echo
  read -r -p "Press Enter to start recording..."
  asciinema rec "$CAST"
  echo "Upload: asciinema upload $CAST"
  exit 0
fi

cat <<'EOF'
== Manual recording (OBS / phone) ==

1. Large terminal font (16–18 pt), dark theme.
2. Run: termvox doctor
3. Run: termvox test --seconds 3
4. Run: cd YOUR_PROJECT && termvox shell --agent cursor
5. F8 or Ctrl+Space → speak one short prompt → confirm → show agent working.
6. Ctrl+\ to exit wrapper.

Storyboard + LinkedIn draft:
  docs/demo.md
  docs/linkedin-post-es.md

Beta checklist for friends:
  docs/beta-test-checklist.md
EOF
