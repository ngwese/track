# `features.yaml`

**Path:** `schema/features.yaml`

Boolean toggles for optional subsystems and enforcement behavior. Values should
match the `features:` block in [`track.yaml`](./track-yaml.md).

## Document shape

Top-level keys (no nesting):

```yaml
efforts: <bool>
components: <bool>
hierarchy: <bool>
relation_enforcement: <bool>
workflows: <bool>
```

## Fields

| Field | Default (template) | Description |
| --- | --- | --- |
| `efforts` | `false` | Enable efforts (sprints, phases, deliveries) and issue `effort` links |
| `components` | `false` | Enable components and issue `component` links |
| `hierarchy` | `false` | Enable `parent` relations and `is_container` types |
| `relation_enforcement` | `false` | Hub rejects transitions that violate `blocks` / `requires` relations |
| `workflows` | `true` | Honor workflow transition rules when defined |

## Interaction with work

When a flag is `false`:

- CLI and hub may hide or reject operations for that subsystem
- Schema may still declare types and states; flags gate **runtime** features

Turn on flags before creating efforts, components, or parent relations in work
YAML.

## Recommended profiles

| Profile | efforts | components | hierarchy | relation_enforcement |
| --- | --- | --- | --- | --- |
| Personal todos | off | off | off | off |
| Software team | on | on | on | on |
| Animation | on | on | on | off (until locked) |
| Home renovation | on | on | off | on |

## Example

```yaml
efforts: true
components: true
hierarchy: true
relation_enforcement: true
workflows: true
```

## See also

- [Efforts and components concept](../../concepts/efforts-and-components.md)
- [Issues concept](../../concepts/issues.md) — relations
