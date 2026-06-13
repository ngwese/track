# track-host

Native launcher for Track. Produces the `track` executable on `PATH`.

## Role

`track-host` is the thin native **host** in [ADR 0001](../../docs/adr/0001-implementation-runtime.md). It is the only code with direct OS access. Before any CLI logic runs, it:

1. Parses host bootstrap flags with **clap** (`--project`, `--log-level`)
2. Discovers the project root (`track.yaml` walk per [SRD §3.2.1](../../docs/SRD.md))
3. Resolves the `track-cli` component from `track-version.yaml` or `TRACK_CLI_VERSION`
4. Preopens user and project storage areas for WASI
5. Instantiates the guest with a clean argv and delegates to `wasi:cli/run`

Command routing, project manifest parsing, and guest flags (`--json`, etc.) live entirely in **`track-cli`**.

## Host bootstrap flags

| Flag / env | Purpose |
|------------|---------|
| `--project PATH` / `TRACK_PROJECT` | Override project-root discovery |
| `--log-level LEVEL` / `TRACK_LOG_LEVEL` | Log level for host and guest (default: `info`) |
| `TRACK_CLI_VERSION` | Override CLI component version (else `track-version.yaml` beside `track.yaml`) |
| `TRACK_CLI_COMPONENT` | Dev override: load this `track_cli.wasm` directly |

Host flags are stripped from argv before the guest runs. See `track help` for the host-options section in usage text.

## Modules

| Module | Responsibility |
|--------|----------------|
| `host_cli.rs` | clap parsing; strip host flags; build guest argv |
| `bootstrap.rs` | Project discovery, CLI version resolution, component path |
| `version_config.rs` | Read `track-version.yaml` (host-only pin file) |
| `preopen.rs` | Storage areas and capability flags from project presence |
| `preopens.rs` | WASI directory preopens + logging |
| `registry_store.rs` | Component cache and `TRACK_CLI_COMPONENT` override |
| `host_impl.rs` | WIT `Host` trait implementations on `HostState` |
| `logging.rs` | `env_logger` initialization |

## Build and run

```bash
cargo build -p track-cli --target wasm32-wasip2
TRACK_CLI_COMPONENT=target/wasm32-wasip2/debug/track_cli.wasm cargo run -p track-host -- version
```

## See also

- [ADR 0002 — Host–guest WIT interfaces](../../docs/adr/0002-host-guest-wit-interfaces.md)
- [Development workflow](../../docs/development.md)
