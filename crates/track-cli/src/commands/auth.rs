use crate::output;
use crate::track::host::auth as host_auth;
use track_types::{WorkspaceListResponse, WorkspaceSummary};

pub fn list(json: bool) -> Result<(), ()> {
    let workspaces = host_auth::list_workspaces()
        .into_iter()
        .map(|w| WorkspaceSummary {
            slug: w.slug,
            hub_url: w.hub_url,
            default_actor: w.default_actor,
        })
        .collect();

    if json {
        output::print_json(&WorkspaceListResponse { workspaces });
    } else if workspaces.is_empty() {
        output::print_text("no workspaces configured; run `track auth login`");
    } else {
        for ws in &workspaces {
            output::print_text(&format!("{} -> {}", ws.slug, ws.hub_url));
        }
    }
    Ok(())
}

pub fn login(json: bool) -> Result<(), ()> {
    if json {
        output::print_json(&track_types::CommandResult {
            ok: false,
            command: "auth login".into(),
            message: Some("auth login not yet implemented in guest (phase 3 stub)".into()),
        });
    } else {
        output::print_text("auth login: not yet implemented (phase 3 stub)");
    }
    Err(())
}
