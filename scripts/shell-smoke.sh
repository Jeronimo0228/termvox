#!/usr/bin/env bash
# Smoke checks for integrated shell helpers and CLI wiring (no live agent PTY).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== shell smoke: unit tests =="
cargo test -p termvox shell:: --locked
cargo test -p termvox-agents session_discover --locked

echo "== shell smoke: build release binary =="
cargo build --release -p termvox --locked -q
export PATH="$ROOT/target/release:$PATH"

echo "== shell smoke: version + doctor json =="
termvox --version
termvox doctor --json | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'hints' in d; assert 'configuration' in d"

echo "shell smoke: OK"
