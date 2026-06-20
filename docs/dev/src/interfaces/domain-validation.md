# Domain validation

Small extension traits on domain and replication crates. Not persistence or
hub boundaries, but swappable for custom validation or classification logic.

## Entity validation (`track-entity`)

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `EntityValidator` | Validate materialized entity state | `DefaultEntityValidator` |

Used during reduction semantic validation. Custom validators can enforce
project-specific rules without changing store traits.

Rustdoc: `track_entity::EntityValidator`

## Event classification (`track-replication`)

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `EventClassifier` | Map payloads to `EventKind` | `DefaultEventClassifier` |
| `EventPayload` | Typed payload marker per event struct | Each struct in `track_replication::payload` |

`EventPayload` is implemented by individual payload types (`ItemCreatePayload`,
`SchemaInitPayload`, …), not by alternate backends.

## Related

- [track-entity crate page](../crates/track-entity.md)
- [track-replication](../crates/track-replication.md)
