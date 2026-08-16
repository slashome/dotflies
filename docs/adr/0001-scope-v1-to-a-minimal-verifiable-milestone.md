# 0001 — Scope v1 to a minimal, verifiable milestone

## Status

Accepted — 2026-08-14

## Context

dotflies has been attempted twice already, and never started:

- A first attempt (24 August 2018) — "A dotfile manager written in GO". One commit,
  `Initial commit`, containing a one-line README. Not a single line of code was ever
  written.
- A second attempt (April 2025) — "All my dotfly profiles". Empty in the strict sense:
  no branch, no commit.

Meanwhile a dotfiles repository from 2019 — plain shell scripts around a flat
manifest — served for years.

The deciding factor was not the language, nor technical ambition. It was **how big
the first push was**. Neither dead attempt contains a single decision: they never had
a scope.

A real configuration already exists, assembled by hand: nine live symlinks covering
kitty, Meld, mpd, ncmpcpp and VS Code. It supplies a set of concrete cases, including
the awkward ones — a *directory* symlink for Meld's GSettings keyfile, and a
`~/.local/bin` wrapper to inject `GSETTINGS_BACKEND`.

## Decision

v1 does **adopt, link and verify**, with an interactive bootstrap.

**In scope:**

- bootstrap on first run: where the dotflies directory lives, whether to version it,
  which forge, assisted creation of the remote repository;
- the manager's own configuration in `$HOME/.config/dotflies/`, declaring where the
  user's directory lives;
- `adopt`: take an existing config file into the repository and put a link in its
  place, without loss;
- symlinks for owned files;
- managed blocks for shared files (see [0003](0003-separate-owned-files-from-shared-files.md));
- wrappers in `~/.local/bin` when a setting is only reachable through an environment
  variable;
- installing software through Homebrew and npm;
- `doctor`: verifying that what was applied is still in place.

**Out of scope:**

- Linux (see [0002](0002-limit-v1-to-macos.md));
- templating / conditional content (see [0005](0005-defer-templating-until-after-v1.md));
- settings held in opaque stores (`defaults write`, dconf);
- secret management;
- multiple profiles per machine.

**v1 acceptance criterion** — not negotiable; it is the definition of done:

> On a fresh Mac, `dotflies` must rebuild the nine current links from scratch, Meld
> included — which means handling a **directory** symlink and installing the
> `~/.local/bin/meld` wrapper.

## Consequences

- The scope is measurable. At any point it is clear whether v1 is finished.
- Meld is the hardest acceptance case. That is deliberate: it combines a directory
  symlink, an environment wrapper, and a setting that fails silently.
- Everything out of scope must still be **designed for**, without being implemented.
  In particular the manifest format carries a platform key from v1.
- A need discovered along the way does not join v1. It becomes an ADR and a later
  release.

## Alternatives considered

**Cover opaque stores (`defaults`, dconf) in v1.** Rejected: it is the most expensive
piece, it requires an inverse capture mechanism to avoid drift, and none of the nine
current cases needs it.

**Start with a homegrown GNU Stow.** Rejected: it only covers owned files, so it
handles neither VS Code, nor `.zshrc`, nor the Meld wrapper — three of the five
programs already in service.

**Scope nothing and improvise.** Rejected: that is precisely what produced `dm` and
`profiles`.
