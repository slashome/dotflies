# 0002 — Limit v1 to macOS and open the Linux port to contribution

## Status

Accepted — 2026-08-14

## Context

dotflies is meant to produce a configuration that reinstalls on macOS **and** Linux.
But the working machine is a Mac, all nine configurations adopted so far are macOS,
and v1 is deliberately small ([0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md)).

Supporting both systems from v1 would double every code path and force a templating
engine immediately, on a project that has already died twice from over-ambition at
the start.

There is, however, an explicit requirement: **moving from one system to the other
must be handled** (Linux to macOS, or the reverse). That requirement is in direct
tension with "macOS first" — handling that migration *is* the cross-platform work.

There is precedent. [`slashome/dotfiles`](https://github.com/slashome/dotfiles) (2019)
hardcoded `trizen` with no notion of platform at all. That single choice made the
repository useless the moment the system changed.

## Decision

**v1 implements macOS only.** Homebrew and npm are the only supported package
managers.

**But the manifest format carries a platform key from v1.** A Linux entry can be
declared in a manifest; v1 validates it, displays it, and skips it at execution time
with an explicit message rather than an error.

Concretely, a manifest never says "the config path is X". It says "on `darwin` the
path is X, on `linux` it is Y". v1 only knows how to resolve `darwin`.

**A call for contribution is published in the README**: the Linux port is explicitly
open, and the manifest format is designed to accept it without a break.

## Consequences

- v1 stays bounded without mortgaging cross-platform support.
- A Linux contributor will not have to break the existing format. They implement a
  resolver for `linux` and a package-manager layer, leaving written manifests alone.
- **Linux ↔ macOS migration does not ship in v1.** That is the accepted consequence
  and the main limitation of this decision. It becomes possible without a rewrite the
  day a `linux` resolver exists.
- Immediate cost: some ceremony in manifests for a single useful platform. That is
  the price of forward compatibility.
- Refusing to hardcode the package manager avoids repeating the 2019 dead end.

## Alternatives considered

**macOS and Linux from v1.** Rejected: doubles the scope and forces templating
immediately, on a project whose historical failure mode is exactly initial size.

**Hardcode macOS, worry about portability later.** Rejected: this is the 2019 mistake.
Catching up costs a rewrite of every manifest; anticipating costs one extra key.

**Say nothing about Linux support.** Rejected: the project's stated goal is
cross-platform. Staying silent would misrepresent the scope to anyone discovering the
repository, and would close a door we explicitly want open.
