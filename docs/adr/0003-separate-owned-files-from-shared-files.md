# 0003 — Separate owned files from shared files, and verify managed blocks on every run

## Status

Accepted — 2026-08-14

## Context

Not every configuration lends itself to a symlink. The nine cases adopted by hand
split according to **who owns the file**:

| Case | Real examples | Observation |
|---|---|---|
| **Owned file** — it would not exist without us | `kitty.conf`, `mpd.conf`, `gruvbox-dark.xml`, the `meld` wrapper | A symlink fits perfectly. |
| **Shared file** — the program writes to it too | VS Code's `settings.json`, `.zshrc` | A symlink is fragile, or simply wrong. |

Shared files raise two distinct problems:

1. **A file symlink is destructible.** An atomic rewrite (`write temp` + `rename`)
   replaces the link with a regular file. That is exactly what GLib does to Meld's
   keyfile — worked around by linking the parent *directory*. VS Code carries the same
   risk on `settings.json`, with no such workaround: `Code/User/` also holds
   `globalStorage/`, `workspaceStorage/` and `History/`, all large and volatile.
2. **We often do not want to own the whole file**, only to add a part of it. That is
   the oh-my-zsh model: a few lines inserted into a `.zshrc` that otherwise still
   belongs to the user or the distribution.

In both cases the failure is **silent**. A software update, a configuration wizard, or
a rewrite from the GUI can remove our additions without a single message. The Meld
case already demonstrated this in another form: preferences were never persisted, and
nothing was ever printed.

Two situations are **not** families and are out of this ADR's remit:

- **A prohibition** — never modify a file inside a program's installation tree. It is
  overwritten on update, and on macOS it invalidates an `.app` signature (`Meld.app`
  is Developer ID signed, hardened runtime).
- **A technique** — when a setting is only reachable through an environment variable,
  install a wrapper in `~/.local/bin`; never modify the launcher shipped by the
  package manager.

The third ownership case — **no file at all**, a setting in an opaque store
(`defaults write`, dconf) — is out of v1 scope
([0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md)).

## Decision

**Two mechanisms, chosen by the manifest, never guessed.**

1. **Owned file → symlink.** On the file where possible, on the **directory** when the
   program rewrites atomically (Meld's keyfile).
2. **Shared file → managed block.** A marker-delimited block inserted into the
   existing file; the rest of the file does not belong to dotflies:

   ```
   # >>> dotflies:<block-name> >>>
   … managed content …
   # <<< dotflies:<block-name> <<<
   ```

   The comment character depends on the target format and is declared in the manifest.
   A managed block is never merged line by line: it is replaced wholesale.

**Block integrity is checked on every run**, not in a separate command nobody
remembers to invoke. Three states are distinguished:

| Observed state | Reading | Behaviour |
|---|---|---|
| Block missing | A software update or a wizard removed it | **Reinsert automatically**, report it |
| Block present, content identical | Nothing to do | Silent |
| Block present, content edited in place | Someone hand-edited *our* block | **Overwrite nothing.** Report, show the diff, require `--force` |

The same check applies to symlinks: a link replaced by a regular file is the exact
signature of an atomic rewrite. It is reported, never silently recreated — overwriting
would destroy settings made through the program's own interface.

`dotflies doctor` runs these checks **without modifying anything**.

## Consequences

- Silently losing a configuration after a software update becomes detectable, and
  usually repaired without intervention.
- VS Code can be handled properly: a managed block instead of a fragile file link, or
  a watched link whose disappearance is reported.
- `.zshrc` becomes adoptable without dispossessing the user of the rest of the file.
- Cost: dotflies must be able to **write into** a file it does not own. That is the
  riskiest part of the product. It requires a systematic backup before modification
  and a way back (`dotflies remove` restores the file without the block).
- Telling "block removed by an update" from "block removed on purpose by the user" is
  technically impossible. Automatic reinsertion decides in favour of the first;
  `dotflies remove` is the clean way to express the second.

## Alternatives considered

**Symlinks everywhere.** Rejected: impossible on a shared file, and destructible by
atomic rewrite. Three of the five programs already in service would not fit.

**Generate shared files wholesale from a versioned source.** Rejected for v1: it is
clean, but it makes dotflies the owner of files like `.zshrc` in full, and it breaks
the editing loop — changing a setting from VS Code's UI would be lost on the next run.

**Line-by-line merging instead of a block.** Rejected: non-deterministic, unreadable
in a diff, and impossible to remove cleanly.

**Check only on demand, through a dedicated command.** Rejected: the failure is silent
by nature. A check you must remember to run would only be run after noticing the
problem — that is, too late. `doctor` exists, but alongside the systematic check, not
instead of it.
