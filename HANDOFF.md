# dotflies — handoff prompt

> Context transferred from a Claude Code session run in `$HOME`, 14 August 2026.
> That session set up the Meld / kitty / mpd / ncmpcpp / VS Code configuration by hand
> and wrote the design notes. The project starts here.

> **This is the origin document, kept for its context — not the current state.**
> For where things stand, read [`PLAN.md`](PLAN.md). Two things here have moved on: the
> language is Rust, not Go ([0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md)),
> and the questions §9 asks have been answered
> ([0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md)).
>
> It was written on the **work Mac**, whose `$HOME` differs from the personal one.
> Everything §3 describes is that machine's state. Check before trusting it elsewhere.

---

I am starting the **dotflies** project in this folder. Here is the whole context.

## 1. What dotflies is

A configuration manager: version the configuration of every program I use, and
reinstall it **with a single command** on a fresh machine, macOS or Linux.

Two distinct things, not to be conflated:

- **`slashome/dotflies`** — *this repository*, the program (the manager).
- **the personal configuration** — my config files, in a separate repository, whose
  default proposed name is `<github-username>/_dotflies`.

## 2. The intended bootstrap — spec to honour

The manager keeps **its own** configuration in `$HOME/.config/dotflies/`. That config
file declares, among other things, **where the user's dotflies directory lives**: the
location is not hardcoded, it is chosen.

On first run, the script asks, in this order:

1. **Where to put the dotflies directory** (with a proposed default).
2. **Do you want to version it?** — the user's choice, never imposed.
3. If yes: **GitHub or another forge?** (GitLab, Codeberg, a bare git remote…).
4. If GitHub: propose `<github-username>/_dotflies` as the repository name, and
   **create it automatically** if the user agrees.

In other words, versioning is a guided option, not a prerequisite. `dotflies` must be
usable with no remote repository at all.

## 3. Current state — to absorb

> ⚠️ **On the work Mac only.** The personal Mac has none of this: no `~/.dotflies`, no
> links, `~/.config/kitty/` empty. See the note at the top of `PLAN.md`.

`~/.dotflies/` already exists, assembled **by hand**, and holds the live configuration:

```
~/.dotflies/
└── configs/
    ├── homapage/ · homepage/   ← pre-existing, typo included, to settle
    ├── kitty/kitty.conf
    ├── meld/{bin,gtk-3.0,gtksourceview-4,glib-2.0}
    ├── mpd/mpd.conf
    ├── ncmpcpp/{config,bindings}
    └── vscode/{settings.json,extensions.txt}
```

⚠️ **Nine live symlinks point into `~/.dotflies/configs/`.** Moving that folder breaks
them. Exact list:

```
~/.local/bin/meld
~/.config/kitty/kitty.conf
~/.config/ncmpcpp/config
~/.config/ncmpcpp/bindings
~/.config/mpd/mpd.conf
~/Library/Application Support/Code/User/settings.json
~/Library/Application Support/org.gnome.Meld/gtk-3.0/gtk.css
~/Library/Application Support/org.gnome.Meld/share/gtksourceview-4/styles/gruvbox-dark.xml
~/Library/Application Support/org.gnome.Meld/glib-2.0/settings          (DIRECTORY link)
```

The design notes have been moved into `docs/` in this repository. What remains to
decide is the fate of `configs/` — it belongs to the personal config repository, not to
this one. Any move must **repoint the links**, not leave them dangling.

## 4. The heart of the problem — five families

This is what rules out a blanket `ln -s`. Full detail in
[`docs/DESIGN.md`](docs/DESIGN.md); summary:

| | Situation | Strategy |
|---|---|---|
| **(a)** | The program reads a dedicated user file (`kitty.conf`, `mpd.conf`) | symlink |
| **(b)** | Opaque store (macOS `defaults`, dconf, SQLite) | declarative script + inverse capture |
| **(c)** | The program rewrites its own config (Meld, VS Code) | **directory** link where possible, otherwise a file link **plus verification** — an atomic `rename` destroys a file link |
| **(d)** | Modifying the installation tree | **forbidden**: overwritten on update, and it invalidates a macOS `.app` signature |
| **(e)** | Injecting environment variables | wrapper in `~/.local/bin`, never the package manager's launcher |

Note that (d) and (e) are not really families: one is a prohibition, the other a
technique. Only (a), (b) and (c) describe *who owns the file*.

Real cases already met, which serve as acceptance tests:

- **Meld** combines (c), (d) and (e). Its bundle redefines `XDG_CONFIG_HOME`, ships no
  dconf (no preference ever persisted, **with no error message whatsoever**), and
  overrides the GTK `settings.ini` from its own key. Lesson: *the documented config
  path is not always the real one*, and *you must verify that a setting took effect*,
  not merely that a file was written.
