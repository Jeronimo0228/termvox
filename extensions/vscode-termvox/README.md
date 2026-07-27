# TermVox for VS Code / Cursor

Minimal extension that wires the TermVox CLI into your editor:

- **Status bar** mic icon (filled when the daemon is running)
- **TermVox: Talk** — runs `termvox talk`
- **Daemon start/stop/status** commands
- Optional **Alt+Space** keybinding while the editor is focused

## Install locally

```bash
cd extensions/vscode-termvox
npm install
npm run compile
```

In VS Code or Cursor: **Extensions → … → Install from VSIX…** after `npm run package`, or use **Developer: Install Extension from Location** and pick this folder.

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `termvox.cliPath` | `termvox` | Path to the CLI binary |
| `termvox.showStatusBar` | `true` | Show the mic status item |

Ensure `termvox` is on your `PATH` (`~/.local/bin`) and the daemon is running:

```bash
termvox daemon start --background
```
