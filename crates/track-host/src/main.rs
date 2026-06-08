mod bootstrap;
mod host_impl;

use anyhow::Result;
use bootstrap::from_argv;
use host_impl::HostState;
use std::env;
use track_host_wit::CliGuest;
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();
    let bootstrap = from_argv(argv)?;
    run_component(bootstrap)
}

fn run_component(bootstrap: bootstrap::Bootstrap) -> Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::from_file(&engine, &bootstrap.component_path).map_err(|err| {
        anyhow::anyhow!(
            "load component {}: {err}",
            bootstrap.component_path.display()
        )
    })?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    CliGuest::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |host| host)?;

    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder.inherit_stdio().inherit_args().inherit_env();
    if let Some(root) = &bootstrap.project_root {
        wasi_builder.preopened_dir(
            root.clone(),
            ".",
            wasmtime_wasi::DirPerms::all(),
            wasmtime_wasi::FilePerms::all(),
        )?;
    }

    let mut store = Store::new(
        &engine,
        HostState::new(bootstrap, wasi_builder.build()),
    );

    let guest = CliGuest::instantiate(&mut store, &component, &linker)
        .map_err(|err| anyhow::anyhow!("instantiate track-cli component: {err}"))?;
    let result = guest
        .wasi_cli_run()
        .call_run(&mut store)
        .map_err(|err| anyhow::anyhow!("call wasi:cli/run: {err}"))?;

    if result.is_err() {
        std::process::exit(1);
    }
    Ok(())
}
