//! Startup bootstrap: user identity and project resolution.

use std::path::PathBuf;

use track_locations::{
    Locations, LocationsOverride, ensure_bucket_dirs, ensure_user_identity,
    resolve_project_locations, resolve_user_locations,
};
use track_project::{
    DiscoveryMethod as ProjectDiscovery, discover_project_root, resolve_init_target,
};

use crate::error::NodeError;

pub use track_locations::UserIdentity;

/// Which command is being run (affects project resolution).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandKind {
    /// `track init`
    Init {
        /// Force re-init.
        force: bool,
        /// Standalone layout.
        standalone: bool,
    },
    /// Commands requiring an existing project.
    RequiresProject,
}

/// Bootstrap inputs from the CLI layer.
#[derive(Clone, Debug)]
pub struct BootstrapRequest {
    /// Process working directory.
    pub cwd: PathBuf,
    /// Optional explicit `--project` path.
    pub explicit_project: Option<PathBuf>,
    /// Command being executed.
    pub command: CommandKind,
    /// User bucket overrides (tests).
    pub locations_override: LocationsOverride,
}

/// Resolved startup state for command handlers.
#[derive(Clone, Debug)]
pub struct BootstrapOutcome {
    /// Process working directory at startup.
    pub cwd: PathBuf,
    /// All storage bucket paths.
    pub locations: Locations,
    /// Loaded or created user identity.
    pub user_identity: UserIdentity,
    /// Resolved project root when applicable.
    pub project_root: Option<PathBuf>,
    /// How the project root was resolved.
    pub discovery_method: DiscoveryMethod,
}

/// Project discovery method exposed to tracing spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryMethod {
    /// No project in scope (e.g. fresh init target only).
    None,
    /// `--project PATH` was provided.
    Explicit,
    /// Walked up from cwd and found `track.yaml`.
    WalkUp,
    /// Init target path resolution (may not exist yet).
    InitTarget,
}

impl From<ProjectDiscovery> for DiscoveryMethod {
    fn from(value: ProjectDiscovery) -> Self {
        match value {
            ProjectDiscovery::Explicit => Self::Explicit,
            ProjectDiscovery::WalkUp => Self::WalkUp,
            ProjectDiscovery::InitTarget => Self::InitTarget,
        }
    }
}

/// Run startup bootstrap.
pub fn bootstrap(request: BootstrapRequest) -> Result<BootstrapOutcome, NodeError> {
    let user = resolve_user_locations(&request.locations_override)?;
    let user_identity = ensure_user_identity(&user)?;
    let (project_root, discovery_method) = resolve_project(&request)?;
    let project = project_root
        .as_ref()
        .map(|root| resolve_project_locations(root));
    let locations = Locations { user, project };
    ensure_bucket_dirs(&locations)?;
    Ok(BootstrapOutcome {
        cwd: request.cwd.clone(),
        locations,
        user_identity,
        project_root,
        discovery_method,
    })
}

fn resolve_project(
    request: &BootstrapRequest,
) -> Result<(Option<PathBuf>, DiscoveryMethod), NodeError> {
    match request.command {
        CommandKind::Init { standalone, .. } => {
            let (root, method) = resolve_init_target(
                &request.cwd,
                request.explicit_project.as_deref(),
                standalone,
            )?;
            Ok((Some(root), DiscoveryMethod::from(method)))
        }
        CommandKind::RequiresProject => {
            let (root, method) =
                discover_project_root(&request.cwd, request.explicit_project.as_deref())?;
            Ok((Some(root), DiscoveryMethod::from(method)))
        }
    }
}
