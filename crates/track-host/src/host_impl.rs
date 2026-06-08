use crate::bootstrap::Bootstrap;
use crate::{lock_store, paths, policy, queue_store, state_store, user_config};
use lock_store::HeldLock;
use policy::CommandPolicy;
use track_host_wit::track::host::{
    auth, capabilities, locations, offline_queue, project_lock, project_state, registry, session,
    user_config as user_config_wit,
};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub struct HostState {
    pub bootstrap: Bootstrap,
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    policy: CommandPolicy,
    project_lock: Option<HeldLock>,
}

impl HostState {
    pub fn new(bootstrap: Bootstrap, wasi_ctx: WasiCtx) -> Self {
        let policy = policy::from_argv(&bootstrap.argv, bootstrap.project_root.as_deref());
        Self {
            bootstrap,
            wasi_ctx,
            resource_table: ResourceTable::new(),
            policy,
            project_lock: None,
        }
    }

    fn project_root(&self) -> Option<&std::path::Path> {
        self.bootstrap.project_root.as_deref()
    }

    fn area_allowed(&self, area: locations::Area) -> bool {
        self.policy.areas.contains(&area)
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

impl session::Host for HostState {
    fn get(&mut self) -> session::Invocation {
        session::Invocation {
            argv: self.bootstrap.argv.clone(),
            cwd: self.bootstrap.cwd.display().to_string(),
            project_root: self
                .bootstrap
                .project_root
                .as_ref()
                .map(|p| p.display().to_string()),
            manifest_path: self
                .bootstrap
                .manifest_path
                .as_ref()
                .map(|p| p.display().to_string()),
            tool_version: self.bootstrap.tool_version.clone(),
            tool_digest: self.bootstrap.tool_digest.clone(),
            host_version: env!("CARGO_PKG_VERSION").to_string(),
            parsed_flags: session::CliFlags {
                json_output: self.bootstrap.parsed.flags.json_output,
                dry_run: self.bootstrap.parsed.flags.dry_run,
                force: self.bootstrap.parsed.flags.force,
                verbose: self.bootstrap.parsed.flags.verbose,
                debug: self.bootstrap.parsed.flags.debug,
            },
            project_override: self.bootstrap.parsed.overrides.project.clone(),
            tool_version_override: self.bootstrap.parsed.overrides.tool_version.clone(),
        }
    }
}

impl capabilities::Host for HostState {
    fn get(&mut self) -> capabilities::CapabilityFlags {
        self.policy.capabilities.clone()
    }
}

impl locations::Host for HostState {
    fn get(&mut self, area: locations::Area) -> Result<locations::PathInfo, locations::Error> {
        if !self.area_allowed(area) {
            return Err(locations::Error {
                code: locations::ErrorCode::AreaUnavailable,
                message: format!("area {area:?} is not available for this command"),
            });
        }
        let native_path = paths::area_path(self.project_root(), area)?;
        std::fs::create_dir_all(&native_path).map_err(|err| locations::Error {
            code: locations::ErrorCode::PermissionDenied,
            message: err.to_string(),
        })?;
        Ok(locations::PathInfo {
            area,
            native_path: native_path.display().to_string(),
            guest_path: paths::guest_mount_name(area).into(),
        })
    }

    fn list_available(&mut self) -> Vec<locations::Area> {
        self.policy.areas.clone()
    }
}

impl auth::Host for HostState {
    fn resolve(&mut self, slug: String) -> Result<auth::WorkspaceAuth, auth::Error> {
        let config = user_config::load().map_err(map_config_error)?;
        let workspace = config
            .workspaces
            .into_iter()
            .find(|w| w.slug == slug)
            .ok_or_else(|| auth::Error {
                code: auth::ErrorCode::UnknownWorkspace,
                message: format!("workspace {slug} is not configured"),
            })?;
        Ok(auth::WorkspaceAuth {
            slug: workspace.slug,
            hub_url: workspace.hub_url,
            token: workspace.token,
            default_actor: workspace.default_actor,
        })
    }

    fn list_workspaces(&mut self) -> Vec<auth::WorkspaceSummary> {
        user_config::load()
            .map(|config| {
                config
                    .workspaces
                    .into_iter()
                    .map(|w| auth::WorkspaceSummary {
                        slug: w.slug,
                        hub_url: w.hub_url,
                        default_actor: w.default_actor,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl user_config_wit::Host for HostState {
    fn read(&mut self) -> Result<String, user_config_wit::Error> {
        user_config::read()
    }

    fn write(&mut self, json: String) -> Result<(), user_config_wit::Error> {
        user_config::write(&json)
    }

    fn upsert_workspace(
        &mut self,
        entry: user_config_wit::WorkspaceEntry,
    ) -> Result<(), user_config_wit::Error> {
        user_config::upsert_workspace(entry)
    }

    fn remove_workspace(&mut self, slug: String) -> Result<(), user_config_wit::Error> {
        user_config::remove_workspace(&slug)
    }
}

impl project_lock::Host for HostState {
    fn acquire(&mut self, blocking: bool) -> Result<(), project_lock::Error> {
        if self.project_lock.is_some() {
            return Err(project_lock::Error {
                code: project_lock::ErrorCode::AlreadyHeld,
                message: "project state lock already held in this invocation".into(),
            });
        }
        let root = self.project_root().ok_or_else(|| project_lock::Error {
            code: project_lock::ErrorCode::IoError,
            message: "no project root discovered".into(),
        })?;
        let held = lock_store::acquire(root, blocking)?;
        self.project_lock = Some(held);
        Ok(())
    }

    fn release(&mut self) -> Result<(), project_lock::Error> {
        let Some(held) = self.project_lock.take() else {
            return Err(project_lock::Error {
                code: project_lock::ErrorCode::Unavailable,
                message: "project state lock is not held".into(),
            });
        };
        held.release()
    }
}

impl project_state::Host for HostState {
    fn read(&mut self) -> Result<String, project_state::Error> {
        state_store::read(self.project_root())
    }

    fn write(&mut self, json: String) -> Result<(), project_state::Error> {
        state_store::write(self.project_root(), &json)
    }
}

impl offline_queue::Host for HostState {
    fn enqueue(&mut self, mutation: offline_queue::Mutation) -> Result<(), offline_queue::Error> {
        queue_store::enqueue(mutation)
    }

    fn list_queued(
        &mut self,
        workspace_slug: Option<String>,
    ) -> Result<Vec<offline_queue::Mutation>, offline_queue::Error> {
        queue_store::list_queued(workspace_slug.as_deref())
    }

    fn drain(
        &mut self,
        workspace_slug: String,
        limit: u32,
    ) -> Result<Vec<offline_queue::Mutation>, offline_queue::Error> {
        queue_store::drain(&workspace_slug, limit)
    }

    fn ack(&mut self, ids: Vec<String>) -> Result<(), offline_queue::Error> {
        queue_store::ack(&ids)
    }

    fn get_status(
        &mut self,
        workspace_slug: String,
    ) -> Result<offline_queue::QueueStatus, offline_queue::Error> {
        queue_store::status(&workspace_slug)
    }
}

impl registry::Host for HostState {
    fn resolve(
        &mut self,
        version: String,
        digest: Option<String>,
    ) -> Result<registry::Artifact, registry::Error> {
        crate::registry_store::resolve(&version, digest.as_deref())
    }
}

fn map_config_error(err: user_config_wit::Error) -> auth::Error {
    auth::Error {
        code: auth::ErrorCode::NotConfigured,
        message: err.message,
    }
}
