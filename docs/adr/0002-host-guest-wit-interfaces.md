# ADR 0002: Host–guest WIT interfaces and on-disk storage scopes

> **Status:** Proposed — defines WIT contracts between `track-host` and `track-cli`, and maps them to user- and project-scoped config, state, and cache locations.

**Date:** 2026-06-07  
**Deciders:** Track maintainers (draft for review)

## Context

[ADR 0001](0001-implementation-runtime.md) splits the Track CLI into a native **host** (`track-host`) and a WebAssembly **guest** (`track-cli`). The host owns OS access; the guest owns command logic, schema validation, push/pull planning, and hub client behavior.

The [SRD](../SRD.md) already defines what Track persists:

- **Declarative project config** — `track.yaml`, `schema/`, and lazily materialized `work/` (§3.2–§3.3)
- **Project sync state** — `.track/state.json` and `.track/state.lock` (§3.7)
- **User secrets** — `~/.config/track/config.json` for workspace URLs and tokens (§5.2, never in Git)
- **Local client cache** — embedded DB (SQLite) for the work index plus optional YAML (§5.1, §5.9)

ADR 0001 left WIT interface design as an open question. Without explicit contracts, host and guest implementations cannot evolve independently, capability restrictions are ambiguous, and agent sandboxes cannot be audited.

This ADR defines:

1. **Six on-disk storage buckets** — user config/state/cache and project config/state/cache
2. **Host → guest interfaces** — invocation control, policy, credentials, and bootstrap configuration
3. **Guest → disk interfaces** — scoped filesystem access and structured state APIs

Canonical WIT source files live in [`wit/track/`](../../wit/track/).

## Decision drivers

1. **Align with SRD layout** — buckets map directly to paths and files already specified in the SRD.
2. **Capability separation** — guest receives only the storage scopes and network policy required for the current command.
3. **Secrets handling** — tokens stay in user config; host mediates reads/writes rather than preopening raw credential files to all invocations.
4. **Offline-first** — user-state APIs support hub mutation queues (PRD §4.3, SRD §6.3).
5. **Interface versioning** — each WIT package is versioned (`@0.1.0`); hosts implement a range of guest import versions.

## On-disk storage model

The host resolves six **buckets**. Paths below are illustrative; the host maps buckets to OS conventions (XDG on Linux, `Library/Application Support` on macOS, `%APPDATA%` / `%LOCALAPPDATA%` on Windows).

### User scope

| Bucket | Typical path | Contents | Git |
|--------|--------------|----------|-----|
| **user-config** | `~/.config/track/` | `config.json` — workspace registry, hub URLs, tokens, default actors | No |
| **user-state** | `~/.local/state/track/` | `cursors.json` (global hub poll cursors), `offline-queue/` (per-workspace mutation queues), `host.json` (diagnostics) | No |
| **user-cache** | `~/.cache/track/` | `components/` (`track-cli` WASM artifacts), `templates/` (fetched init templates) | No |

### Project scope

Project buckets are rooted at the **project root** — the directory that directly contains `track.yaml` (SRD §3.2.1). This is not necessarily the repository root; embedded layouts use `<repo-root>/track/` as the project root.

| Bucket | Resolved path | Contents | Git |
|--------|---------------|----------|-----|
| **project-config** | `<project-root>/` | `track.yaml`, `schema/`, `work/` — issue tracking as code | Yes (subject to `.gitignore`) |
| **project-state** | `<project-root>/.track/` | `state.json` (hashes, materialization registry, event cursor — SRD §3.7), `state.lock` | Usually no |
| **project-cache** | `<project-root>/.track/cache/` | `index.sqlite` (work index), `validate/` (schema validation cache) | No |

**Layout patterns** (SRD §3.2.2):

```
# Standalone — project root is the top-level directory
kitchen/                         # project root
  track.yaml
  schema/ …

# Embedded in a version-controlled repo — customary track/ subdirectory
api-server/                      # repository root (not the project root)
  src/ …
  track/                         # project root
    track.yaml
    schema/ …
    work/ …
    .track/
```

The embedded `track/` layout reduces repo-root naming conflicts and allows `<repo-root>/track` to be a git submodule for independent issue-tracking revision history.

```
~/.config/track/                 USER CONFIG
  config.json
~/.local/state/track/            USER STATE
  cursors.json
  offline-queue/<workspace>/
~/.cache/track/                  USER CACHE
  components/<version>/
  templates/

<project-root>/                  # directory containing track.yaml
  track.yaml                     ┐
  schema/                        │ PROJECT CONFIG (declarative)
  work/                          ┘
  .track/
    state.json                   ─ PROJECT STATE
    state.lock
    cache/
      index.sqlite               ─ PROJECT CACHE
      validate/
```

