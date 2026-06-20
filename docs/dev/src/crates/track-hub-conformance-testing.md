# track-hub-conformance-testing

## Role

Generic **HUB-CONF** lifecycle suite (ADR 0005). Persistent hub crates run
these cases to prove durable state survives process restart.

## Classification

Testing / conformance

## Key dependencies

`track-sync-testing`, `track-sync`, hub and store stack

## Public surface

- `HubConformanceFixture`, `HubConformanceHandle`, `HubConformanceAdmin`
- `SnapshotConformance`, `CompactionConformance`
- `run_core`, `run_admin`, `run_all`
- `conformance_suite!` macro
- HUB-CONF case functions (001–008)

## Implementations in repo

No reference `HubConformanceFixture` impl yet — awaits a durable hub backend
(for example `track-hub-postgres`).

Re-exports `EphemeralHubFixture` / `DurableHubFixture` types from
`track-sync-testing`.

## Related ADRs / SRD

- [ADR 0005](../../adr/0005-hub-implementation-conformance.md)

## When to touch

- New restart or durability requirements for production hubs
- HUB-CONF case catalog changes

See [Implement a new hub service](../guides/new-hub-implementation.md).
