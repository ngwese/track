# Track — Product Requirements Document

**Version:** 0.4 (draft)  
**Status:** Draft for review  
**Last updated:** 2026-06-06

> **Companion document:** Technical design and implementation detail live in the [Software Requirements Document](./SRD.md).

---

## 1. Executive summary

**Track** is a personal issue tracker designed to manage many diverse projects—software development, hardware design, home improvement, trip planning, and more—from a single system. Unlike team-centric tools optimized for one domain, Track treats each project as a **configurable schema** over a shared core model.

Four principles define the product:

1. **Issue tracking as code** — project structure (types, states, workflows, labels, efforts, components) is declared in version-controlled files and pushed to the system, enabling review, sharing, and agent/human co-editing.
2. **CLI-first** — all configuration and CRUD is available through a command-line interface suitable for people, agents, scripts, and headless environments.
3. **Local-first where possible** — each participant maintains a local working copy that remains usable offline; mutations are applied locally first, then reconciled with the hub.
4. **Event-driven convergence** — a **sync hub** (one per workspace) provides a real-time, distributed core so isolated agents, humans, and CI jobs observe and react to the same stream of changes.

Track is built for **simultaneous use by humans and agents**. Agent orchestrators discover actionable work, claim tasks, stream progress from isolated environments, and hand off for human review—or **request human input** when blocked—all through the same CLI and hub APIs that humans use. Workflows such as **agent → human review → post-merge CI** and **agent blocked → needs input → resume** are first-class control flows, not afterthoughts.

---

## 2. Problem statement

Personal project work is fragmented across notebooks, ad-hoc todo apps, team issue trackers (when a "real" project warrants one), and domain-specific tools. Each tool encodes different assumptions:

- Software trackers assume sprints, story points, and bug/feature taxonomies.
- Home or travel planning tools lack dependencies, assignees, or structured workflows.
- Team tools (Linear, Jira, Plane) assume multi-user SaaS, always-on connectivity, and UI-first configuration.

When AI agents participate in project work, the gap widens: agents need scriptable, deterministic interfaces, stable identifiers, and a **coordination point** where orchestrators discover work, agents claim tasks in isolation, and humans or CI confirm completion—not browser automation over a web UI.

**Track** addresses this with one flexible core (issues, dependencies, efforts, components), **per-project schema customization**, **declarative configuration in Git**, a **local-first CLI**, and a **sync hub** that converges participants through an event-driven API.

---

## 3. Vision and goals

### Vision

A personal (or small-group) issue tracker where project structure is as portable and reviewable as application code, operable entirely from the terminal, usable offline on each client, and **kept convergent** across humans, agents, and CI through a workspace sync hub.

### Goals

| ID | Goal |
|----|------|
| G1 | Support heterogeneous project types with per-project issue types, states, workflows, and custom fields |
| G2 | Enable "issue tracking as code": init from template → edit YAML → validate → push |
| G3 | Provide complete CRUD for projects, issues, efforts, and components via CLI |
| G4 | Bias toward offline-capable local clients without sacrificing multi-participant convergence |
| G5 | First-class agent ergonomics: stable IDs, JSON output, claim/lease, progress signals, exit codes |
| G6 | Allow efforts to form dependency graphs for roadmap-style planning |
| G7 | Keep the default experience simple; complexity is opt-in via schema and features |
| G8 | Provide a sync hub with real-time events so orchestrators, agents, humans, and CI converge on shared state |
| G9 | Separate workspace (hub) infrastructure config from per-project issue-tracking-as-code |
| G10 | Support agent execution lifecycle: claim → progress → blocked handoff (`Needs Input`) → human clarification → resume |

### Non-goals (initial releases)

- Replacing full team collaboration suites (rich notifications, granular RBAC, chat)
- Built-in web UI as a primary interface (may come later; CLI + hub API are authoritative in v1)
- Enterprise SSO, audit compliance, or multi-tenant SaaS hosting
- Deep integrations with GitHub/GitLab PR linking (future consideration; CI signal via CLI is in scope)
- Formula/computed fields (defer; see Plane Compose for reference)

