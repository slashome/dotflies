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
        let Some(action) = &entry.action else {
            continue;
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Entry, Mechanism, State};

    /// The concession ADR 0006 makes in prose, turned into a test.
    ///
    /// Every non-`absent` entry below carries a perfectly valid action pointing at a
    /// path that does not exist. If the state gate ever stops holding, these files get
    /// created — and in the real manifests one of those targets is a `.zshrc`. The
    /// assertion is not "apply did the right number of things", it is **nothing was
    /// written**.
    #[test]
    fn apply_writes_nothing_for_any_state_but_absent() {
        let guard = tempfile::tempdir().unwrap();
        let dir = Utf8Path::from_path(guard.path()).unwrap();

        let source = dir.join("source.conf");
        std::fs::write(&source, "font_size 12").unwrap();

        let forbidden = |name: &str, state: State| Entry {
            app: "test".into(),
            mechanism: Mechanism::Link,
            what: name.into(),
            state,
            action: Some(Action::Link {
                source: source.clone(),
                target: dir.join(name),
                kind: LinkKind::File,
            }),
            warnings: Vec::new(),
            note: None,
        };

        let plan = Plan {
            entries: vec![
                forbidden("must-not-appear-ok", State::Ok),
                forbidden("must-not-appear-drifted", State::Drifted("edited".into())),
                forbidden("must-not-appear-conflict", State::Conflict("theirs".into())),
                forbidden("must-not-appear-skipped", State::Skipped("linux".into())),
                Entry {
                    state: State::Absent,
                    ..forbidden("allowed", State::Absent)
                },
            ],
            declared_checks: Vec::new(),
        };

        let outcome = run(&plan, false).unwrap();

        for name in [
            "must-not-appear-ok",
            "must-not-appear-drifted",
            "must-not-appear-conflict",
            "must-not-appear-skipped",
        ] {
            assert!(
                std::fs::symlink_metadata(dir.join(name)).is_err(),
                "{name} was written — the state gate leaked"
            );
        }
        assert!(dir.join("allowed").exists(), "the absent entry was skipped");
        assert_eq!(outcome.done, 1);
    }

    /// A mechanism this build cannot perform is refused, not attempted — so `doctor`
    /// can stay honest about blocks and wrappers while `apply` does only what it can.
    #[test]
    fn an_unimplemented_mechanism_is_refused_and_counted() {
        let plan = Plan {
            entries: vec![Entry {
                app: "zsh".into(),
                mechanism: Mechanism::Block,
                what: "dotflies:zsh in ~/.zshrc".into(),
                state: State::Absent,
                action: Some(Action::NotImplemented("managed blocks")),
                warnings: Vec::new(),
                note: None,
            }],
            declared_checks: Vec::new(),
        };

        let outcome = run(&plan, false).unwrap();
        assert_eq!(outcome.refused, 1);
        assert_eq!(outcome.done, 0);
    }
}
