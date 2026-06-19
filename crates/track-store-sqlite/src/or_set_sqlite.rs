//! OR-set member load/persist helpers for SQLite.

use rusqlite::{Connection, params};
use track_id::TrackUlid;
use track_store::{
    OrSetMember, SetAddOp, SetRemoveOp, SetStamp, StoreError, merge_set_add, merge_set_remove,
};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{row_get, text_to_ulid, ulid_to_text};

/// Load one OR-set member cell from `entity_set_members`.
pub fn load_or_set_member(
    conn: &Connection,
    entity_uuid: &TrackUlid,
    field_name: &str,
    member: &str,
) -> Result<OrSetMember, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT added_by_event_uuid, added_hlc, removed_by_event_uuid, removed_hlc
             FROM entity_set_members
             WHERE entity_uuid = ?1 AND field_name = ?2 AND member_key = ?3",
        )
        .map_err(map_rusqlite_error)?;

    let mut rows = stmt
        .query(params![ulid_to_text(entity_uuid), field_name, member,])
        .map_err(map_rusqlite_error)?;

    let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
        return Ok(OrSetMember::default());
    };

    let added_event: String = row_get(row, 0)?;
    let added_hlc: String = row_get(row, 1)?;
    let removed_event: Option<String> = row_get(row, 2)?;
    let removed_hlc: Option<String> = row_get(row, 3)?;

    let remove = match (removed_event, removed_hlc) {
        (Some(event), Some(hlc)) => Some(load_stamp(conn, &event, &hlc)?),
        _ => None,
    };

    let add_stamp = load_stamp(conn, &added_event, &added_hlc)?;
    let add = if remove
        .as_ref()
        .is_some_and(|r| r.event_uuid == add_stamp.event_uuid && r.hlc_wire == add_stamp.hlc_wire)
    {
        None
    } else {
        Some(add_stamp)
    };

    Ok(OrSetMember { add, remove })
}

/// Apply an add operation with observed-remove merge semantics.
pub fn apply_or_set_add(conn: &Connection, op: &SetAddOp) -> Result<(), StoreError> {
    let mut cell = load_or_set_member(conn, &op.entity_uuid, &op.set_name, &op.member)?;
    merge_set_add(&mut cell, op);
    persist_or_set_member(conn, &op.entity_uuid, &op.set_name, &op.member, &cell)
}

/// Apply a remove operation with observed-remove merge semantics.
pub fn apply_or_set_remove(conn: &Connection, op: &SetRemoveOp) -> Result<(), StoreError> {
    let mut cell = load_or_set_member(conn, &op.entity_uuid, &op.set_name, &op.member)?;
    merge_set_remove(&mut cell, op);
    persist_or_set_member(conn, &op.entity_uuid, &op.set_name, &op.member, &cell)
}

/// List active member keys for one named set on an entity.
pub fn list_active_set_members(
    conn: &Connection,
    entity_uuid: &TrackUlid,
    field_name: &str,
) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT member_key FROM entity_set_members
             WHERE entity_uuid = ?1 AND field_name = ?2",
        )
        .map_err(map_rusqlite_error)?;

    let mut rows = stmt
        .query(params![ulid_to_text(entity_uuid), field_name])
        .map_err(map_rusqlite_error)?;

    let mut members = Vec::new();
    while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
        let member: String = row_get(row, 0)?;
        let cell = load_or_set_member(conn, entity_uuid, field_name, &member)?;
        if cell.is_active() {
            members.push(member);
        }
    }
    Ok(members)
}

fn persist_or_set_member(
    conn: &Connection,
    entity_uuid: &TrackUlid,
    field_name: &str,
    member: &str,
    cell: &OrSetMember,
) -> Result<(), StoreError> {
    if cell.add.is_none() && cell.remove.is_none() {
        conn.execute(
            "DELETE FROM entity_set_members
             WHERE entity_uuid = ?1 AND field_name = ?2 AND member_key = ?3",
            params![ulid_to_text(entity_uuid), field_name, member],
        )
        .map_err(map_rusqlite_error)?;
        return Ok(());
    }

    let added = cell
        .add
        .as_ref()
        .or(cell.remove.as_ref())
        .expect("cell has add or remove stamp");
    let (removed_event, removed_hlc) = match &cell.remove {
        Some(stamp) => (
            Some(ulid_to_text(&stamp.event_uuid)),
            Some(stamp.hlc_wire.as_str()),
        ),
        None => (None, None),
    };

    conn.execute(
        "INSERT INTO entity_set_members (
            entity_uuid, field_name, member_key, added_by_event_uuid,
            added_hlc, removed_by_event_uuid, removed_hlc
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(entity_uuid, field_name, member_key) DO UPDATE SET
            added_by_event_uuid = excluded.added_by_event_uuid,
            added_hlc = excluded.added_hlc,
            removed_by_event_uuid = excluded.removed_by_event_uuid,
            removed_hlc = excluded.removed_hlc",
        params![
            ulid_to_text(entity_uuid),
            field_name,
            member,
            ulid_to_text(&added.event_uuid),
            added.hlc_wire,
            removed_event,
            removed_hlc,
        ],
    )
    .map_err(map_rusqlite_error)?;
    Ok(())
}

fn load_stamp(conn: &Connection, event_text: &str, hlc_wire: &str) -> Result<SetStamp, StoreError> {
    let event_uuid = text_to_ulid(event_text)?;
    let mut stmt = conn
        .prepare("SELECT node_uuid, stream_seq FROM log_events WHERE event_uuid = ?1")
        .map_err(map_rusqlite_error)?;
    let mut rows = stmt
        .query(params![event_text])
        .map_err(map_rusqlite_error)?;
    let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
        return Err(StoreError::ForeignKey(format!(
            "missing log event `{event_text}` for OR-set stamp"
        )));
    };
    Ok(SetStamp {
        event_uuid,
        hlc_wire: hlc_wire.to_string(),
        node_uuid: text_to_ulid(row_get::<String>(row, 0)?.as_str())?,
        stream_seq: row_get::<i64>(row, 1)? as u64,
    })
}
