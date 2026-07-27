# Changelog

All notable changes to TermVox will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project intends to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once public version
compatibility is defined. During alpha development, breaking changes may occur
in minor or patch increments and must be called out explicitly.

## [Unreleased]

### Added

- Professional user documentation for installation, quick start,
  configuration, CLI usage, agents, speech engines, privacy, architecture,
  plugins, compatibility, troubleshooting, and frequently asked questions
- Essential Spanish quick-start documentation
- mdBook navigation and build configuration under `docs/`
- Contribution, conduct, security, support, governance, maintainer, and release
  policies
- Maintained documentary configuration examples

### Changed

- Project documentation now uses the TermVox name, `termvox` command, canonical
  repository URL, and `MIT OR Apache-2.0` licensing consistently
- Compatibility documentation distinguishes six built-in agent adapters,
  direct speech adapters, and generic Parakeet/Vosk sidecar contracts
- Release and security language no longer implies that artifacts, signatures,
  checksums, SBOMs, or package-manager distributions already exist

### Known limitations

- Plugins can be inspected and probed but not selected as interactive agents
- Parakeet and Vosk require separately supplied compatible sidecars
- No stable release or compatibility guarantee has been declared

[Unreleased]: https://github.com/Jeronimo0228/termvox/commits/main
