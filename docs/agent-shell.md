# Agent Shell (`termvox shell`)

Unified voice layer for **every** supported coding-agent CLI. One terminal
session, native upstream TUI, integrated microphone chrome.

## Goal

When a developer runs their agent CLI, they should see:

1. The **real** agent interface (Cursor Agent, Claude Code, Codex, OpenCode, etc.)
2. A **persistent mic bar** owned by TermVox
3. **No second terminal**, no paste-into-other-window workflow

This applies equally to all built-in adapters:

| Agent | Default executable | Shell support |
| --- | --- | --- |
| Cursor | `agent` | Yes |
| Claude Code | `claude` | Yes |
| Codex | `codex` | Yes |
| Gemini | `gemini` | Yes |
| Aider | `aider` | Yes |
| Amp | `amp` | Yes |
| OpenCode | `opencode` | Yes |

Configuration follows the existing `agent = "..."` key and
`[agents.<name>]` profiles (`executable`, `extra_args`, `trust_workspace`).

## Architecture

```text
termvox shell
  ├─ resolve agent from config (same as termvox start)
  ├─ preflight auth (fail fast with login command when missing)
  ├─ spawn upstream CLI in PTY (portable-pty)
  ├─ render mic status bar (crossterm / ratatui)
  ├─ on toggle: AudioRecorder → SpeechEngine → PromptPipeline
  └─ inject transcribed text into PTY stdin (+ optional auto-submit)
```

The upstream binary is **never patched**. TermVox wraps the process and adds
chrome around it.

## Display mode

Agent Shell replaces **companion** mode for interactive CLI use:

| Mode | When to use |
| --- | --- |
| `shell` (default for Cursor and OpenCode presets) | Integrated mic + agent TUI |
| `companion` | Two-window / Wayland fallback |
| `branded` | TermVox-owned session without upstream TUI |

## User experience

```bash
termvox shell                    # uses agent from termvox.toml
termvox shell --agent claude     # override for one session
termvox shell --agent opencode   # OpenCode TUI + mic bar
termvox shell --fresh            # ignore saved workspace session
```

When `[workspace].persist_session` is enabled (default), TermVox saves the
upstream session id to `.termvox/session.json` and passes resume flags on the
next launch in the same project directory.

Example layout:

```text
┌─────────────────────────────────────────┐
│  <upstream agent TUI — unchanged>       │
│  ...                                    │
├─────────────────────────────────────────┤
│ 🎤 TermVox · Cursor · ES · listo · voz Ctrl+Space · salir Ctrl+\ │
└─────────────────────────────────────────┘
```

Mic states: idle → recording → transcribing → injected → idle.

Default voice hotkey: **F8** (configurable via `[shell].hotkey`). On Wayland,
**Ctrl+Space** is added automatically as a fallback (`[shell].alt_hotkeys`).

Leave the wrapper with **`Ctrl+\\`** (`[shell].exit_hotkey`) without sending Ctrl+C
to the agent. **Ctrl+C** is forwarded to the upstream TUI.

## Workspace sessions

When `[workspace].persist_session` is enabled (default), TermVox saves upstream
session ids per agent to `.termvox/session.json` and passes resume flags on the
next launch in the **same project directory**. Use `termvox shell --fresh` to ignore
the saved session. With `discover_session = true` (default), TermVox also probes
agent-local stores scoped to that directory when no id is saved yet (never a
global/latest-session fallback).

## Cursor trust

In shell mode, Cursor receives `-f` automatically so the workspace trust dialog
does not block the TUI. Branded subprocess mode still respects
`agents.cursor.trust_workspace` in config.

## Authentication

`termvox shell` and `termvox start` (non-companion modes) check upstream auth
before launching the agent subprocess. If credentials are missing, TermVox exits
with a clear message and the suggested login command.

Run `termvox doctor` to inspect auth for all installed agents:

```text
[!!] agent/opencode   opencode 0.x — OpenCode has no configured providers
       run: opencode auth login
[ok] agent/claude     1.x (Claude credentials file present)
```

Companion mode skips auth preflight because TermVox only transcribes and pastes
into an external TUI you manage yourself.

## Agent-specific notes

- **Cursor:** shell mode auto-trusts the cwd (`-f`); stdin injection replaces auto-paste.
- **OpenCode:** launches the interactive TUI (`opencode` with no subcommand).
  Session resume is scoped to the **current project directory** (OpenCode
  `project.worktree` / `project_directory`, not the latest global session).
  Use `termvox shell --fresh` when switching repos if you want a new chat.
  One-shot `opencode run --format json` is used by `termvox start` in branded
  mode. Authenticate with `opencode auth login` first.
- **Claude / Codex / Gemini / Amp:** launch interactive TUI, not one-shot
  `-p` mode; reuse JSONL parsers only when the shell forwards structured output.
- **Aider:** plain-text TUI; no streaming parser required for voice injection.

## Optional shims

```bash
termvox install-shim --agent cursor    # ~/.local/bin/agent → termvox shell
termvox install-shim --agent opencode  # ~/.local/bin/opencode → termvox shell
```

Lets users keep typing `agent`, `claude`, or `opencode` while TermVox provides
the mic layer.

## Status

**Stable since v0.1.0-alpha.8** — workspace session resume, localized mic bar,
streaming partial transcripts, and PTY filters for Kitty keyboard protocols.
