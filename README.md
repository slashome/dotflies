# dotflies

A configuration manager: version the configuration of the software you use, and
reinstall it **with a single command** on a fresh machine.

> **Status: starting.** The scope and the manifest format are both settled
> ([`docs/adr/`](docs/adr/)); the code is being written in Rust. See
> [`PLAN.md`](PLAN.md) for where it stands.

## The problem

Dotfiles are not just symlinks. A real configuration runs into at least three distinct
situations, and they do not call for the same answer:

| Situation | Example | Answer |
|---|---|---|
| **Owned file** — it would not exist without you | `kitty.conf` | symlink |
| **Shared file** — the program writes to it too | `settings.json`, `.zshrc` | marker-delimited block |
| **No file at all** — setting held in an opaque store | `defaults write` | replayed command |

Plus two rules that are neither:

- **Never modify a program's installation tree.** It is overwritten on update, and on
  macOS it invalidates an `.app` signature.
- **When a setting is only reachable through an environment variable**, install a
  wrapper in `~/.local/bin` — never modify the launcher shipped by the package
  manager.

The hard part is not applying a configuration. It is noticing that it is gone. A
software update can remove your additions without printing a thing. dotflies verifies
the integrity of what it applied **on every run**.

## v1 scope

`adopt` an existing file, lay links and blocks, generate wrappers, install software
(Homebrew, npm), and `doctor` to check that everything still holds.

Out of v1: templating, opaque stores, secret management, multiple profiles.

## Contributing — the Linux port is open

**v1 targets macOS only**, for lack of a Linux machine to test on. But the manifest
format carries the platform key **from the start**: a `linux` entry can be declared,
and it is validated then skipped at execution time with an explicit message.

In other words, Linux support does not require breaking anything. It requires a
platform resolver and a package-manager layer.

**If you want to take that on, it is yours.** Open an issue.
The full reasoning is in [ADR 0002](docs/adr/0002-limit-v1-to-macos.md).

## Documentation

- [`PLAN.md`](PLAN.md) — status, blockers, milestones
- [`docs/GLOSSARY.md`](docs/GLOSSARY.md) — every term in two sentences, and why it exists
  here. Start there if "managed block" or "drift" means nothing to you yet
- [`docs/adr/`](docs/adr/) — the decisions and why they were made
- [`docs/DESIGN.md`](docs/DESIGN.md) — analysis of the problem
- [`ADR 0007`](docs/adr/0007-adopt-a-declarative-per-program-manifest.md) — the manifest
  format, where your configuration lives, and why

Your configuration lives in its own repository, separate from this one — proposed as
`<your-username>/_dotflies` at first run, and entirely optional: dotflies works with no
remote at all.

## Licence

MIT — see [`LICENSE`](LICENSE).
