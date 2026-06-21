# Home improvement example

Renovation and repair projects benefit from **phase** efforts, **room**
components, and types that distinguish purchases from contractor tasks.

## Goals

- **Types:** Task, Purchase, Inspection
- **Workflow:** planned → ordered → in progress → done
- **Efforts:** renovation phases (demo, rough-in, finish)
- **Components:** rooms or trades (Kitchen, Electrical)
- **Custom fields:** Room, vendor, permit ID

## Feature flags

```yaml
efforts: true
components: true
hierarchy: false
relation_enforcement: true
workflows: true
```

Enable `relation_enforcement` so "install cabinets" cannot complete before
"order cabinets" when linked with `requires`.

## States

```yaml
states:
  Idea:
    group: backlog
    color: "#6b7280"
    allow_issue_creation: true
  Planned:
    group: unstarted
    color: "#3b82f6"
    is_default: true
  Ordered:
    group: started
    color: "#eab308"
  "In Progress":
    group: started
    color: "#f59e0b"
  Done:
    group: completed
    color: "#22c55e"
  Cancelled:
    group: cancelled
    color: "#ef4444"
```

## Workflows

Purchases skip "In Progress"; inspections may jump straight to Done:

```yaml
workflows:
  renovation:
    description: On-site work and planning
    issue_types: [Task, Inspection]
    states: [Idea, Planned, "In Progress", Done, Cancelled]
  procurement:
    description: Ordering materials
    issue_types: [Purchase]
    states: [Idea, Planned, Ordered, Done, Cancelled]
```

## Types

```yaml
types:
  Task:
    description: Work to perform on site
    workflow: renovation
    properties:
      Room:
        type: option
        enum: room
  Purchase:
    description: Material or fixture order
    workflow: procurement
    properties:
      Vendor:
        type: text
      Room:
        type: option
        enum: room
  Inspection:
    description: Code or quality inspection
    workflow: renovation
    properties:
      Permit:
        type: text
```

## Labels

```yaml
labels:
  - name: plumbing
    color: "#3b82f6"
  - name: electrical
    color: "#f59e0b"
  - name: urgent
    color: "#ef4444"
```

## Components and efforts

Suggested work structure (created after schema push):

| Effort | Example scope |
| --- | --- |
| Phase 1 — Demo | Remove cabinets, cap utilities |
| Phase 2 — Rough-in | Electrical, plumbing |
| Phase 3 — Finish | Cabinets, counters, paint |

| Component | Example |
| --- | --- |
| Kitchen | Primary remodel zone |
| Electrical | Cross-cutting trade |

Link issues with `effort` and `component` URNs; relate purchases to install
tasks with `requires`.

## Manifest

```yaml
project:
  key: KITCHEN
  name: Kitchen Renovation
defaults:
  type: Task
  workflow: renovation
```

This example aligns with the SRD home-improvement illustrations (`Room` option
on Task, phase efforts). Adjust keys and names to your property and permits.
