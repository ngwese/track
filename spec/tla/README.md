# Hub sync TLA+ specification

Abstract model of [ADR 0004](../../docs/adr/0004-hub-sync-protocol-and-compaction.md)
hub sync protocol, verified with TLC per
[ADR 0006](../../docs/adr/0006-formal-verification-hub-sync-tlaplus.md).

## Layout

| File | Phase | Purpose |
| --- | --- | --- |
| `HubSync.tla` | 1 | Root spec: `Init`, `Next`, `Spec`, `Inv_*` |
| `HubSync.cfg` | 1 | TLC constants, bounds, invariants |
| `Common.tla` | 0 | Shared operators |
| `Hub.tla` | 0 | Push accept and durable promotion |
| `Node.tla` | 1 | Per-author pull window, persist-before-cursor helpers |
| `Properties.tla` | 0 | Re-exports `Inv_*` from `HubSync.tla` for documentation |
| `Network.tla` | 2 | Message loss, duplication, abort |
| `Snapshots.tla` | 2 | Snapshot bootstrap |
| `Compaction.tla` | 2 | Retention watermarks |
| `run-tlc.sh` | — | Local TLC or Docker wrapper |

## Phase 1 scope (current)

The model covers push, pull, and per-authoring-node cursor rules:

- **Per-authoring-node cursors** — `cursors[syncing][author]` matches ADR 0004
  `known_cursors`.
- **Persist advances cursor** — each `Persist` action updates the cursor for the
  event's authoring node (ADR 0004 §Sync integration loop).
- **Numeric model values in CI** — `HubSync.cfg` uses `Nodes = {1, 2}` and
  `Events = {1, 2, 3}`; authorship is the `Author` operator in `HubSync.tla`.

Remaining abstractions:

- **Atomic push/pull** — no `Network.tla` interleaving; partial failure comes in
  Phase 2.
- **No snapshots or compaction** — stub modules only.

### Properties checked in CI (Phase 1)

| Property | ADR 0006 ID |
| --- | --- |
| `Inv_IdempotentAppend` | `Inv_IdempotentAppend` |
| `Inv_DurableOnlyPull` | `Inv_DurableOnlyPull` |
| `Inv_PersistBeforeCursor` | `Inv_PersistBeforeCursor` |
| `Inv_AcceptedNotPullable` | supports `Inv_DurableOnlyPull` / ack levels |
| `Inv_CursorWithinHub` | cursor sanity |
| `Inv_HubOffsetOrder` | pull page ordering |
| `Inv_PaginationStable` | stable pagination from cursors |
| `Inv_CursorMonotone` | cursor values are valid hub offsets |

Default CI bounds (`HubSync.cfg`) complete in < 1s (~9k distinct states on a
modern laptop, 2026-06-19). Re-benchmark after adding Phase 2–4 modules.

## Phase 0 scope (superseded)

Phase 0 used a single cursor per syncing node. Superseded by Phase 1 above.

## Prerequisites

TLC requires Java 11+. Choose one:

1. **Docker** (no local Java): `ghcr.io/tlaplus/tlaplus:latest`
2. **TLA+ Toolbox** or [tlaplus releases](https://github.com/tlaplus/tlaplus/releases):
   set `TLA_TOOLS_JAR` to `tla2tools.jar`
3. **VS Code TLA+ extension** with TLC installed

Pin the TLC version used in CI in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
(`tlc-hub-sync` job, `tla2tools.jar` v1.8.0). The job runs only when these
paths change: `spec/tla/**`, ADR 0004, ADR 0006, or the workflow file itself.

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
