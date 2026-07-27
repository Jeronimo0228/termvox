# CLI reference

```text
termvox [--config PATH] COMMAND
```

`--config PATH` selects a project-layer configuration file. The global
configuration is still loaded first.

## Commands

### `termvox init [--global] [--force]`

Writes the built-in defaults to `termvox.toml`, or to the OS configuration
directory with `--global`. Existing files are preserved unless `--force` is
explicitly supplied.

### `termvox setup [--global] [--force]`

In an interactive terminal, prompts for language, push-to-talk key, speech
engine, and agent, then writes configuration. With non-terminal stdin, it
writes defaults. Replacement rules match `init`.

### `termvox start [--toggle]`

Starts terminal-focused push-to-talk. Hold the configured key, release it to
transcribe and process the prompt, then confirm when asked. Use `q` or `Ctrl+C`
to exit. With `--toggle`, one key press starts recording and the next stops it.

### `termvox doctor [--json]`

Validates configuration, lists input devices, health-checks the selected speech
engine, probes all six agent CLIs, and reports hotkey capability. Failures are
printed as `[!!]`; unavailable optional agents are printed as `[--]`.

The command currently reports diagnostics but may still exit successfully when
individual checks fail. Read its output; do not use exit status alone as a
health signal. `--json` emits machine-readable diagnostics.

### `termvox plugins [list|inspect ID|test ID]`

- `list` (the default) shows all six built-in adapters and configured plugins.
- `inspect ID` starts an enabled plugin, initializes it, prints its manifest,
  and shuts it down.
- `test ID` also calls `probe` and prints its result.

The command does not discover or install plugins.

### `termvox config [path|show|validate]`

- `path`: print global and project paths
- `show` (the default): print the merged, non-secret TOML configuration
- `validate`: parse, merge, and check supported value and audio constraints

### `termvox test [--seconds N]`

Records for a fixed duration (default: three seconds), trims low-energy edges,
transcribes the result, and prints the transcript. It does not send anything to
a coding agent.

### `termvox record start|stop|toggle`

Provides an external-trigger workflow suitable for a user-created desktop or
window-manager binding:

- `start` creates a per-user marker and records until another process removes it
- `stop` removes that marker
- `toggle` starts or stops according to marker existence

The start process must remain running. The marker is coordination, not
authentication; another process running as the same OS user can manipulate it.

### `termvox models list`

Lists release-reviewed model artifacts from the bundled manifest, including the
default embedded Whisper base model and optional sidecar models.

### `termvox init [--global] [--force] [--preset PRESET]`

Creates a starter config. Presets: `cursor`, `codex`, `claude`, `gemini`, `rust-web`.

Example:

```bash
termvox init --preset cursor --force
```

### `termvox daemon start [--background]`

Unix-only background voice service with global hotkey (default `ALT+SPACE`). See
[Daemon mode](daemon.md).

### `termvox talk`

Toggle recording on a running daemon (IPC).

### `termvox bench [--runs N]`

Print JSON latency report (P50/P95) for the configured Whisper model.

### `termvox models install [ID]`

Downloads and verifies a reviewed model. `default` installs the multilingual
`whisper-tiny` artifact (about 74 MiB) into the configured Whisper model path.
Use `termvox models install accurate` for the larger `whisper-base` model (~142 MiB).

### `termvox models status [ID]`

Reports whether the selected artifact is installed and checksum-verified.

### `termvox models remove [ID]`

Removes the installed artifact from its configured destination.

### `termvox models download URL --sha256 HASH [--destination PATH]`

Downloads bytes, verifies the required 64-character SHA-256, and moves the
verified file into place. Without `--destination`, the configured Whisper model
path is used. TermVox does not supply a trusted URL or hash.

### `termvox update`

Prints version and release-check information. It does not fetch, verify,
install, or replace the executable. The canonical project release page is
<https://github.com/Jeronimo0228/termvox/releases>; releases may be absent.

### `termvox completions SHELL`

Writes completion definitions to stdout for a shell supported by Clap. Redirect
the output to the location required by your shell.

### `termvox manpage [--output PATH]`

Renders a manual page to stdout or writes it to `PATH`.

## Logging

Set `RUST_LOG` to change diagnostic verbosity:

```bash
RUST_LOG=info termvox doctor
RUST_LOG=debug termvox start
```

Logs can expose device names, executable errors, or agent diagnostics. Review
them before sharing publicly.
