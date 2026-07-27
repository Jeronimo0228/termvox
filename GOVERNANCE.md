# Governance

TermVox uses a lightweight, maintainer-led governance model appropriate for an
early-stage project. The current maintainers are listed in
[MAINTAINERS.md](MAINTAINERS.md).

## Principles

- Safety and privacy defaults take priority over convenience.
- Current behavior is distinguished from roadmap intent.
- Decisions and their rationale should be visible in issues, pull requests,
  documentation, or architecture records.
- Compatibility claims require evidence.
- Authority follows sustained responsibility, not volume of contributions.

## Roles

### Contributors

Anyone who reports issues, proposes designs, writes documentation or code,
reviews changes, or helps other users. Contributors have no repository access
by default.

### Reviewers

Trusted contributors who regularly provide accurate reviews in an area.
Reviewers may be invited by maintainers and can recommend approval, but do not
merge unless they are also maintainers.

### Maintainers

Maintainers triage issues, review and merge changes, manage releases and
security reports, moderate community spaces, and steward project direction.
They are expected to follow the documented release and disclosure processes
and to recuse themselves when a conflict of interest prevents fair review.

## Decisions

Routine, reversible changes are decided through pull-request review. The author
should respond to substantive feedback, and one maintainer approval is required
before merge. Authors do not approve their own changes.

The following require a public proposal before implementation:

- Breaking CLI, configuration, or plugin protocol changes
- New network services, telemetry, data persistence, or credential handling
- New dependencies with meaningful security or maintenance impact
- Changes to licensing, governance, contributor policy, or release trust
- Removal of a supported platform, agent, or speech engine

Maintainers seek rough consensus: address material objections and prefer an
approach contributors can support without requiring unanimity. When consensus
cannot be reached, the lead maintainer decides and documents the rationale.
Time-sensitive security fixes may be developed privately and documented at
coordinated disclosure.

## Becoming a maintainer

A candidate should demonstrate sustained contributions, sound technical and
community judgment, reliable review, security and privacy awareness, and
alignment with the Code of Conduct. An existing maintainer proposes the
candidate. Active maintainers approve by consensus and update
`MAINTAINERS.md`.

Repository permissions should be least-privilege and reviewed when roles
change. Access to private vulnerability reports or release credentials is
granted separately when needed.

## Inactivity and removal

A maintainer may step down at any time. Maintainers who expect to be unavailable
for three months should mark themselves inactive. After six months without
project activity or response, remaining maintainers may move them to emeritus
status after attempting contact.

A maintainer may be removed for a serious Code of Conduct violation, misuse of
credentials or authority, repeated disregard of project policy, or sustained
inability to perform the role. The maintainer must be told the reason and given
an opportunity to respond unless immediate access removal is needed to protect
the project.

## Changes to governance

Governance changes use the public-proposal process and require approval from
all active maintainers. During periods with one active maintainer, that
maintainer records the decision and rationale publicly. Governance documents do
not override the project's licenses.
