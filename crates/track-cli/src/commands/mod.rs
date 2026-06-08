pub mod auth;
pub mod schema;

use crate::output;
use crate::router;
use crate::track::host::{
    auth as host_auth, capabilities, locations, offline_queue, project_state, registry,
    session, user_config,
};
use track_types::CommandResult;

pub fn help(json: bool) -> Result<(), ()> {
    let text = "track — CLI-first issue tracker\n\nCommands:\n  version\n  help\n  auth list|login\n  schema validate\n  interfaces (debug)";
    if json {
        output::print_json(&CommandResult {
            ok: true,
            command: "help".into(),
            message: Some(text.into()),
        });
    } else {
        output::print_text(text);
    }
    Ok(())
}

pub fn version(
    invocation: &session::Invocation,
    json: bool,
) -> Result<(), ()> {
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

    output::print_text(&format!("track-cli {} (stub)", invocation.tool_version));
    output::print_text(&format!("argv: {:?}", invocation.argv));
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
    let _ = registry::resolve(&invocation.tool_version, None);
    Ok(())
}
