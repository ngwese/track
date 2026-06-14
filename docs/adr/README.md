# Architecture Decision Records

This directory holds [Architecture Decision Records](https://adr.github.io/) (ADRs) for Track — concise, durable documents that capture significant technical choices, the context that motivated them, and their consequences.

## Format

ADRs use a numbered, immutable sequence. Superseded decisions remain in place; a later ADR references what it replaces.

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