### Project root discovery

The host resolves the project root **before** opening project buckets or reading `tool.version`:

1. **`--project PATH`** — `PATH` is the project root (must contain `track.yaml`, except `track init`).
2. **Upward walk** — from the process working directory, search ancestors for `track.yaml`; the containing directory is the project root.
3. **Failure** — if no manifest is found and the command requires a project, the host exits without loading a guest.

Examples: cwd `api-server/src/` discovers `api-server/track/track.yaml` → project root `api-server/track/`. cwd `kitchen/work/issues/I-…/` discovers `kitchen/track.yaml` → project root `kitchen/`.

`track:session` exposes `project-root` and `manifest-path` (`<project-root>/track.yaml`) to the guest. The guest must not re-walk the filesystem to find the project root.

**Rules:**

- Project buckets are unavailable when no project root is discovered (global commands like `track auth login` use user buckets only).
- `track init` creates the project tree under the target directory (default `./track/` for embedded repos per SRD §3.2.2); host ensures `.track/` exists before guest starts.
- User-cache component artifacts are resolved during host bootstrap (ADR 0001); guest may call `registry.resolve` when pinning or upgrading tool versions.

## Interface architecture

```
┌──────────────── track-host ────────────────────────────────────────┐
│  Parse argv → session                                              │
│  Policy      → capabilities                                        │
│  OS paths    → locations (preopened descriptors per bucket)      │
│  Secrets     → user-config, auth                                   │
│  Bootstrap   → registry (component cache)                          │
│  Concurrency → project-lock, project-state                         │
│  Offline     → offline-queue                                       │
│  WASI p2     → filesystem, cli, clocks, sockets (if allowed)       │
└────────────────────────────┬───────────────────────────────────────┘
                             │ WIT imports
                             ▼
┌──────────────── track-cli (guest) ─────────────────────────────────┐
│  Command routing, schema, push/pull, hub HTTP client               │
│                                                                  │
│  Declarative files  → WASI FS via locations.project-config       │
│  Index DB           → WASI FS via locations.project-cache         │
│  state.json         → project-state (+ project-lock)             │
│  Offline hub ops    → offline-queue (user-state)                 │
│  Credentials        → auth.resolve (not direct config.json read) │
└──────────────────────────────────────────────────────────────────┘
```

### Interface categories

| Category | WIT package | Direction | Purpose |
|----------|-------------|-----------|---------|
| Control | `track:session` | Host → guest | argv, cwd, resolved tool version, parsed flags |
| Control | `track:capabilities` | Host → guest | network/stdio policy, hub allowlist |
| Control | `track:components` | Host ↔ guest | resolve `track-cli` artifacts in user-cache |
| Configuration | `track:config` | Guest → host | read/write `config.json` (validated by host) |
| Configuration | `track:auth` | Guest → host | resolve workspace tokens for hub API calls |
| Storage | `track:locations` | Host → guest | preopened directory descriptors per bucket |
| Storage | `track:state` | Guest → host | read/write `.track/state.json` |
| Storage | `track:lock` | Guest → host | advisory `.track/state.lock` |
| Storage | `track:hub` | Guest → host | durable hub mutation queue in user-state |
| Storage | `wasi:filesystem` | Guest → disk | YAML, SQLite, cache files via bucket descriptors |
| I/O | `wasi:cli`, `wasi:clocks`, `wasi:sockets` | Guest ↔ OS | stdio, time, hub HTTP (when network allowed) |

## Host → guest interfaces (control and configuration)

These interfaces pass **invocation context and policy** into the guest. They are read-only for the guest except where noted.

### `track:session/session`

Provides immutable invocation metadata parsed by the host before component load:

- `argv`, `cwd`, discovered `project-root`, `manifest-path` (`<project-root>/track.yaml`)
- Resolved `tool-version` / `tool-digest` and `host-version`
- Parsed global flags (`--json`, `--dry-run`, `--force`, `--verbose`, `--debug`)
- Overrides: `--project`, `TRACK_TOOL_VERSION`

The guest does not re-parse raw environment variables for track-specific semantics; it consumes `session.get()`.

### `track:capabilities/capabilities`

Declares what the host linked for this run:

| Flag | Effect |
|------|--------|
| `network` | When false, `wasi:sockets` imports are absent or stubbed |
| `hub-allowlist` | When network is true, restrict outbound connections to registered hub URLs |
| `stdin` / `stdout` / `stderr` | Agent/CI runs may disable stdin; JSON mode still uses stdout |

