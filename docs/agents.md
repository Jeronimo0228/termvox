# Coding agents

TermVox launches coding-agent CLIs as child processes. It passes arguments
directly, without invoking a shell, and parses newline-delimited structured
output where available.

## Compatibility matrix

| Agent | Config value | Default executable | Status |
| --- | --- | --- | --- |
| Codex CLI | `codex` | `codex` | Built-in adapter |
| Claude Code | `claude` | `claude` | Built-in adapter |
| Cursor CLI | `cursor` | `agent` | Built-in adapter |
| Gemini CLI | `gemini` | `gemini` | Built-in adapter |
| Aider | `aider` | `aider` | Built-in text adapter |
| Amp | `amp` | `amp` | Built-in adapter |
| OpenCode | `opencode` | `opencode` | Built-in adapter |

## Selecting an agent

Set the top-level `agent` key:

```toml
agent = "cursor"
```

Configure only the active agent's profile under `[agents.<name>]`:

```toml
[agents.cursor]
executable = "/home/you/.local/bin/agent"
trust_workspace = true
extra_args = []

[agents.claude]
# optional executable override or documented extra flags

[agents.codex]
extra_args = []
```

| Field | Applies to | Meaning |
| --- | --- | --- |
| `executable` | All agents | Override the default CLI command or path |
| `extra_args` | All agents | Documented upstream flags inserted before the prompt |
| `trust_workspace` | Cursor only | Pass `-f` for non-interactive workspace trust |
| `display` | Any agent | `branded` (default), `companion`, or `verbose` |
| `copy_to_clipboard` | Any agent | Legacy alias; use `delivery` instead |
| `delivery` | Companion mode | `clipboard`, `paste`, or `both` (Cursor default: `both`) |

### Display modes

| Mode | Behavior |
| --- | --- |
| `branded` | Compact footer styled like the upstream CLI; spawns the agent subprocess |
| `companion` | Voice layer only — transcribe, copy to clipboard, and paste into an external agent TUI (default for Cursor) |
| `verbose` | Legacy multi-line TermVox output |

```toml
[agents.cursor]
trust_workspace = true
display = "companion"   # paste into the Cursor Agent TUI in another pane

[agents.codex]
display = "branded"
```

TermVox never adds undocumented auto-approval flags such as `--yolo` or
`--dangerously`. Use `extra_args` only for flags published by the upstream CLI.

Legacy `[cursor].trust_workspace` is still loaded and mapped to
`[agents.cursor].trust_workspace`.

Run `termvox doctor` to see the selected agent, active profile values, warnings,
and agent-specific hints.

## Current invocation behavior

- **Codex CLI:** `codex exec [resume ID] --json PROMPT`
- **Claude Code:** `claude -p PROMPT --output-format stream-json --verbose`
- **Cursor CLI:** `agent [extra_args...] -p PROMPT --output-format stream-json [-f]`
- **Gemini CLI:** `gemini -p PROMPT --output-format stream-json`
- **Aider:** `aider --message PROMPT`
- **Amp:** `amp -x PROMPT --stream-json`
- **OpenCode:** `opencode run PROMPT --format json [--session ID]`

Resume arguments are added automatically when a remote session ID is known.

For interactive TUI use, prefer `termvox shell --agent opencode` (or any other
agent). Shell mode launches the upstream binary without one-shot flags.

## Authentication

TermVox probes upstream auth non-interactively (environment variables, credential
files, and short-lived CLI checks). Results appear in `termvox doctor` and block
`termvox start` / `termvox shell` when auth is required but missing.

| Agent | Typical signal | Suggested fix |
| --- | --- | --- |
| OpenCode | No `~/.local/share/opencode/auth.json` or API keys | `opencode auth login` |
| Claude | No credentials file or `ANTHROPIC_API_KEY` | `claude login` |
| Codex | No auth file or `OPENAI_API_KEY` | `codex login` |
| Gemini | No auth file or API key env | `gemini auth login` |
| Aider | No LLM API key in environment | `export OPENAI_API_KEY=...` |
| Cursor | Uses Cursor account (unknown if absent) | Sign in via Cursor IDE |
| Amp | Depends on Amp account setup | Configure Amp credentials |

Companion mode does not run auth preflight because the agent subprocess is not
spawned by TermVox.

## Common agent-specific issues

| Agent | Typical non-interactive failure | TermVox setting |
| --- | --- | --- |
| Cursor | Workspace trust required | `agents.cursor.trust_workspace = true` |
| Claude | Not authenticated | Run `claude login` in the same environment |
| Codex | Not authenticated | Complete Codex CLI login first |
| Gemini | Not authenticated | Complete Gemini CLI auth setup first |
| OpenCode | Not authenticated | Run `opencode auth login` first |
| Any on Wayland | Push-to-talk never transcribes | Use `termvox start --toggle` |

## Compatibility caveats

- Aider has no structured-output or resume capability in the current adapter.
- Structured adapters use a forward-compatible JSONL parser; upstream event shapes
  can still change without notice.
- Agent execution is bounded by runtime timeout, total output, and per-frame
  limits from configuration.

An out-of-process [plugin protocol](plugin-system.md) is also under development.
Configured plugins can be listed, inspected, and conformance tested, but they are
not selectable as the `termvox start` coding agent.
