# track-replication

## Role

Replication log envelopes, HLC ordering, event classification, and typed event
payloads.

## Classification

Domain / types

## Key dependencies

`track-id`, `serde`, `time`

## Public surface

- `EventEnvelope`, `EventKind`, `Hlc`, `compare_events`
- `payload::*` — per-event payload structs
- `EventClassifier`, `EventPayload`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `EventClassifier` | `DefaultEventClassifier` |
| `EventPayload` | Each payload struct in `payload` module |

## Related ADRs / SRD

- [ADR 0003](../../adr/0003-domain-model-and-replication-log.md)
- SRD §2.15 (replication events)

## When to touch

- New event kinds or payload shapes
- HLC ordering or envelope validation changes
