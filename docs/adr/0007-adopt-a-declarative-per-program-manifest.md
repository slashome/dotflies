# 0007 — Adopt a declarative per-program manifest, and settle where the user's directory lives

## Status

Accepted — 2026-08-15. Promotes
[`docs/manifest-format-proposal.md`](../manifest-format-proposal.md), which it
replaces as the reference.

## Context

The manifest format is the first brick: `plan`, `apply`, `doctor` and `adopt` all read
it, so nothing can be written before it is frozen. The proposal left **five questions
open**, and those five questions blocked every milestone.

Three prior decisions constrain the answer.
[0002](0002-limit-v1-to-macos.md) makes a platform key mandatory from v1.
[0003](0003-separate-owned-files-from-shared-files.md) requires the mechanism to be
*declared, never guessed*, and integrity to be checked on every run.
[0005](0005-defer-templating-until-after-v1.md) forbids content substitution, so a
manifest describes *placement*, never *content*.

There is also a negative example, close at hand. The May attempt described in
[0006](0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) had no manifest: it walked
a mirror `home/` tree and derived every target from the tree shape. That is what
"guessed" looks like in practice — and it made a platform key structurally
inexpressible, because one tree yields exactly one path. The manifest is the mechanism
that buys back what that model cannot say.

## Decision

### Two locations, never conflated

| What | Where | Versioned |
|---|---|---|
| **The manager** | `slashome/dotflies` — a Rust binary shipped through the Homebrew tap | yes, that repository |
| **The user's configuration** | `~/.config/dotflies/` — a clone of `<username>/_dotflies` | yes, its own repository |

There is no third location and **no configuration file for dotflies itself**. That is the
point of the next section.

### Question 5 — the configuration lives at `~/.config/dotflies`, fixed by convention

Settled on the evidence of the target machine rather than on principle, because two
plausible answers were in play and only one of them has precedent here.

`~/.config/` holds fifteen entries on this machine against three in `~/.local/share/`, so
`.config/<program>` is the dominant convention. But the decisive case is **opencode**,
which already makes exactly the split dotflies needs:

| | Size | Holds |
|---|---|---|
| `~/.config/opencode` | 61 MB | `package.json`, `package-lock.json`, `node_modules` |
| `~/.local/share/opencode` | 2.2 MB | `opencode.db`, `log`, `snapshot`, `repos` |

An **editable, versionable project tree** in `.config`; **machine-local runtime state**
elsewhere. The user's dotflies directory is the first of those two. Reading it as "data"
because dotflies consumes it is the tool's internal view, not the user's — to the user it
is configuration, which is what `XDG_CONFIG_HOME` is for.

Two alternatives were rejected on the same evidence. **`~/.local/share/dotflies`**: no
precedent for a user-edited tree there, and it buries a git repository the user has to
`cd` into. **A bare `~/.config/dotflies.toml`** next to the directory: nothing on this
machine puts a loose file directly in `~/.config` — the four that appear are macOS `._`
resource-fork residue.

**The location is fixed by convention, not configurable — and this reverses part of
[0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md).** That ADR scoped in a
bootstrap question asking "where should the dotflies directory live", and a config file
for the manager "declaring where the user's directory lives". Both are dropped.

The reason is that the config file only ever existed to serve the choice. Make the
location a convention and the file has no content left: it cannot live *inside* the
repository, since it would be a machine-local file in a versioned tree — and every
alternative home for it (`~/.local/state/dotflies/`, a loose `~/.config/dotflies.toml`)
means inventing a second location to hold twenty bytes. Removing the choice removes the
file, the second location, and the bootstrap question in one move.

The escape hatch costs nothing and needs no code: anyone wanting the repository elsewhere
symlinks `~/.config/dotflies` to it. That is one command, and it is the same mechanism
dotflies already uses for everything else.

This also dissolves a question that looked like it needed answering: **there is no
`_dotflies` versus `.dotflies` problem.** On GitHub a user's repositories have no nesting
— they all sit side by side under `slashome/`, so a program and its content repository
cannot share a name, and the content one takes `_`, consistent with `ariane` / `_ariane`.
Locally there is nothing to disambiguate, because the program is a binary in the package
manager's `bin`, not a directory. So the directory is simply `dotflies`, under `.config`
like everything else. Local directory name and repository name routinely differ anyway:
`~/.oh-my-zsh` comes from `ohmyzsh/ohmyzsh`.

Backups taken before writing into a file dotflies does not own are a separate matter, left
open on purpose: nothing writes into a shared file until the managed-block mechanism, and
the backup can as easily sit beside the file it protects. Deciding it now would be
inventing a location before there is anything to put in it.

