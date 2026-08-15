# PLAN — dotflies

> Last updated: 14 August 2026.
> Resume point. Read this first, before `HANDOFF.md` and `docs/`.
> Vocabulary lives in [`docs/GLOSSARY.md`](docs/GLOSSARY.md), not here — this file is
> only about what is done and what is next.

## Where things stand

**Milestones 1 to 5 are done.** `doctor` and `apply` exist, 30 tests pass, and on the
personal Mac `dotflies apply kitty` installed `font-hack-nerd-font` and laid
`~/.config/kitty/kitty.conf` into the repository; `doctor kitty` now exits 0. macOS has
registered the family `Hack Nerd Font Mono`, which is the string `kitty.conf` asks for —
so the config is applied *and* honoured, not merely reported as applied.

The language tripwire below is cleared: `plan` has green tests. Next is milestone 6.


Seven ADRs, all in force. The two that were blocking are settled:

- [0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) — the language
  is **Rust**, not Go. 0004 is superseded and archived; three of its four supporting
  facts did not survive checking.
- [0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md) — the **manifest
  format is frozen**, and the five open questions that blocked every milestone are
  answered. The configuration lives at `~/.config/dotflies/`, fixed by convention — so
  dotflies has no configuration file of its own.

Nothing is blocking. Milestone 1 is open.

## ⚠️ Check the machine before trusting any state described here

The configuration was assembled by hand on the **work Mac** (`$HOME` =
`<work-home>`), which is where `HANDOFF.md` and the `_dotflies` README
were written. Their "nine live symlinks" warning describes *that* machine.

On the **personal Mac** (`<hostname>`, `$HOME` = `<personal-home>`) none of it
exists: `~/.config/kitty/` empty, zero links, no font installed. Which is a gift — that
machine *is* the fresh Mac of the v1 acceptance criterion, so the real test costs nothing
and risks nothing. `slashome/_dotflies` has since been cloned there to
`~/.config/dotflies/`, which is the source, not the applied state.

Never carry a state description across machines. Check.

## Working order

[ADR 0001](docs/adr/0001-scope-v1-to-a-minimal-verifiable-milestone.md)'s finding is
that this project dies of the size of its first push. So the code advances as a
**vertical slice**: kitty applied end to end, then the remaining mechanisms fill in
toward the nine links.

Milestone 1 is not narrowed, though — all six manifests get written, because they cost
nothing and they are what proves the format. Only the *code* is restricted to what kitty
needs.

## Milestones

### 1 — Write the six manifests before the code

`kitty`, `zsh`, `mpd`, `ncmpcpp`, `meld`, `vscode`, in `~/.config/dotflies/configs/<prog>/`.

They are the **executable specification**. If the format cannot express Meld —
directory link, generated wrapper, environment variable — it is wrong, and we find out
before a line of Rust exists. `zsh` has no versioned content yet: its manifest is the
`[[block]]` exemplar.

### 2 — `manifest`

TOML reading, validation, platform resolution. Criterion: all six manifests load, and a
`linux` entry is recognised then marked `skipped` without error
([0002](docs/adr/0002-limit-v1-to-macos.md)). A `target` with no platform key is
rejected.

### 3 — `plan` — the core

Intended versus observed state, six states: `ok`, `absent`, `drifted`, `conflict`,
`skipped`, `warn`. **No disk writes**, covered by tests. This carries all the logic of
[0003](docs/adr/0003-separate-owned-files-from-shared-files.md) and is the most testable
part of the product.

⚠️ This milestone is the language decision's tripwire. **If it has no green tests within
three weeks, the Rust bet is lost** and that fact reopens
[0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) — see its last
consequence. Not a preference, a measurement.

### 4 — `doctor` — first useful deliverable

`plan` plus printing. Nothing else. Deliberately **before** `apply`: it validates the
core against a real machine at zero risk, and it is already worth running.

`[[verify]]` entries are listed as *declared, not run in v1*
([0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md), question 4). That
label is mandatory — without it the feature lies.

### 5 — `apply`, restricted to kitty — **the slice closes here**

File link plus `[install]` through Homebrew. Systematic backup before any write;
`--dry-run` is free since `plan` exists.

`apply` **takes app names** — `dotflies apply kitty`. That is the whole of the per-machine
answer for now, and it is deliberately not a mechanism: see *Deferred decisions*.

Criterion, on the personal Mac: `doctor` reports `absent`, `apply` installs
`font-hack-nerd-font` and lays `~/.config/kitty/kitty.conf`, `doctor` then reports `ok`
— and kitty actually resolves `Hack Nerd Font Mono` instead of falling back.

**At this point the original goal is met** and everything below is filling in.

### 6 — The remaining mechanisms

Directory link (Meld), generated wrapper, managed block. This is the risky part: writing
into a file we do not own. Do not start it before `plan` is covered by tests.

### 7 — `adopt`

Move an existing file into the repository, write the manifest entry, lay the link. This
automates what was done nine times by hand on the work Mac.

### 8 — `pkgmgr`

Homebrew (formula and cask) and npm behind a trait, one implementation per manager. Above
all **no hardcoded command** — that is the 2019 dead end.

