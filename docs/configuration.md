# Configuration

TermVox merges configuration in this order, with later values winning:

1. Built-in defaults
2. Global `termvox/termvox.toml` under the OS configuration directory
3. `termvox.toml` in the current working directory, or the path passed to
   `--config`
4. Supported `TERMVOX_*` environment variables

Typical global paths are `$XDG_CONFIG_HOME/termvox/termvox.toml` on Linux,
`~/Library/Application Support/termvox/termvox.toml` on macOS, and the
equivalent roaming application-data directory on Windows. Run
`termvox config path` for the paths on your machine.

Generate a project file with `termvox init`, a global file with
`termvox init --global`, and validate the merged result with
`termvox config validate`.

## Complete example

```toml
performance_profile = "fast"
speech_engine = "whisper"
agent = "codex"
push_to_talk = "SPACE"
language = "en"
auto_send = false
confirmation = true
permission_profile = "safe"

[audio]
# Omit device to use the OS default input.
# device = "Exact device name reported by termvox doctor"
sample_rate = 16000
max_seconds = 30
vad_threshold_db = -45.0
vad_silence_ms = 400
auto_stop_on_silence = true

[whisper]
model = "/home/user/.local/share/termvox/models/ggml-tiny.bin"
threads = 0
max_threads = 4
prewarm_on_start = false
optimize_for_latency = true

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
endpoint = "https://api.openai.com/v1/audio/transcriptions"
timeout_seconds = 180
max_response_bytes = 2097152

[parakeet]
executable = "termvox-parakeet"
model = "/absolute/path/to/parakeet-model"

[vosk]
executable = "termvox-vosk"
model = "/absolute/path/to/vosk-model"

[runtime]
agent_timeout_seconds = 900
speech_timeout_seconds = 180
shutdown_timeout_seconds = 5
max_output_bytes = 8388608
max_json_frame_bytes = 1048576

[[plugins]]
id = "example"
executable = "/absolute/path/to/plugin"
args = []
enabled = false
env_allowlist = []
timeout_seconds = 30
max_frame_bytes = 1048576

[pipeline]
dictionary = { "api rest" = "REST API", "j w t" = "JWT" }
# prefix = "Project context"
# suffix = "Do not modify generated files."
```

The same maintained example is available at
[`examples/termvox.toml`](https://github.com/Jeronimo0228/termvox/blob/main/examples/termvox.toml).

## Top-level keys

| Key | Allowed/current behavior | Default |
| --- | --- | --- |
| `performance_profile` | `fast`, `balanced`, `accurate`, or `custom` | `fast` |
| `speech_engine` | `whisper` (`whispercpp` legacy alias), `openai`, `parakeet`, or `vosk` | `whisper` |
| `agent` | `codex`, `claude`, `cursor`, `gemini`, `aider`, or `amp` | `codex` |
| `push_to_talk` | `SPACE`, `ENTER`, `TAB`, `F1`–`F24`, or one character | `SPACE` |
| `language` | Passed to the speech engine | `es` |
| `auto_send` | When false, confirmation is required | `false` |
| `confirmation` | When true, confirmation is required | `true` |
| `permission_profile` | `safe`, `workspace-write`, or `custom` | `safe` |

A prompt matching a built-in risk signal always requires confirmation,
regardless of `auto_send` and `confirmation`.

## Audio

`audio.sample_rate` must be from 8,000 through 192,000 Hz.
`audio.max_seconds` must be positive and caps collected source samples.
`audio.vad_threshold_db` controls energy-based trimming; a less negative value
requires louder audio. `audio.device` must exactly match an input device name.

`audio.vad_silence_ms` controls trailing audio retained after the last voiced
20 ms frame. Up to 200 ms of the same window is also retained before the first
voiced frame. This setting trims captured audio; it does not stop recording.

## Speech providers

Embedded Whisper is the default. Install the reviewed model with
`termvox models install default` (~74 MiB tiny) or
`termvox models install accurate` (~142 MiB base). See
[Performance](performance.md).
tilde (`~`) expansion is performed by shells, not TOML parsers, so a quoted
path beginning with `~` may not work. `whisper.threads = 0` selects available
CPU parallelism.

`openai.api_key_env` is an environment-variable name, not the secret itself.
Changing `openai.endpoint` redirects both the audio upload and bearer
credential; treat this as a security-sensitive setting. `timeout_seconds` and
`max_response_bytes` bound the remote request and response.

Parakeet and Vosk use a generic sidecar contract. Their `executable` and
`model` values must point to separately supplied components. See
[Speech engines](speech-engines.md).

## Runtime limits and permissions

`runtime.agent_timeout_seconds` bounds each agent request.
`max_output_bytes` bounds accumulated stdout and captured stderr, and
`max_json_frame_bytes` bounds one JSONL line. Both byte limits must be positive.
`speech_timeout_seconds` and `shutdown_timeout_seconds` are present in the
schema but are not currently applied by CLI orchestration. OpenAI uses its
provider-specific timeout instead.

`permission_profile` communicates intent to adapters. Current built-in adapters
do not translate non-safe profiles into undocumented upstream flags, so this
setting does not grant or enforce agent permissions.

## Agent profiles

Each coding-agent CLI has its own quirks (auth flags, workspace trust, JSON
modes). TermVox keeps shared settings at the top level and agent-specific
options under `[agents.<name>]`, where `<name>` matches the `agent` value.

```toml
agent = "cursor"

[agents.cursor]
executable = "agent"          # optional override
trust_workspace = true          # Cursor only: pass -f for non-interactive trust
extra_args = []                 # inserted before the prompt argument

[agents.claude]
extra_args = []

[agents.codex]
extra_args = []
```

| Key | Applies to | Purpose |
| --- | --- | --- |
| `executable` | Any agent | Override the default CLI binary name or path |
| `extra_args` | Any agent | Documented upstream flags before the prompt |
| `trust_workspace` | Cursor only | Pass `-f` when the CLI requires workspace trust |
| `display` | Any agent | `branded`, `companion`, or `verbose` UI mode |
| `copy_to_clipboard` | Any agent | Auto-copy the prompt in companion mode (default: on for companion) |

Legacy `[cursor].trust_workspace` is still loaded and mapped to
`[agents.cursor].trust_workspace`. Run `termvox doctor` to see the active
profile, display mode, warnings, and hints. See [Coding agents](agents.md) for
per-agent notes and UI behavior.

## Plugins

Each `[[plugins]]` entry requires a non-empty `id` and executable path.
`enabled` controls whether inspect/test can select it. Plugins run in a
per-plugin data directory with a cleared environment; only names in
`env_allowlist` are copied from TermVox. Timeout and frame-size limits apply to
protocol calls. See [Plugin system](plugin-system.md).

## Prompt pipeline

TermVox collapses whitespace, applies literal dictionary replacements in key
order, then adds an optional prefix and suffix separated by blank lines. It
does not infer requirements or correct arbitrary content.

## Environment overrides

Only these overrides are implemented:

```bash
TERMVOX_AGENT=claude
TERMVOX_SPEECH_ENGINE=openai
TERMVOX_LANGUAGE=en
```

Other `TERMVOX_*` names are not currently read. `termvox config show` prints the
merged configuration and does not print the API-key value.
