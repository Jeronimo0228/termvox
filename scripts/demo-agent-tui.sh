#!/usr/bin/env bash
# Fake agent TUI for TermVox demo recordings (runs inside termvox shell PTY).
set -uo pipefail

agent="${1:-opencode}"
cols="${COLUMNS:-100}"

banner() {
  local title color
  case "$agent" in
    cursor)
      title="Cursor Agent"
      color="1;34"
      ;;
    claude)
      title="Claude Code"
      color="1;33"
      ;;
    codex)
      title="Codex CLI"
      color="1;32"
      ;;
    opencode | *)
      title="OpenCode"
      color="1;36"
      agent="opencode"
      ;;
  esac
  printf '\033[%sm%s\033[0m · \033[2mtermvox shell demo\033[0m\n' "$color" "$title"
  printf '\033[2m%s\033[0m\n' "$(printf '─%.0s' $(seq 1 "$cols"))"
  echo
  echo "› Ask anything about this codebase"
  echo
}

respond() {
  local prompt="$1"
  echo
  printf '\033[2mYou:\033[0m %s\n' "$prompt"
  echo
  printf '\033[2m⠋ Thinking…\033[0m\r'
  sleep 1
  printf "\033[K\033[1mAgent:\033[0m I'll refactor the auth module to use structured errors and add tests for the login flow.\n"
  echo
  echo "› "
}

banner

while IFS= read -r line || [[ -n "${line:-}" ]]; do
  line="${line//$'\r'/}"
  [[ -z "$line" ]] && continue
  respond "$line"
done
