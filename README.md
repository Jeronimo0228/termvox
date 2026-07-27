# TermVox

TermVox is a voice interface for terminal coding agents. It records an
utterance, transcribes it, shows the resulting prompt, and sends it to a
selected agent only after the configured confirmation policy allows it.

> **Project status:** TermVox is pre-release software. The source currently
> includes adapters for Codex, Claude, Cursor, Gemini, Aider, and Amp; direct
> Whisper.cpp and OpenAI transcription; and sidecar contracts for Parakeet and
> Vosk. Availability still depends on separately installed upstream tools and
> compatible versions. No release artifacts or signatures are claimed to be
> available.

## Quick start

The only verified installation path in this repository is a source build.
Install [Rust 1.86 or later](https://www.rust-lang.org/tools/install), the
platform audio prerequisites, and one supported coding-agent CLI.

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox

# Fedora/RHEL: sudo dnf install alsa-lib-devel
# Debian/Ubuntu: sudo apt-get install libasound2-dev pkg-config
cargo install --path crates/termvox-cli

termvox init
termvox doctor
termvox start
```

For local transcription, install `whisper-cli`, obtain a compatible GGML model
from a source you trust, and set `[whisper].model` in `termvox.toml`. For remote
transcription, set `speech_engine = "openai"` and export `OPENAI_API_KEY`.

While `termvox start` is focused, hold `Space` to record, release it to
transcribe, review the prompt, and answer the confirmation question. Press `q`
or `Ctrl+C` to exit.

## Compatibility at a glance

| Integration | Status in the current source |
| --- | --- |
| Codex, Claude, Cursor, Gemini, Aider, Amp | Built-in CLI adapters |
| Whisper.cpp, OpenAI speech-to-text | Built-in speech adapters |
| Parakeet, Vosk | Generic local sidecar adapters |
| Third-party JSON-RPC plugins | Explicit configuration plus inspect/test lifecycle |

An adapter in source does not guarantee every upstream version works. Run
`termvox doctor` after upgrading a tool, and see the
[compatibility notes](docs/compatibility.md).

## Documentation

- [Quick start](docs/quick-start.md) · [Inicio rápido en español](docs/es/quick-start.md)
- [Installation](docs/installation.md)
- [Coding agents](docs/agents.md) · [Speech engines](docs/speech-engines.md)
- [Configuration](docs/configuration.md) · [CLI reference](docs/cli-reference.md)
- [Privacy and security](docs/privacy-security.md)
- [Architecture](docs/architecture.md) · [Plugin protocol](docs/plugin-system.md)
- [Troubleshooting](docs/troubleshooting.md) · [Compatibility](docs/compatibility.md) · [FAQ](docs/faq.md)
- [Roadmap](docs/roadmap.md) · [mdBook navigation](docs/SUMMARY.md)

## Community

Read [CONTRIBUTING.md](CONTRIBUTING.md) before proposing a change. Project
governance, support boundaries, security reporting, maintainers, release
process, and changes are documented in the corresponding repository files.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option.
