//! Reading, validating and platform-resolving manifests (ADR 0007).
//!
//! Every rule the format promises is enforced here, and most of them are enforced by
//! serde rather than by hand: `deny_unknown_fields` turns a typo into an error, a
//! missing platform key fails to deserialise, and an unknown `kind` cannot be
//! constructed. What is left over — a `source` that does not exist on disk, a
//! `file_contains` with nothing to look for — is checked in `validate`.

// The format is frozen, but v1 executes only part of it: `[remote]`, `position`,
// `message` and every Linux package key are read and validated so a manifest written
// today stays valid, while the code acting on them lands later. Parsing them is the
// point — dropping the fields would let a typo through in silence (ADR 0007).
#![allow(dead_code)]

use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const ROOT_MANIFEST: &str = "dotflies.toml";
pub const APP_MANIFEST: &str = "manifest.toml";
pub const CONFIGS_DIR: &str = "configs";

#[cfg(target_os = "macos")]
pub const PLATFORM: &str = "darwin";
#[cfg(target_os = "linux")]
pub const PLATFORM: &str = "linux";

/// A value that must be declared per platform. A `target` with no platform key at all
/// is rejected — ADR 0002 rules out the implicit single path, which is the 2019 dead
/// end.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByPlatform {
    pub darwin: Option<String>,
    pub linux: Option<String>,
}

