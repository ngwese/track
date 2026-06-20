# Introduction

The **Track Developer Guide** complements product and architecture documents
with crate-oriented documentation for contributors implementing backends,
bindings, and integrations.

## What this book covers

- High-level responsibilities of each workspace crate
- Which crates define concrete types versus trait boundaries
- An inventory of major interfaces intended for multiple implementations
- Layering diagrams and data-flow context
- Step-by-step guides for new store backends, hub implementations, and
  assembling a memory-backed HTTP hub from existing crates

## What this book does not cover

- Product vision and goals — see [PRD](../../PRD.md)
- Full domain model, file formats, and CLI specification — see [SRD](../../SRD.md)
- Architecture decision rationale — see [ADR index](../../adr/README.md)

Link upward for *why*; use this book for *where* and *how* in the Rust workspace.

## Prerequisites

- Rust **1.96** (see `rust-version` in the workspace root `Cargo.toml`)
- Familiarity with `cargo` workspace commands
- Optional: [mdBook](https://rust-lang.github.io/mdBook/) for local preview

## Build locally

```bash
cargo install mdbook mdbook-mermaid   # mdbook 0.5+, mdbook-mermaid 0.17+
cd docs/dev
mdbook-mermaid install .
mdbook build
mdbook serve   # optional: live reload at http://localhost:3000
```

Built HTML is written to `docs/dev/book/` (gitignored).

## Contributor checklist

When adding or renaming a workspace crate:

1. Add a page under [Crates](./crates/README.md) and update `SUMMARY.md`
2. Update [Crate layering](./architecture/layering.md) if dependencies change
3. Update [Types vs interfaces](./architecture/types-vs-interfaces.md) classification
4. Update relevant [Interfaces](./interfaces/README.md) pages
5. Run `npx markdownlint-cli2 "**/*.md"` from the repo root
6. Run `mdbook build` in `docs/dev/`

## Workspace layout

All library crates live under `crates/` and are listed in the root
`Cargo.toml` `members` array. There is **no application binary crate yet**
(the CLI is planned in SRD §4); a future chapter will cover the CLI crate
when it lands.