### Layout of the user's directory

The existing `configs/<program>/` convention is kept, with each manifest sitting next
to the files it describes:

```
~/.config/dotflies/
├── dotflies.toml                    ← root manifest: inventory and order
└── configs/
    ├── kitty/
    │   ├── manifest.toml
    │   └── kitty.conf
    ├── meld/
    │   ├── manifest.toml
    │   ├── gtk-3.0/gtk.css
    │   ├── gtksourceview-4/styles/gruvbox-dark.xml
    │   └── glib-2.0/settings/keyfile
    ├── mpd/
    ├── ncmpcpp/
    ├── vscode/
    └── zsh/
```

```toml
# ~/.config/dotflies/dotflies.toml
version = 1

apps = ["kitty", "zsh", "mpd", "ncmpcpp", "meld", "vscode"]

[remote]                                          # optional
url = "git@github.com:slashome/_dotflies.git"
```

With no `[remote]`, dotflies runs locally and never mentions Git.

### A program's manifest

Three mechanisms, **always declared, never inferred**
([0003](0003-separate-owned-files-from-shared-files.md)): `link`, `block`, `wrapper`.
`source` is resolved relative to the manifest's own directory.

```toml
name        = "kitty"
description = "Terminal"

[install]
darwin.brew_cask = ["kitty", "font-hack-nerd-font"]
linux.pacman     = ["kitty"]

[[link]]
source        = "kitty.conf"
target.darwin = "~/.config/kitty/kitty.conf"
target.linux  = "~/.config/kitty/kitty.conf"
```

**A `target` with no platform key is rejected at validation.** That is verbose when the
path is identical on both systems, and deliberately so:
[0002](0002-limit-v1-to-macos.md) rules out the implicit single path, which is precisely
the 2019 dead end and the May one.

`[[link]]` takes `kind = "file"` (default) or `kind = "directory"`. The directory form
is not a convenience — it is the only way to survive a program that rewrites its config
through an atomic `rename`, which destroys a file link on first close.

Fields common to every entry: `note`, free text, for recording *why* a
counter-intuitive setting is there. It is not decoration; it is the only place that
reasoning will be reread.

`[install]` keys are enumerated, not open-ended — an unknown one is a validation error,
so a typo cannot silently install nothing:

| Key | Platform | Runs |
|---|---|---|
| `brew_formula` | `darwin` | `brew install <pkg>` |
| `brew_cask` | `darwin` | `brew install --cask <pkg>` |
| `npm_global` | any | `npm install -g <pkg>` |
| `pacman`, `apt`, … | `linux` | declared, `skipped` in v1 ([0002](0002-limit-v1-to-macos.md)) |

A platform with nothing to install simply omits its key. That is not the same as a
`target`, which must always carry one.

**The format cannot express VS Code extension installation**, and that is a known gap
rather than an oversight — see *Consequences*.

### Question 1 — VS Code keeps a watched link

`settings.json` stays a **file link**, and its disappearance is reported as `drifted`.
Slicing a JSON object with textual markers is unsafe even though VS Code tolerates
comments, and the directory-link workaround used for Meld is unavailable here:
`Code/User/` also holds `globalStorage/`, `workspaceStorage/` and `History/`, all large
and volatile. If the link turns out to be destroyed in practice, that observation
becomes a new ADR — not a pre-emptive design.

### Question 2 — wrappers are generated, not versioned

A `[[wrapper]]` declares `exec` and `env`; dotflies writes the script. This keeps
`bin/meld` out of the configuration repository and keeps the *intent* — "inject this
variable" — declarative rather than buried in shell.

```toml
[[wrapper]]
target.darwin = "~/.local/bin/meld"
exec.darwin   = "/Applications/Meld.app/Contents/MacOS/Meld"
env           = { GSETTINGS_BACKEND = "keyfile" }
note          = "The bundle ships no dconf: without the keyfile backend, no preference persists."
```

The day a wrapper needs more than `exec` plus `env`, that is a new ADR.

### Question 3 — manifests live with the user

In `~/.config/dotflies/configs/<program>/manifest.toml`, not inside this repository. They
reference the user's files and the user's platforms; they are configuration, not
product. A catalogue of ready-made manifests shipped with dotflies stays possible
later, and would not conflict.

### Question 4 — `[[verify]]` is declared and validated in v1, but not executed

The middle answer, and the reason is [0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md):
out-of-scope needs must be **designed for without being implemented**.

So v1 parses `[[verify]]`, validates its fields, and `doctor` lists each one as
*declared, not run in v1*. No execution engine ships. The format is frozen — a
manifest written today stays valid — and the scope does not grow.

