# Issues and relations

An **issue** is the core work item: a task, bug, story, purchase order, shot,
or any unit of work your schema defines.

## Identity

Issues have two layers of identity:

| Layer | Field | Purpose |
| --- | --- | --- |
| Canonical | `entity_uuid` | ULID generated at create; stable forever |
| Display | `number` + `identifier` | Human-friendly ref (`KITCHEN-42`); number assigned by hub on first persist |

The canonical ID is used on disk (`work/issues/<entity_uuid>/issue.yaml`) and
in APIs. Display identifiers are for conversation and scripts that prefer short
refs.

## Common properties

Every issue shares a core set of fields regardless of type:

- **Title and description** — summary and markdown body
- **Type and state** — governed by your schema
- **Priority, assignee, labels, dates**
- **Effort and component** — optional links when those features are enabled
- **`properties`** — type-specific custom fields from `types.yaml`

## Relations

Issue-to-issue links are **typed relations**, not ad-hoc fields. Examples:

| Type | Meaning |
| --- | --- |
| `blocks` | Peer cannot start until this issue completes |
| `requires` | This issue cannot complete until peer completes |
| `parent` | Hierarchical grouping (epic → story) when hierarchy is enabled |

Relations materialize in `work/issues/<entity_uuid>/relations.yaml`. When
`features.relation_enforcement` is true, the hub can reject state transitions
that violate `blocks` / `requires` edges.

## Lazy materialization

Not every issue exists on disk at once. Clients **materialize** only the issues
they need into `work/issues/`. Schema always lives in Git; bulk issue history
may stay hub-only until fetched. This keeps large backlogs manageable locally.
