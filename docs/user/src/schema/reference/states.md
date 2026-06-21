# `states.yaml`

**Path:** `schema/states.yaml`

Defines named workflow **states** and their semantic **groups** for aggregation
(burndown, filters, completion detection).

## Document shape

```yaml
states:
  <StateName>:
    group: <group>
    color: "<hex>"
    is_default: <bool>          # optional
    allow_issue_creation: <bool> # optional
```

State names are map keys. Use quotes when names contain spaces (for example
`"In Progress"`).

## State fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `group` | enum | — | Semantic bucket (see below) |
| `color` | string | — | Hex color for display (`#3b82f6`) |
| `is_default` | bool | `false` | Default state for new issues; **exactly one** must be `true` |
| `allow_issue_creation` | bool | `true` | Whether new issues may be created directly in this state |

## State groups

| Group | Meaning | Examples |
| --- | --- | --- |
| `backlog` | Not yet committed | Backlog, Icebox |
| `unstarted` | Committed, not started | Todo, Ready |
| `started` | Active work | In Progress, Review |
| `completed` | Done | Done, Shipped |
| `cancelled` | Will not do | Cancelled, Won't fix |

The hub sets `completed_at` when an issue enters a state whose group is
`completed` or `cancelled`.

## Validation rules

- At least one state required
- Exactly one `is_default: true`
- Every `group` must be a known enum value
- Names referenced in `workflows.yaml` must exist here

## Example

```yaml
states:
  Todo:
    group: unstarted
    color: "#3b82f6"
    is_default: true
    allow_issue_creation: true
  "In Progress":
    group: started
    color: "#f59e0b"
  Done:
    group: completed
    color: "#22c55e"
```

## See also

- [Workflows](./workflows.md) — lists which states each workflow uses
- [Software example](../examples/software-project.md) — Review state in a dev flow
