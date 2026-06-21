# `labels.yaml`

**Path:** `schema/labels.yaml`

Defines flat, project-scoped **labels** (tags). Issues reference labels by
name in their `labels` array.

## Document shape

```yaml
labels:
  - name: <string>
    color: "<hex>"
```

An empty list is valid:

```yaml
labels: []
```

## Label fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | Yes | Unique label name; case-sensitive |
| `color` | string | Yes | Hex color for display |

## Usage on issues

Materialized issues store label names as strings:

```yaml
labels:
  - backend
  - regression
```

Adding a label here does not retroactively tag issues; it makes the name valid
for assignment via CLI or hub API.

## Validation rules

- Label names must be unique within the project
- Unknown label names on issues may be rejected by validators or hub policy
  (behavior depends on release)

## Example

```yaml
labels:
  - name: backend
    color: "#3b82f6"
  - name: urgent-path
    color: "#ef4444"
```

## See also

- [Issues concept](../../concepts/issues.md)
- Domain [examples](../examples/software-project.md) for typical label sets
