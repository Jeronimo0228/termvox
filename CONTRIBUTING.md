# Contributing to TermVox

Thank you for improving TermVox. The project welcomes focused bug reports,
documentation, tests, compatibility research, and code changes.

## Before starting

1. Search existing issues and pull requests in the
   [canonical repository](https://github.com/Jeronimo0228/termvox).
2. For a bug, include a minimal reproduction, OS, Rust version, TermVox
   revision, relevant tool versions, and sanitized diagnostics.
3. Discuss large features, protocol changes, new dependencies, or breaking
   configuration changes before implementation.
4. Never post credentials, private audio, transcripts, or vulnerability
   details in a public issue. Use [SECURITY.md](SECURITY.md) for vulnerabilities.

Roadmap entries are intent, not assignments or acceptance guarantees.

## Development setup

Install Rust 1.88 or later and the platform prerequisites from
[docs/installation.md](docs/installation.md), then:

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
cargo build --workspace
cargo test --workspace
```

Some audio behavior requires real hardware and OS permission. Tests must not
depend on a contributor's microphone, network, paid API, or installed agent
unless clearly marked as opt-in.

## Make a change

- Keep pull requests small and single-purpose.
- Add or update tests for observable behavior.
- Update user documentation and `CHANGELOG.md` when behavior changes.
- State current support accurately. Do not describe planned integrations,
  artifacts, signatures, or packages as available.
- Avoid unrelated formatting or refactoring.
- Preserve conservative defaults and explicit confirmation boundaries.
- Never add secrets, downloaded models, recordings, or generated credentials.

Run the relevant checks before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

For documentation, check internal links and, when mdBook is installed:

```bash
mdbook build docs
```

If a check cannot run in your environment, explain why in the pull request.

## Pull requests

A useful pull request contains:

- The problem and why the change is needed
- The chosen approach and important trade-offs
- Test evidence, including OS or hardware details when relevant
- Security, privacy, compatibility, and migration impact
- Documentation updates

At least one maintainer approval is required. Authors should not approve their
own changes. Maintainers may request changes, split oversized work, or decline
changes that conflict with project scope or maintenance capacity.

## Commit style

Use concise imperative subjects such as `docs: clarify Whisper privacy` or
`fix: preserve agent session id`. Keep generated or mechanical changes separate
when practical. Signed commits are welcome but are not currently required.

## Documentation style

Documentation is written primarily in English, with essential Spanish
translations under `docs/es/`. Use plain language, runnable examples, relative
links, and explicit status labels: **implemented**, **experimental**, or
**planned**. Avoid unverified benchmark, platform, release, and security claims.

## Licensing

By submitting a contribution, you agree that it may be distributed under the
project's dual license, at the recipient's option:

- MIT
- Apache License 2.0

No contributor license agreement is currently required. Only submit work you
have the right to license, and preserve required third-party notices.

All contributors must follow the [Code of Conduct](CODE_OF_CONDUCT.md).
