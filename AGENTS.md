# Agent instructions

This repository is **Track** — a CLI-first, local-first issue tracker with a
sync hub. Read [docs/PRD.md](docs/PRD.md) for product intent and
[docs/SRD.md](docs/SRD.md) for technical design before making structural
changes.

## Markdown

Before any change to a Markdown document (`.md`) can be considered complete, the
file must pass all of the following checks:

1. **Linting** — no markdownlint errors. Run from the repo root:

   ```bash
   npx markdownlint-cli2 "**/*.md"
   ```

   Rules are defined in `.markdownlint-cli2.jsonc` at the repository root.

2. **Trailing whitespace** — no trailing spaces or tabs at the end of a line.

Fix violations in the same change set; do not leave follow-up cleanup for later.

## Rust / Cargo

This repository is a
[Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html).
Individual crates live under `crates/` and are listed in the root
`Cargo.toml` `members` array.

Before any change to files under `crates/` can be considered complete, the
file must pass all of the following checks. Run from the repo root:

1. **Build** — the workspace compiles:

   ```bash
   cargo build --workspace
   ```

2. **Format** — code is formatted with rustfmt (no diff after formatting):

   ```bash
   cargo fmt --all
   ```

3. **Lint** — no Clippy warnings or errors:

   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```

4. **Tests** — all defined tests pass:

   ```bash
   cargo test --workspace
   ```

5. **Change risk** — no CRAP regression vs the committed baseline (policy in
   [`.cargo-crap.toml`](.cargo-crap.toml); see
   [docs/plans/cargo-crap-integration-plan.md](docs/plans/cargo-crap-integration-plan.md)):

   ```bash
   cargo llvm-cov --workspace --lcov --output-path lcov.info
   cargo crap --workspace --lcov lcov.info --baseline crap_baseline.json
   ```

   If scores improve repo-wide, refresh the baseline:

   ```bash
   cargo crap --workspace --lcov lcov.info --format json --output crap_baseline.json
   ```

Fix violations in the same change set; do not leave follow-up cleanup for later.

## Commits

Use **[Conventional Commits](https://www.conventionalcommits.org/)** for every commit.

### Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- **Description:** imperative mood, lowercase, no trailing period (e.g.
  `add claim API`, not `Added claim API.`).
- **Scope:** optional noun in parentheses after the type (e.g. `feat(hub):`,
  `docs(srd):`).
- **Breaking changes:** add `!` after type/scope (`feat(cli)!:`) or a
  `BREAKING CHANGE:` footer.

### Types

| Type | Use for |
| --- | --- |
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

`cli`, `hub`, `infra`, `docs`, `srd`, `prd`, `schema` — use when it clarifies
the diff; omit when the change spans multiple areas.

### Examples

```text
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
