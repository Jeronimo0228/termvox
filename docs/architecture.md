# Architecture

TermVox is a Cargo workspace whose dependencies point inward to
`termvox-core`. The core owns data types and traits, but knows nothing about
CPAL, OpenAI, whisper.cpp, or any coding-agent CLI.

```text
terminal trigger
  -> bounded audio capture
  -> resample + VAD
  -> SpeechEngine
  -> conservative PromptPipeline
  -> confirmation policy
  -> AgentAdapter
  -> normalized AgentEvent stream
```

## Crates

- `termvox-core`: configuration, contracts, events, pipeline, and risk signals.
- `termvox-audio`: CPAL input. The realtime callback only converts samples
  and performs `try_send` into a bounded channel.
- `termvox-speech`: Whisper.cpp, OpenAI, and local sidecar speech adapters.
- `termvox-agents`: six streaming subprocess adapters and JSONL normalization.
- `termvox-plugin-sdk`: versioned JSON-RPC protocol for isolated plugins.
- `termvox-hotkeys`: capability detection, optional native registration, and
  fallback guidance.
- `termvox-cli`: command parsing, terminal PTT, orchestration, and diagnostics.

Official adapters compile into the binary. Third-party plugins execute out of
process so a Rust compiler upgrade cannot break an in-process ABI. Process
separation prevents direct access to TermVox memory, but it is not a sandbox: a
plugin still has the OS user's filesystem and network privileges. Configured
plugins can be initialized, inspected, and probed; they are not yet selectable
as the interactive coding agent.

## Audio path

The CPAL callback converts the first channel of each frame to normalized `f32`
samples and uses a non-blocking send into a bounded channel. A collector caps
the recording by configured duration. On stop, TermVox applies a small low-pass
stage when downsampling, linearly resamples to the target rate, and performs
RMS-threshold edge trimming with configurable hangover in 20 ms frames. This is
energy-based trimming, not a speech classifier.

Native audio can be excluded at build time. Such a binary retains the API but
returns a runtime error for device listing and recording.

## Speech boundary

All four speech selections satisfy the `SpeechEngine` trait. Whisper.cpp is a
local child process receiving a temporary WAV path. Parakeet and Vosk use a
shared sidecar contract. The OpenAI adapter is a bounded HTTP multipart client.
Cancellation drops in-flight work, and child processes are configured to be
killed when their owning handle is dropped.

## Configuration precedence

Defaults are overlaid by global config, project config, and three supported
`TERMVOX_*` environment variables. CLI `--config` replaces the default
`termvox.toml` project path.
Credentials are referenced by environment-variable name and are never stored in
the TOML schema.

## Session model

Interactive startup creates a local session shared across utterances. If a
structured child emits a remote session or thread ID, the adapter records it
and constructs the supported resume argument for later requests. The external
record workflow uses a session per recording process.

Agent subprocesses are started directly, without shell interpolation. stdout
is parsed and emitted incrementally as `AgentEvent` values while stderr is
drained separately. Time, total-output, and per-frame bounds can terminate a
request.

## Stability

The workspace version is **0.1.0-alpha.16**. The configuration schema, plugin
protocol, event normalization, and CLI behavior should be treated as unstable
until the project declares otherwise.
