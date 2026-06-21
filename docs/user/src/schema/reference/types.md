# `types.yaml`

**Path:** `schema/types.yaml`

Defines **issue types** (Task, Bug, Story, Purchase, Shot, …), the workflow
each type uses, and **custom properties** stored on issues under
`properties`.

## Document shape

```yaml
types:
  <TypeName>:
    description: <string>       # optional
    workflow: <WorkflowName>
    is_container: <bool>        # optional
    properties:
      <PropertyName>:
        type: <field_type>
        enum: <catalog>         # when type is option
        required: <bool>        # optional
```

Use `properties: {}` when a type has no custom fields.

## Type fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `description` | string | — | Human-readable type summary |
| `workflow` | string | — | Workflow name from `workflows.yaml` |
| `is_container` | bool | `false` | When `true`, type may be target of `parent` relations (epic-like) |
| `properties` | map | `{}` | Custom fields for this type only |

## Property fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `type` | string | — | Field type tag (see below) |
| `enum` | string | — | Enum catalog name when `type` is `option` |
| `required` | bool | `false` | Whether the field must be set on create |

## Property types (v1)

| Type | Use for |
| --- | --- |
| `text` | Short string (branch name, vendor) |
| `number` | Integer estimate, count |
| `decimal` | Fractional measurements |
| `date` | Calendar date |
| `datetime` | Timestamp |
| `option` | Single choice from enum catalog |
| `boolean` | Flag |
| `url` | Link (design doc, CAD) |
| `email` | Contact |
| `member` | Actor reference |
| `entity_ref` | URN or typed reference to another entity |

Formula/computed fields are deferred (see SRD).

## Materialized issue example

```yaml
type: Task
properties:
  Room: Kitchen
```

Property keys match names declared under the issue's type in this file.

## Validation rules

- At least one type required
- Each `workflow` must exist in `workflows.yaml`
- Each type should appear in its workflow's `issue_types` list
- Property names unique per type
- `defaults.type` in `track.yaml` must name a type here

## Example

```yaml
types:
  Task:
    description: A general task
    workflow: default
    properties: {}
  Bug:
    description: Defect report
    workflow: triage
    properties:
      Severity:
        type: option
        enum: severity
        required: true
```

## See also

- [Workflows](./workflows.md)
- [Home improvement example](../examples/home-improvement.md) — Room option field
- SRD §2.5 for extended property examples
