# cargo-crap integration plan

> **Status:** In progress (Phases 0–1 complete, 2026-06-19)\
> **Tool:** [cargo-crap](https://github.com/minikin/cargo-crap) — CRAP (Change Risk
> Anti-Patterns) metric for Rust\
> **Background:** [Finding Untested Complexity in AI-Generated Rust Code](https://minikin.me/blog/cargo-crap)

This plan adds change-risk measurement to Track: complex, under-tested
functions are surfaced before they become normal. Policy lives in `.cargo-crap.toml`
at the repo root; CI installs pinned tools and runs a fixed command sequence.

## Executive summary

| Layer | Role |
| --- | --- |
| **`.cargo-crap.toml`** | Threshold, exclusions, gate mode, epsilon |
| **`crap_baseline.json`** | Committed snapshot for regression detection |
| **`change-risk` CI job** | Coverage + CRAP report on PRs (Phase 1: report-only) |
| **`AGENTS.md` gate** | Agent completion checklist (Phase 3) |

```text
CRAP(m) = comp(m)² × (1 − cov(m)/100)³ + comp(m)
```

## Goals

1. **Agent completion gate** — extend `AGENTS.md` Rust checklist with coverage
   + CRAP.
2. **PR feedback** — sticky PR comment with regressions and new risky functions.
3. **PR enforcement** — CI fails when scores regress vs `crap_baseline.json`.
4. **Config-driven policy** — no thresholds or exclusions hard-coded in CI YAML.

## Non-goals (initial rollout)

+ Absolute “every function ≤ 30” gate on day one
+ SARIF / Code Scanning upload
+ README badge automation
+ Gating Markdown or TLA+ artifacts

## Gating strategy

| Mode | When | Track fit |
| --- | --- | --- |
| `--fail-regression` + baseline | Primary gate (Phase 2+) | Mature multi-crate workspace |
| `--fail-above` | Optional tighten (Phase 4) | After worst legacy debt addressed |
| Report-only | Bootstrap (Phase 1) | Calibrate CI; no merge blocks |

## Artifacts

### `.cargo-crap.toml`

Repo-root policy file. `cargo crap` discovers it by walking up from CWD; CLI flags
override file values.

Supported keys in cargo-crap 0.2.2: `threshold`, `fail-above`, `fail-regression`,
`missing`, `exclude`, `allow`, `top`, `min`, `epsilon`, `jobs`.

Note: `sort` and `default-excludes` require cargo-crap 0.3.x — pass `--sort file`
on the CLI when upgrading for stabler baseline diffs.

### `crap_baseline.json`

Committed baseline. Regenerate only when scores improve repo-wide:

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info
cargo crap --workspace --lcov lcov.info --format json --output crap_baseline.json
```

### Scope: production crates

Test harness crates are hidden via `allow` path globs (cargo-crap 0.2.2 `--exclude`
is unreliable in `--workspace` mode):

+ `crates/track-sync-testing/**`
+ `crates/track-hub-conformance-testing/**`

`track-hub-memory` stays in scope (ships as a library).

## Command contract

Shared by local dev, agents, and CI:

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info
cargo crap --workspace --lcov lcov.info --baseline crap_baseline.json
```

With `fail-regression = true` in `.cargo-crap.toml`, the second command exits
non-zero on regressions.

### Remediation

1. Regressions on touched functions → add branch-covering tests or simplify logic.
2. New functions above threshold → same; prefer decomposition over `allow`.
3. Baseline refresh → regenerate JSON, commit with explanation in PR body.

### Known exceptions

| Function | CRAP | Reason | Return-to |
| --- | --- | --- | --- |
| `ItemReducer::reduce` | ~35 | 11-arm `item.*` dispatch match; ~97% coverage | Hierarchical match (scalar / OR-set / lifecycle) after `item.*` events stabilize; remove `allow` entry |

## Phase 0 audit (2026-06-19)

Workspace coverage run (`cargo llvm-cov --workspace`) on Rust 1.96.0.

| Metric | Full workspace | Production scope (`allow`) |
| --- | --- | --- |
| Functions analyzed | 745 | 505 |
| Above threshold 30 | 39 | 29 |
| Worst offender | `TrackSqliteStore::list_relations_for_entity` (342.0) | same |

Per-crate crappy counts (production scope):

| Crate | Functions | Crappy |
| --- | --- | --- |
| track-store-sqlite | 66 | 19 |
| track-reduce | 97 | 3 |
| track-entity | 29 | 3 |
| track-materialize-yaml | 20 | 3 |
| track-store | 63 | 1 |
| track-hub, track-sync, track-replication, … | — | 0 |

Observations:

+ **`track-store-sqlite`** dominates risk: many store methods show 0% line coverage
  because integration tests exercise in-memory hubs, not the SQLite adapter directly.
  Regression gating still catches new complexity; absolute scores will look high
  until dedicated SQLite store tests land.
+ **`typed_json_to_field`** (`track-reduce`) is the highest partially-covered
  offender (CC 21, ~40% coverage, CRAP 118).
+ Five integration test files under `crates/*/tests/` had no LCOV match (warning
  only).

## CI integration

### `change-risk` job

Parallel job in `.github/workflows/ci.yml` (llvm-cov rebuilds the workspace).

+ Pin `cargo-llvm-cov@0.8.7` and `cargo-crap@0.2.2`.
+ Policy from `.cargo-crap.toml` only.
+ Phase 1: generate report + PR comment; **no** `fail-regression` in config.
+ Phase 2: uncomment `fail-regression = true` in `.cargo-crap.toml`.

Path filter: skip when only docs/spec/infra change (see workflow).

### PR comment

`--format pr-comment` with `--baseline crap_baseline.json` on pull requests.

## Rollout phases

| Phase | Status | Deliverables | CI | Agents |
| --- | --- | --- | --- | --- |
| **0 — Audit** | Done | Findings above; `.cargo-crap.toml` tuned | — | — |
| **1 — Baseline** | Done | `crap_baseline.json`, `.gitignore`, report-only CI | Comment only | — |
| **2 — Enforce** | Pending | `fail-regression = true` | Fails on regression | — |
| **3 — Complete** | Pending | `AGENTS.md` step 5 | Unchanged | Required |
| **4 — Tighten** | Optional | `fail-above = true` | Absolute threshold | Same |

## Tool versions

| Tool | Version | Notes |
| --- | --- | --- |
| cargo-crap | 0.2.2 | Pinned in CI; 0.3.x adds `sort` in config |
| cargo-llvm-cov | 0.8.7 | Pinned in CI |
| Rust | 1.96.0 | Matches `rust-toolchain.toml` and `rust` CI job |

## Success criteria

+ [x] `.cargo-crap.toml` at repo root; policy not duplicated in CI YAML
+ [x] `crap_baseline.json` committed
+ [x] `change-risk` CI job on PRs (report-only)
+ [ ] PRs fail on CRAP regression (Phase 2)
+ [ ] `AGENTS.md` lists CRAP as Rust step 5 (Phase 3)

## References

+ [cargo-crap README](https://github.com/minikin/cargo-crap)
+ [cargo-crap config module](https://docs.rs/cargo-crap/latest/cargo_crap/config/index.html)
+ [Savoia & Evans — The CRAP Metric (2007)](https://www.artima.com/weblogs/viewpost.jsp?thread=210575)
