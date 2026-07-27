#!/usr/bin/env bash
set -euo pipefail

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
    exit 1
    ;;
esac
