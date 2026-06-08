mod bootstrap;
mod flags;
mod host_impl;
mod lock_store;
mod log;
mod manifest;
mod paths;
mod policy;
mod preopens;
mod queue_store;
mod registry_store;
mod state_store;
mod user_config;

use anyhow::Result;
use bootstrap::{ensure_project, from_argv};
use host_impl::HostState;
use std::env;
use track_host_wit::CliGuest;
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();
    log::trace(format!("argv: {argv:?}"));
    let bootstrap = from_argv(argv)?;
    log::trace(format!(
        "tool_version={} component={}",
        bootstrap.tool_version,
        bootstrap.component_path.display()
    ));
    ensure_project(&bootstrap)?;
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
    preopens::configure(&mut wasi_builder, &bootstrap)?;

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
