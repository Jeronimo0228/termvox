# Local validation report — 2026-07-27

Environment: Fedora Linux x86_64, Rust stable, TermVox `0.1.0-alpha.1`.

## Passed

- Formatting, strict Clippy, all workspace tests, rustdoc, and package dry-run.
- Native Linux build and tests with ALSA/PipeWire.
- `cargo audit`: 305 locked dependencies scanned; no RustSec vulnerability.
- `cargo deny check`: advisories, bans, licenses, and sources passed. Duplicate
  transitive versions remain warnings.
- mdBook generated the HTML documentation successfully.
- Native doctor detected PipeWire/ALSA and physical input devices.
- A one-second microphone test captured 0.98 seconds of voiced audio.
- CLI smoke tests covered JSON doctor output, model listing, completions, and
  manpage generation.
- A CycloneDX 1.5 SBOM containing 304 components was generated and parsed.
- The official `vosk-model-small-es-0.42` archive was downloaded from the Vosk
  catalog and independently hashed as
  `09b239888f633ef2f0b4e09736e3d9936acfd810bc65d53fad45261762c6511f`
  (39,817,833 bytes, Apache-2.0).

## Attempted but not passed

- Claude Code `1.0.33` and Cursor Agent `2026.07.23-e383d2b` were detected.
  Safe no-tool prompt probes produced no output and timed out after 90 seconds,
  so runtime compatibility is not claimed.
- Embedded Whisper inference on Linux with the verified `whisper-base` model
  installed via `termvox models install default`.
- `termvox test --seconds 3` transcribed captured audio locally without
  `whisper-cli` or an API key.

## Unavailable in this environment

- Codex, Gemini CLI, Aider, and Amp executables.
- An OpenAI API credential.
- Whisper.cpp, Parakeet, and Vosk sidecar runtimes.
- macOS and Windows hardware.
- GitHub OIDC, required for genuine keyless Sigstore signing and provenance.

These unavailable or failed routes remain release gates; source-level adapters
and unit fixtures are not substitutes for real integration evidence.
