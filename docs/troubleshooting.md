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

## Whisper executable or model not found

Run `whisper-cli --help` in the same shell. Use an absolute model path; quoted
TOML values beginning with `~` are not reliably shell-expanded. Confirm the
model format matches your Whisper.cpp build.

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
```

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

## `termvox update` does not update

That command is intentionally informational. Build and install the desired
source revision manually. The canonical repository is
<https://github.com/Jeronimo0228/termvox>.

When requesting help, include the TermVox revision, OS, Rust version, relevant
configuration with secrets removed, command output, and exact reproduction
steps. See the
[support policy](https://github.com/Jeronimo0228/termvox/blob/main/SUPPORT.md).
