# 0006 — Write dotflies in Rust, with a shell bootstrap

## Status

Accepted — 2026-08-15. Supersedes
[0004](archives/0004-write-dotflies-in-go-with-a-shell-bootstrap.md).

## Context

[0004](archives/0004-write-dotflies-in-go-with-a-shell-bootstrap.md) chose Go on the day
the scope was settled, and no code followed. The decision was never paid for, which makes
it cheap to revisit — and it had to be revisited before the first line, because a language
choice is the one decision that gets more expensive every day it stands.

It was re-examined from zero, on the explicit assumption that **no existing code counts**:
a 463-line Rust attempt from May 2026 sits in a local checkout with no remote, and the
owner decided not to resume it. "Code already exists" is therefore not an argument here.

Two things were measured rather than assumed.

**Compile times.** On the working machine (Apple M2 Max, 12 cores, cargo 1.97.1):

| Measurement | Result |
|---|---|
| Cold debug build, realistic v1 dependency set (113 crates) | 4.3 s |
| `cargo test`, cold, same set | +3.2 s |
| Synthetic 4 910-line crate, 168 `#[derive(Deserialize)]` types, cold | 1.7 s |
| The same, **incremental** after editing one module | **0.48 s** |
| Incremental `cargo test` loop | **0.45 s** |

**The tap.** `slashome/homebrew-tap` holds `Formula/redlight.rb` (Rust, `depends_on
"rust" => :build` plus `cargo install`, so the user compiles) and `Formula/karaokay.rb`
(Python, virtualenv, eleven pinned `resource` blocks). `RELEASING.md` documents a
source-tarball plus `shasum` flow, and CI runs only `brew style` and `brew audit
--strict --online`.

Three arguments that look decisive are not, and saying so is the point of this ADR:

- **Compile speed does not separate the two languages at this scale.** 0.48 s
  incremental against Go's ~0.15 s is a third of a second per iteration. Since compile
  speed was the last remaining argument for Go, Go is left with none.
- **The tap does not favour either language.** It has **no binary-release machinery for
  any language**. The formula dotflies needs — prebuilt binaries — has to be built from
  scratch whichever language wins. 0004's claim that "the goreleaser → Homebrew path is
  well trodden" was false, but correcting it does not score a point for Rust.
- **`redlight` being Rust is not a language argument.** 0004 said "read its structure
  before inventing another one", which is advice about code organisation. There is a real
  argument nearby — keeping one systems language on the account, for the two tools that
  manipulate files and subprocesses on macOS and Linux — but it is a supporting one, not a
  deciding one.

## Decision

**dotflies is written in Rust**, for three reasons that are specific to this product
rather than general preferences.

**1 — The core of the product is a sum type, and Rust is the only serious candidate that
makes a missed case a compile error.** The plan is a cartesian product: three mechanisms
(`link`, `block`, `wrapper`) × six states (`ok`, `absent`, `drifted`, `conflict`,
`skipped`, `warn`). In Rust that is an `enum` and an exhaustive `match`, so the day a
seventh state appears — and it will,
[0003](0003-separate-owned-files-from-shared-files.md) already distinguishes three
sub-cases of block that the six states flatten — the compiler names **every** site that
must handle it, `doctor`'s printing and `apply`'s branch included. In Go it is a string
constant and a `switch` with `default`: the omission is silent. In this program a silent
omission means writing into a user's `.zshrc` in a case nobody foresaw, which is the
documented number-one failure mode. The language belongs on the side of the net.

**2 — The validation rules frozen by
[0007](0007-adopt-a-declarative-per-program-manifest.md) are free in serde and manual
everywhere else.** The format requires three things: a `target` without a platform key is
rejected, an unknown `[install]` key is an error so a typo cannot silently install
nothing, and a `[[verify]]` `kind` outside `file_exists`/`file_contains` is an error. In
serde those are `Option<T>` (absent and empty stay distinct), `#[serde(deny_unknown_fields)]`
— one line — and an `enum` that fails to deserialise. The specification becomes a data
structure instead of a validator someone forgets to update when the format moves.

