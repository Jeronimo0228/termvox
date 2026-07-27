# Releasing TermVox

This document is a maintainer checklist. It does not assert that any package,
binary, signature, checksum, SBOM, provenance record, or release has already
been published.

## Release principles

- Release only from the canonical repository:
  <https://github.com/Jeronimo0228/termvox>
- Treat credentials, signing keys, and package tokens as least-privilege
  secrets; never store them in the repository.
- Never describe an artifact as signed, notarized, reproducible, or verified
  unless that property was completed and independently checked for that exact
  artifact.
- Do not move or reuse a published version tag.
- Coordinate security releases privately under [SECURITY.md](SECURITY.md).

## 1. Define the release

1. Choose a SemVer version appropriate for compatibility. Pre-1.0 and alpha
   releases must clearly state that interfaces may change.
2. Confirm scope and defer unrelated changes.
3. Review `CHANGELOG.md`, migration notes, known limitations, compatibility,
   and security impact.
4. Verify licensing for new dependencies and contributed assets.

## 2. Prepare a release pull request

Update all version-bearing workspace metadata and documentation consistently.
Move relevant `Unreleased` entries into a dated version section, then create a
fresh empty `Unreleased` section. Do not claim availability before publication.

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --release
mdbook build docs
cargo dist plan
```

Also verify:

- `cargo metadata --no-deps` reports the intended version, repository, and
  `MIT OR Apache-2.0` license expression
- A clean checkout can follow the documented source installation
- All internal documentation links resolve
- `termvox --help`, subcommand help, `termvox config validate`, and
  `termvox doctor` match the docs
- Supported platform and integration claims have current evidence
- No credentials, recordings, models, or private paths are present

Record skipped checks and reasons in the release pull request.

## 3. Approve and tag

After the release pull request is merged, check out the exact canonical commit
and verify the tree is clean. Create an annotated tag:

```bash
git tag -a vX.Y.Z -m "TermVox X.Y.Z"
git push origin vX.Y.Z
```

If the project later adopts signed tags, document the key identity and
verification procedure before calling a tag signed. An ordinary annotated tag
is not a cryptographic signature.

## 4. Build artifacts, if applicable

Source-only releases are acceptable while the artifact pipeline is immature.
If binaries are produced:

1. Build from the tagged commit in isolated, documented environments.
2. Name files with version, OS, architecture, and archive format.
3. Test each archive on a clean supported system.
4. Generate SHA-256 checksums after final packaging.
5. Generate an SBOM and provenance record only with a documented tool and
   retain the underlying evidence.
6. Sign final artifacts only with the approved release key and verify every
   signature independently.

Checksums provide integrity when obtained from a trusted channel; they are not
signatures. Do not advertise code signing or notarization for platforms where
it was not completed.

## 5. Publish

Create release notes from the changelog, including:

- User-visible changes and migration steps
- Supported platforms and integrations actually tested
- Known limitations and unresolved security considerations
- Exact artifact verification instructions, when artifacts exist
- Dual-license notice

Publish package-registry crates only after dry-run packaging and dependency
order are reviewed. Package publication is irreversible; confirm names,
ownership, metadata, included files, and tokens immediately beforehand.

After publication, verify links and artifacts from an unauthenticated clean
environment. Only then update documentation that describes the release as
available.

## 6. Respond to problems

Do not rewrite a published tag or silently replace assets. If metadata or notes
are wrong, correct them transparently. If an artifact is unsafe, remove or mark
it, publish an advisory, and prepare a new patch version. Document impact,
workaround, and superseding version.
