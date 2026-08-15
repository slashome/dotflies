# Glossary

Every term dotflies uses, in two sentences, plus **why it exists here** — because the
general definition is rarely the useful part. Each entry points at a real case from the
configuration this project already manages.

This file explains vocabulary, not progress. For where the work stands, read
[`PLAN.md`](../PLAN.md).

---

### Manifest

A TOML file, one per program, declaring **what goes where and by which mechanism** —
`~/.config/dotflies/configs/kitty/manifest.toml` for kitty.

*Why here:* the alternative is deducing the destination from where you filed the source,
and that cannot express two different paths for two operating systems. VS Code lives at
`~/Library/Application Support/Code/User/settings.json` on macOS and
`~/.config/Code/User/settings.json` on Linux; one folder tree can only produce one of
them. See [ADR 0007](adr/0007-adopt-a-declarative-per-program-manifest.md).

### Owned file, shared file

An **owned** file would not exist without you — `kitty.conf`, `mpd.conf`. A **shared**
file is one the program writes to as well — `.zshrc`, VS Code's `settings.json`.

*Why here:* it decides the mechanism. Owned files get a link; shared files get a managed
block, because taking the whole file would dispossess you of the rest of it. This is the
single distinction [ADR 0003](adr/0003-separate-owned-files-from-shared-files.md) is
built on.

### File link

A symlink at the target path pointing back at the file in your repository. Editing either
path edits the same bytes, which is what keeps your repository the live source of truth.

*Why here:* it is the cheap, correct answer for owned files, and dotflies uses nothing
fancier when it applies.

### Directory link

A symlink on the **parent folder** instead of the file inside it.

*Why here:* Meld. When Meld closes, GLib writes its preferences to a temporary file and
`rename`s it over the old one. `rename` replaces the directory entry — so a file link
becomes a regular file written by GLib, your link is gone, and your preferences stop
reaching the repository, **with no error printed**. Linking the parent folder puts the
link one level above where the rename happens, out of its reach. That is why the Meld
manifest carries `kind = "directory"`.

### Managed block

A section inserted into a file you do not own, fenced by markers:

```sh
# >>> dotflies:zsh >>>
…
# <<< dotflies:zsh <<<
```

The block is replaced wholesale, never merged line by line, and the rest of the file
stays yours.

*Why here:* `~/.zshrc` already holds oh-my-zsh, a `PATH` line you wrote, and a pnpm
block. dotflies only has a claim on a few lines of it.

### Wrapper

A small generated script in `~/.local/bin` that sets environment variables and then
`exec`s the real program.

*Why here:* `Meld.app` ships no dconf, so without `GSETTINGS_BACKEND=keyfile` GLib falls
back to an in-memory backend and every preference is lost on close — silently. The
variable has to be set *before* Meld starts, and no config file can do that. The wrapper
must also win over `/opt/homebrew/bin/meld` in `PATH`, which is what the zsh block is
for. Never modify the launcher the package manager installed
([ADR 0003](adr/0003-separate-owned-files-from-shared-files.md)).

### Plan, and apply

Two passes. **plan** looks at everything and builds a list of intended changes —
*writing nothing at all*. **apply** takes that list and executes it.

*Why here:* with decision and writing interleaved in one loop, `--dry-run` means an
`if` in front of every write, and forgetting one means it writes anyway. Separated,
`--dry-run` is "run plan and stop", `doctor` is "run plan and print", and the decision
logic can be tested without creating a single file. One mechanism, three features.

### The six states

What plan can conclude about one entry: `ok` (conforms), `absent` (nothing there, safe to
lay), `drifted` (our link was replaced, or our block was hand-edited), `conflict`
(something exists and is not ours), `skipped` (declared for another platform), `warn`
(an absolute path was found inside a managed file).

*Why here:* they are the whole contents of `plan/`, and the reason it is the most testable
part of the product.

### Drift

Our **block** or our **wrapper** is there and its content has changed. Reported, **never
silently recreated** — overwriting would destroy whatever the program or you put there.
Repair takes `--force`.

*Why here:* a link can never be drifted, only `conflict`
([ADR 0008](adr/0008-reserve-drift-for-what-can-be-proved-ours.md)). Drift asserts *this
was ours and changed*, and only a block's markers or a wrapper's generated header prove
authorship. A plain file at a link's target proves nothing — see *Conflict*.

### Conflict — and the limit behind it

A file already exists at the target and it is not our link.

*Why here:* **dotflies cannot know who created it.** The filesystem stores no author — the
owner is you either way, and nothing distinguishes "the program wrote its defaults on
first launch" from "you typed it by hand". So the only reliable question is *is this my
symlink?*, and everything answering no is treated identically: it exists, it is not mine,
I do not touch it, I report it. `adopt` is the explicit gesture that supplies the
information the system cannot.

### `adopt`

Take a file that already exists in `$HOME` under management: **move** the real file into
the repository, **lay a link** in its place, **write** the manifest entry.

*Why here:* the order matters. Move-then-link means the file exists somewhere at every
instant — a crash in between costs you a link, not data. Copy-then-delete can lose the
content outright. Two refusals guard it: already a symlink (it is managed already), and
already present in the repository (two versions of one file is your call, not the
program's).

### `doctor`

Run plan, print it, change nothing.

*Why here:* it ships **before** `apply` on purpose. It validates the core against a real
machine at zero risk, and it is already worth running on its own — the acceptance test is
that it correctly diagnoses all nine existing links, Meld's directory link included.

### Platform key

Every `target` in a manifest names its platform: `target.darwin`, `target.linux`. A target
without one is rejected at validation.

*Why here:* v1 resolves `darwin` only, and a `linux` entry is recognised then reported as
`skipped` rather than erroring. The ceremony is deliberate — the 2019 shell version
hardcoded `trizen` with no notion of platform, and became useless the day the system
changed ([ADR 0002](adr/0002-limit-v1-to-macos.md)).

### The three locations

Two, not three. **The manager** — this repository, a binary in your package manager's
`bin`. **Your configuration** — `~/.config/dotflies/`, a clone of `<username>/_dotflies`,
and entirely optional: dotflies works with no remote at all.

*Why here:* the location is fixed by convention, so dotflies has **no configuration file
of its own** — nothing to point anywhere, nothing machine-local to keep out of the
versioned tree. Want it somewhere else? Symlink `~/.config/dotflies` there.

*Why here:* that indirection is what lets your directory sit wherever you want, and lets
versioning be a separate decision. Nothing in the code may assume a path.

### Templating

Generating a file's *content* from a source plus variables, instead of linking it as-is.
**Not in v1** ([ADR 0005](adr/0005-defer-templating-until-after-v1.md)).

*Why here:* two real cases wait for it. `mpd.conf` declares `type "osx"`, which would be
alsa or pipewire on Linux. And `settings.json` hardcodes an absolute `$HOME` — which
already breaks across the two machines this configuration has to serve, since their
usernames differ. The platform key handles *placement*; content is a different problem.
