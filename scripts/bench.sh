#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ ! -f "${TERMVOX_WHISPER_MODEL:-}" ]]; then
  termvox models install default
fi

termvox bench --runs "${TERMVOX_BENCH_RUNS:-3}"
