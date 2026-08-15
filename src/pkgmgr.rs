//! Package managers, behind one shape. No command is ever hardcoded at a call site —
//! that is the 2019 dead end, where `trizen` was baked in and the repository became
//! useless the day the system changed.

use crate::plan::Installed;
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::process::Command;

pub trait PackageManager {
    fn name(&self) -> &'static str;
    /// What is already installed. An absent manager is not an error: it reports nothing.
    fn installed(&self) -> BTreeSet<String>;
    fn install(&self, package: &str) -> Result<()>;
}

pub struct Brew {
    pub cask: bool,
}

impl PackageManager for Brew {
    fn name(&self) -> &'static str {
        if self.cask { "brew --cask" } else { "brew" }
    }

    fn installed(&self) -> BTreeSet<String> {
        let kind = if self.cask { "--cask" } else { "--formula" };
        lines_of(Command::new("brew").args(["list", kind, "-1"]))
    }

    fn install(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("brew");
        cmd.arg("install");
        if self.cask {
            cmd.arg("--cask");
        }
        cmd.arg(package);
        run(cmd, &format!("installing {package} with {}", self.name()))
    }
}

pub struct Npm;

impl PackageManager for Npm {
    fn name(&self) -> &'static str {
        "npm -g"
    }

    fn installed(&self) -> BTreeSet<String> {
        lines_of(Command::new("npm").args(["ls", "-g", "--depth=0", "--parseable"]))
            .into_iter()
            .filter_map(|line| line.rsplit('/').next().map(str::to_string))
            .collect()
    }

    fn install(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("npm");
        cmd.args(["install", "-g", package]);
        run(cmd, &format!("installing {package} with npm"))
    }
}

/// Query every manager once, rather than shelling out per package.
pub fn probe() -> Installed {
    Installed {
        brew_formula: Brew { cask: false }.installed(),
        brew_cask: Brew { cask: true }.installed(),
        npm_global: Npm.installed(),
    }
}

fn lines_of(cmd: &mut Command) -> BTreeSet<String> {
    let Ok(out) = cmd.output() else {
        return BTreeSet::new(); // manager not installed — not an error
    };
    if !out.status.success() {
        return BTreeSet::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn run(mut cmd: Command, what: &str) -> Result<()> {
    let status = cmd.status().with_context(|| what.to_string())?;
    if !status.success() {
        bail!("{what} failed ({status})");
    }
    Ok(())
}
