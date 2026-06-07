# Agent instructions

This repository is **Track** — a CLI-first, local-first issue tracker with a sync hub. Read [docs/PRD.md](docs/PRD.md) for product intent and [docs/SRD.md](docs/SRD.md) for technical design before making structural changes.

## Commits

Use **[Conventional Commits](https://www.conventionalcommits.org/)** for every commit.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- **Description:** imperative mood, lowercase, no trailing period (e.g. `add claim API`, not `Added claim API.`).
- **Scope:** optional noun in parentheses after the type (e.g. `feat(hub):`, `docs(srd):`).
- **Breaking changes:** add `!` after type/scope (`feat(cli)!:`) or a `BREAKING CHANGE:` footer.

### Types

| Type | Use for |
|------|---------|
| `feat` | New user-facing capability |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Tests only |
| `chore` | Build, CI, tooling, deps — no production code change |
| `perf` | Performance improvement |
| `style` | Formatting, whitespace — no logic change |

Prefer `feat` / `fix` / `docs` / `chore` for most work in this repo.

### Scopes (suggested)

`cli`, `hub`, `infra`, `docs`, `srd`, `prd`, `schema` — use when it clarifies the diff; omit when the change spans multiple areas.

### Examples

```
docs(srd): clarify operational telemetry vs comments

feat(hub): append progress entries to operational log

fix(cli): require type prefix when matching partial eids

chore(infra): bump hub compose postgres image

feat(cli)!: rename uuid field to eid across push format

BREAKING CHANGE: materialized issue YAML uses `eid` instead of `uuid`.
```

### Rules

1. One logical change per commit when possible.
2. Do not commit secrets (`.env`, tokens, credentials).
3. Only commit when asked; do not push unless asked.
4. Match existing code style and keep diffs focused.
