use crate::user_config;
use std::path::Path;
use track_host_wit::track::host::{capabilities, locations::Area};

#[derive(Debug, Clone)]
pub struct CommandPolicy {
    pub capabilities: capabilities::CapabilityFlags,
    pub areas: Vec<Area>,
}

pub fn requires_project(argv: &[String]) -> bool {
    let tokens = command_tokens(argv);
    !matches!(
        tokens.first().map(String::as_str),
        None | Some("help")
            | Some("--help")
            | Some("interfaces")
            | Some("version")
            | Some("auth")
            | Some("init")
    )
}

pub fn from_argv(argv: &[String], project_root: Option<&Path>) -> CommandPolicy {
    let tokens = command_tokens(argv);
    let hub_allowlist = hub_allowlist(project_root);

    let (network, areas) = if matches_command(&tokens, &["auth"]) {
        (false, user_areas())
    } else if matches_command(&tokens, &["schema", "validate"]) {
        (false, project_only(&[Area::ProjectConfig]))
    } else if matches_command(&tokens, &["validate"]) {
        (
            false,
            project_only(&[Area::ProjectConfig, Area::ProjectCache]),
        )
    } else if matches_any_command(&tokens, &[&["push"], &["pull"], &["diff"]]) {
        (true, all_areas(project_root.is_some()))
    } else if matches_command(&tokens, &["hub"]) {
        (
            true,
            project_with(&[Area::ProjectState, Area::ProjectCache]),
        )
    } else if matches_command(&tokens, &["issue", "list"]) {
        (false, project_with(&[Area::ProjectCache]))
    } else if matches_command(&tokens, &["issue", "materialize"]) {
        (
            false,
            project_with(&[Area::ProjectConfig, Area::ProjectCache]),
        )
    } else if matches_any_command(&tokens, &[&["init"], &["clone"], &["upgrade"]]) {
        (true, all_areas(project_root.is_some()))
    } else if project_root.is_some() {
        (false, all_areas(true))
    } else {
        (false, user_areas())
    };

    CommandPolicy {
        capabilities: capabilities::CapabilityFlags {
            network,
            hub_allowlist,
            stdin: true,
            stdout: true,
            stderr: true,
        },
        areas,
    }
}

pub fn command_tokens(argv: &[String]) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--project" => {
                i += 2;
            }
            arg if arg.starts_with("--project=") => {
                i += 1;
            }
            "--tool-version" => {
                i += 2;
            }
            arg if arg.starts_with("--tool-version=") => {
                i += 1;
            }
            arg if is_global_flag(arg) => {
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

fn matches_command(tokens: &[String], prefix: &[&str]) -> bool {
    tokens.len() >= prefix.len()
        && prefix
            .iter()
            .zip(tokens)
            .all(|(expected, actual)| actual == expected)
}

fn matches_any_command(tokens: &[String], prefixes: &[&[&str]]) -> bool {
    prefixes
        .iter()
        .any(|prefix| matches_command(tokens, prefix))
}

fn is_global_flag(arg: &str) -> bool {
    matches!(
        arg,
        "--json" | "--dry-run" | "--force" | "--verbose" | "--debug"
    )
}

fn user_areas() -> Vec<Area> {
    vec![Area::UserConfig, Area::UserState, Area::UserCache]
}

fn project_only(areas: &[Area]) -> Vec<Area> {
    areas.to_vec()
}

fn project_with(extra: &[Area]) -> Vec<Area> {
    let mut areas = user_areas();
    areas.extend(extra.iter().copied());
    areas
}

fn all_areas(has_project: bool) -> Vec<Area> {
    let mut areas = user_areas();
    if has_project {
        areas.extend([Area::ProjectConfig, Area::ProjectState, Area::ProjectCache]);
    }
    areas
}

fn hub_allowlist(project_root: Option<&Path>) -> Vec<String> {
    let Some(root) = project_root else {
        return Vec::new();
    };
    let manifest = root.join("track.yaml");
    let Some(workspace_slug) = read_manifest_workspace(&manifest) else {
        return Vec::new();
    };
    let Ok(config) = user_config::load() else {
        return Vec::new();
    };
    config
        .workspaces
        .iter()
        .find(|w| w.slug == workspace_slug)
        .map(|w| vec![w.hub_url.clone()])
        .unwrap_or_default()
}

fn read_manifest_workspace(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;
    value
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_global_flags_and_project_override() {
        let argv = vec![
            "track".into(),
            "--json".into(),
            "--project".into(),
            "/tmp/p".into(),
            "schema".into(),
            "validate".into(),
        ];
        assert_eq!(
            command_tokens(&argv),
            vec!["schema".to_string(), "validate".to_string()]
        );
    }

    #[test]
    fn schema_validate_disables_network() {
        let argv = vec!["track".into(), "schema".into(), "validate".into()];
        let policy = from_argv(&argv, Some(Path::new("/proj")));
        assert!(!policy.capabilities.network);
        assert_eq!(policy.areas, vec![Area::ProjectConfig]);
    }

    #[test]
    fn push_requires_project_root_at_host() {
        assert!(requires_project(&["track".into(), "push".into()]));
        assert!(!requires_project(&[
            "track".into(),
            "auth".into(),
            "login".into()
        ]));
    }
}
