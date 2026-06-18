# Hub sync TLA+ specification

Abstract model of [ADR 0004](../../docs/adr/0004-hub-sync-protocol-and-compaction.md)
hub sync protocol, verified with TLC per
[ADR 0006](../../docs/adr/0006-formal-verification-hub-sync-tlaplus.md).

## Layout

| File | Phase | Purpose |
| --- | --- | --- |
| `HubSync.tla` | 0 | Root spec: `Init`, `Next`, `Spec` |
| `HubSync.cfg` | 0 | TLC constants, bounds, invariants |
| `Common.tla` | 0 | Shared operators |
| `Hub.tla` | 0 | Push accept and durable promotion |
| `Node.tla` | 0 | Pull window, persist-before-cursor helper |
| `Properties.tla` | 0 | Re-exports `Inv_*` from `HubSync.tla` for documentation |
| `Network.tla` | 2 | Message loss, duplication, abort |
| `Snapshots.tla` | 2 | Snapshot bootstrap |
| `Compaction.tla` | 2 | Retention watermarks |
| `run-tlc.sh` | — | Local TLC or Docker wrapper |

## Phase 0 scope (current)

The v0 model covers push, pull, and cursor rules with these **intentional
abstractions**:

- **Numeric model values in CI** — `HubSync.cfg` uses `Nodes = {1, 2}` and
  `Events = {1, 2, 3}`; authorship is the `Author` operator in `HubSync.tla`.
- **Invariants in root module** — TLC accepts one root file; `Inv_*` definitions
  live in `HubSync.tla`. `Properties.tla` re-exports them for plan cross-refs.
- **Single cursor per syncing node** — not yet the per-authoring-node cursor
  map from ADR 0004 §Cursor model (Phase 1).
- **Atomic logical steps** — no `Network.tla` interleaving; partial push/pull
  failure comes in Phase 2.
- **No snapshots or compaction** — stub modules only.

### Properties checked in CI (Phase 0)

| Property | ADR 0006 ID |
| --- | --- |
| `Inv_IdempotentAppend` | `Inv_IdempotentAppend` |
| `Inv_DurableOnlyPull` | `Inv_DurableOnlyPull` |
| `Inv_PersistBeforeCursor` | `Inv_PersistBeforeCursor` |
| `Inv_AcceptedNotPullable` | supports `Inv_DurableOnlyPull` / ack levels |
| `Inv_CursorWithinHub` | cursor sanity |

Default CI bounds (`HubSync.cfg`) complete in ~2s (~108k distinct states on a
modern laptop). Re-benchmark after adding Phase 2–4 modules.

## Prerequisites

TLC requires Java 11+. Choose one:

1. **Docker** (no local Java): `ghcr.io/tlaplus/tlaplus:latest`
2. **TLA+ Toolbox** or [tlaplus releases](https://github.com/tlaplus/tlaplus/releases):
   set `TLA_TOOLS_JAR` to `tla2tools.jar`
3. **VS Code TLA+ extension** with TLC installed

Pin the TLC version used in CI in the implementation plan once CI is wired.

## Run locally

```bash
cd spec/tla
chmod +x run-tlc.sh
./run-tlc.sh
```

Or with explicit `tla2tools.jar`:

```bash
export TLA_TOOLS_JAR=/path/to/tla2tools.jar
java -cp "$TLA_TOOLS_JAR" tlc2.TLC -config HubSync.cfg HubSync.tla
```

## Workflow

1. Change [ADR 0004](../../docs/adr/0004-hub-sync-protocol-and-compaction.md) or
   this model in the same PR when behaviour changes.
2. Run `./run-tlc.sh` until all `INVARIANT` entries in `HubSync.cfg` pass.
3. If TLC emits a counterexample trace, add a minimal `HUB_SYNC-*` integration
   test when the trace maps to deployable Rust behaviour.

## Traceability

See [ADR 0006 §Traceability to integration tests](../../docs/adr/0006-formal-verification-hub-sync-tlaplus.md)
and the [implementation plan](../../docs/plans/adr-0006-formal-verification-implementation-plan.md).
