pub mod auth;
pub mod schema;

use crate::output;
use crate::router;
use crate::track::host::{
    auth as host_auth, capabilities, locations, offline_queue, project_state, registry, session,
    user_config,
};
use track_types::CommandResult;

const GUEST_COMMANDS_HELP: &str = "Commands:
  version
  help
  auth list|login
  schema validate
  interfaces (debug)

Guest options:
  --json       Emit JSON output
  --dry-run    Plan without writing
  --force      Skip confirmation prompts
  --verbose    Verbose output
  --debug      Debug output";

pub fn help(invocation: &session::Invocation, json: bool) -> Result<(), ()> {
    let text = format!(
        "track — CLI-first issue tracker\n\n{}\n\n{}",
        invocation.host_options_help, GUEST_COMMANDS_HELP
    );
    if json {
        output::print_json(&CommandResult {
            ok: true,
            command: "help".into(),
            message: Some(text),
        });
    } else {
        output::print_text(&text);
    }
    Ok(())
}

pub fn version(invocation: &session::Invocation, json: bool) -> Result<(), ()> {
    let response = router::version_line(invocation);
    if json {
        output::print_json(&response);
    } else {
        output::print_text(&format!(
            "track-cli {} (component) host {}",
            response.tool_version, response.host_version
        ));
    }
    Ok(())
}

pub fn interfaces() -> Result<(), ()> {
    let invocation = session::get();
    let caps = capabilities::get();
    let areas = locations::list_available();

    output::print_text(&format!("track-cli {} (stub)", invocation.cli_version));
    output::print_text(&format!("argv: {:?}", invocation.argv));
    output::print_text(&format!("log-level: {}", invocation.log_level));
    if let Some(root) = &invocation.project_root {
        output::print_text(&format!("project-root: {root}"));
    }
    output::print_text(&format!(
        "capabilities: network={} stdout={}",
        caps.network, caps.stdout
    ));
    output::print_text(&format!("areas: {:?}", areas));

    let _ = host_auth::list_workspaces();
    let _ = user_config::read();
    let _ = project_state::read();
    let _ = offline_queue::list_queued(None);
    let _ = registry::resolve(&invocation.cli_version, None);
    Ok(())
}
