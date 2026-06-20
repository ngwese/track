# track-materialize-yaml

## Role

YAML and JSON file projection from reduced entity state (SRD §3). Reads through
`track_store::EntityStore` traits — never touches SQLite directly.

## Classification

Concrete implementation

## Key dependencies

`track-store`, `track-entity`, `serde_yaml`, `sha2`

## Public surface

- `DefaultProjector` — main materialization entry point
- `MaterializeWriter`, `MaterializeSelector`, `YamlExclusionPolicy`
- Path helpers: `issue_yaml_path`, `schema_dir`, `cache_db_path`, …
- `YamlIssueBundle`, `WriteReport`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `MaterializeWriter` | `DefaultProjector` |
| `MaterializeSelector` | `DefaultProjector` |
| `YamlExclusionPolicy` | `DefaultYamlExclusionPolicy` |

## Related ADRs / SRD

- SRD §3 (issue tracking as code — file format)
- [ADR 0003](../../adr/0003-domain-model-and-replication-log.md) (YAML projection)

## When to touch

- On-disk layout or YAML shape changes
- Selective materialization or exclusion policy

See [Reduction and materialize](../interfaces/reduction-and-materialize.md).
