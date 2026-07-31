# npm installation

TermVox publishes an npm package that downloads the native CLI for your platform
and exposes every command (`shell`, `daemon`, `doctor`, `models`, etc.).

## Install globally (recommended)

```bash
npm install -g termvox
```

During `postinstall`, TermVox:

1. Downloads the matching GitHub Release binary (Linux/macOS/Windows x64/arm64)
2. Verifies SHA-256 when the checksum file is available
3. Runs `termvox models install default` (Whisper tiny, ~74 MiB)
4. Writes `termvox init --preset cursor` when no config exists

Then verify:

```bash
termvox --version
termvox doctor
termvox shell
```

## Project-local install

```bash
npm install termvox
npx termvox doctor
npx termvox shell --agent opencode
```

## Editor extension (Cursor / VS Code)

The npm package bundles a minimal editor extension:

```bash
termvox-editor-install
```

This tries `cursor`, `code`, `codium`, and `code-insiders` in order. Manual
install: **Extensions → Install from Location** →
`$(npm root -g)/termvox/editor` (global) or `node_modules/termvox/editor`.

## Skip bootstrap steps

| Variable | Effect |
| --- | --- |
| `TERMVOX_SKIP_BINARY_INSTALL=1` | Do not download the native binary (CI/packaging) |
| `TERMVOX_SKIP_BOOTSTRAP=1` | Skip Whisper model download and `termvox init` |
| `TERMVOX_NPM_PRESET=opencode` | Preset for first-time init |
| `TERMVOX_INSTALL_REPO=owner/repo` | Alternate GitHub release source |

Example for air-gapped or custom binary path:

```bash
TERMVOX_SKIP_BINARY_INSTALL=1 npm install -g termvox
# place your own binary at $(npm root -g)/termvox/vendor/termvox
```

## Publish (maintainers)

1. Create an npm account and organization if needed
2. Add repository secret `NPM_TOKEN` (Automation token with publish)
3. Push a release tag `v*` — workflows `Publish release` and `Publish npm` run
4. Or run **Publish npm** manually from Actions with the tag input

Local dry-run:

```bash
cd packages/termvox
npm test
npm run build:editor
TERMVOX_SKIP_BINARY_INSTALL=1 npm pack
```

## Requirements

- Node.js 18+
- Supported platform triple (see `packages/termvox/lib/platform.js`)
- Microphone permissions for the terminal or daemon
- At least one supported agent CLI on `PATH` for `termvox shell`

See also [installation.md](installation.md) for source builds and shell script install.
