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

Record manually with **OBS Studio** (or similar): full screen + mic, export MP4,
and upload natively to LinkedIn.

### What to show (storyboard)

1. **Hook (5 s)** — Terminal with project open; say what you are about to do.
2. **Install (10 s)** — `npm install -g termvox` (or skip if pre-installed).
3. **Doctor (10 s)** — `termvox doctor` with green mic + speech lines.
4. **Shell (30–50 s)** — `termvox shell`; press F8, speak one sentence
   (*"Add error handling to the login function"*), show transcript + confirm,
   agent starts working.
5. **Close (5 s)** — Repo URL + *"alpha preview — feedback welcome"*.

Tips:

- Use a **large font** (16–18 pt) and high contrast theme.
- Close unrelated notifications.
- Pre-run `termvox doctor` and `termvox test --seconds 2` so Whisper is warm.
- If Cursor/OpenCode auth fails, switch to whichever agent shows `[ok]` in doctor.

### Pre-flight checks

```bash
bash scripts/launch-smoke.sh         # CI-style happy-path checks
```

Upload the video to LinkedIn directly (native video gets more reach than YouTube links).

## Beta tester checklist

Share [beta-test-checklist.md](beta-test-checklist.md) with 2–3 friends before a
wide post. Ask them to file
[beta feedback](../../.github/ISSUE_TEMPLATE/beta-feedback.yml) on GitHub.

## LinkedIn copy

Draft post (Spanish): [linkedin-post-es.md](linkedin-post-es.md)

After recording, attach the video to the post.
