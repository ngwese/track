use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHostCli {
    pub project: Option<PathBuf>,
    pub log_level: String,
    pub guest_argv: Vec<String>,
}

pub fn parse(raw_argv: &[String]) -> Result<ParsedHostCli, String> {
    let (host_argv, guest_argv) = split_argv(raw_argv)?;

    let matches = host_command()
        .try_get_matches_from(host_argv)
        .map_err(|err| err.to_string())?;

    Ok(ParsedHostCli {
        project: matches.get_one::<String>("project").map(PathBuf::from),
        log_level: matches
            .get_one::<String>("log-level")
            .cloned()
            .unwrap_or_else(|| "info".to_string()),
        guest_argv,
    })
}

pub fn host_options_help() -> String {
    let mut help = String::from("Host options:\n");
    help.push_str(&host_command().render_long_help().ansi().to_string());
    help.push_str(
        "\nCLI version is resolved from track-version.yaml (beside track.yaml) \
         or env TRACK_CLI_VERSION.\n",
    );
    help
}

fn host_command() -> Command {
    Command::new("track")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("project")
                .long("project")
                .env("TRACK_PROJECT")
                .value_name("PATH")
                .help("Project root override (directory containing track.yaml)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .env("TRACK_LOG_LEVEL")
                .default_value("info")
                .value_name("LEVEL")
                .help("Log level for the host and guest (e.g. info, debug)"),
        )
}

fn split_argv(raw_argv: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    if raw_argv.is_empty() {
        return Err("missing argv[0]".into());
    }

    let program = raw_argv[0].clone();
    let mut host_argv = vec![program];
    let mut guest_argv = vec!["track".to_string()];
    let mut index = 1;

    while index < raw_argv.len() {
        match raw_argv[index].as_str() {
            "--project" => {
                let Some(path) = raw_argv.get(index + 1) else {
                    return Err("--project requires a path".into());
                };
                host_argv.push("--project".into());
                host_argv.push(path.clone());
                index += 2;
            }
            arg if let Some(path) = arg.strip_prefix("--project=") => {
                if path.is_empty() {
                    return Err("--project requires a path".into());
                }
                host_argv.push(format!("--project={path}"));
                index += 1;
            }
            "--log-level" => {
                let Some(level) = raw_argv.get(index + 1) else {
                    return Err("--log-level requires a level".into());
                };
                host_argv.push("--log-level".into());
                host_argv.push(level.clone());
                index += 2;
            }
            arg if let Some(level) = arg.strip_prefix("--log-level=") => {
                if level.is_empty() {
                    return Err("--log-level requires a level".into());
                }
                host_argv.push(format!("--log-level={level}"));
                index += 1;
            }
            arg => {
                guest_argv.push(arg.to_string());
                index += 1;
            }
        }
    }

    Ok((host_argv, guest_argv))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_host_flags_and_preserves_guest_args() {
        let parsed = parse(&[
            "track".into(),
            "--log-level".into(),
            "debug".into(),
            "--project".into(),
            "/tmp/kitchen".into(),
            "--json".into(),
            "version".into(),
        ])
        .unwrap();

        assert_eq!(parsed.log_level, "debug");
        assert_eq!(parsed.project, Some(PathBuf::from("/tmp/kitchen")));
        assert_eq!(
            parsed.guest_argv,
            vec![
                "track".to_string(),
                "--json".to_string(),
                "version".to_string()
            ]
        );
    }

    #[test]
    fn host_options_help_lists_host_flags() {
        let help = host_options_help();
        assert!(help.contains("Host options:"));
        assert!(help.contains("--project"));
        assert!(help.contains("--log-level"));
        assert!(help.contains("TRACK_CLI_VERSION"));
    }
}
