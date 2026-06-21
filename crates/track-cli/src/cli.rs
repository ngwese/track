//! Clap command definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Track — CLI-first issue tracker.
#[derive(Parser, Debug)]
#[command(name = "track", version, about = "CLI-first issue tracker")]
pub struct Cli {
    /// Project root directory (contains track.yaml).
    #[arg(long, global = true)]
    pub project: Option<PathBuf>,
    /// Machine-readable JSON output on stdout.
    #[arg(long, global = true)]
    pub json: bool,
    /// Log at INFO level on stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,
    /// Log at DEBUG level on stderr.
    #[arg(long, global = true)]
    pub debug: bool,
    /// Log at TRACE level on stderr.
    #[arg(long, global = true, hide = true)]
    pub trace: bool,
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize a new project from a template.
    Init(InitArgs),
    /// Schema commands.
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    /// Push local changes to the sync hub.
    Push(PushArgs),
}

/// Schema subcommands.
#[derive(Subcommand, Debug)]
pub enum SchemaCommand {
    /// Validate schema files offline.
    Validate,
}

/// Init command flags.
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Project key (uppercase identifier).
    pub key: String,
    /// Display name.
    #[arg(long)]
    pub name: Option<String>,
    /// Workspace slug.
    #[arg(long, default_value = "personal")]
    pub workspace: String,
    /// Template name or local path.
    #[arg(long, default_value = "default")]
    pub template: String,
    /// Re-initialize when track.yaml already exists.
    #[arg(long)]
    pub force: bool,
    /// Create project in cwd instead of ./track in repos.
    #[arg(long)]
    pub standalone: bool,
}

/// Push command flags.
#[derive(Parser, Debug)]
pub struct PushArgs {
    /// Plan push without contacting the hub.
    #[arg(long)]
    pub dry_run: bool,
    /// Push schema changes only.
    #[arg(long)]
    pub schema_only: bool,
    /// Push materialized work only.
    #[arg(long)]
    pub work_only: bool,
    /// Exit code 2 when changes would be applied.
    #[arg(long)]
    pub exit_code: bool,
}
