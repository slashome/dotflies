//! Intended state versus observed state. **This module never writes to disk.**
//!
//! That is the single most important property in the codebase (ADR 0007): it is what
//! makes `--dry-run` free rather than a flag to remember, `doctor` the same code path as
//! `apply`, and the riskiest logic in the product testable without side effects.

use crate::manifest::{Install, LinkKind, Manifest, VerifyKind};
use crate::{blocks, paths, wrapper};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Conforms. Nothing to do.
    Ok,
    /// Nothing there; safe to lay.
    Absent,
    /// Provably ours, and changed. Reported, never silently overwritten.
    Drifted(String),
    /// Something is there and it is not ours. Never touched without `--force`.
    Conflict(String),
    /// Declared for another platform.
    Skipped(String),
}

impl State {
    pub fn needs_action(&self) -> bool {
        matches!(self, State::Absent)
    }
    pub fn is_problem(&self) -> bool {
        matches!(self, State::Drifted(_) | State::Conflict(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    Install,
    Link,
    Block,
    Wrapper,
}

impl Mechanism {
    pub fn label(&self) -> &'static str {
        match self {
            Mechanism::Install => "install",
            Mechanism::Link => "link",
            Mechanism::Block => "block",
            Mechanism::Wrapper => "wrapper",
        }
    }
}

/// What `apply` would have to do, expressed so it can be executed without re-deciding.
#[derive(Debug, Clone)]
pub enum Action {
    Link {
        source: Utf8PathBuf,
        target: Utf8PathBuf,
        kind: LinkKind,
    },
    InstallBrewFormula(String),
    InstallBrewCask(String),
    InstallNpmGlobal(String),
    /// Recognised by `plan`, refused by `apply` in this build.
    NotImplemented(&'static str),
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub app: String,
    pub mechanism: Mechanism,
    pub what: String,
    pub state: State,
    pub action: Option<Action>,
    pub warnings: Vec<String>,
    pub note: Option<String>,
}

/// A `[[verify]]` entry: parsed and validated, never executed in v1 (ADR 0007).
#[derive(Debug, Clone)]
pub struct DeclaredCheck {
    pub app: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub entries: Vec<Entry>,
    pub declared_checks: Vec<DeclaredCheck>,
}

impl Plan {
    pub fn actionable(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.state.needs_action())
    }
    pub fn problems(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.state.is_problem())
    }
}

/// Which packages are already present. Injected rather than queried inside `plan`, so
/// tests never shell out.
#[derive(Debug, Default, Clone)]
pub struct Installed {
    pub brew_formula: BTreeSet<String>,
    pub brew_cask: BTreeSet<String>,
    pub npm_global: BTreeSet<String>,
}

pub fn compute(
    manifests: &[Manifest],
    platform: &str,
    installed: &Installed,
) -> Result<Plan> {
    let mut plan = Plan::default();
    let home = paths::home()?;

    for m in manifests {
        if let Some(install) = &m.install {
            plan_install(&mut plan, m, install, platform, installed);
        }

        for link in &m.links {
            let Some(raw_target) = link.target.get(platform) else {
                plan.entries.push(skipped(
                    m,
                    Mechanism::Link,
                    &link.source,
                    &link.target.declared(),
                    platform,
                ));
                continue;
            };
            let target = paths::expand(raw_target)?;
            let source = m.dir.join(&link.source);
            let state = classify_link(&target, &source);
            let warnings = scan_for_absolute_paths(&source, link.kind, &home);

            plan.entries.push(Entry {
                app: m.name.clone(),
                mechanism: Mechanism::Link,
                what: paths::contract(&target),
                action: matches!(state, State::Absent).then(|| Action::Link {
                    source: source.clone(),
                    target: target.clone(),
                    kind: link.kind,
                }),
                state,
                warnings,
                note: link.note.clone(),
            });
        }

        for block in &m.blocks {
            let Some(raw_target) = block.target.get(platform) else {
                plan.entries.push(skipped(
                    m,
                    Mechanism::Block,
                    &block.marker,
                    &block.target.declared(),
                    platform,
                ));
                continue;
            };
            let target = paths::expand(raw_target)?;
            let source = m.dir.join(&block.source);
            let wanted = std::fs::read_to_string(&source).unwrap_or_default();
            let state = classify_block(&target, &block.comment, &block.marker, &wanted);

            plan.entries.push(Entry {
                app: m.name.clone(),
                mechanism: Mechanism::Block,
                what: format!("{} in {}", block.marker, paths::contract(&target)),
                action: matches!(state, State::Absent)
                    .then_some(Action::NotImplemented("managed blocks")),
                state,
                warnings: Vec::new(),
                note: block.note.clone(),
            });
        }

        for w in &m.wrappers {
            let Some(raw_target) = w.target.get(platform) else {
                plan.entries.push(skipped(
                    m,
                    Mechanism::Wrapper,
                    "wrapper",
                    &w.target.declared(),
                    platform,
                ));
                continue;
            };
            let target = paths::expand(raw_target)?;
            let wanted = wrapper::render(w, platform).unwrap_or_default();
            let state = classify_wrapper(&target, &wanted);

            plan.entries.push(Entry {
                app: m.name.clone(),
                mechanism: Mechanism::Wrapper,
                what: paths::contract(&target),
                action: matches!(state, State::Absent)
                    .then_some(Action::NotImplemented("generated wrappers")),
                state,
                warnings: Vec::new(),
                note: w.note.clone(),
            });
        }

        for v in &m.verifies {
            let Some(path) = v.path.get(platform) else {
                continue;
            };
            let what = match v.kind {
                VerifyKind::FileExists => format!("{path} exists"),
                VerifyKind::FileContains => format!(
                    "{path} contains {:?}",
                    v.contains.as_deref().unwrap_or_default()
                ),
            };
            plan.declared_checks.push(DeclaredCheck {
                app: m.name.clone(),
                description: what,
            });
        }
    }

    Ok(plan)
}

fn skipped(
    m: &Manifest,
    mechanism: Mechanism,
    what: &str,
    declared: &[&str],
    platform: &str,
) -> Entry {
    Entry {
        app: m.name.clone(),
        mechanism,
        what: what.to_string(),
        state: State::Skipped(if declared.is_empty() {
            format!("declares no platform, and this is {platform}")
        } else {
            format!("declared for {}, and this is {platform}", declared.join("/"))
        }),
        action: None,
        warnings: Vec::new(),
        note: None,
    }
}

fn plan_install(
    plan: &mut Plan,
    m: &Manifest,
    install: &Install,
    platform: &str,
    installed: &Installed,
) {
    let mut push = |what: String, present: bool, action: Action| {
        plan.entries.push(Entry {
            app: m.name.clone(),
            mechanism: Mechanism::Install,
            what,
            state: if present { State::Ok } else { State::Absent },
            action: (!present).then_some(action),
            warnings: Vec::new(),
            note: None,
        });
    };

    match platform {
        "darwin" => {
            let Some(d) = &install.darwin else { return };
            for p in &d.brew_formula {
                push(
                    format!("brew {p}"),
                    installed.brew_formula.contains(p),
                    Action::InstallBrewFormula(p.clone()),
                );
            }
            for p in &d.brew_cask {
                push(
                    format!("brew --cask {p}"),
                    installed.brew_cask.contains(p),
                    Action::InstallBrewCask(p.clone()),
                );
            }
            for p in &d.npm_global {
                push(
                    format!("npm -g {p}"),
                    installed.npm_global.contains(p),
                    Action::InstallNpmGlobal(p.clone()),
                );
            }
        }
        _ => {
            if install.linux.is_some() {
                plan.entries.push(Entry {
                    app: m.name.clone(),
                    mechanism: Mechanism::Install,
                    what: "packages".into(),
                    state: State::Skipped(format!(
                        "declared for linux, and this build resolves darwin only"
                    )),
                    action: None,
                    warnings: Vec::new(),
                    note: None,
                });
            }
        }
    }
}

/// A link can be `ok`, `absent` or `conflict` — **never `drifted`**.
///
/// Distinguishing "our link was destroyed by an atomic rewrite" from "this file was
/// always here" would need to know who created it, and the filesystem does not record
/// that (see docs/GLOSSARY.md, *Conflict*). Both readings get the same, safe treatment:
/// report it, touch nothing. `drifted` is reserved for blocks and wrappers, where a
/// marker or a generated header *proves* the thing was ours.
pub fn classify_link(target: &Utf8Path, expected_source: &Utf8Path) -> State {
    let meta = match std::fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return State::Absent,
        Err(e) => return State::Conflict(format!("cannot be read: {e}")),
    };

