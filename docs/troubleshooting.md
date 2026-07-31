# Troubleshooting

Start with:

```bash
termvox config path
termvox config validate
termvox doctor
```

Read every doctor line; individual failed checks do not necessarily produce a
nonzero command exit status.

## Build cannot find ALSA

On Debian/Ubuntu install `libasound2-dev` and `pkg-config`. On Fedora/RHEL
install `alsa-lib-devel` and `pkgconf-pkg-config`, then rebuild.

## No input device or permission denied

- Confirm the OS sees the microphone and another application can record.
- Grant microphone access to the terminal application.
- Close applications holding exclusive access to the device.
- Remove `audio.device` to use the OS default, or copy an exact device name
  reported by `termvox doctor`.
- Audio often is unavailable inside containers and remote SSH sessions unless
  explicitly forwarded.

## Push-to-talk does not release

Use a local, focused terminal and try `ENTER`, `TAB`, an `F` key, or a single
character. Some terminal emulators and multiplexers do not deliver key-release
events. Key chords such as `ALT+SPACE` are unsupported.

## “audio contains no voice”

Lower `audio.vad_threshold_db` (make it more negative), move closer to the
microphone, or verify the selected input. `vad_silence_ms` retains audio around
detected speech but does not make quiet audio count as voiced.

## Whisper model not found

Install the reviewed default model:

```bash
termvox models install default
termvox models status default
```

Use an absolute model path in `termvox.toml`; quoted TOML values beginning
with `~` are not reliably shell-expanded. In non-interactive use, TermVox does
not download the model without explicit consent; run the install command first.

## OpenAI authentication or HTTP errors

Confirm the variable named by `openai.api_key_env` is exported in the same
environment as TermVox. Verify `openai.endpoint` carefully: the configured host
receives both audio and bearer credentials. Provider model availability,
quotas, and billing are outside TermVox.

## Agent is “not installed”

Run the expected executable directly:

```bash
codex --version
claude --version
agent --version
gemini --version
aider --version
amp --version
opencode --version
```

## Agent is not authenticated

Run `termvox doctor` and look for `[!!]` lines under `agent/...`. TermVox
suggests the upstream login command when it can detect missing credentials.

Common fixes:

```bash
opencode auth login
claude login
codex login
gemini auth login
export OPENAI_API_KEY=...   # Aider and some OpenCode providers
```

`termvox shell` and `termvox start` (except companion mode) fail fast with the
same message instead of launching a broken agent session.

TermVox currently has no configuration key for a custom built-in agent
executable path. Ensure the command is on `PATH`. If an installed agent starts
failing after an upgrade, its structured-output flags may have changed.

## Parakeet or Vosk sidecar is not ready

TermVox supplies a command contract, not the engine executable or model.
Confirm `termvox-parakeet` or `termvox-vosk` is on `PATH`, the configured model
exists, and the sidecar accepts `--input`, `--model`, `--json`, and optional
`--language`.

## Plugin inspect or test fails

Use an absolute executable path, enable the plugin, and check its stderr.
Plugin stdout must contain only newline-delimited JSON-RPC. The child receives
only environment variables in `env_allowlist`; add required variable names
explicitly without putting secret values in TOML.

## Integrated shell (`termvox shell`)

- **Mic bar disappears** — upstream TUIs redraw the screen; TermVox redraws the bar
  continuously. If it still vanishes, update to the latest release (currently alpha.10).
- **F8 does nothing on Wayland** — use **Ctrl+Space** (`[shell].alt_hotkeys`) or
  configure a global hotkey via the daemon.
- **Cannot exit shell** — use **Ctrl+\\** (`[shell].exit_hotkey`), not Ctrl+C.
- **Cursor trust dialog** — use `termvox shell` (auto-trusts) or set
  `agents.cursor.trust_workspace = true` for branded mode.
- **Session not resumed** — check `.termvox/session.json`; run with `--fresh` to
  debug; ensure `discover_session = true` for agent-local lookup.

## `termvox update` does not update

That command is intentionally informational. Download a newer release from
<https://github.com/Jeronimo0228/termvox/releases> or rebuild from source.

When requesting help, include the TermVox revision, OS, Rust version, relevant
configuration with secrets removed, command output, and exact reproduction
steps. See the
[support policy](https://github.com/Jeronimo0228/termvox/blob/main/SUPPORT.md).
