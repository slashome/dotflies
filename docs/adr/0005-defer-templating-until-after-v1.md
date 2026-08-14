# 0005 — Defer templating until after v1

## Status

Accepted — 2026-08-14

## Context

A **template** is a configuration file whose *content* must vary from machine to
machine. A symlink cannot produce that: the file has to be generated from a source
and a set of variables.

Two real needs exist in the configuration already adopted:

1. **Variation by platform.** `configs/mpd/mpd.conf` declares an audio output of
   `type "osx"` (CoreAudio). On Linux it would be ALSA, PulseAudio or PipeWire.
2. **Hardcoded absolute paths.** `configs/vscode/settings.json` contains
   `<work-home>/…` in two places (`dart.flutterSdkPath` and a
   `yaml.schemas` entry), and `mpd.conf` references `~/Music` and `~/.mpd/`.

The first need disappears as long as v1 targets macOS only
([0002](0002-limit-v1-to-macos.md)). The second only bites if the username changes
between machines — which is not the case here: a fresh Mac will have the same `$HOME`.

## Decision

**No templating engine in v1.** Files are linked or inserted as-is.

Two guardrails, so that deferring does not turn into invisible debt:

- **`dotflies doctor` reports absolute paths** containing the current `$HOME` inside
  managed files. It is a warning, not an error: it makes the problem visible the day
  it matters, without imposing a solution today.
- **The manifest already carries the platform key** ([0002](0002-limit-v1-to-macos.md)).
  The day `mpd.conf` has to diverge, the slot exists.

## Consequences

- v1 avoids a substitution engine, its syntax, its documentation and its edge cases —
  for no benefit at all on the machine v1 targets.
- **Accepted limitation**: the configuration is not portable to a different username.
  Picked up by someone else, it would need a manual pass.
- When templating does arrive, it will have to apply to both mechanisms from
  [0003](0003-separate-owned-files-from-shared-files.md): a generated file can no
  longer be a symlink, it becomes a **produced** file, raising the question of
  rewriting it on every run. This is not a layer you simply add on top.
- The `doctor` warning documents the debt where it lives, rather than in a file nobody
  will reread.

## Alternatives considered

**A full templating engine in v1** (chezmoi-style). Rejected: real cost for a need
that appears on none of the machines v1 targets.

**Minimal `$HOME` substitution only.** Rejected narrowly. It is cheap, but it forces
an immediate decision about which files are *produced* rather than *linked* — which is
exactly the complexity we are trying to defer. The `doctor` warning covers the need
for information without that shift.

**Do nothing and report nothing.** Rejected: hardcoded paths would then surprise us at
the worst possible moment, on a fresh machine.