    if meta.file_type().is_symlink() {
        let pointing = match std::fs::read_link(target) {
            Ok(p) => p,
            Err(e) => return State::Conflict(format!("unreadable symlink: {e}")),
        };
        if pointing == expected_source.as_std_path() {
            return State::Ok;
        }
        // Tolerate a relative or otherwise-spelled link that resolves to the same file.
        if let (Ok(a), Ok(b)) = (target.canonicalize(), expected_source.canonicalize())
            && a == b
        {
            return State::Ok;
        }
        return State::Conflict(format!("symlink pointing at {}", pointing.display()));
    }

    if meta.file_type().is_dir() {
        State::Conflict("a real directory is here, not our link".into())
    } else if meta.file_type().is_file() {
        State::Conflict("a real file is here, not our link".into())
    } else {
        State::Conflict("something that is neither a file nor a directory".into())
    }
}

/// Blocks *can* report `drifted`: the markers prove the block was ours.
pub fn classify_block(target: &Utf8Path, comment: &str, marker: &str, wanted: &str) -> State {
    let Ok(text) = std::fs::read_to_string(target) else {
        return State::Absent;
    };
    match blocks::find(&text, comment, marker) {
        None => State::Absent,
        Some(found) if found.trim() == wanted.trim() => State::Ok,
        Some(_) => State::Drifted("our block is there but its content was edited".into()),
    }
}

