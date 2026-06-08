use std::env;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedFlags {
    pub json_output: bool,
    pub dry_run: bool,
    pub force: bool,
    pub verbose: bool,
    pub debug: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedOverrides {
    pub project: Option<String>,
    pub tool_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgv {
    pub flags: ParsedFlags,
    pub overrides: ParsedOverrides,
}

pub fn parse(argv: &[String]) -> ParsedArgv {
    let mut flags = ParsedFlags::default();
    let mut overrides = ParsedOverrides::default();
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--json" => flags.json_output = true,
            "--dry-run" => flags.dry_run = true,
            "--force" => flags.force = true,
            "--verbose" => flags.verbose = true,
            "--debug" => flags.debug = true,
            "--project" => {
                if let Some(path) = argv.get(i + 1) {
                    overrides.project = Some(path.clone());
                    i += 1;
                }
            }
            arg if let Some(path) = arg.strip_prefix("--project=") => {
                overrides.project = Some(path.to_string());
            }
            "--tool-version" => {
                if let Some(version) = argv.get(i + 1) {
                    overrides.tool_version = Some(version.clone());
                    i += 1;
                }
            }
            arg if let Some(version) = arg.strip_prefix("--tool-version=") => {
                overrides.tool_version = Some(version.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    if overrides.tool_version.is_none() {
        if let Ok(version) = env::var("TRACK_TOOL_VERSION") {
            overrides.tool_version = Some(version);
        }
    }

    ParsedArgv { flags, overrides }
}
