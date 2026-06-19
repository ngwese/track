//! [`EntityStore`] implementation for entity tables.

use indexmap::{IndexMap, IndexSet};
use rusqlite::params;
use track_entity::{
    Claim, Comment, EntityKind, FieldProvenance, FieldValue, ItemHeader, ReducedItem, Relation,
};
use track_id::{Actor, TrackUlid};
use track_store::{CounterAdjustOp, EntityStore, SetAddOp, SetRemoveOp, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{optional_text_to_ulid, row_get, text_to_ulid, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl EntityStore for TrackSqliteStore {
    fn upsert_header(&mut self, header: &ItemHeader) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO entities (
                    entity_uuid, project_uuid, entity_kind, item_type, identifier,
                    number, state_key, archived, schema_version_applied,
                    created_hlc, updated_hlc
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(entity_uuid) DO UPDATE SET
                    project_uuid = excluded.project_uuid,
                    entity_kind = excluded.entity_kind,
                    item_type = excluded.item_type,
                    identifier = excluded.identifier,
                    number = excluded.number,
                    state_key = excluded.state_key,
                    archived = excluded.archived,
                    schema_version_applied = excluded.schema_version_applied,
                    updated_hlc = excluded.updated_hlc",
                params![
                    ulid_to_text(&header.entity_uuid),
                    ulid_to_text(&header.project_uuid),
                    entity_kind_to_str(header.entity_kind),
                    header.item_type,
                    header.identifier,
                    header.number.map(|n| n as i64),
                    header.state_key,
                    i32::from(header.archived),
                    header.schema_version_applied.to_string(),
                    header.created_hlc,
                    header.updated_hlc,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_header(&self, entity_uuid: &TrackUlid) -> Result<Option<ItemHeader>, StoreError> {
        load_header(&self.conn, entity_uuid)
    }

    fn set_scalar_field(
        &mut self,
        entity_uuid: &TrackUlid,
        field: &str,
        value: Option<&FieldValue>,
        provenance: FieldProvenance,
    ) -> Result<(), StoreError> {
        match value {
            Some(v) => {
                let value_json = field_value_inner_json(v)?;
                let value_type = field_value_type(v);
                self.conn
                    .execute(
                        "INSERT INTO entity_fields (
                            entity_uuid, field_name, value_json, value_type,
                            updated_by_event_uuid, updated_hlc
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        ON CONFLICT(entity_uuid, field_name) DO UPDATE SET
                            value_json = excluded.value_json,
                            value_type = excluded.value_type,
                            updated_by_event_uuid = excluded.updated_by_event_uuid,
                            updated_hlc = excluded.updated_hlc",
                        params![
                            ulid_to_text(entity_uuid),
                            field,
                            value_json,
                            value_type,
                            ulid_to_text(&provenance.event_uuid),
                            provenance.hlc_wire,
                        ],
                    )
                    .map_err(map_rusqlite_error)?;
            }
            None => {
                self.conn
                    .execute(
                        "INSERT INTO entity_fields (
                            entity_uuid, field_name, value_json, value_type,
                            updated_by_event_uuid, updated_hlc
                        ) VALUES (?1, ?2, NULL, 'cleared', ?3, ?4)
                        ON CONFLICT(entity_uuid, field_name) DO UPDATE SET
                            value_json = NULL,
                            value_type = 'cleared',
                            updated_by_event_uuid = excluded.updated_by_event_uuid,
                            updated_hlc = excluded.updated_hlc",
                        params![
                            ulid_to_text(entity_uuid),
                            field,
                            ulid_to_text(&provenance.event_uuid),
                            provenance.hlc_wire,
                        ],
                    )
                    .map_err(map_rusqlite_error)?;
            }
        }
        Ok(())
    }

    fn get_scalar_field(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldValue>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT value_json, value_type FROM entity_fields
                 WHERE entity_uuid = ?1 AND field_name = ?2",
            )
            .map_err(map_rusqlite_error)?;
        let mut rows = stmt
            .query(params![ulid_to_text(entity_uuid), field])
            .map_err(map_rusqlite_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };
        let value_json: Option<String> = row_get(row, 0)?;
        let Some(value_json) = value_json else {
            return Ok(None);
        };
        decode_field_value(&row_get::<String>(row, 1)?, &value_json).map(Some)
    }

    fn get_field_provenance(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldProvenance>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT ef.updated_by_event_uuid, ef.updated_hlc, le.node_uuid, le.stream_seq
                 FROM entity_fields ef
                 INNER JOIN log_events le ON le.event_uuid = ef.updated_by_event_uuid
                 WHERE ef.entity_uuid = ?1 AND ef.field_name = ?2",
            )
            .map_err(map_rusqlite_error)?;
        let mut rows = stmt
            .query(params![ulid_to_text(entity_uuid), field])
            .map_err(map_rusqlite_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };
        field_provenance_from_parts(
            row_get(row, 0)?,
            row_get(row, 1)?,
            row_get(row, 2)?,
            row_get(row, 3)?,
        )
        .map(Some)
    }

    fn apply_set_add(&mut self, op: SetAddOp) -> Result<(), StoreError> {
        crate::or_set_sqlite::apply_or_set_add(&self.conn, &op)
    }

    fn apply_set_remove(&mut self, op: SetRemoveOp) -> Result<(), StoreError> {
        crate::or_set_sqlite::apply_or_set_remove(&self.conn, &op)
    }

    fn apply_counter_adjust(&mut self, op: CounterAdjustOp) -> Result<(), StoreError> {
        let inserted = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO entity_counter_adjustments (
                    event_uuid, entity_uuid, field_name, delta,
                    applied_hlc, node_uuid, stream_seq
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    ulid_to_text(&op.event_uuid),
                    ulid_to_text(&op.entity_uuid),
                    op.field,
                    op.delta,
                    op.hlc_wire,
                    ulid_to_text(&op.node_uuid),
                    op.stream_seq as i64,
                ],
            )
            .map_err(map_rusqlite_error)?;
        if inserted == 0 {
            return Ok(());
        }

        let sum: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(delta), 0)
                 FROM entity_counter_adjustments
                 WHERE entity_uuid = ?1 AND field_name = ?2",
                params![ulid_to_text(&op.entity_uuid), op.field],
                |row| row.get(0),
            )
            .map_err(map_rusqlite_error)?;

        let value = FieldValue::Integer(sum);
        let value_json = field_value_inner_json(&value)?;
        let value_type = field_value_type(&value);
        self.conn
            .execute(
                "INSERT INTO entity_fields (
                    entity_uuid, field_name, value_json, value_type,
                    updated_by_event_uuid, updated_hlc
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(entity_uuid, field_name) DO UPDATE SET
                    value_json = excluded.value_json,
                    value_type = excluded.value_type,
                    updated_by_event_uuid = excluded.updated_by_event_uuid,
                    updated_hlc = excluded.updated_hlc",
                params![
                    ulid_to_text(&op.entity_uuid),
                    op.field,
                    value_json,
                    value_type,
                    ulid_to_text(&op.event_uuid),
                    op.hlc_wire,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_set_members(
        &self,
        entity_uuid: &TrackUlid,
        set_name: &str,
    ) -> Result<Vec<String>, StoreError> {
        crate::or_set_sqlite::list_active_set_members(&self.conn, entity_uuid, set_name)
    }

    fn upsert_comment(&mut self, comment: &Comment) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO comments (
                    comment_uuid, entity_uuid, author, body_markdown, created_hlc,
                    superseded_by_comment_version_uuid, deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(comment_uuid) DO UPDATE SET
                    entity_uuid = excluded.entity_uuid,
                    author = excluded.author,
                    body_markdown = excluded.body_markdown,
                    created_hlc = excluded.created_hlc,
                    superseded_by_comment_version_uuid = excluded.superseded_by_comment_version_uuid,
                    deleted = excluded.deleted",
                params![
                    ulid_to_text(&comment.comment_uuid),
                    ulid_to_text(&comment.entity_uuid),
                    comment.author.to_string(),
                    comment.body_markdown,
                    comment.created_hlc,
                    comment.superseded_by.as_ref().map(ulid_to_text),
                    i32::from(comment.deleted),
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_comments(&self, entity_uuid: &TrackUlid) -> Result<Vec<Comment>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT comment_uuid, author, body_markdown, created_hlc,
                        superseded_by_comment_version_uuid, deleted
                 FROM comments WHERE entity_uuid = ?1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(entity_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut comments = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            comments.push(Comment {
                comment_uuid: text_to_ulid(
                    row.get::<_, String>(0)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                entity_uuid: *entity_uuid,
                author: row
                    .get::<_, String>(1)
                    .map_err(map_rusqlite_error)?
                    .parse()
                    .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))?,
                body_markdown: row.get(2).map_err(map_rusqlite_error)?,
                created_hlc: row.get(3).map_err(map_rusqlite_error)?,
                replaces: None,
                superseded_by: optional_text_to_ulid(row.get(4).map_err(map_rusqlite_error)?)?,
                deleted: row.get::<_, i32>(5).map_err(map_rusqlite_error)? != 0,
            });
        }
        Ok(comments)
    }

    fn upsert_relation(&mut self, relation: &Relation) -> Result<(), StoreError> {
        let created_by = latest_log_event_for_project(&self.conn, &relation.project_uuid)?
            .ok_or_else(|| {
                StoreError::ForeignKey(
                    "cannot upsert relation before any log event exists for project".into(),
                )
            })?;

        let attrs_json = relation
            .attrs
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO relations (
                    relation_uuid, project_uuid, relation_kind, from_entity_uuid,
                    to_entity_uuid, attrs_json, created_by_event_uuid, created_hlc,
                    deleted_by_event_uuid, deleted_hlc
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL)
                ON CONFLICT(relation_uuid) DO UPDATE SET
                    project_uuid = excluded.project_uuid,
                    relation_kind = excluded.relation_kind,
                    from_entity_uuid = excluded.from_entity_uuid,
                    to_entity_uuid = excluded.to_entity_uuid,
                    attrs_json = excluded.attrs_json,
                    created_by_event_uuid = excluded.created_by_event_uuid,
                    created_hlc = excluded.created_hlc,
                    deleted_by_event_uuid = NULL,
                    deleted_hlc = NULL",
                params![
                    ulid_to_text(&relation.relation_uuid),
                    ulid_to_text(&relation.project_uuid),
                    relation.relation_kind,
                    ulid_to_text(&relation.from_entity_uuid),
                    ulid_to_text(&relation.to_entity_uuid),
                    attrs_json,
                    ulid_to_text(&created_by),
                    relation.created_hlc,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_relation(&self, relation_uuid: &TrackUlid) -> Result<Option<Relation>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT project_uuid, relation_kind, from_entity_uuid, to_entity_uuid,
                        attrs_json, created_hlc, deleted_by_event_uuid
                 FROM relations WHERE relation_uuid = ?1",
            )
            .map_err(map_rusqlite_error)?;
        let mut rows = stmt
            .query(params![ulid_to_text(relation_uuid)])
            .map_err(map_rusqlite_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };
        let attrs: Option<String> = row_get(row, 4)?;
        Ok(Some(Relation {
            relation_uuid: *relation_uuid,
            project_uuid: text_to_ulid(row_get::<String>(row, 0)?.as_str())?,
            relation_kind: row_get(row, 1)?,
            from_entity_uuid: text_to_ulid(row_get::<String>(row, 2)?.as_str())?,
            to_entity_uuid: text_to_ulid(row_get::<String>(row, 3)?.as_str())?,
            attrs: attrs
                .as_ref()
                .map(|json| serde_json::from_str(json))
                .transpose()
                .map_err(|e| StoreError::Serialization(e.to_string()))?,
            created_hlc: row_get(row, 5)?,
            deleted: row_get::<Option<String>>(row, 6)?.is_some(),
        }))
    }

    fn upsert_claim(&mut self, _claim: &Claim) -> Result<(), StoreError> {
        Ok(())
    }

    fn get_claim(&self, _entity_uuid: &TrackUlid) -> Result<Option<Claim>, StoreError> {
        Ok(None)
    }

    fn list_relations_for_entity(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT relation_uuid, project_uuid, relation_kind, from_entity_uuid,
                        to_entity_uuid, attrs_json, created_hlc, deleted_by_event_uuid
                 FROM relations
                 WHERE (from_entity_uuid = ?1 OR to_entity_uuid = ?1)
                   AND deleted_by_event_uuid IS NULL",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(entity_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut relations = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            let attrs: Option<String> = row.get(5).map_err(map_rusqlite_error)?;
            relations.push(Relation {
                relation_uuid: text_to_ulid(
                    row.get::<_, String>(0)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                project_uuid: text_to_ulid(
                    row.get::<_, String>(1)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                relation_kind: row.get(2).map_err(map_rusqlite_error)?,
                from_entity_uuid: text_to_ulid(
                    row.get::<_, String>(3)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                to_entity_uuid: text_to_ulid(
                    row.get::<_, String>(4)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                attrs: attrs
                    .as_ref()
                    .map(|json| serde_json::from_str(json))
                    .transpose()
                    .map_err(|e| StoreError::Serialization(e.to_string()))?,
                created_hlc: row.get(6).map_err(map_rusqlite_error)?,
                deleted: row
                    .get::<_, Option<String>>(7)
                    .map_err(map_rusqlite_error)?
                    .is_some(),
            });
        }
        Ok(relations)
    }

    fn get_reduced_item(&self, entity_uuid: &TrackUlid) -> Result<Option<ReducedItem>, StoreError> {
        let header = match load_header(&self.conn, entity_uuid)? {
            Some(h) => h,
            None => return Ok(None),
        };

        let fields = load_fields(&self.conn, entity_uuid)?;
        let labels = load_set_members(&self.conn, entity_uuid, "labels")?;
        let assignees = load_assignees(&self.conn, entity_uuid)?;

        Ok(Some(ReducedItem {
            header,
            fields: fields.0,
            field_provenance: fields.1,
            labels,
            assignees,
        }))
    }

    fn list_entity_uuids_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<TrackUlid>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT entity_uuid FROM entities WHERE project_uuid = ?1")
            .map_err(map_rusqlite_error)?;
        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;
        let mut entity_uuids = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            entity_uuids.push(text_to_ulid(row_get::<String>(row, 0)?.as_str())?);
        }
        Ok(entity_uuids)
    }

    fn list_active_relations_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT relation_uuid, project_uuid, relation_kind, from_entity_uuid,
                        to_entity_uuid, attrs_json, created_hlc, deleted_by_event_uuid
                 FROM relations
                 WHERE project_uuid = ?1 AND deleted_by_event_uuid IS NULL",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut relations = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            let attrs: Option<String> = row.get(5).map_err(map_rusqlite_error)?;
            relations.push(Relation {
                relation_uuid: text_to_ulid(row_get::<String>(row, 0)?.as_str())?,
                project_uuid: text_to_ulid(row_get::<String>(row, 1)?.as_str())?,
                relation_kind: row_get(row, 2)?,
                from_entity_uuid: text_to_ulid(row_get::<String>(row, 3)?.as_str())?,
                to_entity_uuid: text_to_ulid(row_get::<String>(row, 4)?.as_str())?,
                attrs: attrs
                    .as_ref()
                    .map(|json| serde_json::from_str(json))
                    .transpose()
                    .map_err(|e| StoreError::Serialization(e.to_string()))?,
                created_hlc: row_get(row, 6)?,
                deleted: false,
            });
        }
        Ok(relations)
    }
}

