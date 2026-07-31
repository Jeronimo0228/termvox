# Changelog

All notable changes to TermVox will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project intends to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once public version
compatibility is defined. During alpha development, breaking changes may occur
in minor or patch increments and must be called out explicitly.

## [Unreleased]

## [0.1.0-alpha.9] - 2026-07-31

### Added

- npm package `termvox` on registry.npmjs.org with postinstall binary bootstrap
- Supply-chain controls: mandatory SHA-256, GitHub URL allowlist, `docs/npm-security.md`
- Trusted Publishing (OIDC) workflow `publish-npm.yml`

### Security

- Removed `TERMVOX_INSTALL_REPO` override from npm installer
- Editor extension validates `termvox.cliPath` for shell metacharacters

[0.1.0-alpha.9]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.8...v0.1.0-alpha.9

## [0.1.0-alpha.8] - 2026-07-30

### Added

- Embedded SQLite (`rusqlite`) for OpenCode session discovery — no external
  `sqlite3` CLI required
- Claude session discovery from `.jsonl` files when no session subdirectory exists
- `usage_hints()` in `termvox-core` — companion→shell, Whisper model/profile
  mismatches, Wayland guidance, workspace session path
- `termvox doctor --json` now includes a `hints` array
- Interactive `termvox setup` prompt to prefer `display = "shell"` for
  Cursor/OpenCode
- CI job `shell-smoke` (`scripts/shell-smoke.sh`) — shell unit tests, release
  build, doctor JSON smoke
- Spanish docs: `docs/es/cli-reference.md`, `docs/es/troubleshooting.md`
- Packaging notes: `docs/packaging.md` (Homebrew sketch, `.deb`, Flatpak outline)

### Changed

- `performance_profile = "balanced"` now selects `ggml-base.bin` (was tiny)
- `performance_profile = "accurate"` also defaults to `ggml-base.bin`

### Fixed

- OpenCode session queries use parameterized SQL instead of string interpolation

## [0.1.0-alpha.7] - 2026-07-28

### Added

- Workspace session persistence (`.termvox/session.json`) for shell, branded, and
  daemon modes with `termvox shell --fresh` to ignore saved sessions
- Agent-local session discovery for Cursor, OpenCode, and Claude when no saved id
  exists (`[workspace].discover_session`, default `true`)
- PTY output heuristics to capture upstream session ids from JSON fragments
- Localized integrated shell bar (`es`/`en`) with agent-themed chrome
- Streaming partial transcripts in the integrated shell bar
- Shared workspace persistence helpers (`workspace.rs`, `session_store.rs`)
- Resume argv mapping for all structured agents (`invocation.rs`)
- `termvox start` routes to `termvox shell` when `display = "shell"`

### Changed

- Cursor shell mode auto-trusts the workspace cwd (`-f`) without extra config
- Shell exit hotkey accepts raw `Ctrl+\` (0x1c) bytes from Linux TTYs
- Branded/daemon runtimes hydrate and persist upstream session ids across runs
- Cursor preset defaults `trust_workspace = true`
- PTY filter blocks bracketed-paste capture (`?2004`) in addition to Kitty keyboard

### Fixed

- Mic bar redraw during transcribing and after upstream TUI repaints
- Clippy and docs for workspace session store APIs

[Unreleased]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.9...main
[0.1.0-alpha.8]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.7...v0.1.0-alpha.8
[0.1.0-alpha.7]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.6...v0.1.0-alpha.7

## [0.1.0-alpha.6] - 2026-07-28

### Fixed

- Shell voice hotkeys when agent TUIs enable the Kitty keyboard protocol

[0.1.0-alpha.6]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.5...v0.1.0-alpha.6

## [0.1.0-alpha.5] - 2026-07-27

### Fixed

- Integrated shell mic bar persistence, recording animation, and exit handling

[0.1.0-alpha.5]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.4...v0.1.0-alpha.5

## [0.1.0-alpha.4] - 2026-07-27

### Added

- Integrated agent shell (`termvox shell`) for all seven built-in CLIs
- OpenCode adapter and auth preflight

[0.1.0-alpha.4]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.3...v0.1.0-alpha.4

## Earlier alpha releases

See git tags `v0.1.0-alpha.1` through `v0.1.0-alpha.3` for initial CI, Windows
builds, and multi-platform release workflow.
