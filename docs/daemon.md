# Daemon mode

Run TermVox in the background with a global hotkey and IPC control.

## Start

```bash
termvox daemon start --background
termvox daemon status
```

Default hotkey: `ALT+SPACE` (configure in `[daemon].hotkey`).

## Talk without focusing the daemon terminal

```bash
termvox talk
```

This toggles recording on the running daemon (same as pressing the global hotkey).

## Stop

```bash
termvox daemon stop
```

## Configuration

```toml
[daemon]
hotkey = "ALT+SPACE"
skip_confirmation = true

[agents.cursor]
display = "companion"
delivery = "both"   # clipboard + auto-paste into focused Cursor window
```

Install paste helpers on Linux:

- Wayland: `wtype` (often via `wtype` package)
- X11: `xdotool`
- uinput: `ydotool`

## Typical Cursor workflow

```bash
# Terminal 1
agent

# Terminal 2
termvox init --preset cursor --force
termvox models install default
termvox daemon start --background
```

Focus the Cursor Agent window, press **Alt+Space**, speak, press again — the prompt is copied and pasted automatically.
