use crate::bootstrap::Bootstrap;
use std::path::PathBuf;
use track_host_wit::track::host::{
    auth, capabilities, locations, offline_queue, project_lock, project_state, registry,
    session, user_config,
};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub struct HostState {
    pub bootstrap: Bootstrap,
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl HostState {
    pub fn new(bootstrap: Bootstrap, wasi_ctx: WasiCtx) -> Self {
        Self {
            bootstrap,
            wasi_ctx,
            resource_table: ResourceTable::new(),
        }
    }

    fn user_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("track")
    }

    fn user_state_dir() -> PathBuf {
        dirs::state_dir()
            .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("track")
    }

    fn user_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("track")
    }

    fn area_path(&self, area: locations::Area) -> Result<PathBuf, locations::Error> {
        match area {
            locations::Area::UserConfig => Ok(Self::user_config_dir()),
            locations::Area::UserState => Ok(Self::user_state_dir()),
            locations::Area::UserCache => Ok(Self::user_cache_dir()),
            locations::Area::ProjectConfig
            | locations::Area::ProjectState
            | locations::Area::ProjectCache => {
                let root = self.bootstrap.project_root.clone().ok_or_else(|| {
                    locations::Error {
                        code: locations::ErrorCode::NotInProject,
                        message: "no project root discovered".into(),
                    }
                })?;
                Ok(match area {
                    locations::Area::ProjectConfig => root,
                    locations::Area::ProjectState => root.join(".track"),
                    locations::Area::ProjectCache => root.join(".track").join("cache"),
                    _ => unreachable!(),
                })
            }
        }
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
            tool_digest: None,
            host_version: env!("CARGO_PKG_VERSION").to_string(),
            parsed_flags: session::CliFlags {
                json_output: false,
                dry_run: false,
                force: false,
                verbose: false,
                debug: false,
            },
            project_override: None,
            tool_version_override: None,
        }
    }
}

impl capabilities::Host for HostState {
    fn get(&mut self) -> capabilities::CapabilityFlags {
        capabilities::CapabilityFlags {
            network: true,
            hub_allowlist: Vec::new(),
            stdin: true,
            stdout: true,
            stderr: true,
        }
    }
}

impl locations::Host for HostState {
    fn get(&mut self, area: locations::Area) -> Result<locations::PathInfo, locations::Error> {
        let native_path = self.area_path(area)?;
        std::fs::create_dir_all(&native_path).map_err(|err| locations::Error {
            code: locations::ErrorCode::PermissionDenied,
            message: err.to_string(),
        })?;
        Ok(locations::PathInfo {
            area,
            native_path: native_path.display().to_string(),
            guest_path: format!("{area:?}"),
        })
    }

    fn list_available(&mut self) -> Vec<locations::Area> {
        let mut areas = vec![
            locations::Area::UserConfig,
            locations::Area::UserState,
            locations::Area::UserCache,
        ];
        if self.bootstrap.project_root.is_some() {
            areas.extend([
                locations::Area::ProjectConfig,
                locations::Area::ProjectState,
                locations::Area::ProjectCache,
            ]);
        }
        areas
    }
}

impl auth::Host for HostState {
    fn resolve(&mut self, slug: String) -> Result<auth::WorkspaceAuth, auth::Error> {
        Err(auth::Error {
            code: auth::ErrorCode::NotConfigured,
            message: format!("workspace {slug} not configured (stub)"),
        })
    }

    fn list_workspaces(&mut self) -> Vec<auth::WorkspaceSummary> {
        Vec::new()
    }
}

impl user_config::Host for HostState {
    fn read(&mut self) -> Result<String, user_config::Error> {
        Ok(r#"{"workspaces":[]}"#.into())
    }

    fn write(&mut self, _json: String) -> Result<(), user_config::Error> {
        Ok(())
    }

    fn upsert_workspace(
        &mut self,
        _entry: user_config::WorkspaceEntry,
    ) -> Result<(), user_config::Error> {
        Ok(())
    }

    fn remove_workspace(&mut self, _slug: String) -> Result<(), user_config::Error> {
        Ok(())
    }
}

impl project_lock::Host for HostState {
    fn acquire(&mut self, _blocking: bool) -> Result<(), project_lock::Error> {
        Ok(())
    }

    fn release(&mut self) -> Result<(), project_lock::Error> {
        Ok(())
    }
}

impl project_state::Host for HostState {
    fn read(&mut self) -> Result<String, project_state::Error> {
        Ok(r#"{"stub":true}"#.into())
    }

    fn write(&mut self, _json: String) -> Result<(), project_state::Error> {
        Ok(())
    }
}

impl offline_queue::Host for HostState {
    fn enqueue(
        &mut self,
        _mutation: offline_queue::Mutation,
    ) -> Result<(), offline_queue::Error> {
        Ok(())
    }

    fn list_queued(
        &mut self,
        _workspace_slug: Option<String>,
    ) -> Result<Vec<offline_queue::Mutation>, offline_queue::Error> {
        Ok(Vec::new())
    }

    fn drain(
        &mut self,
        _workspace_slug: String,
        _limit: u32,
    ) -> Result<Vec<offline_queue::Mutation>, offline_queue::Error> {
        Ok(Vec::new())
    }

    fn ack(&mut self, _ids: Vec<String>) -> Result<(), offline_queue::Error> {
        Ok(())
    }

    fn get_status(
        &mut self,
        _workspace_slug: String,
    ) -> Result<offline_queue::QueueStatus, offline_queue::Error> {
        Ok(offline_queue::QueueStatus {
            pending: 0,
            oldest: None,
        })
    }
}

impl registry::Host for HostState {
    fn resolve(
        &mut self,
        version: String,
        digest: Option<String>,
    ) -> Result<registry::Artifact, registry::Error> {
        Ok(registry::Artifact {
            version,
            digest: digest.unwrap_or_else(|| "stub".into()),
            cache_path: format!(
                "components/{}/track_cli.wasm",
                self.bootstrap.tool_version
            ),
        })
    }
}
