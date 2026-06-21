# `track.yaml` manifest

**Path:** `<project-root>/track.yaml`

The manifest identifies the project, associates it with a workspace, and sets
defaults applied when creating issues.

## Top-level fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `type` | string | Yes | Must be `project` |
| `workspace` | string | Yes | Workspace slug or hub URL for sync |
| `project` | object | Yes | Project identity (see below) |
| `defaults` | object | Yes | Default type and workflow for new issues |
| `template` | string | No | Source template name or URI (for upgrades) |
| `features` | object | No | Feature toggles; should match `schema/features.yaml` |

## `project` object

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `key` | string | Yes | Short uppercase identifier; prefixes display IDs (`KITCHEN-42`) |
| `name` | string | Yes | Human-readable project name |
| `project_uuid` | ULID | Yes | Stable ID; generated at `track init`, preserved on `--force` re-init |
| `description` | string | No | Markdown description |
| `timezone` | string | No | IANA timezone for dates (default `UTC` in template) |

## `defaults` object

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `type` | string | Yes | Must name a type in `schema/types.yaml` |
| `workflow` | string | Yes | Must name a workflow in `schema/workflows.yaml` |

## `features` object

Same keys as [`features.yaml`](./features.md). Keep manifest and schema in sync
so runtime behavior matches declared schema capabilities.

## Example

```yaml
type: project
workspace: personal
project:
  key: KITCHEN
  name: Kitchen Renovation
  project_uuid: 01JHM8X9K2Q4Z0
  description: ""
  timezone: America/Los_Angeles
defaults:
  type: Task
  workflow: default
template: default
features:
  efforts: true
  components: true
  hierarchy: false
  relation_enforcement: true
  workflows: true
```

## See also

- [Creating a project](../../creating-a-project.md)
- [Projects concept](../../concepts/project.md)