Example: `track schema validate` inside an air-gapped project may run with `network: false` and only project buckets available.

### `track:components/registry`

Host ensures `track-cli` artifacts exist in **user-cache** before guest execution (ADR 0001). The guest calls `resolve` when:

- `track init` pins a `tool.version` and records the resolved digest
- `track upgrade` checks for a newer compatible component

Returns `{ version, digest, cache-path }` where `cache-path` is relative to the user-cache bucket root.

## Guest → disk interfaces (state and declarative files)

### `track:locations/locations` (primary filesystem scoping)

The host preopens one directory descriptor per available bucket. The guest uses standard `wasi:filesystem` operations on `path-info.root` — it never receives the full host filesystem.

| Bucket | Guest access pattern |
|--------|---------------------|
| `user-config` | Rarely needed when `user-config` and `auth` WIT are used; available for `track auth` diagnostics |
| `user-state` | Direct file I/O for `cursors.json`; prefer `offline-queue` for mutation queues |
| `user-cache` | Read component artifacts; template cache during `track init` |
| `project-config` | Read/write `track.yaml`, `schema/*.yaml`, `work/**` (materialized entities) |
| `project-state` | Lock file presence; optional direct read — prefer `project-state` WIT for `state.json` |
| `project-cache` | Read/write `index.sqlite`, validation cache files |

`list-available` returns which buckets are linked for the current invocation (e.g. no project buckets for `track auth login`).

### `track:state/project-state`

Structured JSON access to `.track/state.json` (SRD §3.7):

- `read()` — parse-free JSON text
- `write(json)` — atomic replace

Concurrent push/pull must acquire `track:lock/project-lock` before read-modify-write sequences. The host may add atomic `merge` in `@0.2.0` if cross-language guests need host-assisted JSON patching.

### `track:lock/project-lock`

Advisory lock on `.track/state.lock`:

- `acquire(blocking)` → `lock` resource
- `release()` on the resource (also dropped on component exit)

Prevents interleaved `state.json` corruption when a human CLI and background `hub subscribe` flush overlap.

### `track:hub/offline-queue`

Persists hub mutations under **user-state** when the network is unavailable (SRD §6.3):

| Field | Purpose |
|-------|---------|
| `workspace-slug` | Routes flush to correct hub credentials |
| `project-eid` | Optional scope for project-specific mutations |
| `method`, `path`, `body` | HTTP-shaped hub API request |
| `idempotency-key` | SRD §5.4 idempotent writes |

Operations: `enqueue`, `list`, `drain`, `ack`, `status`.

### `wasi:filesystem` (declarative config and project cache)

The bulk of on-disk manipulation uses WASI filesystem APIs on bucket descriptors:

| Data | Bucket | Examples |
|------|--------|----------|
| Project manifest + schema | `project-config` | `track.yaml`, `schema/types.yaml` |
| Materialized work | `project-config` | `work/issues/<eid>/issue.yaml` |
| Work index | `project-cache` | `index.sqlite` (guest-embedded SQLite) |
| Validation cache | `project-cache` | Invalidated on `schema.updated` hub events |

This keeps YAML and SQLite logic in the guest while the host enforces directory boundaries.

## Configuration and credential interfaces

### `track:config/user-config`

Mediated access to **user-config** `config.json`:

- `read()` / `write(json)` — full document
- `upsert-workspace` / `remove-workspace` — `track auth login` / logout paths

The host validates JSON shape before persist and restricts file permissions (0600). The guest should prefer this over raw filesystem reads of `config.json` so schema validation stays centralized.

### `track:auth/auth`

Short-lived credentials for hub API calls:

- `resolve(slug)` → `{ hub-url, token, default-actor }` for the workspace named in `track.yaml`
- `list()` → workspace summaries without tokens

Tokens are never written to project repos. For `track auth login`, the guest writes via `user-config.upsert-workspace`; for normal commands, `auth.resolve` supplies ephemeral tokens for the invocation.

## Command → bucket and interface matrix

Illustrative mapping for v0.1 commands (SRD §4). The host may narrow capabilities further per subcommand.

| Command | Project buckets | User buckets | Key WIT imports |
|---------|-----------------|--------------|-----------------|
| `track auth login` | — | config, state | `user-config`, `session` |
| `track init` | config, state, cache | cache, config | `locations`, `registry`, `project-state`, `session` |
| `track schema validate` | config | — | `locations` (project-config only) |
| `track push` / `pull` | all project | config, state, cache | `locations`, `auth`, `project-state`, `project-lock`, `offline-queue`, `capabilities` (network) |
| `track issue list` | cache | config | `locations`, `auth`, `capabilities` |
| `track issue materialize` | config, cache | config | `locations`, `auth`, `project-state`, `capabilities` |
| `track hub subscribe` | state, cache | config, state | `auth`, `offline-queue`, `capabilities` |

