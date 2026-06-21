# Projects

A **project** is a named container with its own key, schema, and work. Examples:
`KITCHEN` (home renovation), `FILM` (animation), `API` (software service).

## Project root

The **project root** is the directory that directly contains `track.yaml`. The
CLI discovers it by walking up from your current directory, or you can pass
`--project PATH`.

Two common layouts:

**Standalone** — dedicated folder or repository:

```text
kitchen/                 # project root
├── track.yaml
├── schema/
├── work/
└── .track/
```

**Embedded in a source repo** — customary `track/` subdirectory:

```text
my-app/                  # repository root
├── src/
└── track/               # project root
    ├── track.yaml
    ├── schema/
    └── work/
```

`track init` defaults to `./track/` when run from a repository root unless you
pass `--standalone` or an explicit path.

## Manifest fields

The manifest (`track.yaml`) holds identity and defaults:

| Field | Purpose |
| --- | --- |
| `project.key` | Short uppercase ID; prefixes issue identifiers (`KITCHEN-42`) |
| `project.name` | Display name |
| `project.project_uuid` | Stable ULID; generated at init |
| `workspace` | Workspace slug this project syncs to |
| `defaults.type` | Default issue type for new issues |
| `defaults.workflow` | Default workflow name |
| `features.*` | Feature toggles mirrored from schema (see reference) |

Full field reference: [`track.yaml`](../schema/reference/track-yaml.md).

## Templates

Projects start from a **template**—a directory of schema files and manifest
stubs. The built-in `default` template provides a minimal task workflow. You
tailor copies under `schema/` rather than editing the template in place.
