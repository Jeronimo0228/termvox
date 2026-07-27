# Security model

The maintained security and privacy guide is
[Privacy and security](privacy-security.md). The repository's vulnerability
reporting and disclosure policy is the
[security policy](https://github.com/Jeronimo0228/termvox/blob/main/SECURITY.md).

In short:

- TermVox is a transport and confirmation boundary, not an agent sandbox.
- Local Whisper processing uses temporary files and trusted native components.
- Remote transcription uploads audio to the configured endpoint.
- Agent CLIs retain their own permissions, credentials, and data policies.
- Risk phrase matching is only a warning heuristic.
- External plugins and trigger processes run with the current OS user's
  authority.
