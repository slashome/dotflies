//! Executing a plan. Every decision was already taken in `plan`; this module only acts.
//!
//! What it refuses to do matters as much as what it does: it never touches a `conflict`
//! or a `drifted` entry, because both mean something is there that dotflies did not put
//! there — or no longer recognises.

use crate::manifest::LinkKind;
use crate::pkgmgr::{Brew, Npm, PackageManager};
use crate::plan::{Action, Plan};
use crate::{paths, ui};
use anyhow::{Context, Result, bail};
use camino::Utf8Path;

pub struct Outcome {
    pub done: usize,
    pub refused: usize,
}

pub fn run(plan: &Plan, dry_run: bool) -> Result<Outcome> {
    let mut outcome = Outcome {
        done: 0,
        refused: 0,
    };

    for entry in plan.actionable() {
        let Some(action) = &entry.action else { continue };

        if let Action::NotImplemented(what) = action {
            ui::refused(entry, &format!("{what} are not implemented in this build"));
            outcome.refused += 1;
            continue;
        }

        if dry_run {
            ui::would(entry);
            outcome.done += 1;
            continue;
        }

        perform(action).with_context(|| format!("applying {} for {}", entry.what, entry.app))?;
        ui::did(entry);
        outcome.done += 1;
    }

    Ok(outcome)
}

fn perform(action: &Action) -> Result<()> {
    match action {
        Action::Link {
            source,
            target,
            kind,
        } => lay_link(source, target, *kind),
        Action::InstallBrewFormula(p) => Brew { cask: false }.install(p),
        Action::InstallBrewCask(p) => Brew { cask: true }.install(p),
        Action::InstallNpmGlobal(p) => Npm.install(p),
        Action::NotImplemented(_) => Ok(()),
    }
}

/// Lay one symlink. Only ever called for an `absent` target, so there is nothing here to
/// destroy — the check that guarantees that lives in `plan`, not in a flag passed down.
fn lay_link(source: &Utf8Path, target: &Utf8Path, kind: LinkKind) -> Result<()> {
    match kind {
        LinkKind::File if !source.is_file() => {
            bail!("{source} is not a file")
        }
        LinkKind::Directory if !source.is_dir() => {
            bail!("{source} is not a directory")
        }
        _ => {}
    }

    if let Some(parent) = target.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {parent}"))?;
    }

    if std::fs::symlink_metadata(target).is_ok() {
        bail!(
            "{} appeared between planning and applying — rerun doctor",
            paths::contract(target)
        );
    }

    std::os::unix::fs::symlink(source, target)
        .with_context(|| format!("linking {} -> {source}", paths::contract(target)))
}

pub fn summarise(outcome: &Outcome, plan: &Plan, dry_run: bool) -> String {
    let verb = if dry_run { "would apply" } else { "applied" };
    let problems = plan.problems().count();
    let mut parts = vec![format!("{} {}", outcome.done, verb)];
    if outcome.refused > 0 {
        parts.push(format!("{} refused", outcome.refused));
    }
    if problems > 0 {
        parts.push(format!("{problems} left alone"));
    }
    parts.join(", ")
}
