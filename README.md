# Track

[![CI](https://github.com/ngwese/track/actions/workflows/ci.yml/badge.svg)](https://github.com/ngwese/track/actions/workflows/ci.yml)

Personal issue tracking for heterogeneous projects—software, hardware, home
improvement, travel, and more—from one system. **Issue tracking as code**,
**CLI-first**, **local-first**, with a **sync hub** that keeps humans, agents,
and CI convergent.

> **Status:** PRD and SRD approved (v0.4). Sync hub MVP and CLI
> implementation in progress; see [docs/adr/](docs/adr/) for runtime decisions.
>
> **Warning:** Track is experimental and incomplete. APIs, on-disk formats, and
> sync behavior will change without notice. Do not rely on it for production
> work—it may eat your lunch.

## Why Track

Team trackers (Linear, Jira, Plane) assume SaaS, UI-first config, and a single
domain. Personal work spans many domains with different schemas. When agents
join the workflow, you need scriptable APIs, stable identifiers, and a
coordination point—not browser automation.

Track provides:

- **Per-project schema** — types, states, workflows, labels, efforts, components
- **Declarative projects in Git** — validate locally, push to the hub
- **Lazy materialization** — fetch only the issues you need into `work/issues/<eid>/`
- **Agent orchestration** — claim, progress telemetry, human handoff, resume

## Principles

1. **Issue tracking as code** — structure in version-controlled YAML
2. **CLI-first** — every operation available to humans and agents (`--json`,
   exit codes)
3. **Local-first** — offline-capable clients; deliberate sync via push/pull
4. **Event-driven convergence** — workspace sync hub with subscribe/poll APIs

## Documentation

| Document | Description |
| --- | --- |
| [docs/PRD.md](docs/PRD.md) | Vision, goals, personas, product principles |
| [docs/SRD.md](docs/SRD.md) | Domain model, file formats, CLI, hub API, MVP |
| [docs/user/](docs/user/) | User guide (mdBook): concepts, schema authoring, reference |
| [docs/dev/](docs/dev/) | Developer guide (mdBook): crates, traits, implementation guides |
| [docs/adr/](docs/adr/) | Architecture decision records |
| [infra/README.md](infra/README.md) | Deploy a sync hub workspace |
| [AGENTS.md](AGENTS.md) | Agent workflow; [Conventional Commits](https://www.conventionalcommits.org/) |
