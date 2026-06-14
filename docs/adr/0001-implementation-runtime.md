# ADR 0001: Implementation runtime (WASIp2 + WebAssembly components)

> **Status:** Deferred — revisit when per-project or per-type configurable CLI logic is a clear requirement.

**Date:** 2026-06-07  
**Deciders:** Track maintainers (draft for review)

## Deferral

A proof of concept on branch `feat/adr-0001-implementation-plan` demonstrated that the WASIp2 host/guest split is technically viable, but the costs outweigh the benefits for the current MVP:

- **Startup overhead** — pushing most or all CLI functionality into a WebAssembly guest added significant invocation latency compared with a native binary.
- **Split argv surface** — some flags were handled by the native host while others were handled by the guest, which duplicated parsing logic and produced a confusing CLI structure.
- **Bootstrap complexity** — the host had to parse project metadata (including tool version overrides) before loading the guest, coupling version selection to project layout earlier than necessary.

**Conclusion:** Focus on core Track functionality in a native CLI first. Revisit WebAssembly when there is a clearer need to support configurable logic per project or project type.

## Context

Track is a CLI-first, local-first issue tracker with a sync hub. Participants include humans, agents, scripts, and CI jobs operating across heterogeneous machines — developer laptops, CI runners, VM images, and container images built for agent sandboxes.

The [PRD](../PRD.md) and [SRD](../SRD.md) establish product goals (issue tracking as code, lazy materialization, agent ergonomics) but do not yet specify how the **CLI client** is built, packaged, or executed. That choice has outsized impact:

- **Distribution surface.** A traditional native CLI must be built, published, and kept current for every OS and CPU architecture where Track might run. Agent images and CI environments multiply that burden.
- **Evolution cadence.** Track will introduce breaking changes at the hub API, workspace config, and on-disk project schema levels over time. Forcing every project and environment onto the latest CLI version creates friction and blocks gradual migration.
- **Host capability boundary.** When tool logic is fetched or selected at runtime, it is desirable to constrain what that logic can do on the host (filesystem scope, network access, config paths) and to abstract OS differences (e.g. XDG config dirs vs macOS Application Support).

This ADR records the decision for the **CLI client implementation runtime**. It does **not** decide the sync hub server stack; the hub remains a separately deployed, long-lived service (see [infra/README.md](../../infra/README.md)).

## Decision drivers

1. **Minimize per-environment maintenance** — one portable artifact for Track logic; thin, infrequently updated native launchers per OS/arch.
2. **Per-project tool versioning** — a project declares which Track implementation version it expects; bootstrap resolves and runs that version without upgrading the whole machine.
3. **Capability separation** — a small native **host** exposes a limited, auditable set of host operations; the bulk of Track logic runs as a **guest** with no direct OS access beyond what the host grants via interfaces.

Secondary drivers aligned with the SRD:

- Async I/O for hub sync, event subscribe, and concurrent materialization (SRD §8 performance targets).
- Portability across macOS, Linux, and Windows best-effort (SRD §8).
- Agent/CI suitability: deterministic JSON output, stable exit codes, headless operation (PRD G5).

## Considered options

### Option A — Native CLI binary (Rust or Go), one build per platform

Ship a single statically linked or mostly-static binary per `(os, arch)` via package managers, GitHub releases, and container base images.

**Pros:** Lowest runtime overhead; simplest debugging; familiar distribution story.  
**Cons:** Every environment must install and upgrade the full CLI; breaking tool changes are global; no built-in sandbox between “tool update” and “what the tool can touch on disk/network”; N×M build matrix grows with platforms and agent image variants.

### Option B — Interpreted runtime (Node, Python)

Distribute Track as source or package; require a language runtime on the host.

**Pros:** Fast iteration; some portability within a runtime ecosystem.  
**Cons:** Runtime version drift across environments; heavy base images for agents; weak sandbox story; dependency and packaging friction for end users.

### Option C — WASI Preview 1 module (non-component)

Compile Track to `wasm32-wasi` as a single core module; host with Wasmtime or similar.

**Pros:** One WASM artifact; some sandboxing via WASI imports.  
**Cons:** Preview 1 lacks the Component Model; custom host APIs are ad hoc; weaker interface versioning story; async host/guest integration is less mature than Preview 2.

### Option D — WASIp2 target + WebAssembly Component Model (chosen)

Split implementation into:

