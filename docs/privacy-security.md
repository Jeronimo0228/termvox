# Privacy and security

TermVox is a transport and confirmation boundary. It is not a sandbox for
coding agents, speech engines, models, or plugins.

## Data flow

1. CPAL captures microphone samples in memory.
2. TermVox converts to mono, resamples, and trims low-energy edges.
3. Whisper.cpp, Parakeet, and Vosk modes write a temporary WAV for a local
   subprocess. OpenAI mode uploads a WAV to the configured HTTPS endpoint.
4. The transcript is transformed by explicit pipeline rules and printed.
5. After policy confirmation, the prompt is passed as a direct argument to the
   selected agent CLI in the current working directory.
6. Structured agent output is parsed and printed.

The current TermVox source has no telemetry client and does not intentionally
save audio, transcripts, prompt history, or API-key values. This does not mean
the system as a whole is ephemeral: terminals can retain scrollback; shells and
process monitors can expose arguments or environment variables; crashes can
leave temporary files; and speech providers or coding agents can log, transmit,
or retain data under their own policies.

## Local and remote transcription

Whisper.cpp and configured sidecars keep transcription local to their
executables, but those executables and models are outside TermVox's trust
boundary. Install them from trusted sources. Temporary WAV and output files are
removed after normal completion, handled failure, or handled cancellation;
deletion is best-effort and not secure erasure.

The OpenAI adapter sends the full captured utterance and bearer credential to
`openai.endpoint`. A custom endpoint receives both. Verify the URL and review
the provider's terms before speaking secrets.

## Agent permissions

TermVox invokes agents without a shell and does not add broad auto-approval
flags. It also does not translate permission profiles into undocumented
upstream flags. These controls reduce some accidental exposure but do not make
prompts safe. Each agent acts according to its own configuration, approvals,
credentials, and sandbox.

The prompt scanner recognizes a small set of strings such as `rm`, `sudo`,
`git push`, `docker prune`, and `terraform destroy`. A match forces
confirmation. This is a warning heuristic, not semantic analysis, and both
false positives and false negatives are expected.

## Safer operation

- Keep `confirmation = true` and `auto_send = false`.
- Use least-privilege agent credentials and review the agent's own tool policy.
- Start TermVox in the intended repository; that directory becomes the agent's
  working directory.
- Do not speak secrets into a remote engine.
- Pin and verify model downloads with a trusted SHA-256.
- Treat configuration changes, especially endpoints and plugin executable
  paths, as code changes.
- Review transcript and prompt text before sending.
- Keep TermVox, agents, Whisper.cpp, and the operating system patched.

## Plugins and external triggers

The plugin client executes only an explicitly configured file and uses
newline-delimited JSON-RPC over child-process standard I/O. It clears the child
environment except for explicitly allowlisted variables, sets a dedicated
working directory, and enforces call and frame limits. This is process
isolation, not a security sandbox; a plugin still has the normal filesystem and
network privileges of the TermVox OS user. The CLI can initialize, inspect, and
probe configured plugins, but does not select one as the interactive coding
agent.

External recording uses a marker in a per-user runtime or temporary directory.
Any process able to act as that OS user may create or remove it. Do not treat it
as an authorization mechanism.

To report a vulnerability privately, follow the
[security policy](https://github.com/Jeronimo0228/termvox/blob/main/SECURITY.md).
