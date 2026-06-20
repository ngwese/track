# track-id

## Role

Stable identity primitives for Track: ULIDs, URNs, actors, stream IDs, and node
UUIDs.

## Classification

Domain / types

## Key dependencies

None (leaf crate). External: `ulid`, `nutype`, `serde`, `strum`.

## Public surface

- `TrackUlid`, `NodeUuid`, `EntityUrn`, `StreamId`, `SchemaVersion`
- `Actor`, `EntityType`
- `IdError`

## Implementations in repo

No traits. Pure newtypes and validation helpers.

## Related ADRs / SRD

- [ADR 0003 §Identity model](../../adr/0003-domain-model-and-replication-log.md)
- SRD §2.2

## When to touch

- Adding a new identifier type used across domain and wire formats
- Changing ULID/URN validation rules
