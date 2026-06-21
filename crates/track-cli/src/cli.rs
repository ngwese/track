//! Clap command definitions.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Flags available on every `track` command.
#[derive(Args, Debug, Clone)]
pub struct GlobalOptions {
    /// Project root directory (contains track.yaml).
    #[arg(
        long,
        global = true,
        help_heading = "Global options",
        display_order = 1000
    )]
    pub project: Option<PathBuf>,
    /// Machine-readable JSON output on stdout.
    #[arg(
        long,
        global = true,
        help_heading = "Global options",
        display_order = 1001
    )]
    pub json: bool,
    /// Log at INFO level on stderr.
    #[arg(
        short,
        long,
        global = true,
        help_heading = "Global options",
        display_order = 1002
    )]
    pub verbose: bool,
    /// Log at DEBUG level on stderr.
    #[arg(
        long,
        global = true,
        help_heading = "Global options",
        display_order = 1003
    )]
    pub debug: bool,
    /// Log at TRACE level on stderr.
    #[arg(
        long,
        global = true,
        hide = true,
        help_heading = "Global options",
        display_order = 1004
    )]
    pub trace: bool,
}

/// Track — CLI-first issue tracker.
#[derive(Parser, Debug)]
#[command(name = "track", version, about = "CLI-first issue tracker")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,
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
    #[arg(long, display_order = 1)]
    pub name: Option<String>,
    /// Workspace slug.
    #[arg(long, default_value = "personal", display_order = 2)]
    pub workspace: String,
    /// Template name or local path.
    #[arg(long, default_value = "default", display_order = 3)]
    pub template: String,
    /// Re-initialize when track.yaml already exists.
    #[arg(long, display_order = 4)]
    pub force: bool,
    /// Create project in cwd instead of ./track in repos.
    #[arg(long, display_order = 5)]
    pub standalone: bool,
}

/// Push command flags.
#[derive(Parser, Debug)]
pub struct PushArgs {
    /// Plan push without contacting the hub.
    #[arg(long, display_order = 1)]
    pub dry_run: bool,
    /// Push schema changes only.
    #[arg(long, display_order = 2)]
    pub schema_only: bool,
    /// Push materialized work only.
    #[arg(long, display_order = 3)]
    pub work_only: bool,
    /// Exit code 2 when changes would be applied.
    #[arg(long, display_order = 4)]
    pub exit_code: bool,
}
