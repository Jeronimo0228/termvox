## TermVox v0.1.0-alpha.15 — CI green, shell TUI stability

Pin: `TERMVOX_VERSION=v0.1.0-alpha.15`

### What's new in alpha.15

- Fix CI: rustfmt + clippy on session store (cast truncation)
- Suppress Whisper/ggml AMX stderr so OpenCode TUI stays clean in `termvox shell`
- Defer Whisper prewarm until first voice toggle in shell mode
- `termvox shell --fresh` skips mid-session discovery
- Workspace sessions scoped per project **and** per agent (`session.json` v2)

### Upgrade

```bash
npm install -g termvox@latest
# or
TERMVOX_VERSION=v0.1.0-alpha.15 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
termvox doctor
```
