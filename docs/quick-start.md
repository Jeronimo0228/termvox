# Quick start

This guide uses the current source tree. TermVox does not yet promise published
packages or prebuilt binaries.

## 1. Install prerequisites

Install Rust 1.88 or later, microphone development libraries, and one supported
agent:

- `codex` for Codex CLI
- `claude` for Claude Code
- `agent` for Cursor CLI
- `gemini` for Gemini CLI
- `aider` for Aider
- `amp` for Amp
- `opencode` for OpenCode

Local speech-to-text is embedded. It does not require `whisper-cli`, an API key,
or a paid speech service.

## 2. Build and initialize

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
cargo install --path crates/termvox-cli

termvox init
termvox models install default
termvox config validate
termvox doctor
```

`termvox init` writes defaults to `termvox.toml`. `termvox setup` provides
interactive choices when stdin is a terminal. Both refuse to replace an
existing file unless `--force` is passed.

## 3. Select transcription

The free local default is:

```toml
speech_engine = "whisper"

[whisper]
model = "/absolute/path/to/ggml-base.bin" # optional override
threads = 0 # use available CPU parallelism
```

`termvox models install default` downloads the reviewed multilingual base model
(about 142 MiB), verifies its SHA-256, and stores it in the TermVox data
directory. Interactive recording commands ask before downloading it when
missing; non-interactive commands print the explicit install command instead.

For OpenAI:

```toml
speech_engine = "openai"

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
endpoint = "https://api.openai.com/v1/audio/transcriptions"
```

```bash
export OPENAI_API_KEY="your-key"
```

OpenAI mode uploads each captured utterance to the configured endpoint.

## 4. Speak to the agent

```bash
termvox start
```

Hold the configured key (`Space` by default), speak, and release. TermVox prints
the transcript and processed prompt. The default policy asks before sending.
Answer `y` or `yes` to continue; any other answer cancels. Press `q` or
`Ctrl+C` to leave interactive mode.

Run a short end-to-end microphone and transcription test with:

```bash
termvox test --seconds 3
```

Next: [configuration](configuration.md), [agents](agents.md), and
[privacy and security](privacy-security.md).
