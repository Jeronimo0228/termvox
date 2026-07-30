## TermVox v0.1.0-alpha.8 — doctor hints, balanced Whisper, embedded SQLite

Pin: `TERMVOX_VERSION=v0.1.0-alpha.8`

### What's new in alpha.8

- OpenCode session discovery uses embedded SQLite (no `sqlite3` in PATH)
- `balanced` performance profile maps to `ggml-base.bin`
- `termvox doctor` prints contextual hints (shell vs companion, Wayland, models)
- Interactive setup can default Cursor/OpenCode to integrated shell
- CI shell-smoke job; Spanish CLI reference and troubleshooting docs
- Packaging notes for Homebrew, `.deb`, and Flatpak contributors

### Upgrade

```bash
TERMVOX_VERSION=v0.1.0-alpha.8 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
termvox models install accurate   # if using balanced/accurate profile
termvox doctor
```