**3 — This program meets none of the hard parts of Rust, and that is verified rather than
hoped.**
<!-- Note: 0007 later fixed the configuration location by convention, which removes the
     `config.rs` this reason once cited as reusable. The evidence below is unaffected —
     it is about what the code contains, not about reusing it. -->
 The May attempt's 463 lines contain no lifetime annotations, no generics, no
`Arc`/`Mutex`, no `async`, no `unsafe`. Its `classify()` is the core function of dotflies
and it is eighteen lines of `match` over `fs::symlink_metadata`. The domain is a tree walk
with no shared state, no concurrency and no hot loop, so the expensive parts of Rust never
show up. That is the one thing worth keeping from that attempt — not code, but evidence.

**`tools/install.sh` stays POSIX shell**, with the bounded scope 0004 gave it, which was
never in question:

1. detect OS and architecture;
2. check for and install Homebrew if missing;
3. install the `dotflies` binary;
4. hand over to `dotflies bootstrap`.

**No business logic in shell.** The interactive bootstrap is Rust, inside `dotflies
bootstrap`. The script stays readable in one screenful: it is code users are invited to
run straight from a URL.

**The bootstrap URL is settled on `main`**, since 0004 left it pointing at `master` and
the repository exists on `main`. It would have 404'd on first use.

**Libraries**, resolved against crates.io rather than quoted from memory: `clap` 4.6
(derive), `toml` 0.9, `serde` 1.0, `anyhow` 1.0, `thiserror` 2.0, `dirs` 6.0, `inquire`
0.7, `owo-colors` 4.3, `insta` 1.48, `assert_cmd` 2.2 with `predicates` 3.1, `tempfile`
3.27, `camino` 1.2.

Two of those are decisions, not defaults. **`thiserror` in `plan/`, `anyhow`
everywhere else**: `plan`'s errors are asserted on in tests, so they need named variants;
elsewhere a context string is enough. **`camino::Utf8PathBuf` throughout**: the real
friction for a non-Rust-fluent author on this program is not borrowing, it is the
`Path`/`PathBuf`/`OsStr`/`&str`/`String` shuffle. `camino` removes almost all of it, at
the price of refusing non-UTF-8 paths — an acceptable constraint on the targets this
manages.

## Consequences

- A single binary with no runtime, for a tool whose job is to repair a fresh or broken
  machine. This is also what rules out the runtime-based candidates.
- `insta` turns `doctor`'s report into a snapshot test, which is exactly the shape of that
  milestone's deliverable.
- **Distribution has to be built, and it is not free.** The formula dotflies needs exists
  in the tap for no language. It means a release workflow producing
  `aarch64-apple-darwin` and `x86_64-apple-darwin` archives, a prebuilt-binary formula
  (`on_arm`/`on_intel`, allowed in a third-party tap even though `homebrew-core` forbids
  binary-only formulae), and an update to `RELEASING.md`. `cargo-dist` generates and
  maintains all of it — latest 0.32.0, May 2026, actively developed — but it is
  **single-vendor**, and the fallback if it is abandoned is hand-writing ~40 lines of
  GitHub Actions plus a formula template. Two things to check at that milestone: the
  generated formula must pass the `brew audit --strict --online` the tap's CI runs (a
  missing `license` is the classic failure), and it must stay a **formula**, not a cask —
  a cask reopens Gatekeeper on an unnotarised binary.
- **The Linux port gets harder to give away.** [0002](0002-limit-v1-to-macos.md) opens it
  to contribution; Rust narrows the pool, and a contributor has to install the toolchain
  too. The honest mitigation is that the code in question is `match` over file metadata
  plus one trait implementation — the most approachable subset of Rust there is — and that
  this must be **written into `CONTRIBUTING.md`**, not hoped for.