---

## 4. Core principles

### 4.1 Issue tracking as code

Project creation and structural changes happen by **tailoring a template**—a directory of YAML files describing global properties, schema, and optionally seed work (efforts, issues). The template is validated locally, then **pushed** to update runtime configuration.

Benefits (aligned with [Plane Compose](https://developers.plane.so/dev-tools/plane-compose)):

- Structural changes produce Git diffs and PRs
- Templates are shareable across users and instances via Git URLs
- Agents can propose schema changes as commits
- Schema and work have distinct lifecycles (schema changes rarely; work changes constantly)

**Design stance:** Local declared files are the **preferred source of truth** for project structure. Remote/UI-originated structural changes are supported via explicit **import/pull** operations, not silent bidirectional merge.

### 4.2 CLI-first

Every operation available to a user must be available to an agent:

- Human: `track issue create --title "..." --type Story`
- Agent: same command, plus `--json`, `--force`, `--dry-run`, documented exit codes
- CI: non-interactive auth, `track push --exit-code`

No feature is CLI-exclusive to humans; no feature requires a GUI.

### 4.3 Local-first where possible

- Each client (human laptop, agent sandbox, CI runner) maintains a **local store** and optional declarative project files
- Reads and writes succeed offline against local state; the CLI queues outbound mutations when the hub is unreachable
- Sync is **deliberate** for bulk reconciliation: push, pull, status, diff
- Conflict detection on reconcile; client chooses merge strategy (merge vs force), mirroring Plane Compose's pull semantics

Local-first optimizes individual responsiveness; it does **not** mean the system is single-user or offline-only.

### 4.4 Event-driven convergence (sync hub)

A **workspace** maps to a **sync hub**—the authoritative coordination point for all projects registered to that workspace. The hub:

- Accepts mutations from CLI clients (push, direct API writes, CI signals)
- Persists canonical state and an append-only **event log**
- Fans out events in near real time (SSE, WebSocket, or long-poll) so participants converge
- Exposes a **poll API** with cursors for orchestrators that cannot maintain persistent connections

This hybrid model supports agent orchestration: isolated agents report progress immediately; humans and CI observe the same event stream; local caches update incrementally without requiring a full pull.

**Operational vs declarative work:** Short-lived **claim / progress / release** signals are durable hub-backed telemetry for orchestrators (see SRD §2.15). **Comments** are durable discussion records for humans and agents—including explicit requests for human input. They serve different purposes and emit different events.

### 4.5 Workspace vs project configuration

**Project configuration** (schema, issues, efforts, components) lives in independent project directories under version control—issue tracking as code.

**Workspace configuration** (hub deployment, auth, retention, event transport, agent registry) is **infrastructure as code** stored separately—typically in an infrastructure repository—not co-mingled with project directories. A project's `track.yaml` references its workspace; it does not embed hub infra.

---

## 5. User personas

| Persona | Needs |
|---------|-------|
| **Solo maker** | One CLI, many projects (app, workshop, house), minimal setup |
| **Agent operator** | Scriptable issue CRUD, JSON I/O, stable IDs, template iteration in Git |
| **Technical PM** | Roadmap via effort dependencies, schema evolution via PRs |
| **Agent orchestrator** | Poll/subscribe hub events, dispatch work, track claims and leases |
| **CI runner** | Post-merge signal: transition issue, attach evidence, release claim |
| **Occasionally connected user** | Local edits offline; push and subscribe when hub reachable |

---

## 6. Related documents

| Document | Audience | Contents |
|----------|----------|----------|
| [SRD.md](./SRD.md) | Engineering | Domain model, file formats, CLI, architecture, functional/NFR requirements, MVP, milestones |
| [infra/README.md](../infra/README.md) | Operations | Sync hub deployment (infra-as-code) |

Product goals (§3) map to engineering deliverables in SRD §9 (MVP) and SRD §10 (milestones). Agent handoff workflows (G10) are specified in SRD §6.7. Open questions tracked in SRD §11 until resolved and promoted here.
