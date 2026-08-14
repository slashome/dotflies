# PLAN — dotflies

> Last updated: 14 August 2026.
> Resume point. Read this first, before `HANDOFF.md` and `docs/`.

## Where things stand

No code. The scope is settled (five ADRs), the manifest format is proposed but **not
approved**. The real configuration already exists, assembled by hand: nine live
symlinks under `~/.dotflies/configs/`, which serve as the target.

## What is blocking — settle this first

Five open questions in
[`docs/manifest-format-proposal.md`](docs/manifest-format-proposal.md#7-open-questions).
Nothing can start before they are answered.

| # | Question | Recommendation |
|---|---|---|
| 1 | VS Code: watched link or managed block? | **watched link** — marker-delimited blocks inside JSON are unsafe |
| 2 | Wrapper generated or versioned script? | **generated** from `exec` + `env` |
| 3 | Manifests with the user or inside dotflies? | **with the user**, they reference the user's files |
| 4 | Is `[[verify]]` in v1? | **undecided** — it is what would have caught the Meld bug, but it is extra scope. Owner's call. |
| 5 | Default user directory: `~/_dotflies` or `~/.dotflies`? | to fix |

Once settled, the proposal becomes **ADR 0006** and milestone 1 starts.

## Milestones

Ordered so that something useful ships early, and so that risky code lands after
testable code.

### 1 — Freeze the format, write the manifests before the code

Write the six real manifests: `kitty`, `zsh`, `mpd`, `ncmpcpp`, `meld`, `vscode`.

They are the **executable specification**: if the format cannot express Meld —
directory link, wrapper with an environment variable — then it is wrong, and we find
out before a single line of Go exists.

### 2 — `internal/manifest`

TOML reading, validation, platform resolution. Criterion: all six manifests load, and
a `linux` entry is recognised then marked `skipped` without error
([ADR 0002](docs/adr/0002-limit-v1-to-macos.md)).

### 3 — `internal/plan` — the core

Intended vs observed state, six states: `ok`, `absent`, `drifted`, `conflict`,
`skipped`, `warn`. **No disk writes.** This is the most testable part of the product,
and it carries all the logic from
[ADR 0003](docs/adr/0003-separate-owned-files-from-shared-files.md).

### 4 — `doctor` — first useful deliverable

`plan` plus printing. Nothing else.

Deliberately **before** `apply`: it validates `plan/` against the reality of the
machine at zero risk, and it already gives you a tool worth running. Criterion:
`doctor` correctly diagnoses the nine current links, including Meld's directory link.

### 5 — `apply`

Links (file and directory), generated wrapper, managed block. Systematic backup before
any write. `--dry-run` is free, since `plan` already exists.

This is the risky milestone: writing into a file we do not own. Do not start it before
`plan` is covered by tests.

### 6 — `adopt`

Move an existing file into the repository, write the manifest entry, lay the link.
This automates what has been done nine times by hand.

### 7 — `internal/pkgmgr`

Homebrew (formula + cask) and npm. An interface with one implementation per manager —
above all, no hardcoded command. That is the 2019 dead end.

### 8 — `bootstrap`

First run: directory location, version it or not, forge, assisted creation of
`<github-username>/_dotflies`. Must work **with no remote repository at all**.

### 9 — Distribution

`tools/install.sh` (POSIX shell, four steps, readable in one screenful), goreleaser,
publishing to `slashome/homebrew-tap`.

⚠️ The planned bootstrap URL points at the `master` branch, while GitHub creates
repositories on `main`. **Settle it here**, or it 404s:

```
https://raw.githubusercontent.com/slashome/dotflies/master/tools/install.sh
```

### 10 — v0.1.0

README with the Linux call for contribution
([ADR 0002](docs/adr/0002-limit-v1-to-macos.md)), then tag.

## v1 definition of done

> On a fresh Mac, `dotflies` rebuilds the nine current links from scratch, Meld
> included.

Nothing else is required to call v1 finished. Any need discovered along the way
becomes an ADR and a later release — that is
[ADR 0001](docs/adr/0001-scope-v1-to-a-minimal-verifiable-milestone.md), and it is what
the two previous attempts lacked.

## Deferred decisions

- Fate of `~/.dotflies/configs/`: migrate to the future `_dotflies` repository, or stay
  local until the manager exists.
- `configs/homapage` vs `configs/homepage` — duplicate, with a typo.
- `karaokay` vs ncmpcpp's lyrics binding — functional overlap.
- Linux ↔ macOS migration — out of v1 per
  [ADR 0002](docs/adr/0002-limit-v1-to-macos.md).
- Secrets (SSH/GPG keys, tokens) — out of v1.

## Resuming work

```sh
cd ~/workspace/projects/slashome/dotflies
```

Then, in a Claude Code session:

```
Read PLAN.md, then all of docs/adr/, then docs/manifest-format-proposal.md.

We resume at the blocker: the five open questions. Put them to me one at a time
with your recommendation, then turn the proposal into ADR 0006 and start
milestone 1 — writing the six real manifests before any code.

Keep your explanations separate from your questions. No Go until the six
manifests are approved.
```

⚠️ Before touching anything: **nine live symlinks point into `~/.dotflies/configs/`**
(exact list in [`HANDOFF.md`](HANDOFF.md#3-current-state--to-absorb)), one of them a
**directory** link for Meld. Breaking them breaks a configuration in daily use.
