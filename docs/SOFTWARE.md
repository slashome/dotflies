# dotflies — software inventory

This file merges the former `~/workspace/dotfiles/install.txt` with the current state
of the configuration. It holds the list of what `dotflies` must install and configure.

Statuses: **adopted** = configuration versioned under `configs/` and linked ·
**to adopt** = installed but configuration not taken over yet · **to settle** =
relevance to confirm.

## Adopted

| Program | Configuration | Platform | Note |
|---|---|---|---|
| **kitty** | `configs/kitty/kitty.conf` | macOS + Linux | Terminal. Source of truth for the Gruvbox palette. |
| **meld** | `configs/meld/` | macOS (bundle) | GUI diff. Gruvbox theme plus a `GSETTINGS_BACKEND` wrapper. On Linux the native package applies: configuration to redo entirely. |
| **mpd** | `configs/mpd/mpd.conf` | macOS + Linux | Non-portable audio output (`type "osx"`). |
| **ncmpcpp** | `configs/ncmpcpp/{config,bindings}` | macOS + Linux | MPD client. `error.log` not versioned. |
| **VS Code** | `configs/vscode/{settings.json,extensions.txt}` | macOS + Linux | Config path differs by OS. |

## To adopt — inherited from `~/workspace/dotfiles/install.txt`

Original contents of the file, kept verbatim:

```
# Ranger: File browser
ranger

# NCurses Music Player Client (Plus Plus): featureful ncurses based MPD client
ncmpcpp

# Qutebrowser: Vim-based web browser
qutebrowser

# Rxvt: Terminal
rxvt-unicode
```

Disposition:

| Program | Decision |
|---|---|
| **ranger** | To adopt. Configuration already present in `slashome/dotfiles` (2019): `rc.conf`, `bookmarks`, `tagged`. |
| **ncmpcpp** | ✅ already adopted above. |
| **qutebrowser** | **To settle.** Still in use? 2019 configuration available. |
| **rxvt-unicode** | **Obsolete.** X11 only, replaced by kitty. Not to be carried over. |

## Personal ecosystem to integrate

These are not configurations but programs of our own that the installer will have to
lay down:

| Repository | Role | Relation to dotflies |
|---|---|---|
| [`slashome/karaokay`](https://github.com/slashome/karaokay) | Synchronised lyrics in the terminal, via MPD | **Directly overlaps** `configs/ncmpcpp/`: the `7 → show_lyrics` binding and ncmpcpp's `*_lyrics` options do the same job. One or the other. |
| [`slashome/redlight`](https://github.com/slashome/redlight) | USB sync daemon, macOS + Linux | Precedent for homegrown cross-platform code; worth reading for structure. |
| [`slashome/homebrew-tap`](https://github.com/slashome/homebrew-tap) | Homebrew tap | **Ready-made distribution channel** for `dotflies` and `karaokay` on macOS. |
| [`slashome/scriptr`](https://github.com/slashome/scriptr) | Script launcher written in Go | Overlaps the `script-launcher.sh` of the 2019 repository. |

## Archive — the 2019 list (`slashome/dotfiles`, Arch Linux + i3)

Kept for reference. **Not to be reused as-is**: it targets Arch (`trizen`), X11 and an
i3 environment that is no longer in use.

- System / interface: `xorg-server`, `i3-gaps`, `i3lock-color-git`, `polybar`,
  `python-pywal`, `dunst`, `nautilus`, `stacer`
- Terminal: `rxvt-unicode-pixbuf`, `rxvt-unicode-terminfo`, `urxvt-perls`, `zsh`,
  `thefuck`
- Applications: `boostnote`, `buttercup-desktop`, `qutebrowser`, `irssi`, `task`, `khal`
- Development: `vim`, `sublime-text-dev`, `mycli`
- Files: `ranger`, `ncdu`

Still worth salvaging: the `ranger`, `vim` and `zsh` configurations, and the
script-launcher idea.

## To inventory

Installed on the machine but absent from this document — to review:

- [ ] shell (`zsh`, aliases, prompt)
- [ ] `git` (`~/.gitconfig`)
- [ ] Homebrew: generate a `Brewfile`
- [ ] Cursor (installed alongside VS Code — separate config, to take over or not)
- [ ] `~/.claude/` (agents, skills, hooks)
- [ ] `lnav`, `htop`, `ranger`, `flameshot`, `mpv`, `OpenRGB` (present in `~/.config/`)
