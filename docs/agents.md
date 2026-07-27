# Coding agents

TermVox launches coding-agent CLIs as child processes. It passes arguments
directly, without invoking a shell, and parses newline-delimited structured
output where available.

## Compatibility matrix

| Agent | Config value | Expected executable | Status |
| --- | --- | --- | --- |
| Codex CLI | `codex` | `codex` | Built-in adapter |
| Claude Code | `claude` | `claude` | Built-in adapter |
| Cursor CLI | `cursor` | `agent` | Built-in adapter |
| Gemini CLI | `gemini` | `gemini` | Built-in adapter |
| Aider | `aider` | `aider` | Built-in text adapter |
| Amp | `amp` | `amp` | Built-in adapter |

“Built-in adapter” means invocation and output parsing exist in source. It is
not a guarantee of compatibility with every upstream CLI version. These tools
evolve independently, and TermVox has not published a version-by-version test
matrix.

## Selecting an agent

Set the top-level `agent` key:

```toml
agent = "codex" # claude, cursor, gemini, aider, or amp
```

For a one-off environment override:

```bash
TERMVOX_AGENT=cursor termvox doctor
```

`termvox doctor` probes all six current executables with `--version`.
`termvox plugins` prints the same adapter availability at a glance.

## Current invocation behavior

- **Codex CLI:** `codex exec [resume ID] --json PROMPT`
- **Claude Code:** `claude -p PROMPT --output-format stream-json --verbose`
  with `--resume ID` when a remote session ID is known
- **Cursor CLI:** `agent -p PROMPT --output-format stream-json` with
  `--resume ID` when available
- **Gemini CLI:** `gemini -p PROMPT --output-format stream-json` with
  `--resume ID` when available
- **Aider:** `aider --message PROMPT`; plain output is emitted as message events
- **Amp:** `amp -x PROMPT --stream-json` with `--resume ID` when available

Cursor is currently invoked without an explicit TermVox sandbox flag; its
effective policy comes from the installed Cursor CLI and environment.

These details may change as upstream tools change. TermVox does not add
`--force`, `--yolo`, or automatic approval flags. The `permission_profile`
setting is carried through the request, but non-safe values are not translated
to undocumented agent flags. The invoked agent still controls authentication,
sandboxing, permissions, tool policy, and side effects.

`termvox start` creates one local session and reuses it across utterances until
exit. Structured adapters can capture a remote session ID and add their
documented resume argument to later requests. The external `termvox record`
workflow creates a session for each recording process.

## Compatibility caveats

- Aider has no structured-output or resume capability in the current adapter.
- The other adapters use a generic forward-compatible JSONL parser; upstream
  event shapes can still lead to missing or misclassified output.
- Agent execution is bounded by runtime timeout, total output, and per-frame
  limits from configuration.
- Built-in executable paths are fixed command names and are not configurable.

An out-of-process [plugin protocol](plugin-system.md) is also under
development. Configured plugins can be listed, inspected, and conformance
tested, but they are not selectable as the `termvox start` coding agent.
