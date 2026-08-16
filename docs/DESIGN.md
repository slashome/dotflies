# dotflies — design notes

> Status: local directory `~/.dotflies`, not versioned yet.
> The installer described here **is still to be written**.
>
> See also [`SOFTWARE.md`](SOFTWARE.md) — inventory of programs to install.

## 0. Prior art — what already exists

An audit of our own 30 repositories. Four are directly relevant:

| Repository | Date | State | Verdict |
|---|---|---|---|
| A dotfile manager | 2018 | **Empty** (README only): "A dotfile manager written in GO" | The project has been attempted twice and abandoned at the intention stage both times. **The real risk is not technical, it is scope**: these notes must stay small enough to be executable. |
| A dotfiles repository | 2019 | **Real and complete**: Arch + i3, `install.sh`, `program_list.sh`, per-program configs | Prior art to reuse (see below). |
| A profiles repository | 2025 | **Empty**: "All my dotfly profiles" | The name `profiles` is already taken, and the notion is worth keeping. |
| [`homebrew-tap`](https://github.com/slashome/homebrew-tap) | 2026 | Active | **Distribution channel already in place** for `dotflies` on macOS. |

### What the 2019 dotfiles repository already solved

Its `install.sh` architecture prefigures section 6 almost exactly:

- a declarative `PROGRAMS[]` array, commented and grouped by category
  (`program_list.sh`) — that is the **manifest**;
- a loop that queries the package manager (`trizen -Qi`) and installs only what is
  missing — that is **idempotence**;
- a **per-program hook**: if `./<prog>/install.sh` exists, it is sourced — that is the
  extension point absorbing cases (b), (d) and (e);
- a `createLink` helper centralising link creation, with a privileged mode for
  `/usr/bin`.

What has to change: the package manager is hardcoded (`trizen`, AUR), there is no
notion of platform, no `--dry-run`, no after-the-fact verification, and no way back
(uninstallation).

→ **Reuse the skeleton, not the code.** And treat "install packages" as already
validated by experience; the new work is portability and verification.

### Cross-platform precedent

An existing USB sync daemon of ours, running on macOS and Linux and actively
maintained, is the most recent precedent for homegrown code that has to work on both
systems. Read its structure before inventing another one.

## 1. Goal

A complete, versioned configuration of every program in use, reinstallable **with a
single command** on a fresh machine, macOS or Linux.

Target:

```sh
git clone <repository> ~/.dotflies && ~/.dotflies/install
```

…and get the terminal, editors, Git tooling, audio players and GUI applications back
exactly as they were.

## 2. The central problem: editing a program's default config vs creating a config file

This is the crux, and it is what rules out a blanket `ln -s`. Programs fall into five
families, and they do not call for the same strategy.

### (a) The program reads a dedicated user file — the ideal case

`~/.config/kitty/kitty.conf`, `~/.config/mpd/mpd.conf`, `~/.gitconfig`, `~/.zshrc`.
The file does not exist by default; we create it, it is ours, nothing overwrites it.

→ **Symlink** from the dotflies directory. Reversible, readable, diffable.

### (b) The program stores settings in an opaque store

macOS `defaults`/plists, dconf on Linux, SQLite databases, binary application state.
There is no "source file" to symlink: versioning the artefact produces no meaningful
diff and corrupts easily.

→ **A declarative application script**: `defaults write …`, `dconf load < dump.ini`,
`jq` to merge JSON. The repository versions the *intent*, not the artefact. The inverse
operation ("capture the current state") is equally necessary, otherwise the
configuration drifts silently.

### (c) The program rewrites its own config file

Meld and VS Code both do this: the file is written by the application as soon as a
preference is touched in the UI, and it sometimes mixes **durable preferences** (theme,
font) with **volatile state** (window geometry, recent files).

Two traps:

1. An atomic rewrite (`write temp` + `rename`) **destroys a symlink placed on the
   file**. Workaround: symlink the parent **directory** — provided that directory does
   not also hold volatile data.
2. Volatile state pollutes the Git history every time the program is closed.

→ Depending on the case: a directory symlink, or a file symlink **plus a verification**
after the application's first write. For the noise: a targeted `.gitignore`, or a Git
`clean` filter that strips volatile keys on commit. To be settled.

### (d) Files inside the program's own installation have to be modified

**Forbidden on principle.** Two disqualifying reasons:

- any package update overwrites the modification, silently;
- on macOS, modifying a signed `.app` (hardened runtime) **invalidates the signature**
  and the application refuses to launch. `Meld.app` is Developer ID signed as
  `SW3D6BB6A6`: touching it would have broken the binary.

→ Always look for the user-side extension point. If there is none, fall through to (e).

### (e) Environment variables have to be injected at launch

Some behaviours are only reachable through the environment (`GSETTINGS_BACKEND`,
`GTK_THEME`, `EDITOR`…). Editing the launcher shipped by the package manager falls
under (d): Homebrew will replace it on the next update.

→ **A personal wrapper in `~/.local/bin`**, placed ahead of `/opt/homebrew/bin` on the
`PATH`. It survives updates and stays versionable.
Known limitation: on macOS, launching from Finder or the Dock does not inherit the
shell `PATH`, so the wrapper is bypassed. Possible workarounds: `launchctl setenv` via
a LaunchAgent, or a small redirecting `.app`.

## 3. Case study: Meld (the reference)

Meld alone combines (c), (d) and (e).

| File in the repository | Links to | Family |
|---|---|---|
| `configs/meld/bin/meld` | `~/.local/bin/meld` | (e) wrapper exporting `GSETTINGS_BACKEND=keyfile` |
| `configs/meld/gtk-3.0/gtk.css` | `~/Library/Application Support/org.gnome.Meld/gtk-3.0/gtk.css` | (a) |
| `configs/meld/gtksourceview-4/styles/gruvbox-dark.xml` | `…/org.gnome.Meld/share/gtksourceview-4/styles/gruvbox-dark.xml` | (a) |
| `configs/meld/glib-2.0/settings/` | `…/org.gnome.Meld/glib-2.0/settings` | (c) **directory link** |

Transferable lessons:

- The `Meld.app` bundle redefines `XDG_CONFIG_HOME` to
  `~/Library/Application Support/org.gnome.Meld`. **The documented config path is not
  always the real one**: the program will need a way to check it (inspecting the
  launcher, `lsof`, `fs_usage`) rather than assuming.
- The bundle ships no dconf: GSettings fell back to an in-memory backend and **no
  preference ever persisted**. A setting can fail without a single error message — the
  installer must **verify** its effects, not merely write files.
- Meld forces `gtk-application-prefer-dark-theme` from its own key at startup,
  overriding the GTK `settings.ini`. **The most specific configuration layer wins**;
  identify which one that is before writing.
- The versioned `keyfile` also holds `[org/gnome/meld/window-state]`, which changes
  every time the window closes. Noise to handle (see 2.c).

## 4. The other adopted programs

| Program | Files | Destination | Family | Notes |
|---|---|---|---|---|
| **kitty** | `kitty.conf` | `~/.config/kitty/` | (a) | Source of truth for the Gruvbox palette, reused by Meld. A shared theme should derive from it rather than duplicate it. |
| **ncmpcpp** | `config`, `bindings` | `~/.config/ncmpcpp/` | (a) | `error.log` left in place: volatile state, never versioned. |
| **mpd** | `mpd.conf` | `~/.config/mpd/` | (a) | Contains absolute paths (`~/Music`, `~/.mpd/`) and an audio output of `type "osx"` — **must become platform-conditional**: ALSA/PulseAudio/PipeWire on Linux. The first concrete case requiring a template. |
| **VS Code** | `settings.json`, `extensions.txt` | `~/Library/Application Support/Code/User/` | (c) | See below. |

### VS Code, a point of caution

`settings.json` is rewritten by the application every time a setting changes through
the UI. Symlinking the `User/` **directory** is not viable: it also holds
`globalStorage/`, `workspaceStorage/` and `History/`, all large and volatile. So the
link sits on the file, which exposes trap 2.c.1.

→ After the first setting change from the UI, check:

```sh
ls -l ~/Library/Application\ Support/Code/User/settings.json   # must still be a link
```

If the link has been replaced by a regular file, we move to a (b)-style strategy:
generate `settings.json` from a versioned source rather than link it.

Extensions cannot be symlinked either (`~/.vscode/extensions/`, downloaded content).
They are captured as a list:

```sh
code --list-extensions > configs/vscode/extensions.txt              # capture
xargs -n1 code --install-extension < configs/vscode/extensions.txt  # restore
```

The same logic applies to Homebrew (`Brewfile`), global npm, and so on: **capture a
list, not artefacts**.

## 5. macOS / Linux constraints

The same program has neither the same paths nor the same mechanisms:

| | macOS | Linux |
|---|---|---|
| Config | `~/Library/Application Support/`, `~/.config/` | `~/.config/` (XDG) |
| Data | `~/Library/Application Support/` | `~/.local/share/` |
| Cache | `~/Library/Caches/` | `~/.cache/` |
| System settings | `defaults write` | dconf / gsettings |
| Packages | Homebrew (formula + cask) | apt / dnf / pacman / flatpak |
| Fonts | `~/Library/Fonts/` | `~/.local/share/fonts/` |
| Services | launchd | systemd --user |
| mpd audio output | CoreAudio (`type "osx"`) | ALSA / PulseAudio / PipeWire |
| VS Code | `~/Library/Application Support/Code/User/` | `~/.config/Code/User/` |

→ A program's manifest must carry **per-platform** paths, not a single path. And it
must allow for a program existing on only one of the two, or differing in content (the
`mpd.conf` case).

## 6. What the manager will have to do

- **A manifest per program** (one declarative file per tool): package to install,
  links per platform, pre/post hooks, checks.
- **Templates** for files whose content differs by platform.
- **Idempotence**: replayable at any time with no side effects.
- **`--dry-run`**: show the actions without performing them.
- **Adoption**: take an existing config into the repository and lay the link in its
  place, without loss — exactly the operation performed by hand for the five current
  programs, which should be automated (`dotflies adopt <file>`).
- **Uninstall / restore**: remove the links, put the files back.
- **Backup**: never overwrite an existing file without moving it aside first.
- **Post-installation verification**: check that the setting actually took effect (see
  the Meld dconf case, and the VS Code symlink case).
- **Package lists**: Brewfile, VS Code extensions, global npm, casks.
- **Secrets**: SSH/GPG keys, tokens and `.npmrc` do not belong in a plaintext
  repository. Decide early: `age`/`sops`, system keychain, or keep them out.

## 7. Decision to make: existing tool or homegrown?

To settle before writing a line:

| Option | For | Against |
|---|---|---|
| **GNU Stow** | trivial, pure symlinks | only covers (a) — nothing for (b), (d), (e) |
| **chezmoi** | cross-platform, templates, secrets, `chezmoi add` | copies files instead of linking them; a clumsier editing loop |
| **yadm** | Git repository straight on `$HOME`, per-OS alternates | more rustic, no package management |
| **nix-darwin + home-manager** | genuine reproducibility, macOS **and** Linux | steep learning curve, Homebrew coexistence to manage |
| **Homegrown (shell/Python/Go)** | fits the (b)/(d)/(e) cases actually encountered | everything to write and maintain |

Likely path: an existing base for (a) and (b), plus a homegrown layer for wrappers,
templates and verification.

**A word of caution**: `dm` (2018) and `profiles` (2025) are two homegrown attempts
that stayed empty, while `dotfiles` (2019) — plain shell and a flat manifest — actually
served. The deciding factor appears to be the size of the initial undertaking, not the
language.

## 7 bis. Where to publish

| Option | For | Against |
|---|---|---|
| **The existing `profiles` repository** | name already reserved and described for this, empty repository | "profiles" names a narrower notion (per-machine profiles) than the project |
| **A new `dotflies` repository** | the project's name, a clean start | one more repository |
| **Reuse the 2019 dotfiles repository** | keeps the 2019 history, prior art in the same tree | five years of dead Arch/i3 content to purge first |
| **The 2018 `dm` repository** | short name, already "dotfile manager" | does not say "dotflies" |

Distribution of the binary once written: our Homebrew tap on macOS.
A promising split: keep the **manager** (`dotflies`, published through the tap) apart
from the **configuration** (`_dotflies`).

## 8. Current state

```
~/.dotflies/
└── configs/
    ├── homapage/               ← pre-existing (misspelled name)
    │   ├── kubernetes.yaml
    │   ├── logs/
    │   └── settings.yaml
    ├── homepage/               ← pre-existing, empty
    ├── kitty/kitty.conf
    ├── meld/                   ← Gruvbox theme aligned with kitty
    │   ├── bin/meld
    │   ├── glib-2.0/settings/keyfile
    │   ├── gtk-3.0/gtk.css
    │   └── gtksourceview-4/styles/gruvbox-dark.xml
    ├── mpd/mpd.conf
    ├── ncmpcpp/{config,bindings}
    └── vscode/{settings.json,extensions.txt}
```

The documentation (`DESIGN.md`, `SOFTWARE.md`) has been moved into `docs/` of the
`slashome/dotflies` repository. Only the configuration itself remains in the personal
directory.

Merged sources:

- `~/workspace/dotfiles/install.txt` → reproduced verbatim in
  [`SOFTWARE.md`](SOFTWARE.md), folder removed.

Next:

- [ ] settle 7 bis (where to publish) — **done**: a new `dotflies` repository
- [ ] settle 7 (existing tool or homegrown) — **done**: homegrown, see
      [ADR 0006](adr/0006-write-dotflies-in-rust-with-a-shell-bootstrap.md)
      (which superseded ADR 0004: the language is Rust, not Go)
- [ ] settle `configs/homapage` vs `configs/homepage` (duplicate, typo)
- [ ] recover the still-relevant configs from the 2019 repository (ranger, zsh, vim)
- [ ] settle our lyrics tool vs ncmpcpp's lyrics binding (functional overlap)
- [ ] adopt `~/.gitconfig`, `~/.zshrc`, `~/.claude/`
- [ ] capture a `Brewfile`
- [ ] handle the `window-state` noise in Meld's keyfile
- [ ] make `mpd.conf` platform-conditional
