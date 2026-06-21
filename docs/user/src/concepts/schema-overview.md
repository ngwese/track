# Schema files

Schema declares **how work is shaped** in a project—not the work itself. Files
live under `schema/` and cross-reference each other by name.

```text
schema/
├── states.yaml       # workflow columns and semantic groups
├── labels.yaml       # flat tags
├── workflows.yaml    # bind types to states (and optional transitions)
├── types.yaml        # issue types and custom properties
└── features.yaml     # toggle efforts, components, hierarchy, etc.
```

## Authoring order

Edit in dependency order so references resolve:

1. [`states.yaml`](../schema/reference/states.md)
2. [`labels.yaml`](../schema/reference/labels.md)
3. [`workflows.yaml`](../schema/reference/workflows.md)
4. [`types.yaml`](../schema/reference/types.md)
5. [`features.yaml`](../schema/reference/features.md)
6. [`track.yaml`](../schema/reference/track-yaml.md) defaults

## Validate before push

Cross-file rules—exactly one default state, workflow states must exist, types
must reference valid workflows—are checked locally:

```bash
track schema validate
```

Fix reported paths and codes, then push when hub sync is available.

## Relationship diagram

```text
states.yaml ──► workflows.yaml ──► types.yaml ──► track.yaml
features.yaml ────────────────────────────────► track.yaml
labels.yaml ──► issues reference label names by string
```

Schema is **name-keyed** in YAML (for example state name `In Progress`). Work
entities use ULIDs on disk. That split keeps schema readable in diffs while
keeping issue identity stable at scale.
