# Demo and launch materials

Use this page to record a LinkedIn demo, onboard beta testers, and verify the
**happy path** before a public post.

## Pre-rendered 4K demos

| File | Content |
| --- | --- |
| [`termvox-demo-4k-linkedin.mp4`](assets/termvox-demo-4k-linkedin.mp4) | **~55 s, 3840×2160** — **real `termvox shell` mic bar** + OpenCode demo TUI |
| [`termvox-demo-shell-4k.mp4`](assets/termvox-demo-shell-4k.mp4) | ~18 s raw shell recording (asciinema → 4K) |
| [`termvox-demo-4k.mp4`](assets/termvox-demo-4k.mp4) | VHS tape (commands only, no mic bar) |

### Regenerate shell demo (recommended for LinkedIn)

Shows the **integrated shell**: upstream agent area + TermVox mic bar on the last row
(Listening → Transcribing → Confirm → injected prompt).

```bash
bash scripts/record-shell-demo-4k.sh opencode   # or: cursor, claude, codex
```

Uses `termvox shell --demo --demo-auto` (no real agent CLI required). Manual recording
with real Cursor/OpenCode:

```bash
termvox shell --agent cursor          # real agent + mic bar; press F8 yourself
termvox shell --demo --agent opencode # fake agent UI + real mic bar
```

Requires: `asciinema`, `ffmpeg`, Rust toolchain. Downloads `agg` into `.cache/bin/`.

### Regenerate VHS command demo

```bash
bash scripts/render-demo-4k.sh
```

## Happy path (recommended first install)

Target audience: developers who already use **Cursor CLI** or **OpenCode** on
Linux or macOS.

```bash
npm install -g termvox
termvox doctor
termvox init --preset cursor    # or: --preset opencode
termvox models install default  # ~74 MiB, skipped if postinstall ran
cd your-project
termvox shell --agent cursor    # or: --agent opencode
```

Inside `termvox shell`:

| Action | Key |
| --- | --- |
| Toggle voice | **F8** or **Ctrl+Space** (Wayland) |
| Exit TermVox wrapper | **Ctrl+\\** |

Speak a short prompt in your language, review the transcript, confirm when
asked, and watch the agent receive it in the same terminal.

Quick mic test without an agent:

```bash
termvox test --seconds 3
```

## Record a 45–90 second LinkedIn video

### What to show (storyboard)

1. **Hook (5 s)** — Terminal with project open; say what you are about to do.
2. **Install (10 s)** — `npm install -g termvox` (or skip if pre-installed).
3. **Doctor (10 s)** — `termvox doctor` with green mic + speech lines.
4. **Shell (30–50 s)** — `termvox shell`; press F8, speak one sentence
   (*"Add error handling to the login function"*), show transcript + confirm,
   agent starts working.
5. **Close (5 s)** — Repo URL + *"alpha preview — feedback welcome"*.

### Recording tools

| Tool | Best for |
| --- | --- |
| **OBS Studio** | Full screen + mic; export MP4 for LinkedIn |
| **VHS (4K terminal cast)** | Scripted MP4 at 3840×2160 — see below |
| **asciinema** | Terminal-only cast (`scripts/record-demo.sh --asciinema`) |
| **Phone camera** | Acceptable if terminal text is readable |

### 4K VHS render (terminal-only MP4)

For a reproducible, high-resolution terminal demo without manual OBS capture:

```bash
bash scripts/render-demo-4k.sh
```

This installs [VHS](https://github.com/charmbracelet/vhs) via `go` when missing,
requires **ffmpeg**, builds `termvox` if needed, and renders
`docs/assets/termvox-demo-4k.mp4` from `docs/assets/termvox-demo.tape`
(Catppuccin Mocha, 3840×2160, 60 fps). The tape runs `termvox --version`,
`termvox doctor`, `termvox test --seconds 2`, then documents the shell
**F8** voice flow via on-screen comment lines.

Tips:

- Use a **large font** (16–18 pt) and high contrast theme.
- Close unrelated notifications.
- Pre-run `termvox doctor` and `termvox test --seconds 2` so Whisper is warm.
- If Cursor/OpenCode auth fails, switch to whichever agent shows `[ok]` in doctor.

### Automated prep script

```bash
bash scripts/record-demo.sh          # human-readable steps + smoke checks
bash scripts/record-demo.sh --asciinema   # record terminal cast (requires asciinema)
bash scripts/launch-smoke.sh         # CI-style happy-path checks
```

Upload the video to LinkedIn directly (native video gets more reach than YouTube links).

**Shortcut:** attach [`docs/assets/termvox-demo-4k-linkedin.mp4`](assets/termvox-demo-4k-linkedin.mp4) (~55 s, real mic bar) or re-render with `bash scripts/record-shell-demo-4k.sh`.

## Beta tester checklist

Share [beta-test-checklist.md](beta-test-checklist.md) with 2–3 friends before a
wide post. Ask them to file
[beta feedback](../../.github/ISSUE_TEMPLATE/beta-feedback.yml) on GitHub.

## LinkedIn copy

Draft post (Spanish): [linkedin-post-es.md](linkedin-post-es.md)

After recording, attach the video and replace `{DEMO_VIDEO}` placeholders in the draft.
