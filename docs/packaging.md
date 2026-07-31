# Packaging notes

TermVox ships official binaries via GitHub Releases and `cargo-dist` (see
`.github/workflows/publish-release.yml`). Community packaging is welcome; the
paths below are documented for contributors, not maintained install channels
yet.

## Official install paths

### npm (recommended)

```bash
npm install -g termvox
```

See [npm.md](../docs/npm.md) for bootstrap behavior, editor extension, and CI skip flags.

### Install script

```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

Pin a version:

```bash
TERMVOX_VERSION=v0.1.0-alpha.8 curl -fsSL .../install.sh | bash
```

## From source (Cargo)

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
cargo install --path crates/termvox-cli --locked
termvox models install default
```

Linux hosts need ALSA development headers (`libasound2-dev` on Debian/Ubuntu,
`alsa-lib-devel` on Fedora).

## Homebrew (formula sketch)

A third-party formula can wrap the release tarball:

```ruby
class Termvox < Formula
  desc "Voice bridge for coding-agent CLIs"
  homepage "https://github.com/Jeronimo0228/termvox"
  url "https://github.com/Jeronimo0228/termvox/releases/download/v0.1.0-alpha.8/termvox-x86_64-apple-darwin.tar.xz"
  sha256 "REPLACE_WITH_RELEASE_SHA"
  license "MIT"

  def install
    bin.install "termvox"
  end

  test do
    assert_match "termvox", shell_output("#{bin}/termvox --version")
  end
end
```

`cargo-dist` already lists `homebrew` in `[workspace.metadata.dist].installers`;
upstream tap publication is planned post-beta.

## Debian `.deb` (manual)

1. Build release: `cargo build --release -p termvox --locked`
2. Stage:

```text
termvox_0.1.0-alpha.8_amd64/
  usr/local/bin/termvox
  usr/share/doc/termvox/copyright
```

3. `dpkg-deb --build termvox_0.1.0-alpha.8_amd64`

Whisper models are not bundled; run `termvox models install` after install.

## Flatpak (outline)

- Runtime: `org.freedesktop.Platform`
- Finish-args: `--device=all` (microphone), `--filesystem=home`
- Entry: `termvox shell` or `termvox daemon start`
- Wayland: document F8/Ctrl+Space inside the sandbox terminal; global hotkeys
  may require `--socket=session-bus` and a portal-compatible compositor.

Flatpak is not published from this repository yet.

## Windows

Use the `.zip` from Releases or `cargo install` on MSVC with LLVM/CMake (see CI
workflow). PowerShell install script ships alongside the shell installer.

## Plugins as shell agents

Custom plugins remain subprocess JSON-RPC adapters (`termvox plugins`). Running
a plugin inside `termvox shell` is on the roadmap; today use `display = "branded"`
or companion delivery for plugin agents.