### 9 — `bootstrap`

First run: directory location, version it or not, forge, assisted creation of
`<github-username>/_dotflies`. Must work **with no remote at all**.

### 10 — Distribution

`tools/install.sh` (POSIX shell, four steps, one screenful) and publication to
`slashome/homebrew-tap`.

⚠️ **The formula dotflies needs exists in the tap for no language.** `redlight.rb` builds
with `cargo install`, which imposes the Rust toolchain and a multi-minute compile on every
user — a build in the middle of the "single command on a fresh machine" promise.
`karaokay.rb` is a virtualenv. Neither is the model.

What has to be built here: a release workflow producing `aarch64-apple-darwin` and
`x86_64-apple-darwin` archives, and a prebuilt-binary formula (`on_arm`/`on_intel`, which
a third-party tap allows even though `homebrew-core` forbids binary-only formulae).
`cargo-dist` generates and maintains both — but it is single-vendor, so budget for
hand-writing ~40 lines of Actions plus a formula template if it stalls. Then: the
generated formula must pass the `brew audit --strict --online` the tap's CI runs on every
formula (a missing `license` is the classic failure), and it must stay a **formula, not a
cask** — a cask reopens Gatekeeper on an unnotarised binary. Update `RELEASING.md`, which
only documents the source-tarball path.

Target for the end user: `brew install slashome/tap/dotflies` → ~2 MB download, no
compile, no runtime.

The bootstrap URL is settled on `main`, not `master`:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/slashome/dotflies/main/tools/install.sh)"
```

### 11 — v0.1.0

README with the Linux call for contribution
([0002](docs/adr/0002-limit-v1-to-macos.md)), then tag.

## v1 definition of done

> On a fresh Mac, `dotflies` rebuilds the nine current links from scratch, Meld
> included.

Nothing else is required. Any need discovered along the way becomes an ADR and a later
release — that is
[0001](docs/adr/0001-scope-v1-to-a-minimal-verifiable-milestone.md), and it is what the
three previous attempts lacked.

## Deferred decisions

- **Which programs a given machine manages — needs its own ADR.** `dotflies.toml` carries
  one `apps` list, versioned, therefore identical on every machine. But the personal Mac
  has kitty and nothing else, while the work Mac has all six: running `apply` with no
  argument there would install Meld, mpd, ncmpcpp and VS Code on a machine that wants none
  of them. **The repository may hold everything; the machine decides what it uses** — that
  decoupling is the answer, and it is what makes a shared repository workable at all.
  Deliberately unresolved for now: `apply` takes app names, which unblocks the work without
  inventing a mechanism. The real one has two shapes, each with a real cost — a
  hostname-keyed section in `dotflies.toml` (fully versioned, but the hostname is
  `<hostname>.home` and hostnames change), or a machine-local list (robust, but it
  reintroduces a file outside the repository, which is exactly what fixing the location
  just removed).
- **Software version differences between machines** — same trigger, different problem, and
  the two must not be conflated. Placement is handled by the platform key; a config whose
  *content* has to differ is templating, already deferred by
  [0005](docs/adr/0005-defer-templating-until-after-v1.md). Do not solve it with the
  mechanism above.

- **[ADR 0005](docs/adr/0005-defer-templating-until-after-v1.md) rests on a premise that
  is false across these two machines.** It defers templating partly because "a fresh Mac
  will have the same `$HOME`" — but the work Mac is `<work-home>` and the
  personal one `<personal-home>`. No impact on kitty, whose config holds no absolute path.
  It breaks `vscode` (`dart.flutterSdkPath`, a `yaml.schemas` entry) and `mpd` the moment
  they are applied on the personal Mac. Needs a new ADR then — not before.
- **The work Mac is now off-convention.** Its configuration sits at `~/.dotflies/`, while
  [0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md) settled the default at
  `~/.config/dotflies/`. The content is already versioned in `slashome/_dotflies`, so the
  move itself is trivial — but **nine live symlinks point into the old path** and every one
  of them has to be repointed, Meld's directory link included. Do this with `apply` once it
  exists, not by hand; it is a good first real test of it.
- `configs/homapage` — misspelled folder, and not in the six manifests. To settle.
- `karaokay` versus ncmpcpp's lyrics binding — functional overlap.
- Linux ↔ macOS migration — out of v1 per
  [0002](docs/adr/0002-limit-v1-to-macos.md).
- Secrets (SSH/GPG keys, tokens) — out of v1.
- The `dotflies-rust-legacy` checkout next to this repository is the May attempt, with no
  remote and two commits. Five specific pieces are lifted from it
  ([0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md)); delete it
  once they are in.

## Resuming work

```sh
cd ~/workspace/projects/slashome/dotflies
```

Then, in a Claude Code session:

```
Read PLAN.md, then docs/adr/0006 and 0007.

Check the machine first — the state described in HANDOFF.md is the work Mac's, not
necessarily this one.

Continue at the current milestone. The format is frozen: if something cannot be
expressed, that is a new ADR, not an improvisation in the manifest.
```
