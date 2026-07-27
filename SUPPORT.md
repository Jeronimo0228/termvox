# Support

TermVox is maintained by a small open-source project. Support is community-led,
best effort, and has no guaranteed response time.

## Before asking

1. Read the [quick start](docs/quick-start.md),
   [compatibility notes](docs/compatibility.md), and
   [troubleshooting guide](docs/troubleshooting.md).
2. Run:

   ```bash
   termvox config validate
   termvox doctor
   ```

3. Search issues in
   <https://github.com/Jeronimo0228/termvox/issues>.
4. Reproduce with the latest default-branch source when practical.

## Where to ask

Use a GitHub issue for reproducible bugs, documentation problems, and focused
feature proposals:

<https://github.com/Jeronimo0228/termvox/issues/new>

Include:

- TermVox version or commit
- OS, architecture, terminal, and Rust version
- Agent CLI and speech engine, including their versions
- Relevant sanitized configuration
- Exact commands, expected behavior, actual behavior, and full error text
- Minimal reproduction and whether it is consistent

Text is preferred over screenshots because it is searchable and accessible.

## Keep reports safe

Before posting, remove API keys, tokens, user names, private paths, recordings,
transcripts, prompts, repository content, and agent output that contains
secrets. Debug logs may contain device names and subprocess errors.

Security vulnerabilities and accidental secret exposure do not belong in
public support channels. Follow [SECURITY.md](SECURITY.md).
Conduct reports follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Boundaries

The project cannot provide:

- Guaranteed response or resolution times
- Private consulting or environment administration
- Support for unofficial binaries, modified forks, or untrusted plugins
- Provider billing, quota, account, or data-retention support
- Support for agent actions outside TermVox's transport boundary
- Recovery of lost data or credentials

Questions about Codex, Claude, Cursor, OpenAI, Whisper.cpp, operating systems,
or hardware may need to be reported to the relevant upstream project after a
TermVox-specific cause is ruled out.