```toml
[[verify]]
kind        = "file_contains"
path.darwin = "~/Library/Application Support/org.gnome.Meld/glib-2.0/settings/keyfile"
contains    = "prefer-dark-theme=true"
message     = "Meld's dark theme is not active — is the keyfile backend in place?"
```

Two kinds, and only two — `file_exists` and `file_contains`. Both are checkable without
a terminal, which rules out the obvious-looking alternatives: `kitty +list-fonts` and
`kitty --debug-config` both need a tty and fail under a `doctor` run. A `kind` outside
this list is a validation error; widening it is a new ADR.

This is the one answer that will look wrong in hindsight if the Meld class of bug
recurs before the engine lands: the check that would have caught it will be sitting in
the manifest, unread. Accepted, with that risk stated.

### Managed blocks

```toml
[[block]]
source        = "zshrc.block"
target.darwin = "~/.zshrc"
target.linux  = "~/.zshrc"
marker        = "dotflies:zsh"
comment       = "#"
position      = "end"
```

Produces, inside a file that otherwise remains the user's:

```sh
# >>> dotflies:zsh >>>
export PATH="$HOME/.local/bin:$PATH"
# <<< dotflies:zsh <<<
```

A block is replaced wholesale, never merged line by line.

### Plan, then apply

**Every command computes a plan first** — the differences between intended and observed
state — and only then applies anything, if at all.

| Command | Does what |
|---|---|
| `dotflies doctor` | computes the plan, prints it, **writes nothing** |
| `dotflies apply --dry-run` | same |
| `dotflies apply` | computes the plan, applies it |
| `dotflies adopt <file>` | moves the file into the repository, writes the manifest entry, lays the link |

One mechanism buys `--dry-run`, `doctor` and the every-run verification
[0003](0003-separate-owned-files-from-shared-files.md) requires. That is what keeps v1
small. Every plan entry carries a state:

| State | Meaning | What `apply` does |
|---|---|---|
| `ok` | conforms | nothing |
| `absent` | link, block or wrapper missing | lays it |
| `drifted` | our block or wrapper is there and its content changed — see [0008](0008-reserve-drift-for-what-can-be-proved-ours.md) | **nothing** — reports, requires `--force` |
| `conflict` | a file exists at the target and is not ours | back up, then lay — or refuse |
| `skipped` | entry for another platform | nothing, explicit message ([0002](0002-limit-v1-to-macos.md)) |
| `warn` | absolute path containing the current `$HOME` found in a managed file | nothing ([0005](0005-defer-templating-until-after-v1.md)) |

### Crate layout

```
dotflies/
├── Cargo.toml
├── src/
│   ├── main.rs           entry point
│   ├── cli.rs            clap derive definitions
│   ├── paths.rs          `~/.config/dotflies` and tilde expansion — a constant, not a config
│   ├── manifest.rs       reading, validation, platform resolution
│   ├── plan.rs           intended vs observed state  ← the core
│   ├── apply.rs          executes a plan; decides nothing
│   ├── blocks.rs         finding and rendering a marker-delimited block
│   ├── wrapper.rs        generating a wrapper, and proving one is ours
│   ├── pkgmgr.rs         brew, npm — a trait, one implementation per manager
│   └── ui.rs             reporting
├── tools/install.sh      shell bootstrap, four steps ([0006](0006-write-dotflies-in-rust-with-a-shell-bootstrap.md))
├── docs/adr/
└── README.md             including the Linux call for contribution ([0002](0002-limit-v1-to-macos.md))
```

**A module is a file until it earns a directory.** `foo.rs` and `foo/mod.rs` are the same
module path in Rust, so promoting one later moves a file and changes no import anywhere.
Seven directories holding one file each would be ceremony bought on credit, and this
project's documented failure mode is paying for structure before it is needed
([0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md)). Only `plan` and `manifest`
are near the size where the split pays.

Two modules here were not foreseen, and both exist because `plan` needs to answer a
question `apply` will later act on — so the logic has to be shared, and pure:

- **`blocks`** finds and renders a marker-delimited section. `plan` reads the markers to
  decide, `apply` will write them.
- **`wrapper`** renders the exact bytes a wrapper would contain, and answers *is this one
  ours?* through its generated header. That header is load-bearing: it is what lets a
  wrapper report `drifted` where a plain link can only report `conflict`.

Two modules named in earlier drafts are deliberately absent:

- **`verify`** has no module because it has no engine. Question 4 above keeps `[[verify]]`
  parsed and reported but never executed in v1, so its parsing lives in `manifest.rs` with
  every other field, and its reporting in `plan.rs` and `ui.rs` with every other output. A
  module holding no behaviour would advertise one.
