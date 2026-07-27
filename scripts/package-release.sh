#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 BINARY OUTPUT.{tar.gz|zip}" >&2
  exit 2
fi

binary=$1
output=$2

if [[ ! -f "$binary" ]]; then
  echo "error: binary does not exist: $binary" >&2
  exit 2
fi

if [[ ! "${SOURCE_DATE_EPOCH:-}" =~ ^[0-9]+$ ]]; then
  echo "error: SOURCE_DATE_EPOCH must be set to a Unix timestamp" >&2
  exit 2
fi

if command -v python3 >/dev/null 2>&1; then
  python_cmd=python3
elif command -v python >/dev/null 2>&1; then
  python_cmd=python
else
  echo "error: Python 3 is required to create reproducible archives" >&2
  exit 1
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
"$python_cmd" "$script_dir/reproducible-archive.py" "$binary" "$output"
echo "created $output"
