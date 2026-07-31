# Quick start

TermVox **0.1.0-alpha.10** is available on [npm](https://www.npmjs.com/package/termvox)
and as pre-built binaries for Linux, macOS, and Windows on
[GitHub Releases](https://github.com/Jeronimo0228/termvox/releases). You can also
build from source.

## 1. Install

**Recommended (npm):**

```bash
npm install -g termvox
termvox doctor
termvox-editor-install   # optional Cursor/VS Code mic status bar
```

**Shell installer:**

```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

Pin a version:

```bash
TERMVOX_VERSION=v0.1.0-alpha.10 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

**From source:**

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
cargo install --path crates/termvox-cli
```

Install prerequisites: Rust 1.88+, microphone libraries (see [installation](installation.md)),
and at least one [supported agent CLI](agents.md):

| Agent | Executable |
| --- | --- |
| Codex | `codex` |
| Claude Code | `claude` |
| Cursor | `agent` |
| Gemini | `gemini` |
| Aider | `aider` |
| Amp | `amp` |
| OpenCode | `opencode` |

Local speech-to-text is embedded (Whisper). No API key is required for the default path.

## 2. Initialize

```bash
termvox init --preset cursor    # or opencode, claude, codex, gemini, rust-web
termvox models install default
termvox config validate
termvox doctor
```

`termvox init` writes `termvox.toml`. Use `--force` to replace an existing file.
`termvox setup` provides interactive prompts when stdin is a terminal.

## 3. Recommended workflow — integrated shell

For Cursor, OpenCode, Claude Code, Codex, and other TUIs, use the integrated mic bar:

```bash
cd your-project
termvox shell
```

Or pick an agent for one session:

```bash
termvox shell --agent opencode
termvox shell --agent cursor -- --model gpt-5
```

| Action | Key |
| --- | --- |
| Toggle voice | **F8** or **Ctrl+Space** (Wayland fallback) |
| Leave TermVox wrapper | **Ctrl+\\** |
| Send Ctrl+C to agent | **Ctrl+C** (does not exit TermVox) |

TermVox saves the upstream chat id in `.termvox/session.json` and resumes it the
next time you open the same project. Use `termvox shell --fresh` to start clean.

See [Agent shell](agent-shell.md) for display modes, shims, and agent-specific notes.

## 4. Alternative — branded / companion mode

Set `display = "branded"` (default for Codex/Claude/Gemini) or `display = "companion"`
(two-window paste) under `[agents.<name>]` in `termvox.toml`, then:

```bash
termvox start
```

Hold **Space** (default push-to-talk), speak, release. Confirm with `y` when prompted.
Press `q` or **Ctrl+C** to exit.

When `display = "shell"`, `termvox start` delegates to `termvox shell` automatically.

## 5. Quick microphone test

```bash
termvox test --seconds 3
```

Records, transcribes, and prints text without contacting an agent.

## 6. Speech engine (optional)

Default (local, free):

```toml
speech_engine = "whisper"

[whisper]
model = "/absolute/path/to/ggml-tiny.bin"   # optional override
threads = 0
```

For OpenAI transcription:

```toml
speech_engine = "openai"

[openai]
api_key_env = "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="your-key"
```

Next: [configuration](configuration.md), [CLI reference](cli-reference.md),
[agents](agents.md), [troubleshooting](troubleshooting.md).
