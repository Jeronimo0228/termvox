# Performance

TermVox defaults to the **`fast`** profile so voice commands stay responsive on
modest hardware without loading large models into RAM.

## Profiles

| Profile | Model | RAM (approx.) | Best for |
| --- | --- | --- | --- |
| `fast` (default) | `ggml-tiny.bin` (~74 MiB) | Lowest | Short voice commands, daily use |
| `balanced` | `ggml-base.bin` (~142 MiB) | Medium | Better accuracy on longer phrases |
| `accurate` | `ggml-base.bin` (~142 MiB) | Higher | Noisy rooms, rare terms |
| `custom` | Your choice | Varies | Manual tuning |

```toml
performance_profile = "fast"   # default

# Or opt into quality mode:
# performance_profile = "accurate"
```

Install the matching model:

```bash
termvox models install default    # whisper-tiny (~74 MiB)
termvox models install accurate   # whisper-base (~142 MiB)
```

## What `fast` changes

- **`whisper-tiny`** instead of base (~50% smaller on disk and in RAM)
- **`audio.max_seconds = 30`** — bounded capture buffer
- **`audio.auto_stop_on_silence = true`** — live VAD ends recording early
- **`whisper.prewarm_on_start = false`** — lazy model load (lower idle RAM)
- **`whisper.optimize_for_latency = true`** — short-utterance Whisper flags
- **`whisper.max_threads = 4`** — caps decoder threads

Override any field explicitly and set `performance_profile = "custom"` to keep
your values.

## Tuning keys

```toml
[audio]
max_seconds = 30
vad_silence_ms = 400
auto_stop_on_silence = true

[whisper]
model = "/path/to/ggml-tiny.bin"
threads = 0
max_threads = 4
prewarm_on_start = false
optimize_for_latency = true
```

Run `termvox doctor` to see the active profile, model path, and agent display
mode.
