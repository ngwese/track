# Development workflow

## Quick start

```bash
make build    # wasm guest + native host
make test     # unit + integration tests
make run      # cargo run -p track-host (default help)
make ci       # build + test + smoke version
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `TRACK_CLI_COMPONENT` | Override path to `track_cli.wasm` (dev/CI) |
| `TRACK_TOOL_VERSION` | Override manifest `tool.version` |
| `TRACK_LOG` | Host trace lines on stderr (`[track-host] …`) |
| `TRACK_DEV_NATIVE` | Reserved; native router binary is a future fast path |

## Logging across host and guest

- **Host:** set `TRACK_LOG=1` for bootstrap and component resolution traces.
- **Guest:** pass `--debug` or `--verbose`; the router can expand output in later releases.

Full stack debugging always runs the wasm guest under `track-host` for parity with production.

## Related docs

- [ADR-0001 implementation plan](plans/adr-0001-implementation-plan.md)
- [Air-gapped distribution](distribution/air-gapped.md)
