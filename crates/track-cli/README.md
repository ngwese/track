# track-cli

Track CLI logic compiled as a **WebAssembly component** (`wasm32-wasip2`).

## Role

`track-cli` is the **guest** in [ADR 0001](../../docs/adr/0001-implementation-runtime.md). It contains command routing, schema validation, push/pull planning, hub client behavior, and agent-oriented output — everything that should be versioned per project and portable across machines.

The guest has no direct OS access. It imports:

- **`track:host/*`** — session context, storage areas, credentials, project state, offline queue, component registry ([ADR 0002](../../docs/adr/0002-host-guest-wit-interfaces.md))
- **WASI Preview 2** — stdio, filesystem, clocks, sockets (linked by `track-host`, not declared in the guest WIT world)

The component entry point is `wasi:cli/run`.

## Artifact

Building produces a dynamic library packaged as a component:

```bash
cargo build -p track-cli --target wasm32-wasip2
# → target/wasm32-wasip2/debug/track_cli.wasm
```

`track-host` loads this file at runtime (from the build tree, user cache, or `TRACK_CLI_COMPONENT`).

## Current status

Feasibility **stub**: prints bootstrap metadata from host imports and exercises WIT calls when invoked as `track interfaces`. Real subcommands (`init`, `push`, `pull`, …) are planned in the [implementation plan](../../docs/plans/adr-0001-implementation-plan.md).

## Dependencies

- **wit-bindgen** — guest bindings from `wit/track/` (`cli-guest` world)
- **track-wit-deps** (build) — ensures WASI WIT deps exist before bindgen

## See also

- WIT sources: [`wit/track/`](../../wit/track/)
- [Implementation plan](../../docs/plans/adr-0001-implementation-plan.md)
