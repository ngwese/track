# `workflows.yaml`

**Path:** `schema/workflows.yaml`

A **workflow** binds issue **types** to an ordered set of **states** and
optionally restricts **transitions** between them.

## Document shape

```yaml
workflows:
  <WorkflowName>:
    description: <string>       # optional
    issue_types: [<TypeName>, …]
    states: [<StateName>, …]
    transitions:                # optional
      <FromState>:
        - to: <ToState>
```

## Workflow fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `description` | string | No | Human-readable summary |
| `issue_types` | string[] | Yes | Types governed by this workflow; names from `types.yaml` |
| `states` | string[] | Yes | States available in this workflow; names from `states.yaml` |
| `transitions` | map | No | Allowed moves from each source state |

## Transition entries

Each key under `transitions` is a **from** state name. Values are lists of
targets:

```yaml
transitions:
  Todo:
    - to: "In Progress"
    - to: Cancelled
  "In Progress":
    - to: Done
    - to: Todo
```

When `transitions` is **omitted**, all state changes among listed states are
permitted (convenient during development). Explicit transitions support stricter
process gates (for example mandatory Review before Done).

When `features.workflows` is `false`, transition enforcement may be relaxed at
runtime; keep the flag `true` for production software flows.

## Validation rules

- Workflow names must be unique
- Every `states` entry must exist in `states.yaml`
- Every `issue_types` entry must exist in `types.yaml` (cross-check after types
  are authored)
- Every type's `workflow` field must name a workflow that lists that type in
  `issue_types`
- Transition `to` targets must appear in `states`

## Example

```yaml
workflows:
  default:
    description: Standard task flow
    issue_types:
      - Task
    states:
      - Backlog
      - Todo
      - "In Progress"
      - Done
      - Cancelled
```

## See also

- [States](./states.md)
- [Types](./types.md)
- [Features](./features.md) — `workflows` toggle
