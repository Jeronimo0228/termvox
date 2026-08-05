# Speech engines

TermVox turns microphone audio into text before it reaches a coding agent.
Choose an engine with `speech_engine` in `termvox.toml`. The default is local
**embedded Whisper** (no API key).

For quality tuning, profiles, VAD, and Spanish setup see
**[Performance and STT quality](performance.md)** and the
**[Spanish STT guide](es/stt.md)**.

## Compatibility matrix

| Engine | Config value | Processing location | Status |
| --- | --- | --- | --- |
| Embedded Whisper | `whisper` (`whispercpp` legacy alias) | In-process, local CPU | Default built-in adapter |
| OpenAI speech-to-text | `openai` | Configured HTTPS service | Built-in adapter |
| Parakeet | `parakeet` | Local sidecar subprocess | Generic sidecar adapter |
| Vosk | `vosk` | Local sidecar subprocess | Generic sidecar adapter |

## Embedded Whisper

TermVox embeds Whisper.cpp through `whisper-rs`. Inference runs on the local CPU
in a worker thread — audio never needs to leave the device and no API key is
required.

### Install models

```bash
termvox models install default      # ggml-tiny.bin  (~74 MiB)  — profile fast
termvox models install accurate     # ggml-base.bin  (~142 MiB) — balanced / accurate
termvox models list
termvox models status accurate
```

Models are stored under the TermVox data directory (Linux:
`~/.local/share/termvox/models/`), not in Git. Interactive commands may offer a
download when a model is missing; non-interactive runs do not download without
consent.

### Configure

```toml
speech_engine = "whisper"
language = "es"                         # always set your spoken language
performance_profile = "balanced"        # see performance.md

[whisper]
model = "/absolute/path/to/ggml-base.bin"
threads = 0                             # 0 = use available CPUs (capped by max_threads)
max_threads = 6
prewarm_on_start = true
optimize_for_latency = false
use_gpu = false
streaming = true
```

| Key | Meaning |
| --- | --- |
| `model` | Absolute path to a `ggml-*.bin` file |
| `threads` | Decoder threads; `0` = auto |
| `max_threads` | Cap when `threads = 0` |
| `prewarm_on_start` | Load model early (`termvox shell` still defers until first F8) |
| `optimize_for_latency` | Faster short utterances; slightly worse on long phrases |
| `use_gpu` | Baseline builds keep CPU for portable Linux/macOS/Windows |
| `streaming` | Show partial text in the mic bar while decoding |

Use absolute paths. Quoted `~` in TOML is not expanded by the parser.

### Quality tips

| Goal | Action |
| --- | --- |
| Better Spanish / longer prompts | `performance_profile = "balanced"` + `ggml-base.bin` |
| Max built-in quality | `performance_profile = "accurate"` + disable auto-stop silence |
| Wrong words / cutoffs | Tune `[audio]` VAD — see [performance.md](performance.md) |
| Product names mangled | Add `[pipeline].dictionary` replacements |

Full walkthrough: [Performance and STT quality](performance.md).

## OpenAI

The remote adapter uploads a WAV multipart body to the configured endpoint with
bearer auth and requests `verbose_json`:

```toml
speech_engine = "openai"
language = "es"

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
endpoint = "https://api.openai.com/v1/audio/transcriptions"
timeout_seconds = 180
max_response_bytes = 2097152
```

```bash
export OPENAI_API_KEY="sk-..."
termvox test --seconds 3
```

The API key is read from the named environment variable. TermVox does not save
it in configuration. Changing `openai.endpoint` sends audio and credentials to
that host — only use endpoints you trust. Review the provider’s pricing,
retention, and privacy terms before enabling this engine.

## Parakeet and Vosk sidecars

TermVox provides the same generic local sidecar contract for both engines. It
does not bundle an inference runtime, model, or official sidecar executable.
Defaults expect `termvox-parakeet` or `termvox-vosk` on `PATH`.

```toml
speech_engine = "parakeet" # or "vosk"

[parakeet]
executable = "termvox-parakeet"
model = "/absolute/path/to/parakeet-model"

[vosk]
executable = "termvox-vosk"
model = "/absolute/path/to/vosk-model"
```

The sidecar is invoked as:

```text
SIDECAR --input INPUT.wav --model MODEL --json [--language LANGUAGE]
```

It must exit successfully and write one JSON object to stdout with a string
`text` field and optional string `language`. TermVox writes a temporary WAV and
deletes it after handled completion, failure, or cancellation.

The reviewed model manifest (`termvox models list`) includes commit-pinned
Whisper tiny/base artifacts and the portable Spanish
`vosk-model-small-es-0.42` archive, with license, size, and SHA-256 metadata.

## Choosing an engine

| Need | Engine |
| --- | --- |
| Free, offline, private | `whisper` (default) |
| Higher cloud accuracy, accept upload | `openai` |
| Experimental local sidecars | `parakeet` / `vosk` (bring your own binary) |

No speech-engine choice changes the coding agent’s own network or data-handling
behavior.

## Related

- [Performance and STT quality](performance.md)
- [Configuration](configuration.md)
- [Guía STT (Español)](es/stt.md)
- [Troubleshooting](troubleshooting.md)
