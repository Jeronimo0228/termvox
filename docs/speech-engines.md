# Speech engines

## Compatibility matrix

| Engine | Config value | Processing location | Status |
| --- | --- | --- | --- |
| Embedded Whisper | `whisper` (`whispercpp` legacy alias) | In-process, local CPU | Default built-in adapter |
| OpenAI speech-to-text | `openai` | Configured HTTPS service | Built-in adapter |
| Parakeet | `parakeet` | Local sidecar subprocess | Generic sidecar adapter |
| Vosk | `vosk` | Local sidecar subprocess | Generic sidecar adapter |

## Embedded Whisper

TermVox embeds Whisper.cpp through `whisper-rs`. Audio inference runs on the
local CPU in a blocking worker, so captured audio never needs to leave the
device and no API key is required. No temporary WAV or external `whisper-cli`
process is used.

Install and inspect the reviewed default model with:

```bash
termvox models install default
termvox models status default
```

Configure it with:

```toml
speech_engine = "whisper"
language = "en"

[whisper]
model = "/absolute/path/to/ggml-base.bin" # optional override
threads = 0
```

The default multilingual `ggml-base.bin` is about 142 MiB and is stored in the
TermVox data directory, not Git or the executable. `threads = 0` selects
available CPU parallelism. The baseline build intentionally disables GPU
acceleration for consistent Linux, macOS, and Windows behavior.

## OpenAI

The remote adapter sends a WAV multipart upload to the configured endpoint,
using bearer authentication and requesting `verbose_json`:

```toml
speech_engine = "openai"
language = "en"

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
endpoint = "https://api.openai.com/v1/audio/transcriptions"
timeout_seconds = 180
max_response_bytes = 2097152
```

The API key is read from the named environment variable. TermVox does not save
it in configuration, but your shell, process manager, or OS may expose or log
environment variables. Before use, review the service's current availability,
pricing, retention, and privacy terms.

Because the endpoint is configurable, setting a non-OpenAI URL sends audio and
credentials to that host. Only use endpoints you trust.

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

“Generic sidecar adapter” is narrower than native engine support: users must
provide or implement a compatible executable, and this repository does not
claim compatibility with a particular Parakeet or Vosk distribution.

Run `termvox models list` to inspect release-reviewed model artifacts. The
manifest includes commit-pinned multilingual Whisper base and tiny models plus
the portable Spanish `vosk-model-small-es-0.42` archive, with license, size, and
SHA-256 metadata.

## Choosing an engine

Use embedded Whisper for the free local default. Use OpenAI only when a remote
service is explicitly acceptable. No speech-engine choice changes the coding
agent's own network or data-handling behavior.
