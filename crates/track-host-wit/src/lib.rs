wasmtime::component::bindgen!({
    world: "cli-guest",
    path: "../../wit/track",
    require_store_data_send: true,
    with: {
        "wasi": wasmtime_wasi::p2::bindings,
    },
});
