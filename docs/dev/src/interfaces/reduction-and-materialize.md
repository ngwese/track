# Reduction and materialize

Traits in [`track-reduce`](../../crates/track-reduce.md) and
[`track-materialize-yaml`](../../crates/track-materialize-yaml.md). Each has a
single primary in-repo implementation today, but traits allow extension and
testing without changing call sites.

## Reduction (`track-reduce`)

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `EventReducer` | Apply one event kind to store traits | `ItemReducer`, `SchemaReducer`, `CommentReducer`, `RelationReducer`, `ExecutionReducer`, `BlobReducer` |
| `RegisterMerge<T>` | LWW register merge | `merge::LwwRegister<T>` |
| `OrSetMerge` | OR-set add/remove merge | `merge::OrSet` |

`ReductionEngine` coordinates reducers over a `StoreHandles` bundle.

## Materialization (`track-materialize-yaml`)

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `MaterializeWriter` | Write projected YAML/JSON files | `DefaultProjector` |
| `MaterializeSelector` | Choose which entities to materialize | `DefaultProjector` |
| `YamlExclusionPolicy` | Exclude fields from YAML output | `DefaultYamlExclusionPolicy` |

Materializers read through `track_store::EntityStore` — never SQLite directly.

## Data flow

```text
EventEnvelope (track-replication)
  → EventReducer (track-reduce)
    → EntityStore, SchemaStore, … (track-store traits)
      → MaterializeWriter (track-materialize-yaml)
        → work/issues/<eid>/*.yaml
```

## When to add a new trait implementor

- **New event kind:** add an `EventReducer` impl and register it in
  `ReductionEngine`.
- **New output format:** implement `MaterializeWriter` / `MaterializeSelector`
  in a sibling crate (keep YAML logic in `track-materialize-yaml`).

Reduction **correctness** is tested in `track-reduce` and integration tests;
store **trait semantics** are tested via STORE-CONF on backends.

## Related

- [track-reduce crate page](../crates/track-reduce.md)
- [track-materialize-yaml](../crates/track-materialize-yaml.md)
- [Store traits](./store.md)
