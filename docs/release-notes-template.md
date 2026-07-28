## TermVox v0.1.0-alpha.4 — integrated agent shell and OpenCode

**TermVox** is an open-source CLI that turns your microphone into a fast, local speech-to-text bridge for **Cursor**, Claude Code, Codex, Gemini, OpenCode, and other agent CLIs.

### Install in one line

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.ps1 | iex
```

### What's new in alpha.4

- **`termvox shell`** — unified voice layer for all seven built-in agents (PTY + mic bar, F8 hotkey)
- **OpenCode adapter** — branded JSON mode and interactive shell TUI
- **Auth preflight** — `termvox doctor` and shell/start fail fast with login hints when credentials are missing
- **`termvox install-shim`** — Unix wrapper so `claude`, `opencode`, etc. launch through TermVox

### What's included

- Pre-built binaries for **Linux**, **macOS**, and **Windows**
- Embedded **Whisper** (offline, privacy-first) — default `whisper-tiny` for low latency
- **Cursor companion mode**: transcribe → copy → auto-paste into the Cursor window
- Background **daemon** + `termvox talk` hotkey workflow
- **VS Code / Cursor extension** (`extensions/vscode-termvox/`)
- Signed release artifacts + SHA-256 checksums

### Quick start (integrated shell)

```bash
termvox models install default
termvox init --preset opencode --force   # or cursor, claude, codex, …
termvox doctor
termvox shell
```

Docs: https://github.com/Jeronimo0228/termvox
