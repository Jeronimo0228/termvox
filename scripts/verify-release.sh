#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 [--write-checksums] [DIST_DIR]" >&2
}

write_checksums=false
if [[ ${1:-} == "--write-checksums" ]]; then
  write_checksums=true
  shift
fi
if [[ $# -gt 1 ]]; then
  usage
  exit 2
fi

dist_dir=${1:-dist}
if [[ ! -d "$dist_dir" ]]; then
  echo "error: release directory does not exist: $dist_dir" >&2
  exit 2
fi

shopt -s nullglob
archives=("$dist_dir"/termvox-v*.tar.gz "$dist_dir"/termvox-v*.zip)
if [[ ${#archives[@]} -eq 0 ]]; then
  echo "error: no TermVox release archives found in $dist_dir" >&2
  exit 1
fi

if $write_checksums; then
  (
    cd "$dist_dir"
    files=(termvox-v*.tar.gz termvox-v*.zip)
    printf '%s\n' "${files[@]}" | LC_ALL=C sort | xargs sha256sum
  ) > "$dist_dir/SHA256SUMS"
  echo "wrote $dist_dir/SHA256SUMS"
  exit 0
fi

for archive in "${archives[@]}"; do
  checksum="$archive.sha256"
  sbom="$archive.cdx.json"
  bundle="$archive.sigstore.json"

  for companion in "$checksum" "$sbom" "$bundle"; do
    if [[ ! -s "$companion" ]]; then
      echo "error: missing or empty companion file: $companion" >&2
      exit 1
    fi
  done

  (cd "$dist_dir" && sha256sum --check "$(basename "$checksum")")

  case "$archive" in
    *.tar.gz)
      contents=$(tar -tzf "$archive")
      ;;
    *.zip)
      unzip -tqq "$archive"
      contents=$(unzip -Z1 "$archive")
      ;;
  esac
  binary=termvox
  [[ "$archive" == *.zip ]] && binary=termvox.exe
  required=(
    "$binary"
    README.md
    LICENSE-MIT
    LICENSE-APACHE
    termvox.1
    completions/termvox.bash
    completions/termvox.zsh
    completions/termvox.fish
    completions/termvox.ps1
    completions/termvox.elv
  )
  for path in "${required[@]}"; do
    if [[ ! $'\n'"$contents"$'\n' == *$'\n'"$path"$'\n'* ]]; then
      echo "error: missing $path in $archive" >&2
      exit 1
    fi
  done

  python_cmd=python3
  command -v "$python_cmd" >/dev/null 2>&1 || python_cmd=python
  "$python_cmd" -c \
    'import json,sys; data=json.load(open(sys.argv[1], encoding="utf-8")); assert data.get("bomFormat") == "CycloneDX"' \
    "$sbom"

  if command -v cosign >/dev/null 2>&1; then
    cosign verify-blob \
      --bundle "$bundle" \
      --certificate-identity-regexp \
        '^https://github\.com/Jeronimo0228/termvox/\.github/workflows/release\.yml@refs/(heads|tags)/.+' \
      --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
      "$archive"
  elif [[ ${CI:-} == "true" ]]; then
    echo "error: cosign is required in CI" >&2
    exit 1
  else
    echo "warning: cosign unavailable; skipped signature verification for $archive" >&2
  fi
done

echo "verified ${#archives[@]} release archive(s)"
