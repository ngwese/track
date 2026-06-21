# Introduction

**Track** is a personal issue tracker for many kinds of work—software,
animation, home renovation, travel, hardware, and more—from one CLI-first
system. Each project declares its own **schema** (issue types, states,
workflows, labels) in version-controlled YAML. You edit locally, validate
offline, and push changes to a **workspace sync hub** when you are ready to
share state with other clients, agents, or CI.

## What you get

- **Issue tracking as code** — structure lives in Git beside (or inside) your
  project, not in a web UI.
- **Per-project customization** — a kitchen remodel and a Rust service can use
  different types, states, and custom fields without separate tools.
- **Local-first CLI** — create and validate projects without network access;
  sync when you choose.
- **Shared core model** — issues, typed relations, optional efforts and
  components, and stable identifiers work the same everywhere.

## Who this guide is for

This book explains concepts and schema authoring for **people using Track** to
organize work. It complements:

| Document | Purpose |
| --- | --- |
| [PRD](../../PRD.md) | Product vision and goals |
| [SRD](../../SRD.md) | Full technical specification |
| [Developer guide](../dev/) | Rust crates and hub implementation |

## Build this book locally

```bash
cargo install mdbook --locked --version 0.5.3
cd docs/user
mdbook build
mdbook serve   # optional: http://localhost:3000
```

Built HTML is written to `docs/user/book/` (gitignored).

## Status

Track is under active development. CLI commands and on-disk formats may change.
This guide tracks the intended user experience; where behavior is not yet
implemented, pages note the gap.
