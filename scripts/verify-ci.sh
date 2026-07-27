#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 || ( $# -eq 1 && $1 != "--full" ) ]]; then
  echo "usage: $0 [--full]" >&2
  exit 2
fi

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

echo "checking scripts"
for script in scripts/*.sh; do
  bash -n "$script"
done
python3 -c \
  'from pathlib import Path; path=Path("scripts/reproducible-archive.py"); compile(path.read_text(), str(path), "exec")'

echo "checking Cargo metadata and formatting"
cargo metadata --locked --no-deps --format-version 1 >/dev/null
cargo fmt --all --check

if command -v actionlint >/dev/null 2>&1; then
  echo "checking GitHub Actions workflows"
  actionlint
else
  echo "warning: actionlint is not installed; workflow lint skipped" >&2
fi

if command -v cargo-deny >/dev/null 2>&1; then
  echo "checking dependency policy"
  cargo deny check
else
  echo "warning: cargo-deny is not installed; dependency policy check skipped" >&2
fi

if [[ ${1:-} == "--full" ]]; then
  echo "running full local CI"
  cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
  cargo test --workspace --all-targets --locked
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked
  cargo package --workspace --locked
fi

echo "CI configuration checks complete"
