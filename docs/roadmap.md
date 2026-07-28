# Roadmap

This roadmap communicates intent, not a promise of scope or dates. An item is
not available until it is implemented, tested, documented, and reflected in
the current compatibility matrix.

## Current alpha foundation

- Terminal-focused push-to-talk and an external start/stop marker
- CPAL microphone capture, filtered resampling, and energy-based trimming
- Whisper.cpp and OpenAI transcription adapters
- Generic Parakeet and Vosk sidecar adapters
- Codex, Claude, Cursor, Gemini, Aider, Amp, and OpenCode CLI adapters
- **`termvox shell`:** integrated Agent Shell for all seven built-in agents —
  upstream TUI in a PTY plus a persistent TermVox mic bar and stdin injection
- Non-interactive upstream auth probes (`termvox doctor`, shell/start preflight)
- Explicit prompt transformations and risk-triggered confirmation
- Layered configuration, diagnostics, audio test, and verified model download
- Streaming agent events with runtime limits and session reuse
- Configured JSON-RPC plugin inspection and conformance probing

These components exist in source but remain subject to alpha-level change.

## Compatibility and hardening targets

- Tested OS, architecture, terminal, agent-version, and model matrices
- Maintained Parakeet and Vosk sidecar implementations or verified references
- More reliable terminal and platform trigger integration
- Streaming transcription and richer event rendering
- Adapter-specific parsers and contract tests for upstream CLI versions
- Windows support for `termvox install-shim`

## Extensibility targets

- Selecting external plugins as interactive agents
- Stable lifecycle schemas and deeper conformance tests
- A reviewable distribution model with integrity metadata before any registry
- Profiles, reusable prompt context, and carefully designed opt-in history

## Release-engineering targets

- Reproducible multi-platform builds
- Checksums, software bills of materials, provenance, and artifact signing
- Installation and rollback documentation
- A stable protocol and configuration schema for 1.0
- Accessibility review and hardware-acceleration guidance

Self-update remains advisory until signed artifact verification and atomic
rollback are available.
