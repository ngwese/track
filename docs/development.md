# Development workflow

## Quick start

```bash
./scripts/setup.sh
make build    # wasm guest + native host
make test     # unit + integration tests
make run      # cargo run -p track-host (default help)
make ci       # build + test + smoke version
```

## Host vs guest argv

The native `track` binary (host) parses bootstrap flags with clap, strips them, and passes a clean argv to the wasm guest:

| Layer | Flags / config |
|-------|----------------|
| **Host** | `--project`, `--log-level`, `TRACK_PROJECT`, `TRACK_LOG_LEVEL`, `TRACK_CLI_VERSION`, `track-version.yaml` |
| **Guest** | Subcommands, `--json`, `--dry-run`, `--force`, `--verbose`, `--debug`, project manifest parsing |

Run `track help` to see host options in a separate section above guest commands.

## Environment variables

| Variable | Purpose |
|----------|---------|
| `TRACK_CLI_COMPONENT` | Override path to `track_cli.wasm` (dev/CI) |
| `TRACK_CLI_VERSION` | Override CLI component version (else `track-version.yaml`) |
| `TRACK_PROJECT` | Override project-root discovery |
| `TRACK_LOG_LEVEL` | Log level for host and guest (`info`, `debug`, …) |
| `TRACK_DEV_NATIVE` | Reserved; native router binary is a future fast path |

## Logging

- **Host:** `--log-level debug` or `TRACK_LOG_LEVEL=debug` enables `env_logger` output for project discovery, preopens, and component resolution.
- **Guest:** receives `log-level` via session; also supports `--debug` / `--verbose` in argv.

## Related docs

- [ADR-0001 implementation plan](plans/adr-0001-implementation-plan.md)
- [Air-gapped distribution](distribution/air-gapped.md)
