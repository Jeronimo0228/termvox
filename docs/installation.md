# Installation

## Support level

TermVox is pre-release software. Building from this repository is the only
installation method verified by the project documentation. Do not assume that
GitHub release assets, package-manager formulas, checksums, signatures, or
SBOMs exist.

All platforms need:

- Git
- Rust 1.88 or later, including Cargo
- A working microphone and OS permission to use it
- One [supported coding-agent CLI](agents.md)
- About 142 MiB of disk space for the default local Whisper model

## Linux

Install the ALSA development package before building:

```bash
# Debian or Ubuntu
sudo apt-get update
sudo apt-get install build-essential pkg-config libasound2-dev

# Fedora or RHEL
sudo dnf install gcc pkgconf-pkg-config alsa-lib-devel
```

Then build:

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
cargo install --path crates/termvox-cli
```

TermVox uses terminal key events, not a desktop-wide keyboard hook. Keep its
terminal focused while using push-to-talk.

## macOS

Install the Xcode Command Line Tools and Rust:

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Open a new shell, then clone and build as shown above. macOS may ask the
terminal application—not the `termvox` executable by name—for microphone
permission. Review it under **System Settings → Privacy & Security →
Microphone**.

## Windows

Install Rust with the 64-bit MSVC toolchain and the Microsoft C++ Build Tools.
In PowerShell:

```powershell
git clone https://github.com/Jeronimo0228/termvox.git
Set-Location termvox
cargo install --path crates/termvox-cli
termvox.exe init
termvox.exe doctor
```

Allow microphone access for desktop applications in Windows privacy settings.
The current project does not claim a tested installer or code-signed Windows
binary.

## Speech engine setup

### Embedded Whisper (default)

No separate inference executable is needed. Install the reviewed multilingual
base model with:

```bash
termvox models install default
termvox models status default
```

TermVox downloads from a commit-pinned upstream URL, verifies its SHA-256, then
atomically moves the file into its data directory. Use
`termvox models remove default` to remove it. Interactive recording commands
offer the download when it is missing; non-interactive use never consents on
your behalf.

### OpenAI

No local model is needed. Set `speech_engine = "openai"` and export the
environment variable named by `openai.api_key_env`. Usage may incur provider
charges and is subject to the provider's terms and data handling.

### Parakeet or Vosk

TermVox does not install either engine. Supply a compatible local sidecar
executable and model, then configure `[parakeet]` or `[vosk]`. The required
command and JSON contract are documented in
[Speech engines](speech-engines.md).

## Upgrade or uninstall

From a new checkout of the desired revision:

```bash
cargo install --path crates/termvox-cli --force
cargo uninstall termvox
```

`termvox update` is informational only. It does not download or replace the
running executable.
