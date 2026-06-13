use crate::commands;
use crate::output;
use crate::track::host::session::{self, Invocation};
use track_types::{CommandResult, VersionResponse};

#[derive(Debug, Clone, Default)]
struct GuestFlags {
    json_output: bool,
    dry_run: bool,
    force: bool,
    verbose: bool,
    debug: bool,
}

fn command_tokens(argv: &[String]) -> (GuestFlags, Vec<String>) {
    let mut flags = GuestFlags::default();
    let mut tokens = Vec::new();
    let mut index = 1;
    while index < argv.len() {
        match argv[index].as_str() {
            "--json" => flags.json_output = true,
            "--dry-run" => flags.dry_run = true,
            "--force" => flags.force = true,
            "--verbose" => flags.verbose = true,
            "--debug" => flags.debug = true,
            token => tokens.push(token.to_string()),
        }
        index += 1;
    }
    (flags, tokens)
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
    let (flags, tokens) = command_tokens(&invocation.argv);

    if tokens.is_empty()
        || matches_command(&tokens, &["help"])
        || matches_command(&tokens, &["--help"])
    {
        return commands::help(&invocation, flags.json_output);
    }
    if matches_command(&tokens, &["version"]) {
        return commands::version(&invocation, flags.json_output);
    }
    if matches_command(&tokens, &["interfaces"]) {
        return commands::interfaces();
    }
    if matches_command(&tokens, &["auth", "list"]) {
        return commands::auth::list(flags.json_output);
    }
    if matches_command(&tokens, &["auth", "login"]) {
        return commands::auth::login(flags.json_output);
    }
    if matches_command(&tokens, &["schema", "validate"]) {
        return commands::schema::validate(&invocation, flags.json_output);
    }

    {
        let command = tokens.join(" ");
        if flags.json_output {
            output::print_json(&CommandResult {
                ok: false,
                command,
                message: Some("unknown command (phase 3 router stub)".into()),
            });
        } else {
            output::print_text(&format!(
                "track-cli {}: unknown command `{command}`; try `track help`",
                invocation.cli_version
            ));
        }
        Err(())
    }
}

pub fn version_line(invocation: &Invocation) -> VersionResponse {
    VersionResponse {
        cli_version: track_types::CLI_VERSION.into(),
        tool_version: invocation.cli_version.clone(),
        host_version: invocation.host_version.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_guest_flags_from_clean_argv() {
        let (flags, tokens) = command_tokens(&[
            "track".into(),
            "--json".into(),
            "schema".into(),
            "validate".into(),
        ]);
        assert!(flags.json_output);
        assert_eq!(tokens, vec!["schema".to_string(), "validate".to_string()]);
    }
}
