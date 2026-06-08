# track-wit-deps

Build-time helper that vendors WASI WIT dependencies into `wit/deps/`.

## Role

Track’s WIT package (`wit/track/`) depends on upstream WASI Preview 2 interfaces (cli, io, clocks, filesystem, random, sockets). Those are declared in:

- `wit/deps.toml` — dependency manifest (URLs / versions)
- `wit/deps.lock` — pinned checksums for reproducible fetches

This crate’s `build.rs` runs `wit_deps::lock_sync!("../../wit")` so `wit/deps/` is populated before **wit-bindgen** or **wasmtime bindgen** parse `wit/track/`.

The generated `wit/deps/` directory is **gitignored**; only the manifest and lock file are committed.

## Consumers

Added as a `[build-dependencies]` entry by crates that compile against WIT:

- **track-host-wit** — Wasmtime host bindgen
- **track-cli** — wit-bindgen guest bindings

Each consumer also has a minimal `build.rs` that emits `cargo:rerun-if-changed` for `deps.toml` and `deps.lock`, so Cargo rebuilds when pins change.

## Crate surface

There is no runtime API. The library target exists only so Cargo can run this crate’s build script when it is listed as a build-dependency.

## Updating WASI pins

1. Edit `wit/deps.toml`
2. Run `wit-deps update` (or any build that pulls in this crate)
3. Commit the updated `wit/deps.lock`

## See also

- [wit-deps](https://github.com/bytecodealliance/wit-deps)
- [Implementation plan](../../docs/plans/adr-0001-implementation-plan.md)