/// Wrappers behave like blocks: the generated header proves authorship.
pub fn classify_wrapper(target: &Utf8Path, wanted: &str) -> State {
    let Ok(text) = std::fs::read_to_string(target) else {
        return match std::fs::symlink_metadata(target) {
            Ok(_) => State::Conflict("something is here that we cannot read".into()),
            Err(_) => State::Absent,
        };
    };
    if text == wanted {
        State::Ok
    } else if wrapper::is_ours(&text) {
        State::Drifted("we generated this wrapper and it no longer matches".into())
    } else {
        State::Conflict("a script is here that dotflies did not generate".into())
    }
}

/// ADR 0005's guardrail, widened by what the two real machines showed.
///
/// The ADR asks for absolute paths containing the *current* `$HOME`. That catches
/// portability debt on the machine that authored the file — but a path holding a
/// *different* user's home is not debt, it is already broken here. Both are reported,
/// and they are not the same message.
fn scan_for_absolute_paths(source: &Utf8Path, kind: LinkKind, home: &Utf8Path) -> Vec<String> {
    if kind == LinkKind::Directory {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(source) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for prefix in ["/Users/", "/home/"] {
        let mut rest = text.as_str();
        while let Some(at) = rest.find(prefix) {
            rest = &rest[at..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .unwrap_or(rest.len());
            let found = &rest[..end];
            if seen.insert(found.to_string()) {
                if found.starts_with(home.as_str()) {
                    out.push(format!(
                        "hardcodes {found} — same machine today, breaks on any other $HOME"
                    ));
                } else {
                    out.push(format!(
                        "hardcodes {found}, which is NOT this machine's $HOME ({home}) — already broken here"
                    ));
                }
            }
            rest = &rest[end.max(1)..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> (tempfile::TempDir, Utf8PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        (dir, path)
    }

    #[test]
    fn a_missing_target_is_absent() {
        let (_g, dir) = tmp();
        assert_eq!(
            classify_link(&dir.join("nope"), &dir.join("src")),
            State::Absent
        );
    }

    #[test]
    fn our_link_is_ok() {
        let (_g, dir) = tmp();
        let source = dir.join("kitty.conf");
        fs::write(&source, "font_size 12").unwrap();
        let target = dir.join("linked.conf");
        std::os::unix::fs::symlink(&source, &target).unwrap();
        assert_eq!(classify_link(&target, &source), State::Ok);
    }

    /// The reason `symlink_metadata` is used rather than `metadata`: a link whose
    /// destination is gone must read as "a link is here", not as "nothing is here".
    #[test]
    fn a_broken_link_is_not_mistaken_for_absent() {
        let (_g, dir) = tmp();
        let source = dir.join("gone.conf");
        fs::write(&source, "x").unwrap();
        let target = dir.join("linked.conf");
        std::os::unix::fs::symlink(&source, &target).unwrap();
        fs::remove_file(&source).unwrap();

        assert_eq!(
            classify_link(&target, &source),
            State::Ok,
            "still our link, merely pointing at a file we are about to restore"
        );
        assert_ne!(classify_link(&target, &dir.join("other")), State::Absent);
    }

    #[test]
    fn a_real_file_at_the_target_is_a_conflict_never_absent() {
        let (_g, dir) = tmp();
        let target = dir.join("kitty.conf");
        fs::write(&target, "hand written").unwrap();
        assert!(matches!(
            classify_link(&target, &dir.join("src")),
            State::Conflict(_)
        ));
    }

    #[test]
    fn a_link_pointing_elsewhere_is_a_conflict() {
        let (_g, dir) = tmp();
        let other = dir.join("other.conf");
        fs::write(&other, "x").unwrap();
        let target = dir.join("kitty.conf");
        std::os::unix::fs::symlink(&other, &target).unwrap();
        assert!(matches!(
            classify_link(&target, &dir.join("ours.conf")),
            State::Conflict(_)
        ));
    }

    #[test]
    fn an_edited_block_is_drifted_because_the_markers_prove_it_was_ours() {
        let (_g, dir) = tmp();
        let target = dir.join(".zshrc");
        fs::write(
            &target,
            "# >>> dotflies:zsh >>>\nexport PATH=tampered\n# <<< dotflies:zsh <<<\n",
        )
        .unwrap();
        assert!(matches!(
            classify_block(&target, "#", "dotflies:zsh", "export PATH=ours"),
            State::Drifted(_)
        ));
    }

    #[test]
    fn an_untouched_block_is_ok_and_a_file_without_one_is_absent() {
        let (_g, dir) = tmp();
        let target = dir.join(".zshrc");
        fs::write(
            &target,
            "alias ll='ls'\n# >>> dotflies:zsh >>>\nexport PATH=ours\n# <<< dotflies:zsh <<<\n",
        )
        .unwrap();
        assert_eq!(
            classify_block(&target, "#", "dotflies:zsh", "export PATH=ours"),
            State::Ok
        );

        let bare = dir.join("bare");
        fs::write(&bare, "alias ll='ls'\n").unwrap();
        assert_eq!(
            classify_block(&bare, "#", "dotflies:zsh", "export PATH=ours"),
            State::Absent
        );
    }

    #[test]
    fn a_foreign_script_where_a_wrapper_goes_is_a_conflict_not_drift() {
        let (_g, dir) = tmp();
        let target = dir.join("meld");
        fs::write(&target, "#!/bin/sh\nexec /usr/bin/meld\n").unwrap();
        assert!(matches!(
            classify_wrapper(&target, "#!/bin/sh\n# generated by dotflies\n"),
            State::Conflict(_)
        ));
    }

    #[test]
    fn absolute_paths_are_reported_and_a_foreign_home_reads_differently() {
        let (_g, dir) = tmp();
        let source = dir.join("settings.json");
        fs::write(
            &source,
            r#"{"dart.flutterSdkPath": "/Users/someone-else/workspace/flutter"}"#,
        )
        .unwrap();

        let warnings =
            scan_for_absolute_paths(&source, LinkKind::File, Utf8Path::new("/Users/me"));
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].contains("already broken here"),
            "got: {}",
            warnings[0]
        );
    }

    #[test]
    fn a_portable_config_produces_no_warning() {
        let (_g, dir) = tmp();
        let source = dir.join("kitty.conf");
        fs::write(&source, "font_family Hack Nerd Font Mono\nfont_size 12.0\n").unwrap();
        assert!(
            scan_for_absolute_paths(&source, LinkKind::File, Utf8Path::new("/Users/me"))
                .is_empty()
        );
    }
}
