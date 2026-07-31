# TermVox (npm)

Install the full TermVox CLI — voice for Codex, Claude, Cursor, Gemini, Aider,
Amp, and OpenCode — with one command:

```bash
npm install -g termvox
```

This package downloads the native binary for your OS from GitHub Releases during
`postinstall`, bootstraps the default Whisper model, and writes a starter
`termvox.toml`.

## Quick start

```bash
npm install -g termvox
termvox doctor
termvox shell --agent cursor
termvox-editor-install   # optional Cursor/VS Code extension
```

Inside `termvox shell`, use **F8** or **Ctrl+Space** (Wayland) for voice.
Exit the wrapper with **Ctrl+\\**.

## All CLI commands

Every feature of the Rust CLI is available through the `termvox` binary:

| Command | Purpose |
| --- | --- |
| `termvox shell` | Integrated agent TUI + mic bar |
| `termvox start` | Branded/companion/shell session |
| `termvox daemon start` | Background voice + global hotkey |
| `termvox talk` | Toggle recording on a running daemon |
| `termvox doctor` | Health checks and hints |
| `termvox test` | Microphone + transcription smoke test |
| `termvox models install` | Download Whisper models |
| `termvox init` / `setup` | Create configuration |
| `termvox plugins` | Inspect JSON-RPC plugins |

Run `termvox --help` for the full list.

## Environment variables

| Variable | Effect |
| --- | --- |
| `TERMVOX_SKIP_BINARY_INSTALL=1` | Skip downloading the native binary (packaging/CI) |
| `TERMVOX_SKIP_BOOTSTRAP=1` | Skip model download and `termvox init` |
| `TERMVOX_NPM_PRESET=cursor` | Preset for first-time `termvox init` |
| `TERMVOX_INSTALL_REPO=owner/repo` | Override GitHub release source |

## Requirements

- Node.js 18+
- macOS (Intel/Apple Silicon), Linux (x64/arm64), or Windows x64
- Microphone access
- At least one supported coding-agent CLI on `PATH`

## Docs

https://github.com/Jeronimo0228/termvox/tree/main/docs

## License

MIT
