#!/usr/bin/env bash
set -euo pipefail

validate_version() {
  local version=$1
  if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]]; then
    echo "error: invalid SemVer version: $version" >&2
    exit 2
  fi
}

if [[ ${1:-} == "--validate-version" ]]; then
  [[ $# -eq 2 ]] || {
    echo "usage: $0 --validate-version VERSION" >&2
    exit 2
  }
  validate_version "$2"
  exit 0
fi

if [[ $# -ne 1 ]]; then
  echo "usage: $0 VERSION" >&2
  exit 2
fi

version=$1
validate_version "$version"

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

if command -v python3 >/dev/null 2>&1; then
  python_cmd=python3
else
  python_cmd=python
fi

metadata=$(cargo metadata --no-deps --format-version 1)
workspace_version=$(
  "$python_cmd" -c \
    'import json,sys; d=json.load(sys.stdin); versions={p["version"] for p in d["packages"]}; assert len(versions)==1, versions; print(versions.pop())' \
    <<<"$metadata"
)
if [[ "$version" != "$workspace_version" ]]; then
  echo "error: requested version $version does not match workspace version $workspace_version" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "warning: working tree is dirty; dry-run output is not releaseable" >&2
fi

target=$(rustc -vV | awk '/^host:/ {print $2}')
case "$target" in
  *windows*) extension=zip; binary="target/$target/dist/termvox.exe" ;;
  *) extension=tar.gz; binary="target/$target/dist/termvox" ;;
esac

echo "checking package contents"
cargo package --workspace --locked

echo "building locked dist profile for $target"
cargo build --profile dist --locked --target "$target" --bin termvox

export SOURCE_DATE_EPOCH
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
dist_dir=target/release-dry-run
asset="termvox-v${version}-${target}.${extension}"
rm -rf "$dist_dir"
mkdir -p "$dist_dir"

scripts/package-release.sh "$binary" "$dist_dir/$asset"
scripts/package-release.sh "$binary" "$dist_dir/$asset.repro.${extension}"
if ! cmp -s "$dist_dir/$asset" "$dist_dir/$asset.repro.${extension}"; then
  echo "error: repeated archive creation was not reproducible" >&2
  exit 1
fi
rm "$dist_dir/$asset.repro.${extension}"
(cd "$dist_dir" && sha256sum "$asset" > "$asset.sha256")

echo "dry run complete: $dist_dir/$asset"
echo "SBOM, keyless signature, and GitHub attestation are generated only by the release workflow."
