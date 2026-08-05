# Demo and launch materials

Use this page to record a LinkedIn demo, onboard beta testers, and verify the
**happy path** before a public post.

## Happy path (recommended first install)

Target audience: developers who already use **Cursor CLI** or **OpenCode** on
Linux or macOS.

```bash
npm install -g termvox
termvox doctor
termvox init --preset cursor    # or: --preset opencode
termvox models install default  # ~74 MiB; use `accurate` for better STT
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

STT quality: see [performance.md](performance.md) and [es/stt.md](es/stt.md).

## Record a 45–90 second LinkedIn video

### 1. Capture (OBS / GNOME)

Record the shell session manually with **OBS Studio** (preferred) or the desktop
recorder. Aim for **1920×1080 or 3840×2160**, **≥30 fps**, and a bitrate high
enough that terminal text stays sharp (OBS: x264, CRF ~18, or CBR ≥12 Mbps at
1080p). Export MP4.

GNOME’s built-in recorder often produces **low-bitrate / low-fps** clips that
look soft when upscaled — prefer OBS for the public cut.

### 2. What to show (storyboard)

1. **Hook (5 s)** — Terminal with project open; say what you are about to do.
2. **Install (10 s)** — `npm install -g termvox` (or skip if pre-installed).
3. **Doctor (10 s)** — `termvox doctor` with green mic + speech lines.
4. **Shell (30–50 s)** — `termvox shell`; press F8, speak one sentence, show
   transcript + confirm, agent starts working.
5. **Close (5 s)** — Repo URL + *"alpha preview — feedback welcome"*.

Tips:

- Use a **large font** (16–18 pt) and high contrast theme.
- Close unrelated notifications.
- Pre-run `termvox doctor` and `termvox test --seconds 2` so Whisper is warm.
- If Cursor/OpenCode auth fails, switch to whichever agent shows `[ok]` in doctor.

### 3. Package for LinkedIn / pitch (practical cut)

Builds a **terminal-only** pitch video:

1. Animated setup: `init` → `config show` → `models install` → `doctor` → `shell`
2. Your screen recording (full width, light grade; ANSI glitch fragments covered)

No marketing title cards or burned-in captions.

```bash
/usr/bin/python3.14 scripts/render-linkedin-demo.py \
  --source "$HOME/Vídeos/Grabaciones de la pantalla/tu-grabacion.mp4" \
  --out-dir "$HOME/Vídeos/termvox-demo"
```

Outputs:

- `termvox-linkedin-4k.mp4` — archive master (3840×2160)
- `termvox-linkedin-1080p.mp4` — **preferred upload** for LinkedIn / decks

Keep the pitch text in the LinkedIn post body ([linkedin-post-es.md](linkedin-post-es.md)),
not burned into the video.

### Pre-flight checks

```bash
bash scripts/launch-smoke.sh         # CI-style happy-path checks
termvox --help                       # CLI help + manpage share the same copy
```

Upload the video to LinkedIn directly (native video gets more reach than YouTube links).

## Beta tester checklist

Share [beta-test-checklist.md](beta-test-checklist.md) with 2–3 friends before a
wide post. Ask them to file
[beta feedback](../../.github/ISSUE_TEMPLATE/beta-feedback.yml) on GitHub.

## LinkedIn copy

Draft post (Spanish): [linkedin-post-es.md](linkedin-post-es.md)

After recording, attach the video to the post.