fn latest_log_event_for_project(
    conn: &rusqlite::Connection,
    project_uuid: &TrackUlid,
) -> Result<Option<TrackUlid>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT event_uuid FROM log_events
             WHERE project_uuid = ?1 ORDER BY hlc DESC LIMIT 1",
        )
        .map_err(map_rusqlite_error)?;
    let mut rows = stmt
        .query(params![ulid_to_text(project_uuid)])
        .map_err(map_rusqlite_error)?;
    let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
        return Ok(None);
    };
    text_to_ulid(row_get::<String>(row, 0)?.as_str()).map(Some)
}

fn entity_kind_to_str(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Issue => "issue",
        EntityKind::Effort => "effort",
        EntityKind::Component => "component",
    }
}

fn parse_entity_kind(s: &str) -> Result<EntityKind, StoreError> {
    s.parse()
        .map_err(|_| StoreError::Serialization(format!("unknown entity_kind: {s}")))
}

fn field_value_type(value: &FieldValue) -> &'static str {
    match value {
        FieldValue::String(_) => "string",
        FieldValue::Integer(_) => "integer",
        FieldValue::Decimal(_) => "decimal",
        FieldValue::Boolean(_) => "boolean",
        FieldValue::Date(_) => "date",
        FieldValue::DateTime(_) => "datetime",
        FieldValue::Member(_) => "member",
        FieldValue::EntityRef(_) => "entity_ref",
        FieldValue::Json(_) => "json",
    }
}

