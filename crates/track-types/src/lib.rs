use serde::Serialize;

pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub cli_version: String,
    pub tool_version: String,
    pub host_version: String,
}

#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub ok: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u8,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceSummary>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceSummary {
    pub slug: String,
    pub hub_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_actor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SchemaValidateResponse {
    pub valid: bool,
    pub manifest: Option<String>,
}
