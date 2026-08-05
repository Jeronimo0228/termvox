# Release readiness

TermVox is published as **`0.1.0-alpha.15`** (alpha). This page tracks what is
already shipped versus what still needs manual evidence before a beta or V1 claim.

## Published today

| Channel | Status |
| --- | --- |
| [npm `termvox`](https://www.npmjs.com/package/termvox) | `latest` → alpha.10 (Trusted Publishing / OIDC) |
| [GitHub Releases](https://github.com/Jeronimo0228/termvox/releases) | Linux, macOS, Windows archives per tag |
| CI on `main` | fmt, clippy, cross-platform tests, npm pack, supply-chain job |
| Release workflow on tags | Multi-platform builds, SHA-256, Sigstore bundles, SBOM fallback |

Install paths: [npm](npm.md), [shell script](../scripts/install.sh),
[quick start](quick-start.md). STT tuning: [performance](performance.md) ·
[STT ES](es/stt.md).

## Automated gates (green on `main`)

- `cargo fmt --all --check`
- Clippy with `-D warnings`, all targets/features
- Workspace tests (Linux, macOS, Windows × stable + MSRV 1.88)
- Feature matrix checks (`default`, `--no-default-features`, `--all-features`)
- `cargo doc`, `cargo package --workspace --locked`
- Shell smoke script (`scripts/shell-smoke.sh`)
- npm package build, test, and pack in CI

## Artifact inventory

- Seven Rust workspace crates and the `termvox` binary
- npm wrapper with postinstall bootstrap and bundled editor extension
- Shell and PowerShell installers
- Bash/Zsh/Fish/PowerShell/Elvish completions and `termvox(1)` manpage (CLI-generated)
- Per-release SHA-256 sums and Sigstore attestations on GitHub Releases

Homebrew, Flatpak, `.deb`, and crates.io publication remain **documented only**
(see [packaging](packaging.md)); they are not maintained install channels yet.

## Required before beta or V1

- [x] Publish verified release archives and npm package.
- [x] CI on all primary platforms with locked dependencies.
- [x] Supply-chain controls on npm postinstall (SHA-256, URL allowlist).
- [ ] Manual microphone capture and permission tests on macOS and Windows for each release.
- [ ] Record evidence for each supported route in the seven-agent matrix
      (Codex, Claude Code, Cursor, Gemini, Aider, Amp, OpenCode).
- [ ] Record evidence for embedded Whisper, OpenAI, Parakeet, and Vosk on every
      platform claimed in the compatibility table.
- [ ] Clean `install → setup → doctor → shell` run for each release archive.
- [ ] Maintainer sign-off on compatibility matrix and release notes.

An adapter existing in source is not evidence that an upstream version or
hardware path has passed these manual gates.
