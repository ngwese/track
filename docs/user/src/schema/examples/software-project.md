# Software project example

A typical application team wants distinct issue types, sprint grouping, service
components, and enforced dependency relations.

## Goals

- **Types:** Story, Bug, Task (Task for chores)
- **Workflow:** backlog → active → review → done
- **Efforts:** two-week sprints
- **Components:** one per deployable service or repo
- **Relations:** `blocks` / `requires` enforced on transition

## Feature flags

`schema/features.yaml`:

```yaml
efforts: true
components: true
hierarchy: true
relation_enforcement: true
workflows: true
```

Mirror the same toggles under `features:` in `track.yaml`.

## States

Add a **Review** state in the `started` group between active work and completion:

```yaml
states:
  Backlog:
    group: backlog
    color: "#6b7280"
    allow_issue_creation: true
  Todo:
    group: unstarted
    color: "#3b82f6"
    is_default: true
  "In Progress":
    group: started
    color: "#f59e0b"
  Review:
    group: started
    color: "#a855f7"
  Done:
    group: completed
    color: "#22c55e"
  Cancelled:
    group: cancelled
    color: "#ef4444"
```

## Workflows

Separate **delivery** flow for stories vs lightweight **triage** for bugs:

```yaml
workflows:
  delivery:
    description: Story and task delivery
    issue_types: [Story, Task]
    states: [Backlog, Todo, "In Progress", Review, Done, Cancelled]
  triage:
    description: Bug fix flow
    issue_types: [Bug]
    states: [Backlog, Todo, "In Progress", Done, Cancelled]
```

Add explicit `transitions` later if you need to forbid skipping Review.

## Types and custom fields

```yaml
types:
  Story:
    description: User-facing increment
    workflow: delivery
    is_container: true
    properties:
      Estimate:
        type: number
      Branch:
        type: text
  Bug:
    description: Defect
    workflow: triage
    properties:
      Severity:
        type: option
        enum: severity
  Task:
    description: Chore or infra work
    workflow: delivery
    properties: {}
```

Define enum catalogs in a future `schema/enums.yaml` when supported; until then,
option fields may reference inline enum names as your validator allows.

## Labels

```yaml
labels:
  - name: backend
    color: "#3b82f6"
  - name: frontend
    color: "#22c55e"
  - name: regression
    color: "#ef4444"
```

## Manifest defaults

```yaml
defaults:
  type: Task
  workflow: delivery
```

## What to add in work (later)

- Create **components** for `api`, `web`, `worker` with repository URLs
- Open **efforts** per sprint; link issues via `effort`
- Use **parent** relations for epic → story hierarchy

Validate with `track schema validate` before push.
