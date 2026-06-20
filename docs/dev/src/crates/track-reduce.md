# track-reduce

## Role

Deterministic event reducers and CRDT merge primitives. Bridges the
replication log to materialized entity state via `track-store` traits.

## Classification

Concrete implementation

## Key dependencies

`track-store`, `track-entity`, `track-replication`, `track-hub-protocol`,
`track-id`, `track-store-memory` (for `ReductionEngine` test/progress wiring)

## Public surface

- `ReductionEngine` — coordinates reduction over store handles
- Per-kind reducers: `ItemReducer`, `SchemaReducer`, `CommentReducer`, …
- `merge::{LwwRegister, OrSet, OrMap, PnCounter}`
- `EventReducer`, `ReduceContext`, `ReduceOutcome`, `ReduceError`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `EventReducer` | Six reducers (item, schema, comment, relation, execution, blob) |
| `RegisterMerge<T>` | `LwwRegister<T>` |
| `OrSetMerge` | `OrSet` |

## Related ADRs / SRD

- [ADR 0003](../../adr/0003-domain-model-and-replication-log.md)
- SRD §5.1 (local reducers)

## When to touch

- New event kinds requiring reduction logic
- Merge policy changes for CRDT fields
- Quarantine or semantic validation behaviour

See [Reduction and materialize](../interfaces/reduction-and-materialize.md).
