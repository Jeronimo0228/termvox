# Compatibility

TermVox is `0.1.0-alpha.1`. Compatibility claims below describe source
implementations, not a completed release certification matrix.

## Operating systems

| Platform | Source support | Important dependency |
| --- | --- | --- |
| Linux | Intended | ALSA development libraries |
| macOS | Intended | Xcode Command Line Tools; microphone permission |
| Windows | Intended | MSVC Rust toolchain and C++ Build Tools |

Audio uses CPAL's default host. Hardware, drivers, remote sessions, containers,
and audio servers can affect device availability. There is no published list of
tested OS versions or architectures.

## Rust

The workspace declares Rust 1.86 and edition 2024. Older toolchains are
unsupported.

## Coding agents

Built-in adapters exist for Codex CLI, Claude Code, Cursor CLI, Gemini CLI,
Aider, and Amp. They depend on upstream arguments and output shapes that may
change without TermVox. The repository does not pin or certify agent CLI
versions. Aider uses plain text and does not support session resume in the
current adapter. See [Coding agents](agents.md).

## Speech-to-text

Whisper.cpp compatibility means an executable accepting the documented
`whisper-cli` arguments and a compatible GGML model. OpenAI compatibility means
an endpoint implementing the multipart transcription shape used by the
adapter. Alternative endpoints are not tested or endorsed.

Parakeet and Vosk use a generic sidecar command contract. TermVox does not
bundle or certify a sidecar or model for either engine. See
[Speech engines](speech-engines.md).

## Terminals and keys

Interactive push-to-talk requires press and release events from a focused
terminal. Supported configured keys are `SPACE`, `ENTER`, `TAB`, `F1` through
`F24`, and a single character. Chords such as `ALT+SPACE` are not supported.
Terminal emulators, multiplexers, and remote shells may not report release
events reliably.

## Known pre-release gaps

- No claimed prebuilt, signed, or notarized artifacts
- No package-manager distribution claimed
- No stable configuration or plugin API guarantee
- Configured plugins can be inspected and tested but cannot drive
  `termvox start`
- Global-hotkey support is detected by diagnostics, but interactive start still
  uses focused terminal events; use external record commands for OS bindings
- `termvox doctor` reports failed checks in output without necessarily returning
  a failing process status