fn field_value_inner_json(value: &FieldValue) -> Result<String, StoreError> {
    let tagged =
        serde_json::to_value(value).map_err(|e| StoreError::Serialization(e.to_string()))?;
    let inner = tagged
        .get("value")
        .ok_or_else(|| StoreError::Serialization("missing field value inner".into()))?;
    serde_json::to_string(inner).map_err(|e| StoreError::Serialization(e.to_string()))
}

fn decode_field_value(value_type: &str, value_json: &str) -> Result<FieldValue, StoreError> {
    let inner: serde_json::Value =
        serde_json::from_str(value_json).map_err(|e| StoreError::Serialization(e.to_string()))?;
    let tagged = serde_json::json!({ "type": value_type, "value": inner });
    serde_json::from_value(tagged).map_err(|e| StoreError::Serialization(e.to_string()))
}

fn load_header(
    conn: &rusqlite::Connection,
    entity_uuid: &TrackUlid,
) -> Result<Option<ItemHeader>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT project_uuid, entity_kind, item_type, identifier, number,
                    state_key, archived, schema_version_applied, created_hlc, updated_hlc
             FROM entities WHERE entity_uuid = ?1",
        )
        .map_err(map_rusqlite_error)?;

    let mut rows = stmt
        .query(params![ulid_to_text(entity_uuid)])
        .map_err(map_rusqlite_error)?;

    let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
        return Ok(None);
    };

    Ok(Some(ItemHeader {
        entity_uuid: *entity_uuid,
        project_uuid: text_to_ulid(row_get::<String>(row, 0)?.as_str())?,
        entity_kind: parse_entity_kind(&row_get::<String>(row, 1)?)?,
        item_type: row_get(row, 2)?,
        identifier: row_get(row, 3)?,
        number: row_get::<Option<i64>>(row, 4)?.map(|n| n as u64),
        state_key: row_get(row, 5)?,
        archived: row_get::<i32>(row, 6)? != 0,
        schema_version_applied: row_get::<String>(row, 7)?
            .parse()
            .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))?,
        created_hlc: row_get(row, 8)?,
        updated_hlc: row_get(row, 9)?,
    }))
}

