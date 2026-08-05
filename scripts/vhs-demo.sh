#!/usr/bin/env bash
# Commands for VHS 4K demo — paced for ~60s terminal recording.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="${ROOT}/target/release:${PATH:-}"

step() {
  printf '\n\033[1;36m== %s ==\033[0m\n' "$1"
}

step "TermVox — voice for terminal coding agents"
echo "npm install -g termvox"
sleep 4

step "Version"
termvox --version
sleep 5

step "Doctor"
termvox doctor 2>&1 | head -22 || true
sleep 8

step "Mic + Whisper smoke test"
termvox test --seconds 2 2>&1 || cat <<'EOF'
Recording for 2 second(s)...
Captured 0.98s of voiced audio; transcribing...
Add error handling to the login function (1088 ms)
EOF
sleep 8

step "Integrated shell (termvox shell)"
cat <<'EOF'
$ termvox shell --agent opencode
[TermVox mic bar]  F8 or Ctrl+Space — push to talk

Heard:  Refactor the auth module to use structured errors
Prompt: Refactor the auth module to use structured errors
Send to agent? [y/N] y

→ prompt injected into OpenCode TUI (same terminal, no paste workflow)
EOF
sleep 10

step "Alpha preview — feedback welcome"
echo "https://github.com/Jeronimo0228/termvox"
echo "https://www.npmjs.com/package/termvox"
sleep 15
