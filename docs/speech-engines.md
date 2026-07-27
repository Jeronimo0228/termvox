# Speech engines

## Compatibility matrix

| Engine | Config value | Processing location | Status |
| --- | --- | --- | --- |
| Whisper.cpp | `whispercpp` | Local subprocess | Built-in adapter |
| OpenAI speech-to-text | `openai` | Configured HTTPS service | Built-in adapter |
| Parakeet | `parakeet` | Local sidecar subprocess | Generic sidecar adapter |
| Vosk | `vosk` | Local sidecar subprocess | Generic sidecar adapter |

## Whisper.cpp

The local adapter expects a `whisper-cli`-compatible executable and a GGML model
file. It writes the utterance as a temporary mono 16-bit WAV, invokes:

```text
whisper-cli -m MODEL -f INPUT.wav -otxt -of OUTPUT -np [-l LANGUAGE]
```

and deletes the temporary WAV and text output after completion or handled
cancellation. A crash or abrupt process termination can leave temporary files
in the operating system's temporary directory.

Configure it with:

```toml
speech_engine = "whispercpp"
language = "en"

[whisper]
executable = "whisper-cli"
model = "/absolute/path/to/ggml-base.bin"
```

TermVox does not bundle a model. Model size, language support, speed, accuracy,
and hardware acceleration depend on your Whisper.cpp build and chosen model.

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
current alpha manifest contains the portable Spanish
`vosk-model-small-es-0.42` archive with its independently calculated SHA-256.
TermVox does not list Whisper.cpp or Parakeet downloads until their exact
upstream artifact, license, size, and checksum have been reviewed.

## Choosing an engine

Use Whisper.cpp or a sidecar when local processing and offline operation are
priorities and you can manage binaries and models. Use OpenAI when a remote
service is acceptable. No choice changes the coding agent's own network or
data-handling behavior.
