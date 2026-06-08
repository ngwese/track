# track-host

Native launcher for Track. Produces the `track` executable on `PATH`.

## Role

`track-host` is the thin native **host** in [ADR 0001](../../docs/adr/0001-implementation-runtime.md). It is the only code with direct OS access. Before any CLI logic runs, it:

1. Parses the invocation and discovers the project root (`track.yaml` walk per [SRD §3.2.1](../../docs/SRD.md))
2. Resolves the `track-cli` WebAssembly component path
3. Configures Wasmtime with WASI Preview 2 and `track:host/*` imports
4. Instantiates the guest and delegates to `wasi:cli/run`

The bulk of Track behavior lives in **`track-cli`** (the guest component), versioned per project.

## Modules

| Module | Responsibility |
|--------|----------------|
| `bootstrap.rs` | argv, `--project`, project-root discovery, component path resolution (`TRACK_CLI_COMPONENT` or `target/wasm32-wasip2/debug/track_cli.wasm`) |
| `host_impl.rs` | WIT `Host` trait implementations on `HostState` |
| `user_config.rs` | `~/.config/track/config.json` read/write/validate |
| `policy.rs` | Per-command capabilities and area visibility (ADR 0002 matrix) |
| `lock_store.rs` | Advisory `.track/state.lock` via `fs2` |
| `state_store.rs` | `.track/state.json` atomic read/write |
| `queue_store.rs` | Hub mutation queue under user-state |
| `paths.rs` | Storage area → native path mapping |
| `main.rs` | Wasmtime engine, linker setup, WASI context, guest instantiation |

## Dependencies

- **`track-host-wit`** — generated Wasmtime bindings and `CliGuest` linker API
- **Wasmtime 45** + **wasmtime-wasi** — component runtime and WASI p2

## Build and run

```bash
# Build guest first (or set TRACK_CLI_COMPONENT)
cargo build -p track-cli --target wasm32-wasip2

cargo run -p track-host
cargo run -p track-host -- interfaces
```

## See also

- [ADR 0002 — Host–guest WIT interfaces](../../docs/adr/0002-host-guest-wit-interfaces.md)
- [Implementation plan](../../docs/plans/adr-0001-implementation-plan.md)
