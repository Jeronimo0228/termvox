## TermVox v0.1.0-alpha.7 — workspace sessions and shell polish

**TermVox** is an open-source CLI that turns your microphone into a fast, local
speech-to-text bridge for **Cursor**, Claude Code, Codex, Gemini, OpenCode, Aider,
and Amp.

### Install

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.ps1 | iex
```

Pin: `TERMVOX_VERSION=v0.1.0-alpha.7`

### What's new in alpha.7

- **Workspace session resume** — `.termvox/session.json` per project; `--fresh` to reset
- **Session discovery** — Cursor transcripts, OpenCode sqlite, Claude projects, PTY heuristics
- **Shell polish** — localized mic bar (es/en), partial streaming, Cursor auto-trust in shell
- **Branded/daemon persist** — subprocess modes hydrate and save upstream session ids
- **PTY stability** — Kitty keyboard/mouse/bracketed-paste filtering; `Ctrl+\` exit fix

### Quick start

```bash
termvox models install default
termvox init --preset cursor --force
termvox doctor
cd your-project
termvox shell
```

Voice: **F8** or **Ctrl+Space** (Wayland). Exit wrapper: **Ctrl+\\**.

Docs: https://github.com/Jeronimo0228/termvox/tree/main/docs
