## TermVox v0.1.0-alpha.2 — voice prompts for coding agents

**TermVox** is an open-source CLI that turns your microphone into a fast, local speech-to-text bridge for **Cursor**, Claude Code, Codex, Gemini, and other agent CLIs.

### Install in one line

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.ps1 | iex
```

### What's included

- Pre-built binaries for **Linux**, **macOS**, and **Windows**
- Embedded **Whisper** (offline, privacy-first) — default `whisper-tiny` for low latency
- **Cursor companion mode**: transcribe → copy → auto-paste into the Cursor window
- Background **daemon** + `termvox talk` hotkey workflow
- **VS Code / Cursor extension** (`extensions/vscode-termvox/`)
- Signed release artifacts + SHA-256 checksums

### Quick start (Cursor)

```bash
termvox models install default
termvox init --preset cursor --force
termvox daemon start --background
termvox talk   # or Alt+Space with the extension
```

Docs: https://github.com/Jeronimo0228/termvox
