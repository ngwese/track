# Architecture Decision Records

This directory holds [Architecture Decision Records](https://adr.github.io/)
(ADRs) for Track — concise, durable documents that capture significant technical
choices, the context that motivated them, and their consequences.

## Format

ADRs use a numbered, immutable sequence. Superseded decisions remain in place; a
later ADR references what it replaces.

| Field | Meaning |
|-------|---------|
| **Status** | `Proposed` → `Accepted` → (`Deferred` \| `Deprecated` \| `Superseded`) |
| **Date** | When the decision was recorded |
| **Deciders** | Who approved (or is reviewing) the decision |

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-implementation-runtime.md) | Implementation runtime (WASIp2 + WebAssembly components) | Deferred |
| [0002](0002-host-guest-wit-interfaces.md) | Host–guest WIT interfaces and on-disk storage scopes | Deferred |
| [0003](0003-domain-model-and-replication-log.md) | Domain model and replication log | Proposed (reconciled with SRD v0.5) |
| [0004](0004-hub-sync-protocol-and-compaction.md) | Hub sync protocol, cursors, acknowledgements, and compaction | Proposed (supersedes SRD Appendix D) |
| [0005](0005-hub-implementation-conformance.md) | Hub implementation conformance suite (restart durability) | Proposed |
| [0006](0006-formal-verification-hub-sync-tlaplus.md) | Formal verification of hub sync protocol (TLA+) | Proposed |
