# How Track is organized

Track separates **where you sync** (workspace) from **what you track**
(project). A workspace provides a sync hub—a coordination point for events,
claims, and shared state. A project is a directory tree with a manifest
(`track.yaml`), schema files, and lazily materialized work under `work/`.

```text
Workspace (sync hub)
└── Project[]          independent directories
    ├── track.yaml     manifest + workspace association
    ├── schema/        types, states, workflows, labels, features
    └── work/            issues, efforts, components (materialized on demand)
```

## Typical workflow

1. **Initialize** a project from a template (`track init`).
2. **Tailor** schema YAML to match your domain (types, states, workflows).
3. **Validate** cross-file references locally (`track schema validate`).
4. **Push** schema (and later work) to the workspace hub.
5. **Operate** day to day via CLI—create issues, transition states, assign
   work—while keeping YAML in sync with the hub.

Schema changes are infrequent and reviewable; issue updates are frequent. Keeping
them in separate files reflects that lifecycle.

## Key terms

| Term | Meaning |
| --- | --- |
| **Workspace** | Sync hub scope; projects associate with one workspace |
| **Project** | Named container with its own schema and work (`KITCHEN`, `APP`) |
| **Issue** | Core work item with title, state, type, and optional custom fields |
| **Schema** | Declarative config: states, workflows, types, labels, feature toggles |
| **Effort** | Time- or goal-oriented grouping (sprint, phase, trip leg) |
| **Component** | Structural artifact (subsystem, room, scene, PCB block) |
| **Relation** | Typed link between issues (blocks, requires, parent, …) |

The following chapters expand each concept. The [schema reference](../schema/reference/states.md)
documents every YAML field.
