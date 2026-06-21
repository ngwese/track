# Workspaces and sync

A **workspace** maps to a **sync hub**—the authoritative coordination point for
all projects registered to that workspace. When you run `track init`, you
associate the project with a workspace slug (for example `personal`). That slug
identifies which hub receives your pushes.

## What the hub does

The hub:

- Accepts replication events from clients (schema updates, issue mutations)
- Persists a durable log and derived entity projections
- Assigns display identifiers (for example `KITCHEN-42`) on first persist
- Enforces claims and leases when agents work on issues
- Broadcasts changes so other clients converge

## Workspace vs project configuration

**Project configuration**—schema, issues, efforts, components—lives in project
directories under version control. **Workspace configuration**—hub deployment,
auth, retention, transport—is separate infrastructure (see SRD appendix on hub
infra-as-code). A project's `track.yaml` references its workspace; it does not
embed hub deployment details.

## Offline and local-first

You can validate schema and plan pushes without contacting the hub. Local
state under `.track/` tracks sync position and caches indexes. When offline,
read and edit materialized work; reconcile with `track push` / `track pull` when
connected.

> **Note:** Live hub sync is still rolling out. Today you can initialize
> projects, validate schema offline, and dry-run push planning. See
> [Creating a project](../creating-a-project.md) for current CLI coverage.
