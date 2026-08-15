use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "dotflies",
    version,
    about = "Version your configuration, reinstall it in one command"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Compare your configuration to this machine and report. Writes nothing, ever.
    Doctor(Scope),

    /// Apply your configuration. Only ever acts on what is absent.
    Apply(Apply),
}

#[derive(clap::Args, Debug)]
pub struct Scope {
    /// Limit to these programs. Defaults to every program in dotflies.toml.
    pub apps: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct Apply {
    /// Limit to these programs. Defaults to every program in dotflies.toml.
    pub apps: Vec<String>,

    /// Show what would happen and stop. Free, because `plan` already exists.
    #[arg(long)]
    pub dry_run: bool,
}
