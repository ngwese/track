# Track

Personal issue tracking for heterogeneous projects—software, hardware, home improvement, travel, and more—from one system. **Issue tracking as code**, **CLI-first**, **local-first**, with a **sync hub** that keeps humans, agents, and CI convergent.

> **Status:** Design and infrastructure drafts. Application code (CLI, hub) is not implemented yet.

## Why Track

Team trackers (Linear, Jira, Plane) assume SaaS, UI-first config, and a single domain. Personal work spans many domains with different schemas. When agents join the workflow, you need scriptable APIs, stable identifiers, and a coordination point—not browser automation.

Track provides:

- **Per-project schema** — types, states, workflows, labels, efforts, components
- **Declarative projects in Git** — validate locally, push to the hub
- **Lazy materialization** — fetch only the issues you need into `work/issues/<eid>/`
- **Agent orchestration** — claim, progress telemetry, human handoff, resume

## Principles

1. **Issue tracking as code** — structure in version-controlled YAML
2. **CLI-first** — every operation available to humans and agents (`--json`, exit codes)
3. **Local-first** — offline-capable clients; deliberate sync via push/pull
4. **Event-driven convergence** — workspace sync hub with subscribe/poll APIs

## Documentation

| Document | Description |
|----------|-------------|
| [docs/PRD.md](docs/PRD.md) | Vision, goals, personas, product principles |
| [docs/SRD.md](docs/SRD.md) | Domain model, file formats, CLI, hub API, MVP |
| [docs/adr/](docs/adr/) | Architecture decision records |
| [infra/README.md](infra/README.md) | Deploy a sync hub workspace |
| [AGENTS.md](AGENTS.md) | Agent workflow; [Conventional Commits](https://www.conventionalcommits.org/) |
