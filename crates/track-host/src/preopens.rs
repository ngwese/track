use crate::bootstrap::Bootstrap;
use crate::{paths, preopen};
use anyhow::Result;
use wasmtime_wasi::WasiCtxBuilder;

pub fn configure(wasi_builder: &mut WasiCtxBuilder, bootstrap: &Bootstrap) -> Result<()> {
    for area in preopen::preopened_areas(bootstrap.project_root.as_deref()) {
        let native_path = paths::area_path(bootstrap.project_root.as_deref(), area)
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        std::fs::create_dir_all(&native_path)?;
        let guest_mount = paths::guest_mount_name(area);
        ::log::info!(
            "preopening storage area area={area:?} native_path={} guest_mount={guest_mount}",
            native_path.display()
        );
        wasi_builder.preopened_dir(
            native_path,
            guest_mount,
            wasmtime_wasi::DirPerms::all(),
            wasmtime_wasi::FilePerms::all(),
        )?;
    }

    Ok(())
}
