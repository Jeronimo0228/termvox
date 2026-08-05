# Compatibility

TermVox is `0.1.0-alpha.15`. Compatibility claims below describe source
implementations and published install channels, not a completed release
certification matrix. See [release readiness](release-readiness.md) for manual
QA gates still open before beta.

## Published install channels

| Channel | Status |
| --- | --- |
| [npm `termvox`](https://www.npmjs.com/package/termvox) | `latest` → alpha.10; postinstall downloads GitHub Release binary |
| [GitHub Releases](https://github.com/Jeronimo0228/termvox/releases) | Linux, macOS, Windows archives per tag |
| Shell / PowerShell installers | Documented in [installation](installation.md) |
| Homebrew, Flatpak, `.deb`, crates.io | **Not** maintained channels yet ([packaging](packaging.md)) |

Release archives ship **SHA-256 checksums** and **Sigstore/cosign** bundles
(`.sigstore.json`). npm postinstall verifies SHA-256 when checksum files are
present. macOS notarization is **not** claimed.

## Operating systems

| Platform | Source support | Important dependency |
| --- | --- | --- |
| Linux | Intended | ALSA / PipeWire; mic permission for terminal app |
| macOS | Intended | Xcode CLT; microphone permission for terminal |
| Windows | Intended | x64 MSVC build; mic privacy settings |

Audio uses CPAL's default host. Hardware, drivers, remote sessions, containers,
and audio servers can affect device availability. There is no published list of
tested OS versions or architectures yet.

## Rust

The workspace declares Rust 1.88 and edition 2024. Older toolchains are
unsupported.

## Coding agents

Built-in adapters exist for Codex CLI, Claude Code, Cursor CLI, Gemini CLI,
Aider, Amp, and OpenCode. They depend on upstream arguments and output shapes that may
change without TermVox. The repository does not pin or certify agent CLI
versions. Aider uses plain text and does not support session resume in the
current adapter. See [Coding agents](agents.md).

**Recommended for first try:** Cursor CLI (`agent`) or OpenCode — best tested
with `termvox shell`. See [demo happy path](demo.md).

## Speech-to-text

Embedded Whisper is the free local default. It runs in-process on the CPU.
The **fast** profile uses `ggml-tiny.bin` (~74 MiB); **balanced/accurate** use
`ggml-base.bin` (~142 MiB). Install with `termvox models install default`.

OpenAI compatibility means an endpoint implementing the multipart transcription
shape used by the adapter. Alternative endpoints are not tested or endorsed.

Parakeet and Vosk use a generic sidecar command contract. TermVox does not
bundle or certify a sidecar or model for either engine. See
[Speech engines](speech-engines.md).

## Terminals and keys

Interactive push-to-talk requires press and release events from a focused
terminal. Supported configured keys are `SPACE`, `ENTER`, `TAB`, `F1` through
`F24`, and a single character. Chords such as `ALT+SPACE` are not supported in
companion mode; **`termvox shell`** supports **F8** and **Ctrl+Space** (Wayland).

Terminal emulators, multiplexers, and remote shells may not report release
events reliably.

## Known alpha limitations

- No stable configuration or plugin API guarantee
- Configured plugins can be inspected and tested but cannot drive `termvox start`
- Global hotkeys need `termvox daemon start` or compositor support on Wayland
- `termvox doctor` reports failed checks in output without always exiting nonzero
- Agent × platform matrix lacks maintainer-signed evidence (see release readiness)
- `termvox install-shim` is Unix-only today

An adapter existing in source is not evidence that an upstream version or
hardware path has passed manual release gates.
