# Security policy

## Supported versions

TermVox is pre-release software and has not declared a stable supported release
line. Security fixes are developed on the default branch. Older commits,
forks, locally modified builds, third-party packages, speech models, agent CLIs,
and external plugins are not supported by this policy.

| Version | Security support |
| --- | --- |
| Default branch | Best effort |
| Published releases, if any | See the release notes for that release |
| Older source revisions | Not supported |

This table does not assert that a release or artifact has been published.

## Reporting a vulnerability

Do not open a public issue, discussion, or pull request for an undisclosed
vulnerability.

Use GitHub's private vulnerability reporting for the canonical repository:

<https://github.com/Jeronimo0228/termvox/security/advisories/new>

If that feature is unavailable, use the private contact method on the current
maintainer's GitHub profile in [MAINTAINERS.md](MAINTAINERS.md). Send only
enough initial information to establish a private channel.

Include:

- A clear description and security impact
- Affected revision or version and platform
- Reproduction steps or a minimal proof of concept
- Required configuration, agent, speech engine, or plugin
- Whether user interaction or special privileges are needed
- Suggested remediation, if known
- Any disclosure deadline or coordination constraints

Remove real credentials, private recordings, and unrelated personal data.

## What to expect

The project will make a best effort to:

1. Acknowledge the report and confirm a private contact channel.
2. Reproduce and assess scope and severity.
3. Coordinate a fix, tests, documentation, and disclosure.
4. Credit the reporter if requested and safe.

Response times are not guaranteed because the project currently has limited
maintainer capacity. Please avoid public disclosure until a fix and advisory
can be coordinated, while recognizing that reporters retain control of their
own disclosure decisions.

## Security scope

Useful reports include vulnerabilities in TermVox's:

- Subprocess argument and output handling
- Configuration and environment-variable boundaries
- Model download verification and file placement
- Temporary audio-file lifecycle
- Plugin protocol parsing and process lifecycle
- Confirmation or risk-policy bypasses
- Exposure of audio, transcripts, prompts, or credentials

The following are generally out of scope unless TermVox makes them worse:

- Vulnerabilities solely in an upstream agent CLI, CPAL backend, Whisper.cpp,
  model, operating system, or remote API
- Social engineering without a TermVox vulnerability
- Denial of service requiring the same user's control of the process or files
- Reports that only note that coding agents can execute tools with permissions
  the user granted them
- Missing hardening features already documented as pre-release limitations

Do not test against systems, accounts, microphones, APIs, or data you do not
own or have explicit permission to use.

For the user-facing threat model, see
[docs/privacy-security.md](docs/privacy-security.md).