- **`bootstrap`** is a later milestone and simply not written.

Go's `internal/` has no Rust equivalent and needs none: a binary-only crate makes every
module private by default, which is the same guarantee by construction.

**`plan/` never writes to disk.** That is what makes the riskiest part of the product —
writing into a file we do not own — testable without side effects, and it is the single
most important line in this ADR.

### Why TOML

Comments are native and survive round-trips, which matters because the 2019
`program_list.sh` that actually served was a **heavily commented** array — that shape is
preserved deliberately. No indentation traps, explicit typing, and `[[link]]` reads well
four in a row. YAML would be acceptable. JSON is ruled out: no comments.

## Consequences

- The format is frozen, so the six real manifests can be written **before** any code,
  and they become the executable specification. If it cannot express Meld — directory
  link, generated wrapper, environment variable — it is wrong, and that is discovered
  at zero cost.
- `plan` being pure makes the core of the product testable, and `doctor` ships before
  `apply` — a useful tool at no risk.
- **Cost: ceremony.** Every target is written twice, once per platform, for a v1 that
  resolves only one of them. That is paid on every entry of every manifest, forever, to
  keep [0002](0002-limit-v1-to-macos.md)'s door open.
- **Cost: three mechanisms to implement**, where symlinks alone would have been one.
  Unavoidable — three of the five programs already in service do not fit a symlink.
- **`[[verify]]` will be dead weight in v1**: parsed, validated, displayed, never run.
  Users can write checks that do nothing, and that is a real trap for anyone but the
  author. `doctor` must label them explicitly, or the feature lies.
- **One location, one file fewer, one bootstrap question fewer.** The cost is that a user
  who wants the repository elsewhere has to symlink rather than answer a prompt — and it
  reverses part of [0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md), which is
  recorded above rather than left to be discovered.
- **`config.rs` disappears.** [0006](0006-write-dotflies-in-rust-with-a-shell-bootstrap.md)
  lists it among the pieces lifted from the May attempt; with a fixed location it reduces
  to a constant and a tilde expansion. One of the five salvaged items was salvaging a
  problem that no longer exists.
- Deciding on a watched link for VS Code accepts a known risk rather than designing
  around it. If `settings.json` does get replaced by an atomic rewrite, the answer is a
  new ADR, and until then the cheaper design stands.
- **`[install]` cannot express VS Code extensions**, and writing the six manifests is
  what surfaced it — which is the whole point of doing that before any code.
  `configs/vscode/extensions.txt` is a captured list of 38 extensions with no mechanism
  to reinstall it. Homebrew and npm do not cover `code --install-extension`, and
  [0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md) names only those two. So
  the list stays captured and inert in v1, recorded as such in the manifest, and the
  third channel becomes its own ADR. Deliberately not smuggled into this one.
- **Choosing generated wrappers (question 2) makes a versioned file redundant.**
  `configs/meld/bin/meld` exists in `_dotflies` as a hand-written script; the
  `[[wrapper]]` entry now reproduces it from `exec` plus `env`. Keeping both would mean
  two sources of truth for the same wrapper, so the file has to go — and its comment,
  which is where the dconf reasoning was written down, must survive as the entry's
  `note` first.

## Alternatives considered

**Derive targets from a mirror `home/` tree**, GNU Stow style, as the May attempt did.
Rejected: it cannot express a platform key at all, it guesses where
[0003](0003-separate-owned-files-from-shared-files.md) demands a declaration, and it
handles only owned files — which excludes `.zshrc`, VS Code and the Meld wrapper.
Convenient at first, structurally incapable of the acceptance criterion.

**One single manifest for every program** instead of one per program. Rejected: it grows
to hundreds of lines, and it separates a manifest from the files it describes — so
`adopt` would edit a distant central file and diffs would stop being readable per
program. The 2019 `program_list.sh` was that single file, and it worked at that scope;
it does not survive three mechanisms and a platform key.

**Infer the mechanism** — a symlink by default, a block if the target already exists.
Rejected: this is exactly what [0003](0003-separate-owned-files-from-shared-files.md)
forbids. The cost of guessing wrong is silent destruction of a user's configuration,
and the only distinguishing information — who owns the file — is not observable from
disk.

**YAML instead of TOML.** Rejected narrowly: acceptable, and better at nesting, but
`target.darwin` reads worse in it, and its indentation traps are a poor property in a
file people hand-edit rarely and under pressure.

**Ship the manifests with dotflies** rather than with the user. Rejected as the default:
they name the user's paths and platforms. Kept as a possible later catalogue, which this
format accommodates without change.