## WIT packaging and worlds

WIT import paths take the form `package-name/interface-name`. The **package** is a short namespace; the **interface** names the specific capability. When both matched (`track:session/session`) it was an artifact of one-interface-per-file naming, not a requirement. Prefer the shorter pattern used elsewhere — e.g. `track:lock/project-lock`, `track:components/registry` — where the package groups related interfaces and the interface name stays specific.

Source files: [`wit/track/`](../../wit/track/)

| File | Package | Interface | Import path |
|------|---------|-----------|-------------|
| `session.wit` | `track:session@0.1.0` | `session` | `track:session/session` |
| `capabilities.wit` | `track:capabilities@0.1.0` | `capabilities` | `track:capabilities/capabilities` |
| `locations.wit` | `track:locations@0.1.0` | `locations` | `track:locations/locations` |
| `auth.wit` | `track:auth@0.1.0` | `auth` | `track:auth/auth` |
| `config.wit` | `track:config@0.1.0` | `user-config` | `track:config/user-config` |
| `lock.wit` | `track:lock@0.1.0` | `project-lock` | `track:lock/project-lock` |
| `state.wit` | `track:state@0.1.0` | `project-state` | `track:state/project-state` |
| `hub.wit` | `track:hub@0.1.0` | `offline-queue` | `track:hub/offline-queue` — package reserved for hub-coupled interfaces (queue now; HTTP client, event subscribe, etc. later) |
| `components.wit` | `track:components@0.1.0` | `registry` | `track:components/registry` |
| `world.wit` | `track:world@0.1.0` | `cli-guest`, `host` | — |

The `cli-guest` world imports all `track:*` interfaces plus WASI Preview 2 (`wasi:cli/run` export). The `host` world exports the `track:*` implementations. WASI dependency versions pin to `@0.2.0` to match Wasmtime WASIp2 examples (ADR 0001).

### Versioning policy

- Bump the `@0.x.0` package version on breaking WIT changes.
- Hosts must implement all `track:*@0.1.0` exports before loading guests that import `@0.1.0`.
- Guests declare required import versions in their component manifest; host refuses incompatible combinations with a clear error.

## Consequences

### Positive

- **Auditable sandbox** — agent deployments can inspect exactly which buckets and network hosts a command receives.
- **SRD alignment** — buckets correspond to documented paths; no parallel config story.
- **Independent evolution** — host adds `offline-queue` flush strategies or `auth` token refresh without recompiling guest logic (within the same major WIT version).
- **Testability** — mock host implementations can implement the `host` world for guest unit tests without a real filesystem.

### Negative / trade-offs

- **Two-layer storage APIs** — guests must know when to use WIT (`project-state`, `offline-queue`) vs raw WASI FS (YAML, SQLite).
- **Host implementation surface** — nine `track:*` interfaces plus WASI linking is substantial for v0.1.
- **Token exposure** — `auth.resolve` passes tokens into guest memory; mitigated by short-lived invocations and future host-mediated HTTP WIT (ADR 0001 open question).

### Follow-up work

- Normative bucket paths and `.track/cache/` layout — documented in SRD §3.2.1–§3.2.3.
- Add `tool.version` block to SRD §3.3 `track.yaml` (per ADR 0001).
- Decide whether hub HTTP stays in guest (`wasi:sockets`) or moves to `track:http@0.2.0` host export.
- Add WIT conformance tests: mock host + guest component in CI.

## Compliance

- CI validates WIT syntax (`wit-deps` / `wasm-tools component wit`).
- Integration tests cover bucket listing per command class (with/without project root).
- Capability tests verify guest cannot open paths outside `locations` descriptors.
- Lock tests verify concurrent `project-state` writes serialize via `project-lock`.

## References

- [ADR 0001: Implementation runtime](0001-implementation-runtime.md)
- [Track SRD §3.2–§3.7](../SRD.md) — project root discovery, layout patterns, `track.yaml`, `state.json`
- [Track SRD §5.1–§5.9](../SRD.md) — local client, workspace association, storage
- [Track PRD §4.3](../PRD.md) — local-first, offline mutation queue
- [WIT specification](https://component-model.bytecodealliance.org/design/wit.html)
- [`wit/track/`](../../wit/track/) — canonical interface definitions

## Related decisions

- [ADR 0001](0001-implementation-runtime.md) — WASIp2 runtime split (superseded open question #2 on WIT package layout)
