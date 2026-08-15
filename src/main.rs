mod apply;
mod blocks;
mod cli;
mod manifest;
mod paths;
mod pkgmgr;
mod plan;
mod ui;
mod wrapper;

use anyhow::{Context, Result};
use clap::Parser;

fn main() -> Result<()> {
    match cli::Cli::parse().command {
        cli::Command::Doctor(scope) => doctor(&scope.apps),
        cli::Command::Apply(args) => run_apply(&args.apps, args.dry_run),
    }
}

fn load(apps: &[String]) -> Result<plan::Plan> {
    let root = paths::root()?;
    let manifests = manifest::load_all(&root, apps)
        .with_context(|| format!("reading the configuration in {root}"))?;
    let installed = pkgmgr::probe();
    plan::compute(&manifests, manifest::PLATFORM, &installed)
}

fn doctor(apps: &[String]) -> Result<()> {
    let plan = load(apps)?;
    ui::report(&plan);
    ui::summary(&plan);
    ui::mechanism_hint(&plan);

    // Exit non-zero when the machine does not match the configuration, so doctor is
    // usable from a script. Problems and pending work both count.
    if plan.problems().next().is_some() || plan.actionable().next().is_some() {
        std::process::exit(1);
    }
    Ok(())
}

fn run_apply(apps: &[String], dry_run: bool) -> Result<()> {
    let plan = load(apps)?;

    if plan.actionable().next().is_none() {
        println!("nothing to do");
        ui::summary(&plan);
        return Ok(());
    }

    let outcome = apply::run(&plan, dry_run)?;
    println!("\n{}", apply::summarise(&outcome, &plan, dry_run));

    if plan.problems().next().is_some() {
        println!("run `dotflies doctor` to see what was left alone, and why");
    }
    Ok(())
}
