# Animation project example

Animation production tracks shots, assets, and deliveries across sequences.
States reflect review gates; components map to sequences or departments.

## Goals

- **Types:** Shot, Asset, Task
- **Workflow:** layout → animation → lighting → comp → approved
- **Efforts:** weekly or milestone **deliveries** (client reviews)
- **Components:** sequences or reels
- **Custom fields:** frame range, department, version

## Feature flags

```yaml
efforts: true
components: true
hierarchy: true
relation_enforcement: false
workflows: true
```

Relation enforcement is often off during exploratory layout; enable it for
locked delivery schedules.

## States

Use groups that match dailies and client review:

```yaml
states:
  Backlog:
    group: backlog
    color: "#6b7280"
  Layout:
    group: started
    color: "#3b82f6"
    is_default: true
  Animation:
    group: started
    color: "#f59e0b"
  Lighting:
    group: started
    color: "#eab308"
  Comp:
    group: started
    color: "#a855f7"
  Approved:
    group: completed
    color: "#22c55e"
  Omit:
    group: cancelled
    color: "#ef4444"
```

## Workflows

One primary pipeline for shots; simpler flow for assets and tasks:

```yaml
workflows:
  shot_pipeline:
    description: Standard shot work
    issue_types: [Shot]
    states: [Backlog, Layout, Animation, Lighting, Comp, Approved, Omit]
  support:
    description: Assets and production tasks
    issue_types: [Asset, Task]
    states: [Backlog, Layout, Approved, Omit]
```

## Types

```yaml
types:
  Shot:
    description: Single shot in a sequence
    workflow: shot_pipeline
    properties:
      Frame range:
        type: text
      Department:
        type: option
        enum: department
  Asset:
    description: Rig, prop, or environment
    workflow: support
    properties:
      Version:
        type: text
  Task:
    description: Production coordination
    workflow: support
    properties: {}
```

## Labels

```yaml
labels:
  - name: hero
    color: "#f59e0b"
  - name: fx
    color: "#8b5cf6"
  - name: client-note
    color: "#ef4444"
```

## Components and efforts (work layer)

When creating work (not schema):

- **Component** per sequence (`SEQ_010`, `SEQ_020`) or reel
- **Effort** per delivery week; link shots due that review
- **Parent** relations group shots under sequence epics if using hierarchy

## Manifest

```yaml
project:
  key: FILM
  name: Short Film Production
defaults:
  type: Shot
  workflow: shot_pipeline
```

Tailor colors and state names to your studio vocabulary; keep semantic
`group` values stable for hub progress aggregation.
