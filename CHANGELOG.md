# Changelog

All notable changes to TermVox will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project intends to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once public version
compatibility is defined. During alpha development, breaking changes may occur
in minor or patch increments and must be called out explicitly.

## [Unreleased]

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

[Unreleased]: https://github.com/Jeronimo0228/termvox/compare/v0.1.0-alpha.7...main
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