1. **`track-host`** — minimal native launcher per `(os, arch)` embedding a WASI Preview 2–capable runtime (initial reference: [Wasmtime](https://docs.wasmtime.dev/examples-wasip2.html) with `wasmtime-wasi` p2 bindings).
2. **`track-cli`** — Track logic compiled to `wasm32-wasip2` as a WebAssembly **component**, loaded at runtime according to project metadata.

Host and guest communicate through:

- **Standard WASI Preview 2 imports** (`wasi:filesystem`, `wasi:cli`, `wasi:clocks`, `wasi:sockets`, …) for portable OS abstraction, configured narrowly by the host.
- **Custom component interfaces** (WIT) for Track-specific capabilities — e.g. resolving `~/.config/track/`, project root discovery, hub credential access, structured logging — versioned independently of the guest implementation.

**Pros:** Single `wasm32-wasip2` artifact for Track logic; project-level version pinning; explicit capability surface; async Rust on host and guest; component interfaces are modular and evolvable.  
**Cons:** Newer toolchain and ecosystem; startup and FFI overhead vs native; debugging across host/guest boundary; requires defining and maintaining WIT contracts.

## Decision

Adopt **Option D**: implement the Track CLI as a **WASIp2 WebAssembly component** (`track-cli`) executed by a **thin native host** (`track-host`).

### Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  track-host  (native; built per os+arch, distributed rarely)     │
│                                                                  │
│  • Parse argv; discover project root via track.yaml (SRD §3.2.1) │
│  • Resolve track-cli version (project pin → cache → fetch)       │
│  • Wasmtime (or equivalent) + WASI Preview 2 linker              │
│  • Configure WASI imports (narrow FS, net, stdio, env)           │
│  • Implement track:* WIT imports (config, credentials, …)        │
└────────────────────────────┬─────────────────────────────────────┘
                             │ component imports (WIT + WASI p2)
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│  track-cli  (wasm32-wasip2 component; versioned per project)     │
│                                                                  │
│  • Command routing, schema validate, push/pull planning          │
│  • Local store, YAML materialization, hub HTTP client            │
│  • JSON output, exit codes, agent-oriented behavior              │
└──────────────────────────────────────────────────────────────────┘
```

### Host bootstrap

`track-host` performs all pre-guest work. The guest component is not loaded until bootstrap completes (or fails with a host-level error).

**Phase 1 — Parse invocation**

1. User or agent runs `track …` (the host binary on `PATH`).
2. Host parses `argv`, global flags (`--json`, `--dry-run`, `--project`, …), and determines whether the subcommand requires a project.

**Phase 2 — Discover project root** (when required)

Per [SRD §3.2.1](../SRD.md):

1. If `--project PATH` is set, `PATH` is the project root.
2. Otherwise, walk from the process working directory toward the filesystem root until a `track.yaml` file is found; the **directory containing that file** is the project root (e.g. `kitchen/track.yaml` → root is `kitchen/`; `api-server/track/track.yaml` → root is `api-server/track/`, not `api-server/`).
3. If discovery fails and the command requires a project, the host exits before loading a guest.

The host records `project-root` and `manifest-path` (`<project-root>/track.yaml`) in `track:session` when a project is in scope.

**Phase 3 — Resolve guest component**

1. If a project is in scope, read `tool.version` from `track.yaml` (see below). Commands without a project (e.g. `track auth login`) use the host's default or latest compatible `track-cli` version.
2. Ensure the matching `track-cli` component is present in user-cache (content-addressed or semver-keyed); fetch from a release registry or mirror if missing and network policy allows.
3. Map [storage buckets](0002-host-guest-wit-interfaces.md) to absolute paths: user buckets from OS conventions; project buckets from the discovered project root.

**Phase 4 — Instantiate guest**

1. Configure WASI Preview 2 imports (scoped preopens, optional sockets per command policy).
2. Link `track:*` WIT exports (session, locations, auth, …).
3. Instantiate the resolved component and delegate execution (e.g. `wasi:cli/run`).
4. Propagate guest exit code to the host process.

### Per-project version pinning

The SRD already calls for schema format versioning in `track.yaml` (§8). Extend the project manifest with an explicit **tool** pin so each project directory controls which Track implementation runs:

```yaml
# track.yaml (illustrative; exact field names TBD in SRD)
type: project
workspace: personal
tool:
  version: "0.1.0"                    # semver of track-cli component
  # optional: digest, channel, or URL override for air-gapped mirrors
project:
  key: KITCHEN
  # ...
```

Rules (intent):

- **Default:** `track init` writes the tool version that created the project.
- **Pin is authoritative** for normal invocations inside the project tree; the host must not silently run a newer global install.
- **Override:** host supports `--tool-version` / `TRACK_TOOL_VERSION` for maintainers and CI matrix testing.
- **Schema vs tool version** remain distinct: `apiVersion` (or equivalent) describes on-disk YAML layout; `tool.version` describes which CLI component interprets it.

### Host capability model

The host is the **only** code with direct OS access. Responsibilities:

| Responsibility | Mechanism |
|----------------|-----------|
| Map config dir across OSes | `track:config` WIT (returns base path; guest opens files via WASI FS preopens) |
| Scope filesystem access | WASI preopens: discovered project root (SRD §3.2.1), `.track/` under that root, user buckets — not arbitrary `~/` |
| Network to sync hub | `wasi:sockets` or host-mediated HTTP WIT; optional per-workspace allowlist |
| Credentials | Host reads `~/.config/track/config.json`; exposes token to guest via WIT handle, not env vars in project repos |
| Stdio / exit code | `wasi:cli` for agent-friendly stdout/stderr; guest exit propagates to host process |
| Clock / randomness | `wasi:clocks`, `wasi:random` as needed |

The guest **must not** depend on platform-specific paths, native FFI, or undeclared imports. Adding a new host capability requires a **versioned WIT interface** and host support; older `track-cli` components continue to run against hosts that still provide older interface versions.

### Implementation language

- **Host:** Rust, embedding Wasmtime with `wasmtime-wasi` Preview 2 bindings. Async host functions align with hub SSE subscribe and concurrent HTTP (see Wasmtime WASIp2 async example).
- **Guest (`track-cli`):** Rust, compiled with `wasm32-wasip2` target and the Component Model toolchain (`cargo component` or equivalent). Business logic shares types and pure modules with host-side tests where practical.

Hub server code is **out of scope** for this ADR; it may remain native Rust or another stack suited to long-running services.

### Distribution

| Artifact | Audience | Update cadence |
|----------|----------|----------------|
| `track-host` | Installed once per machine/image | Rare; security/runtime bumps |
| `track-cli` `.wasm` component | Resolved per project; cached under host cache dir | Per project pin; global cache deduplicates |

Agent and CI images need only **`track-host` + cache directory`** (or a pre-warmed cache layer). Projects pin their own CLI version without rebuilding images.

## Consequences

### Positive

- **One Track logic build** (`wasm32-wasip2`) runs everywhere the host runs — laptops, Linux CI, agent containers.
- **Gradual ecosystem migration** — Project A stays on `track-cli 0.1` while Project B adopts `0.2` with schema changes.
- **Reduced image churn** — VM/container images ship a stable host; Track versions arrive as cached WASM artifacts.
- **Auditable sandbox** — Security-sensitive deployments can disable sockets, narrow preopens, or swap WIT implementations without recompiling Track logic.
- **Interface-stable evolution** — Component Model + WIT gives explicit import/export versioning between host and guest.

### Negative / trade-offs

- **Cold-start cost** — WASM instantiation adds latency vs native binary; mitigate with component caching and warm CI caches.
- **Operational complexity** — Two artifacts, WIT contracts, and a release pipeline for components.
- **Debug ergonomics** — Stack traces span host and guest; investment in logging and test harnesses is required.
- **Ecosystem maturity** — WASIp2 and async components are newer than plain native Rust; toolchain pins and fallbacks must be documented.
- **Windows** — WASI portability helps, but host packaging and FS semantics still need explicit validation (SRD: Windows best-effort).

### Neutral

- Sync hub implementation and deployment are unchanged.
- On-disk YAML formats and hub API remain defined in the SRD; this ADR only affects **how the CLI is built and run**.

## Compliance

How we will verify this decision is working:

- Golden tests run `track-cli` as a component under the host in CI (same path as production).
- A matrix job runs host builds on `linux-amd64`, `linux-arm64`, `macos-amd64`, `macos-arm64` (and Windows when ready).
- Integration tests pin a fixture project to a fixed `tool.version` and assert the host loads that artifact.
- Capability tests assert guest cannot read paths outside configured preopens.

## Open questions (follow-up ADRs or SRD updates)

1. **Component registry** — GitHub Releases, OCI artifacts, or self-hosted mirror? Content-addressing vs semver tags?
2. **WIT package layout** — monorepo `wit/track/` vs published `track:host` world version scheme.
3. **Offline and air-gapped** — vendoring `track-cli.wasm` into `.track/` vs host cache only.
4. **Developer workflow** — `cargo run` native dev mode vs always-component for parity.
5. **Hub client TLS** — guest-owned via `wasi:sockets` vs host-mediated HTTP with certificate store injection.
6. **Exact `track.yaml` fields** — finalize `tool` block schema and interaction with SRD `apiVersion`.

## References

- [Wasmtime WASIp2 example](https://docs.wasmtime.dev/examples-wasip2.html) — host instantiation, sync and async
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [WASI Preview 2](https://github.com/WebAssembly/WASI/blob/main/preview2/README.md)
- [Track PRD](../PRD.md) — CLI-first, local-first, agent ergonomics
- [Track SRD](../SRD.md) — `track.yaml`, versioning (§8), portability (§8)

## Related decisions

- [ADR 0002](0002-host-guest-wit-interfaces.md) — host–guest WIT interfaces, project root discovery, and on-disk storage scopes