impl ByPlatform {
    pub fn get(&self, platform: &str) -> Option<&str> {
        match platform {
            "darwin" => self.darwin.as_deref(),
            "linux" => self.linux.as_deref(),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.darwin.is_none() && self.linux.is_none()
    }

    pub fn declared(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.darwin.is_some() {
            out.push("darwin");
        }
        if self.linux.is_some() {
            out.push("linux");
        }
        out
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    pub version: u32,
    pub apps: Vec<String>,
    #[serde(default)]
    pub remote: Option<Remote>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Remote {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub install: Option<Install>,
    #[serde(default, rename = "link")]
    pub links: Vec<Link>,
    #[serde(default, rename = "block")]
    pub blocks: Vec<Block>,
    #[serde(default, rename = "wrapper")]
    pub wrappers: Vec<Wrapper>,
    #[serde(default, rename = "verify")]
    pub verifies: Vec<Verify>,

    /// The directory this manifest was read from. `source` fields resolve against it.
    #[serde(skip)]
    pub dir: Utf8PathBuf,
}

/// `[install]` keys are enumerated, not open-ended: a typo must not silently install
/// nothing (ADR 0007).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Install {
    #[serde(default)]
    pub darwin: Option<Darwin>,
    #[serde(default)]
    pub linux: Option<Linux>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Darwin {
    #[serde(default)]
    pub brew_formula: Vec<String>,
    #[serde(default)]
    pub brew_cask: Vec<String>,
    #[serde(default)]
    pub npm_global: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Linux {
    #[serde(default)]
    pub pacman: Vec<String>,
    #[serde(default)]
    pub apt: Vec<String>,
    #[serde(default)]
    pub npm_global: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    #[default]
    File,
    /// The only form that survives a program rewriting its config through an atomic
    /// rename — see Meld in docs/GLOSSARY.md.
    Directory,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub source: String,
    #[serde(default)]
    pub kind: LinkKind,
    pub target: ByPlatform,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[default]
    End,
    Start,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub source: String,
    pub target: ByPlatform,
    pub marker: String,
    pub comment: String,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wrapper {
    pub target: ByPlatform,
    pub exec: ByPlatform,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Two kinds and only two, both checkable without a terminal (ADR 0007). Parsed and
/// validated in v1, never executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyKind {
    FileExists,
    FileContains,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verify {
    pub kind: VerifyKind,
    pub path: ByPlatform,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

pub fn load_root(root: &Utf8Path) -> Result<Root> {
    let path = root.join(ROOT_MANIFEST);
    if !path.exists() {
        bail!(
            "no {ROOT_MANIFEST} at {} — is {} a dotflies configuration?",
            path,
            root
        );
    }
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
    let parsed: Root = toml::from_str(&text).with_context(|| format!("parsing {path}"))?;
    if parsed.version != 1 {
        bail!(
            "{path} declares version {}, and this build only understands version 1",
            parsed.version
        );
    }
    Ok(parsed)
}

pub fn load_app(root: &Utf8Path, app: &str) -> Result<Manifest> {
    let dir = root.join(CONFIGS_DIR).join(app);
    let path = dir.join(APP_MANIFEST);
    if !path.exists() {
        bail!("{app} is listed in {ROOT_MANIFEST} but {path} does not exist");
    }
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
    let mut parsed: Manifest = toml::from_str(&text).with_context(|| format!("parsing {path}"))?;
    parsed.dir = dir;
    validate(&parsed).with_context(|| format!("validating {path}"))?;
    Ok(parsed)
}

/// Load every app listed in the root manifest, or only the ones named.
pub fn load_all(root: &Utf8Path, only: &[String]) -> Result<Vec<Manifest>> {
    let declared = load_root(root)?;

    let wanted: Vec<String> = if only.is_empty() {
        declared.apps.clone()
    } else {
        for name in only {
            if !declared.apps.iter().any(|a| a == name) {
                bail!(
                    "{name} is not listed in {ROOT_MANIFEST} (declared: {})",
                    declared.apps.join(", ")
                );
            }
        }
        only.to_vec()
    };

    wanted.iter().map(|app| load_app(root, app)).collect()
}

fn validate(m: &Manifest) -> Result<()> {
    let mut problems: Vec<String> = Vec::new();

    for (i, link) in m.links.iter().enumerate() {
        if link.target.is_empty() {
            problems.push(format!(
                "[[link]] #{}: target declares no platform — write target.darwin and/or target.linux",
                i + 1
            ));
        }
        let source = m.dir.join(&link.source);
        match link.kind {
            LinkKind::File if !source.is_file() => problems.push(format!(
                "[[link]] #{}: source {} is not a file",
                i + 1,
                source
            )),
            LinkKind::Directory if !source.is_dir() => problems.push(format!(
                "[[link]] #{}: source {} is declared kind = \"directory\" but is not one",
                i + 1,
                source
            )),
            _ => {}
        }
    }

    for (i, block) in m.blocks.iter().enumerate() {
        if block.target.is_empty() {
            problems.push(format!("[[block]] #{}: target declares no platform", i + 1));
        }
        let source = m.dir.join(&block.source);
        if !source.is_file() {
            problems.push(format!(
                "[[block]] #{}: source {} is missing",
                i + 1,
                source
            ));
        }
        if block.marker.trim().is_empty() {
            problems.push(format!("[[block]] #{}: marker is empty", i + 1));
        }
        if block.comment.trim().is_empty() {
            problems.push(format!("[[block]] #{}: comment is empty", i + 1));
        }
    }

    for (i, wrapper) in m.wrappers.iter().enumerate() {
        if wrapper.target.is_empty() {
            problems.push(format!(
                "[[wrapper]] #{}: target declares no platform",
                i + 1
            ));
        }
        if wrapper.exec.is_empty() {
            problems.push(format!("[[wrapper]] #{}: exec declares no platform", i + 1));
        }
    }

    for (i, verify) in m.verifies.iter().enumerate() {
        if verify.path.is_empty() {
            problems.push(format!("[[verify]] #{}: path declares no platform", i + 1));
        }
        if verify.kind == VerifyKind::FileContains && verify.contains.is_none() {
            problems.push(format!(
                "[[verify]] #{}: kind = \"file_contains\" needs a `contains` value",
                i + 1
            ));
        }
    }

    if problems.is_empty() {
        return Ok(());
    }
    bail!("{}", problems.join("\n  "));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Manifest> {
        toml::from_str::<Manifest>(text).map_err(Into::into)
    }

    #[test]
    fn a_target_without_a_platform_key_is_rejected() {
        let err = parse(
            r#"
            name = "kitty"
            [[link]]
            source = "kitty.conf"
            target = "~/.config/kitty/kitty.conf"
            "#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("target"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn an_unknown_install_key_is_an_error_not_a_silent_no_op() {
        let err = parse(
            r#"
            name = "kitty"
            [install]
            darwin.brew_cazk = ["kitty"]
            "#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("brew_cazk"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn an_unknown_verify_kind_is_rejected() {
        let err = parse(
            r#"
            name = "meld"
            [[verify]]
            kind = "command_output_contains"
            path.darwin = "/tmp/x"
            "#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("kind"), "unexpected error: {err}");
    }

    #[test]
    fn a_linux_only_target_parses_and_resolves_to_nothing_on_darwin() {
        let m = parse(
            r#"
            name = "kitty"
            [[link]]
            source = "kitty.conf"
            target.linux = "~/.config/kitty/kitty.conf"
            "#,
        )
        .unwrap();
        let target = &m.links[0].target;
        assert!(!target.is_empty(), "the entry is well formed");
        assert_eq!(target.declared(), vec!["linux"]);
        assert_eq!(target.get("darwin"), None, "so darwin skips it");
    }

    #[test]
    fn link_kind_defaults_to_file_and_directory_is_explicit() {
        let m = parse(
            r#"
            name = "meld"
            [[link]]
            source = "a"
            target.darwin = "~/a"
            [[link]]
            source = "b"
            kind = "directory"
            target.darwin = "~/b"
            "#,
        )
        .unwrap();
        assert_eq!(m.links[0].kind, LinkKind::File);
        assert_eq!(m.links[1].kind, LinkKind::Directory);
    }

    #[test]
    fn a_wrapper_carries_its_environment() {
        let m = parse(
            r#"
            name = "meld"
            [[wrapper]]
            target.darwin = "~/.local/bin/meld"
            exec.darwin = "/Applications/Meld.app/Contents/MacOS/Meld"
            env = { GSETTINGS_BACKEND = "keyfile" }
            "#,
        )
        .unwrap();
        assert_eq!(
            m.wrappers[0]
                .env
                .get("GSETTINGS_BACKEND")
                .map(String::as_str),
            Some("keyfile")
        );
    }

    #[test]
    fn file_contains_without_contains_fails_validation() {
        let mut m = parse(
            r#"
            name = "meld"
            [[verify]]
            kind = "file_contains"
            path.darwin = "/tmp/x"
            "#,
        )
        .unwrap();
        m.dir = Utf8PathBuf::from("/nonexistent");
        let err = validate(&m).unwrap_err();
        assert!(err.to_string().contains("contains"), "unexpected: {err}");
    }
}
