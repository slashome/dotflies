# Architecture Decision Records (ADR)

A log of dotflies' design decisions. One file per decision: what prompted it, what was
chosen, what it costs, and what was ruled out.

The value is not the decision itself but its **context**. Six months from now the
question will not be "what did we pick" but "why, and does it still hold".

## Lifecycle

An ADR at the root of this folder is in force. When a decision is revisited we do not
edit the old file and we do not delete it: we write a new ADR, flip the old one to
`Superseded` with a link to its successor, and move it into [`archives/`](archives/).
Obsolete reasoning stays readable — it is usually what explains a constraint that has
since become mysterious.

| Status | Meaning |
| --- | --- |
| `Proposed` | Written, not yet settled. |
| `Accepted` | In force. |
| `Superseded` | Replaced by a later ADR. Archived. |
| `Deprecated` | No longer relevant, no successor. Archived. |

## Writing convention

[Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
format: **Status**, **Context**, **Decision**, **Consequences**, **Alternatives
considered**.

The *Consequences* section must carry the **negative** effects as much as the positive
ones. An ADR that costs nothing is usually an ADR that settled nothing.

Files are named `NNNN-imperative-title.md`, four-digit sequential numbering, never
reused after archiving.

## Decisions in force

| № | Decision | Status | Date |
| --- | --- | --- | --- |
| [0001](0001-scope-v1-to-a-minimal-verifiable-milestone.md) | Scope v1 to a minimal, verifiable milestone | Accepted | 2026-08-14 |
| [0002](0002-limit-v1-to-macos.md) | Limit v1 to macOS and open the Linux port to contribution | Accepted | 2026-08-14 |
| [0003](0003-separate-owned-files-from-shared-files.md) | Separate owned files from shared files, and verify managed blocks on every run | Accepted | 2026-08-14 |
| [0005](0005-defer-templating-until-after-v1.md) | Defer templating until after v1 | Accepted | 2026-08-14 |
| [0006](0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) | Write dotflies in Rust, with a shell bootstrap | Accepted | 2026-08-14 |
| [0007](0007-adopt-a-declarative-per-program-manifest.md) | Adopt a declarative per-program manifest, and settle where the user's directory lives | Accepted | 2026-08-14 |

## Archived

| № | Decision | Status | Replaced by |
| --- | --- | --- | --- |
| [0004](archives/0004-write-dotflies-in-go-with-a-shell-bootstrap.md) | Write dotflies in Go, with a shell bootstrap | Superseded | [0006](0006-write-dotflies-in-rust-with-a-shell-bootstrap.md) |

## Adding one

Take the next number (archives included), write the five sections, add the row above.
If the decision replaces another, archive the old one and drop it from the table.