type EntityFields = (
    IndexMap<String, FieldValue>,
    IndexMap<String, FieldProvenance>,
);

fn load_fields(
    conn: &rusqlite::Connection,
    entity_uuid: &TrackUlid,
) -> Result<EntityFields, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT ef.field_name, ef.value_json, ef.value_type,
                    ef.updated_by_event_uuid, ef.updated_hlc, le.node_uuid, le.stream_seq
             FROM entity_fields ef
             INNER JOIN log_events le ON le.event_uuid = ef.updated_by_event_uuid
             WHERE ef.entity_uuid = ?1",
        )
        .map_err(map_rusqlite_error)?;

    let mut fields = IndexMap::new();
    let mut provenance = IndexMap::new();

    let mut rows = stmt
        .query(params![ulid_to_text(entity_uuid)])
        .map_err(map_rusqlite_error)?;

    while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
        let field_name: String = row.get(0).map_err(map_rusqlite_error)?;
        let value_json: Option<String> = row.get(1).map_err(map_rusqlite_error)?;
        if let Some(value_json) = value_json {
            let value_type: String = row.get(2).map_err(map_rusqlite_error)?;
            fields.insert(
                field_name.clone(),
                decode_field_value(&value_type, &value_json)?,
            );
            provenance.insert(field_name, field_provenance_from_row(row)?);
        }
    }

    Ok((fields, provenance))
}

