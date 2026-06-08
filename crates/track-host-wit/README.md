# track-host-wit

Wasmtime host bindings for the `track:host` WIT package.

## Role

This crate bridges **`track-host`** and the WIT contracts in [`wit/track/`](../../wit/track/). It wraps a single `wasmtime::component::bindgen!` invocation against the **`cli-guest`** world:

- From the guest’s perspective, `track:host/*` interfaces are **imports**
- From the host’s perspective, those same interfaces are **exports** to implement

The generated API includes:

- `CliGuest` — instantiate a guest component, call `wasi:cli/run`
- `CliGuest::add_to_linker` — register host implementations with a Wasmtime `Linker`
- `track::host::*` trait modules — one `Host` trait per WIT interface (`session`, `locations`, `auth`, …)

`track-host` implements those traits on `HostState` in `host_impl.rs`.

## Why a separate crate?

- Keeps bindgen output isolated from the binary crate
- Allows `track-host` to depend on generated types without re-running bindgen in every host module
- Compile-time WIT path is fixed relative to `wit/track/`

## Build

Built automatically as a dependency of `track-host`:

```bash
cargo build -p track-host-wit
```

Requires `wit/deps/` to be populated (handled by **track-wit-deps** via `[build-dependencies]`).

## See also

- [ADR 0002 — Host–guest WIT interfaces](../../docs/adr/0002-host-guest-wit-interfaces.md)
- Consumer: [`track-host`](../track-host/README.md)
