use crate::bootstrap::Bootstrap;
use crate::{paths, policy};
use anyhow::Result;
use wasmtime_wasi::WasiCtxBuilder;

pub fn configure(wasi_builder: &mut WasiCtxBuilder, bootstrap: &Bootstrap) -> Result<()> {
    let command_policy =
        policy::from_argv(&bootstrap.argv, bootstrap.project_root.as_deref());

    for area in &command_policy.areas {
        let native_path = paths::area_path(bootstrap.project_root.as_deref(), *area)
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        std::fs::create_dir_all(&native_path)?;
        wasi_builder.preopened_dir(
            native_path,
            paths::guest_mount_name(*area),
            wasmtime_wasi::DirPerms::all(),
            wasmtime_wasi::FilePerms::all(),
        )?;
    }

    Ok(())
}