fn field_provenance_from_row(row: &rusqlite::Row<'_>) -> Result<FieldProvenance, StoreError> {
    field_provenance_from_parts(
        row_get::<String>(row, 3)?,
        row_get::<String>(row, 4)?,
        row_get::<String>(row, 5)?,
        row_get::<i64>(row, 6)?,
    )
}

fn field_provenance_from_parts(
    event_text: String,
    hlc_wire: String,
    node_text: String,
    stream_seq: i64,
) -> Result<FieldProvenance, StoreError> {
    Ok(FieldProvenance {
        event_uuid: text_to_ulid(&event_text)?,
        hlc_wire,
        node_uuid: text_to_ulid(&node_text)?,
        stream_seq: stream_seq as u64,
    })
}

fn load_set_members(
    conn: &rusqlite::Connection,
    entity_uuid: &TrackUlid,
    field_name: &str,
) -> Result<IndexSet<String>, StoreError> {
    crate::or_set_sqlite::list_active_set_members(conn, entity_uuid, field_name)
        .map(|v| v.into_iter().collect())
}

fn load_assignees(
    conn: &rusqlite::Connection,
    entity_uuid: &TrackUlid,
) -> Result<IndexSet<Actor>, StoreError> {
    let keys = crate::or_set_sqlite::list_active_set_members(conn, entity_uuid, "assignees")?;
    keys.into_iter()
        .map(|s| {
            s.parse()
                .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))
        })
        .collect()
}
