//! Track CLI entrypoint.

mod cli;
mod output;
mod tracing_init;

use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Command, InitArgs, PushArgs, SchemaCommand};
use tracing::info_span;
use track_node::{
    BootstrapRequest, CommandKind, InitRequest, PushRequest, SchemaValidateRequest, bootstrap,
    init, push, schema_validate,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();
    tracing_init::init_tracing(cli.verbose, cli.debug, cli.trace);

    let cwd = std::env::current_dir().context("read current directory")?;
    let log_level = tracing_init::resolved_level(cli.verbose, cli.debug, cli.trace);

    let bootstrap_span = info_span!(
        "track.bootstrap",
        cwd = %cwd.display(),
        explicit_project = tracing::field::Empty,
        log_level = %log_level,
        project_root = tracing::field::Empty,
        discovery_method = tracing::field::Empty,
        node_uuid = tracing::field::Empty,
        default_actor = tracing::field::Empty,
        user_config = tracing::field::Empty,
    );
    let _bootstrap_guard = bootstrap_span.enter();

    let command_kind = match &cli.command {
        Command::Init(args) => CommandKind::Init {
            force: args.force,
            standalone: args.standalone,
        },
        Command::Schema { .. } | Command::Push(_) => CommandKind::RequiresProject,
    };

    if let Some(project) = &cli.project {
        bootstrap_span.record("explicit_project", project.display().to_string());
    }

    let outcome = bootstrap(BootstrapRequest {
        cwd: cwd.clone(),
        explicit_project: cli.project.clone(),
        command: command_kind,
        locations_override: track_locations::LocationsOverride::default(),
    })
    .context("bootstrap failed")?;

    tracing::info!(
        cwd = %cwd.display(),
        explicit_project = cli.project.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
        log_level = %log_level,
        node_uuid = %outcome.user_identity.node_uuid,
        default_actor = %outcome.user_identity.default_actor,
        user_config = %outcome.locations.user.config.display(),
        "track startup"
    );

    if let Some(root) = &outcome.project_root {
        bootstrap_span.record("project_root", root.display().to_string());
    }
    bootstrap_span.record(
        "discovery_method",
        format!("{:?}", outcome.discovery_method),
    );
    bootstrap_span.record("node_uuid", outcome.user_identity.node_uuid.to_string());
    bootstrap_span.record(
        "default_actor",
        outcome.user_identity.default_actor.to_string(),
    );
    bootstrap_span.record(
        "user_config",
        outcome.locations.user.config.display().to_string(),
    );

    let json = cli.json;
    match cli.command {
        Command::Init(args) => run_init(json, outcome, args),
        Command::Schema { command } => match command {
            SchemaCommand::Validate => run_schema_validate(json, outcome),
        },
        Command::Push(args) => run_push(json, outcome, args),
    }
}

fn run_init(
    json: bool,
    outcome: track_node::BootstrapOutcome,
    args: InitArgs,
) -> anyhow::Result<ExitCode> {
    let span = info_span!(
        "track.init",
        key = %args.key,
        template = %args.template,
        force = args.force,
        standalone = args.standalone,
    );
    let _guard = span.enter();
    let response = init(InitRequest {
        bootstrap: outcome,
        key: args.key,
        name: args.name,
        workspace: args.workspace,
        template: args.template,
        force: args.force,
        standalone: args.standalone,
    })
    .context("failed to initialize project")?;
    output::print_init(json, &response);
    Ok(ExitCode::SUCCESS)
}

fn run_schema_validate(
    json: bool,
    outcome: track_node::BootstrapOutcome,
) -> anyhow::Result<ExitCode> {
    let span = info_span!("track.schema.validate");
    let _guard = span.enter();
    let response = schema_validate(SchemaValidateRequest { bootstrap: outcome })
        .context("schema validation failed")?;
    output::print_schema_validate(json, &response);
    if response.valid {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn run_push(
    json: bool,
    outcome: track_node::BootstrapOutcome,
    args: PushArgs,
) -> anyhow::Result<ExitCode> {
    let span = info_span!(
        "track.push",
        dry_run = args.dry_run,
        schema_only = args.schema_only,
        work_only = args.work_only,
    );
    let _guard = span.enter();
    let response = push(PushRequest {
        bootstrap: outcome,
        dry_run: args.dry_run,
        schema_only: args.schema_only,
        work_only: args.work_only,
        exit_code: args.exit_code,
    })
    .context("push failed")?;
    output::print_push(json, &response);
    if args.exit_code && response.would_apply {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
