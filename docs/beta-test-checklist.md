# Beta test checklist

Thank you for trying TermVox during the alpha preview. This checklist takes
about **15 minutes**. Please report results via
[GitHub beta feedback](../.github/ISSUE_TEMPLATE/beta-feedback.yml).

## Your environment

Fill in before testing:

| Field | Your value |
| --- | --- |
| OS + version | |
| Architecture (x64 / arm64) | |
| Terminal app | |
| TermVox version (`termvox --version`) | |
| Agent CLI + version | |
| Install method (npm / script / release) | |

## Install path

- [ ] `npm install -g termvox` (or script installer) completed without errors
- [ ] `termvox --version` prints `0.1.0-alpha.10` or newer
- [ ] `termvox doctor` shows `[ok]` for **microphone** and **speech/whisper**
- [ ] At least one agent line shows `[ok]` (not `[--]` or `[!!]`)

## Speech smoke test

```bash
termvox test --seconds 3
```

- [ ] Captured audio (non-zero seconds)
- [ ] Printed transcript (any language)
- [ ] Completed in under 30 seconds on your machine

## Integrated shell (main feature)

```bash
cd a-git-repo-you-use-daily
termvox shell --agent cursor   # or opencode / claude / codex
```

- [ ] Agent TUI appeared inside TermVox wrapper
- [ ] Mic bar visible at bottom (may flicker — note if it disappears)
- [ ] **F8** or **Ctrl+Space** started recording
- [ ] Spoke a short prompt; transcript appeared
- [ ] After confirm, agent received the prompt
- [ ] **Ctrl+\\** exited TermVox wrapper cleanly

## Optional

- [ ] `termvox-editor-install` (Cursor/VS Code extension)
- [ ] `termvox daemon start --background` + global hotkey (Linux/macOS)

## Report blockers

Note anything that failed with:

1. Exact command
2. Full terminal output (redact secrets)
3. What you expected vs what happened

File: https://github.com/Jeronimo0228/termvox/issues/new?template=beta-feedback.yml
