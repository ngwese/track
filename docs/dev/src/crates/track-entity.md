# track-entity

## Role

Materialized domain entities and schema types — logical state projected by
reducers, not log envelopes or storage rows.

## Classification

Domain / types

## Key dependencies

`track-id`, `indexmap`, `serde`, `time`

## Public surface

- `schema::*` — `CanonicalSchema`, field/enum definitions, schema operations
- `work::*` — `ReducedItem`, `Claim`, `Comment`, `Relation`, `FieldValue`, …
- `validation::*` — `EntityValidator`, `ConflictReport`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `EntityValidator` | `DefaultEntityValidator` |

## Related ADRs / SRD

- [ADR 0003 §Domain model](../../adr/0003-domain-model-and-replication-log.md)
- SRD §2

## When to touch

- New materialized entity fields or schema constructs
- Semantic validation rules during reduction
