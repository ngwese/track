use crate::commands;
use crate::output;
use crate::track::host::session::{self, Invocation};
use track_types::{CommandResult, VersionResponse};

pub fn command_tokens(argv: &[String]) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--project" | "--tool-version" => i += 2,
            arg if arg.starts_with("--project=")
                || arg.starts_with("--tool-version=")
                || is_global_flag(arg) =>
            {
                i += 1;
            }
            _ => {
                tokens.push(argv[i].clone());
                i += 1;
            }
        }
    }
    tokens
}

fn is_global_flag(arg: &str) -> bool {
    matches!(
        arg,
        "--json" | "--dry-run" | "--force" | "--verbose" | "--debug"
    )
}

fn matches_command(tokens: &[String], prefix: &[&str]) -> bool {
    tokens.len() >= prefix.len()
        && prefix
            .iter()
            .zip(tokens)
            .all(|(expected, actual)| actual == expected)
}

pub fn run() -> Result<(), ()> {
    let invocation = session::get();
    let json = invocation.parsed_flags.json_output;
    let tokens = command_tokens(&invocation.argv);

    if tokens.is_empty()
        || matches_command(&tokens, &["help"])
        || matches_command(&tokens, &["--help"])
    {
        return commands::help(json);
    }
    if matches_command(&tokens, &["version"]) {
        return commands::version(&invocation, json);
    }
    if matches_command(&tokens, &["interfaces"]) {
        return commands::interfaces();
    }
    if matches_command(&tokens, &["auth", "list"]) {
        return commands::auth::list(json);
    }
    if matches_command(&tokens, &["auth", "login"]) {
        return commands::auth::login(json);
    }
    if matches_command(&tokens, &["schema", "validate"]) {
        return commands::schema::validate(&invocation, json);
    }

    {
        let command = tokens.join(" ");
        if json {
            output::print_json(&CommandResult {
                ok: false,
                command,
                message: Some("unknown command (phase 3 router stub)".into()),
            });
        } else {
            output::print_text(&format!(
                "track-cli {}: unknown command `{command}`; try `track help`",
                invocation.tool_version
            ));
        }
        Err(())
    }
}

pub fn version_line(invocation: &Invocation) -> VersionResponse {
    VersionResponse {
        cli_version: track_types::CLI_VERSION.into(),
        tool_version: invocation.tool_version.clone(),
        host_version: invocation.host_version.clone(),
    }
}
