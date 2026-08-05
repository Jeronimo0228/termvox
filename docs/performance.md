# Performance and STT quality

TermVox defaults to the **`fast`** profile (`ggml-tiny`) so voice stays
responsive on modest hardware. If transcription feels inaccurate — especially
in Spanish or with longer prompts — switch to **`balanced`** or **`accurate`**
and install the base Whisper model.

## Quick upgrade (recommended)

```bash
# 1. Install the larger model (~142 MiB) if needed
termvox models install accurate
termvox models status accurate

# 2. Edit config (see paths below)
termvox config path
```

In your **global** or **project** `termvox.toml`:

```toml
performance_profile = "balanced"
language = "es"   # or "en" — always set your spoken language

[whisper]
model = "/home/YOU/.local/share/termvox/models/ggml-base.bin"
optimize_for_latency = false
prewarm_on_start = true
max_threads = 6

[audio]
max_seconds = 60
vad_silence_ms = 700
vad_threshold_db = -42.0
auto_stop_on_silence = true
```

Replace `YOU` with your username, or run `termvox models status accurate` and
copy the absolute path it prints.

Validate and check:

```bash
termvox config validate
termvox doctor          # must show ggml-base.bin + balanced/accurate
termvox test --seconds 4
```

Speak a short coding sentence during the test. You should see a meaningful
transcript, not only `[Música]` / silence placeholders.

## Where to put the config

TermVox merges layers (later wins):

1. Built-in defaults  
2. **Global** — `termvox config path` → `global:`  
   Linux: `~/.config/termvox/termvox.toml`  
3. **Project** — `./termvox.toml` in the current directory  
4. Environment variables (`TERMVOX_*`)

```bash
termvox init --global          # create global file
# or
termvox init                   # create ./termvox.toml in a project
termvox config path            # show both paths
termvox config show            # show merged effective config
```

Use **global** for STT quality defaults on every project. Use a project file
only when one repo needs different settings.

> If both files exist, an explicit `whisper.model = "...tiny..."` in the project
> file will override a global `balanced` profile. Keep paths consistent.

## Profiles

| Profile | Model | Latency | Accuracy | Best for |
| --- | --- | --- | --- | --- |
| `fast` (default) | `ggml-tiny.bin` (~74 MiB) | Lowest | Lowest | Short English commands, low RAM |
| `balanced` | `ggml-base.bin` (~142 MiB) | Medium | Good | Daily use, Spanish, longer phrases |
| `accurate` | `ggml-base.bin` (~142 MiB) | Higher | Best of built-in | Noisy rooms, technical terms |
| `custom` | Your choice | Manual | Manual | Full control (profile does not auto-tune) |

```toml
performance_profile = "balanced"
```

### What each profile sets

| Setting | `fast` | `balanced` | `accurate` |
| --- | --- | --- | --- |
| Whisper model | tiny | base | base |
| `audio.max_seconds` | 30 | 60 | 120 |
| `audio.vad_silence_ms` | 400 | 600 | (unchanged default) |
| `audio.auto_stop_on_silence` | true | true | **false** |
| `whisper.prewarm_on_start` | false | true | true |
| `whisper.optimize_for_latency` | true | true | **false** |
| `whisper.max_threads` | 4 | 6 | 0 (all CPUs) |

Profile auto-tuning only fills fields that still match built-in defaults.
If you hard-coded `whisper.model` to `ggml-tiny.bin`, change that path too
(or set `performance_profile = "custom"` and manage everything yourself).

Install matching models:

```bash
termvox models install default     # tiny — for fast
termvox models install accurate    # base — for balanced / accurate
termvox models list
termvox models status default
termvox models status accurate
```

## Language

Always set the language you speak. Wrong language hurts accuracy a lot:

```toml
language = "es"   # Spanish
# language = "en"
```

This value is passed to Whisper (and other engines) for every transcription.

## Audio / VAD tuning

```toml
[audio]
# device = "Exact name from termvox doctor"   # omit = OS default
sample_rate = 16000          # Whisper expects 16 kHz; keep this
max_seconds = 60             # hard cap on one utterance
vad_threshold_db = -42.0     # less negative = needs louder speech
vad_silence_ms = 700         # silence before auto-stop (when enabled)
auto_stop_on_silence = true  # false = you stop with F8 / hotkey only
```

| Symptom | Try |
| --- | --- |
| Cuts off mid-sentence | Raise `vad_silence_ms` (700–1000) or set `auto_stop_on_silence = false` |
| Captures room noise / music | Raise `vad_threshold_db` (e.g. `-40`) or move closer to the mic |
| “audio contains no voice” | Lower `vad_threshold_db` (e.g. `-50`), check mic device |
| First words missing | Speak a beat after pressing F8; raise `vad_silence_ms` |
| Long prompts truncated | Raise `max_seconds` or use `accurate` |

Pick the input device with `termvox doctor` (microphone section) and copy the
exact name into `audio.device` if the default is wrong.

## Whisper keys

```toml
[whisper]
model = "/absolute/path/to/ggml-base.bin"
threads = 0                  # 0 = auto from available CPUs
max_threads = 6              # cap when threads = 0
prewarm_on_start = true      # load model early (shell defers until first F8)
optimize_for_latency = false # true = faster, slightly worse on long phrases
use_gpu = false              # baseline builds keep CPU for portability
streaming = true             # partials in the mic bar while decoding
```

Use **absolute paths** for `model`. Quoted `~` in TOML is not expanded.

### “AMX is not ready to be used!”

Harmless ggml/CPU notice on some Linux CPUs. TermVox **alpha.14+** suppresses it
so it does not corrupt agent TUIs. Voice still works without AMX.

## Dictionary (post-STT corrections)

Literal replacements after transcription — useful for product names and acronyms:

```toml
[pipeline]
dictionary = { "api rest" = "REST API", "jay double u t" = "JWT", "open code" = "OpenCode" }
# prefix = "Context: this is a Rust CLI project."
# suffix = "Ask before changing dependencies."
```

## Cloud STT (optional)

If local Whisper is still not enough and you accept uploading audio:

```toml
speech_engine = "openai"
language = "es"

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
```

```bash
export OPENAI_API_KEY="sk-..."
termvox test --seconds 3
```

Never put the API key inside `termvox.toml`. See [Speech engines](speech-engines.md).

## Verify end-to-end

```bash
termvox config show | head -40
termvox doctor
termvox test --seconds 4
termvox shell --agent opencode --fresh   # or cursor
```

In the shell: **F8** (or **Ctrl+Space** on Wayland) → speak → release → confirm.

## Related

- [Speech engines](speech-engines.md) — Whisper, OpenAI, Parakeet, Vosk  
- [Configuration](configuration.md) — full key reference  
- [Troubleshooting](troubleshooting.md) — mic, models, VAD failures  
- [Guía STT en español](es/stt.md)
