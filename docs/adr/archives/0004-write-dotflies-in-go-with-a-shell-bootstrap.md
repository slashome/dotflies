# 0004 — Write dotflies in Go, with a shell bootstrap

## Status

Superseded — 2026-08-14 — by
[0006](../0006-write-dotflies-in-rust-with-a-shell-bootstrap.md).

Three of the four facts this ADR rested on did not survive checking. The tap has no
goreleaser and no binary-release path **for any language**, so "well trodden" was wrong
— though correcting it favours neither language. The daemon cited here as the
structural model is itself Rust, so this ADR's own citation did not support its
conclusion. And the "wants tests" argument settles compiled-versus-shell, not
Go-versus-Rust.

What stays true is everything about the **shell bootstrap**: those four steps are carried
over into 0006 unchanged, and they are why this file is worth rereading.

## Context

The intended entry point is a single command:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/slashome/dotflies/master/tools/install.sh)"
```

It must install dotflies, install the software, and apply its configuration.

The choice of language was called into question by history: a 2018 attempt announced
"A dotfile manager written in GO" and never got past its README, while a 2019 dotfiles
repository, in shell, actually served.

Checked: the 2018 attempt has **exactly one commit**, `Initial commit` of 24 August 2018,
containing a one-line README. **No Go was ever written.** The repository is therefore
not evidence about Go — it is evidence about the absence of scope, already addressed
by [0001](../0001-scope-v1-to-a-minimal-verifiable-milestone.md).

Facts in favour of Go in this specific context:

- Our Homebrew tap already exists and is active. The goreleaser → Homebrew formula
  path is well trodden.
- The product is not a sequence of chained commands. It compares states, detects
  broken links, parses managed blocks, produces a report and proposes repairs
  ([0003](../0003-separate-owned-files-from-shared-files.md)). That kind of logic wants
  tests, and tests badly in shell.
- The riskiest mechanism in the product — writing into a file we do not own —
  deserves unit tests over awkward cases.
- An existing macOS + Linux daemon of ours is a precedent for homegrown cross-platform
  structure.

One constraint is unavoidable: **the bootstrap script cannot be Go**, since it runs
before the binary exists.

## Decision

**dotflies is written in Go.** The binary ships through our Homebrew tap.

**`tools/install.sh` stays POSIX shell**, and its scope is strictly bounded:

1. detect OS and architecture;
2. check for and install Homebrew if missing;
3. install the `dotflies` binary (through the tap, or by fetching a release);
4. hand over to `dotflies bootstrap`.

**No business logic in shell.** The interactive bootstrap — where the dotflies
directory lives, whether to version it, which forge, creating the remote repository
proposed as `<github-username>/_dotflies` — is Go, inside `dotflies bootstrap`.

The bootstrap script must stay readable in one screenful. It is code users are invited
to execute straight from a URL; they must be able to read it first.

## Consequences

- A single static binary, no runtime dependency, installable through the tap already
  in place.
- The verification logic becomes testable, which is decisive for the "write into a
  shared file" part.
- Real cost: for a scope this small, Go is more code than the shell equivalent. That
  is accepted — shell would have hit a ceiling at the first non-trivial state logic.
- There will always be **two languages** in the repository. The boundary must stay
  sharp, or shell will reclaim ground at every convenience.
- A Linux contributor ([0002](../0002-limit-v1-to-macos.md)) inherits a typed, tested
  codebase rather than a script to reverse-engineer.
- **To settle when the repository is created**: the bootstrap URL points at the
  `master` branch. GitHub creates repositories on `main` today. Either create a
  `master` branch or fix the URL — otherwise the install command returns a 404.

## Alternatives considered

**All shell, as in 2019.** Rejected: state comparison, managed-block parsing and the
`doctor` report get unreadable and untestable fast. The 2019 version only laid links
and called a package manager — a distinctly narrower scope than v1.

**Python.** Rejected: requires an interpreter and environment management on the target
machine, which contradicts the goal of a single command on a fresh system.

**Rust.** Rejected: no decisive gain here, and the goreleaser → Homebrew chain is more
direct in Go.

**An existing tool (chezmoi, Stow, nix-darwin).** Covered in the design notes. Stow
only handles owned files; chezmoi copies instead of linking and breaks the editing
loop; nix-darwin is a bigger undertaking than the project itself.