- **VS Code**: `settings.json` linked as a file, to be watched (trap (c)).
- **mpd**: the first non-portable file (`type "osx"` vs ALSA/PulseAudio/PipeWire) —
  which is what forces per-platform templating.

## 5. Prior art — read before writing code

I audited my 30 repositories. Four matter:

| Repository | State | Lesson |
|---|---|---|
| A dotfile manager (2018) | **empty**, "A dotfile manager written in GO" | abandoned attempt |
| A dotfiles repository (2019) | **real and complete**, Arch + i3 | the only version that ever served |
| A profiles repository (2025) | **empty**, "All my dotfly profiles" | abandoned attempt |
| Our Homebrew tap (active) | Homebrew tap | distribution channel already in place |

**The signal: two "manager" attempts left empty, one plain shell version that actually
served for years.** The deciding factor looks like the size of the initial push, not
the language. Aim small and usable.

The 2019 `install.sh` already prefigures the target architecture: a declarative,
commented `PROGRAMS[]` manifest, an idempotent loop that installs only what is missing,
a **per-program `./<prog>/install.sh` hook** (the extension point that absorbs (b), (d)
and (e)), and a centralised `createLink` helper. What it lacks: portability (`trizen`
hardcoded), `--dry-run`, after-the-fact verification, uninstallation.
→ **Reuse the skeleton, not the code.**

See also our USB sync daemon: the most recent homegrown code running on macOS **and**
Linux — read its structure before inventing another one.

## 6. What the manager will have to do

- A declarative manifest per program: package, links **per platform**, hooks, checks
- Templates for files whose content differs by OS
- Idempotence, `--dry-run`
- `adopt <file>`: take an existing config into the repository and lay the link without
  loss (the operation I did nine times by hand)
- Uninstall / restore; never overwrite without backing up
- **Post-installation verification** that the setting took effect
- Package lists: Brewfile, VS Code extensions, global npm
- Secrets kept out of the plaintext repository (`age`/`sops`, system keychain, or
  exclusion)

## 7. Decisions already made — see `docs/adr/`

The scope was settled on 14 August 2026. **Read `docs/adr/` before anything else**;
these decisions are binding:

| № | Decision |
|---|---|
| [0001](docs/adr/0001-scope-v1-to-a-minimal-verifiable-milestone.md) | v1 scope: `adopt` + links + managed blocks + wrappers + Homebrew/npm install + `doctor`. Acceptance criterion: **rebuild the nine current links from scratch, Meld included**. |
| [0002](docs/adr/0002-limit-v1-to-macos.md) | macOS only, but a **platform key in the manifest from v1**. Linux call for contribution in the README. |
| [0003](docs/adr/0003-separate-owned-files-from-shared-files.md) | Owned file → link; shared file → **managed block** with markers. Integrity check **on every run**, not only on demand. |
| ~~0004~~ | ~~**Go**~~ — superseded by 0006. |
| [0005](docs/adr/0005-defer-templating-until-after-v1.md) | No templating in v1; `doctor` reports absolute paths. |
| [0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) | **Rust**, with `tools/install.sh` in POSIX shell strictly bounded to bootstrapping. |
| [0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md) | Manifest format frozen; the configuration lives at `~/.config/dotflies/`, fixed by convention, so dotflies has no config file of its own. |

Intended entry point — settled on `main` by
[0006](docs/adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md), since the
repository exists on `main` and the original `master` URL would have 404'd:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/slashome/dotflies/main/tools/install.sh)"
```

## 8. Still open

- **Fate of `~/.dotflies/configs/`**: migrate to the future `_dotflies` repository, or
  stay local until the manager exists.
- **`configs/homapage` vs `configs/homepage`**: duplicate with a typo.
- **Our lyrics tool vs ncmpcpp's lyrics binding**: functional overlap to settle.
- **Linux ↔ macOS migration**: explicitly out of v1 per 0002, to be replanned.

## 9. What I want from you now

> **Done — 14–15 August 2026.** All four points were carried out; the manifest format was
> proposed, then settled as
> [ADR 0007](docs/adr/0007-adopt-a-declarative-per-program-manifest.md). The §8 items
> were put back to the owner rather than decided here, as asked, and the surviving ones
> are tracked under *Deferred decisions* in [`PLAN.md`](PLAN.md). Kept below as the
> record of what was asked.

1. Read [`PLAN.md`](PLAN.md), then all of `docs/adr/`, then
   [`docs/DESIGN.md`](docs/DESIGN.md) and [`docs/SOFTWARE.md`](docs/SOFTWARE.md).
2. Look at the 2019 dotfiles repository — especially `install.sh`, `program_list.sh`
   and `scripts/core/` — to absorb what worked.
3. Propose **the manifest format** and the Go repository layout. It is the first brick:
   everything else depends on it, and 0002 requires it to carry the platform key from
   the start.
4. Keep your explanations separate from your questions. Do not settle the items in §8
   for me: put them to me.

No code until we agree on the manifest format.
