# Hub sync TLA+ specification

Abstract model of [ADR 0004](../../docs/adr/0004-hub-sync-protocol-and-compaction.md)
hub sync protocol, verified with TLC per
[ADR 0006](../../docs/adr/0006-formal-verification-hub-sync-tlaplus.md).

## Layout

| File | Phase | Purpose |
| --- | --- | --- |
| `HubSync.tla` | 4 | Root spec: `Init`, `Next`, `Spec`, `Inv_*` |
| `HubSync.cfg` | 4 | TLC constants, bounds, invariants, liveness |
| `Common.tla` | 0 | Shared operators |
| `Hub.tla` | 0 | Push accept and durable promotion |
| `Node.tla` | 1–4 | Pull window, absolute offsets, persist-before-cursor |
| `Properties.tla` | 0 | Re-exports `Inv_*` from `HubSync.tla` for documentation |
| `Network.tla` | 2 | Push/pull stream helpers; partial commit and abort |
| `Snapshots.tla` | 3 | Published snapshot coverage and bootstrap cursors |
| `Compaction.tla` | 4 | Watermarks, prefix compaction, tombstone guards |
| `run-tlc.sh` | — | Local TLC or Docker wrapper |

## Phase 4 scope (current)

The model covers push, pull, cursors, partial streams, snapshots, and
compaction:

- **Per-authoring-node cursors** — `cursors[syncing][author]` with absolute hub
  offsets (`compactedThrough` + tail index).
- **Streaming push/pull** — bounded batches with mid-stream abort (Phase 2).
- **Published snapshots** — `PublishSnapshot` records `snapshotCoverage`,
  `snapshotThrough`, and per-author `snapshotCursors`; republish only when the hub
  grows past the prior snapshot boundary.
- **Snapshot bootstrap** — `BootstrapFromSnapshot` hydrates `localLog` and
  cursors from coverage; `ColdResetNode` models an inactive replica.
- **Compaction** — `ReportWatermark` + `CompactPrefix` below snapshot boundary
  when active replicas have caught up; archived events keep absolute offsets in
  `archivedOffsets`.
- **Tombstones** — `TombstoneEvents` (event `3` in CI) must remain in snapshot
  or tail after compaction.
- **Active replicas only** — sync actions require `nodeActive[node]`.

Remaining abstractions:

- No transport-level drop/duplicate (streaming abort only).
- Single workspace/project snapshot (no multi-scope snapshots).
- Hub admin actions are atomic (`PublishSnapshot`, `CompactPrefix`).

### Properties checked in CI (Phase 4)

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
| `Inv_PartialPush` | durable push prefix only |
| `Inv_PartialPull` | pull buffer ahead of cursor |
| `Inv_MalformedLine` | partial push/pull safety |
| `Inv_NoSilentLoss` | active replicas retain compacted history |
| `Inv_CompactionSafe` | archived prefix covered by snapshot |
| `Inv_TombstoneRetained` | tombstones survive compaction |
| `Inv_BootstrapCoverage` | snapshot bootstrap hydration |
| `Live_InactiveBootstrap` | bounded inactive-replica bootstrap liveness |

Default CI bounds (`HubSync.cfg`) complete in ~40s (~132k distinct states on a
modern laptop, 2026-06-19).

## Phase 2 scope (superseded)

Phase 2 added streaming push/pull without snapshots or compaction. Superseded by
Phase 4 above.

## Phase 1 scope (superseded)

Phase 1 used atomic push/pull. Superseded by Phase 2 streaming.

## Phase 0 scope (superseded)

Phase 0 used a single cursor per syncing node. Superseded by Phase 1.

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
2. Run `./run-tlc.sh` until all `INVARIANT` and `PROPERTY` entries in
   `HubSync.cfg` pass.
3. If TLC emits a counterexample trace, add a minimal `HUB_SYNC-*` integration
   test when the trace maps to deployable Rust behaviour.

## Traceability

See [ADR 0006 §Traceability to integration tests](../../docs/adr/0006-formal-verification-hub-sync-tlaplus.md)
and the [implementation plan](../../docs/plans/adr-0006-formal-verification-implementation-plan.md).
