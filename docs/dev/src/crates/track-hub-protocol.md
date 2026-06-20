# track-hub-protocol

## Role

Hub sync protocol records (ADR 0004): push/pull requests and responses, cursors,
offsets, NDJSON helpers, snapshot and compaction types. Framing-independent.

## Classification

Domain / wire types

## Key dependencies

`track-replication`, `track-id`

## Public surface

- Push/pull: `PushRequest`, `PushResponse`, `PullRequest`, `PulledEvent`
- Cursors: `CursorSet`, `NodeCursor`, `HubOffset`
- Modules: `ndjson`, `snapshot`, `compaction`
- `TRACK_PROTOCOL_VERSION`, version header constants

## Implementations in repo

No traits — serde records and protocol helpers.

## Related ADRs / SRD

- [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)
- SRD Appendix D (hub API sketch)

## When to touch

- Protocol version bumps or wire shape changes
- New snapshot or compaction metadata fields