- **No exploratory loop.** Every "what does `symlink_metadata` return on a directory link
  under APFS?" is a compile cycle rather than a one-liner. On a domain where the truth is
  empirical — the Meld bundle redefines `XDG_CONFIG_HOME`, GLib renames, VS Code might too
  — that is real friction. Mitigation: a throwaway `tests/probe.rs` run with `cargo test
  -- --nocapture`, at 0.45 s a turn.
- **Two languages in the repository**, unchanged from 0004. The `tools/install.sh`
  boundary is the line that gets eaten first, and it has to stay at four steps.
- **The cost Rust does not pay but that must be named**: the author writes faster in
  TypeScript. This decision bets that a half-second compile loop and the absence of Rust's
  hard parts outweigh that. **The falsification condition is explicit**: if the `plan`
  milestone has no green tests within three weeks, that bet is lost, and *that fact* —
  not a preference — reopens this ADR.
- This is the fourth start on the same idea.
  [0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md)'s finding is that the project
  dies of the size of its first push, not of its language. A language change must not
  become a rewrite debate: it cost one ADR, and it stops here.

## Alternatives considered

**Keep Go, as 0004 decided.** Rejected: nothing is left. Its compile-speed advantage is a
third of a second at this scale, its distribution costs exactly as much as Rust's, and it
is the only serious candidate that cannot make an unhandled state a compile error — in the
one program where an unhandled state means writing into a file it does not own. Its real
advantage, trivial cross-compilation, applies to a feature that is out of v1 scope and
handled in Rust with per-target CI runners.

**All shell, as in 2019.** Rejected — and the reason is sharper than 0004's. The 2019
manifest was a bash `PROGRAMS[]` array that the shell **sourced**: parsing cost zero.
[0007](0007-adopt-a-declarative-per-program-manifest.md) froze a TOML format using dotted
keys (`target.darwin`) and inline tables (`env = { GSETTINGS_BACKEND = "keyfile" }`). No
POSIX shell TOML parser handles both; it would take `tomlq` (so Python), `dasel`/`taplo`
(so a Go or Rust binary — which settles it), or hand-rolled awk that would need testing,
which is the thing shell tests worst. **The shell's historical advantage evaporated the
moment the manifest stopped being shell.** It keeps exactly one job: the four steps of
`tools/install.sh`.

**TypeScript on Node.** The strongest runner-up, and genuinely defensible: it is the
author's language, `node:fs` covers `lstat`/`symlink`/`readlink`/atomic `rename`,
discriminated unions give the exhaustiveness Go lacks, `vitest` plus `memfs` test `plan`
well, and an esbuild single-file bundle installs *faster* than the tap's current Rust
formula. Rejected on one dirimant point: dotflies exists to repair a fresh or broken
machine, and depending on a runtime whose version, `PATH` and manager (nvm, fnm, volta,
brew) are part of what dotflies is supposed to manage is a circular dependency.

**Python.** Rejected, and the tap prices it exactly: `Formula/karaokay.rb` carries a
virtualenv and eleven pinned `resource` blocks to regenerate on every release. The same
circular-runtime objection applies.

**Zig.** Rejected without hesitation: no mature TOML parser, no clap equivalent, a standard
library still shifting between minor versions, and no gain here — there is no binary-size
constraint and no allocation control to win.

**Resume the May Rust attempt rather than restart.** Rejected: it is a mirror-tree model,
incompatible with [0002](0002-limit-v1-to-macos.md) and
[0003](0003-separate-owned-files-from-shared-files.md), and three of five mechanisms are
absent from it. It is kept as evidence (reason 3 above), not as a starting point.

**An existing tool (chezmoi, Stow, nix-darwin).** Unchanged from 0004 and covered in the
design notes: Stow handles owned files only; chezmoi copies instead of linking and breaks
the editing loop; nix-darwin is a bigger undertaking than the project itself.
