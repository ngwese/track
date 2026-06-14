CREATE TABLE nodes (
  node_uuid TEXT PRIMARY KEY,
  created_hlc TEXT NOT NULL,
  last_seen_hlc TEXT
);

CREATE TABLE log_events (
  event_uuid TEXT PRIMARY KEY,
  workspace_uuid TEXT NOT NULL,
  project_uuid TEXT NOT NULL,
  node_uuid TEXT NOT NULL,
  actor TEXT NOT NULL,
  stream_id TEXT NOT NULL,
  stream_seq INTEGER NOT NULL,
  hlc TEXT NOT NULL,
  deps_json TEXT,
  schema_version TEXT NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  received_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  reduced INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (node_uuid) REFERENCES nodes(node_uuid)
);

CREATE UNIQUE INDEX idx_log_events_node_stream_seq
  ON log_events(node_uuid, stream_id, stream_seq);
CREATE INDEX idx_log_events_project_hlc
  ON log_events(project_uuid, hlc);
CREATE INDEX idx_log_events_stream_hlc
  ON log_events(stream_id, hlc);

CREATE TABLE replica_progress (
  node_uuid TEXT PRIMARY KEY,
  last_event_uuid TEXT,
  last_hlc TEXT,
  last_stream_seq INTEGER,
  FOREIGN KEY (node_uuid) REFERENCES nodes(node_uuid)
);

CREATE TABLE schema_versions (
  project_uuid TEXT NOT NULL,
  schema_version TEXT NOT NULL,
  base_event_uuid TEXT,
  schema_json TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  is_snapshot INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (project_uuid, schema_version)
);

CREATE TABLE entities (
  entity_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  item_type TEXT,
  identifier TEXT,
  number INTEGER,
  state_key TEXT,
  archived INTEGER NOT NULL DEFAULT 0,
  schema_version_applied TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  updated_hlc TEXT NOT NULL
);

CREATE INDEX idx_entities_project_kind
  ON entities(project_uuid, entity_kind);
CREATE INDEX idx_entities_project_state
  ON entities(project_uuid, state_key);

CREATE TABLE entity_fields (
  entity_uuid TEXT NOT NULL,
  field_name TEXT NOT NULL,
  value_json TEXT,
  value_type TEXT NOT NULL,
  updated_by_event_uuid TEXT NOT NULL,
  updated_hlc TEXT NOT NULL,
  PRIMARY KEY (entity_uuid, field_name),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid),
  FOREIGN KEY (updated_by_event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE entity_set_members (
  entity_uuid TEXT NOT NULL,
  field_name TEXT NOT NULL,
  member_key TEXT NOT NULL,
  added_by_event_uuid TEXT NOT NULL,
  removed_by_event_uuid TEXT,
  added_hlc TEXT NOT NULL,
  removed_hlc TEXT,
  PRIMARY KEY (entity_uuid, field_name, member_key),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE comments (
  comment_uuid TEXT PRIMARY KEY,
  entity_uuid TEXT NOT NULL,
  author TEXT NOT NULL,
  body_markdown TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  superseded_by_comment_version_uuid TEXT,
  deleted INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE relations (
  relation_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  relation_kind TEXT NOT NULL,
  from_entity_uuid TEXT NOT NULL,
  to_entity_uuid TEXT NOT NULL,
  attrs_json TEXT,
  created_by_event_uuid TEXT NOT NULL,
  deleted_by_event_uuid TEXT,
  created_hlc TEXT NOT NULL,
  deleted_hlc TEXT,
  FOREIGN KEY (from_entity_uuid) REFERENCES entities(entity_uuid),
  FOREIGN KEY (to_entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE INDEX idx_relations_from
  ON relations(from_entity_uuid, relation_kind);
CREATE INDEX idx_relations_to
  ON relations(to_entity_uuid, relation_kind);

CREATE TABLE blobs (
  blob_uuid TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  mime_type TEXT NOT NULL,
  file_name TEXT NOT NULL,
  created_by_event_uuid TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  FOREIGN KEY (created_by_event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE blob_links (
  blob_uuid TEXT NOT NULL,
  entity_uuid TEXT NOT NULL,
  role TEXT NOT NULL,
  linked_by_event_uuid TEXT NOT NULL,
  unlinked_by_event_uuid TEXT,
  linked_hlc TEXT NOT NULL,
  unlinked_hlc TEXT,
  PRIMARY KEY (blob_uuid, entity_uuid, role),
  FOREIGN KEY (blob_uuid) REFERENCES blobs(blob_uuid),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE quarantined_events (
  event_uuid TEXT PRIMARY KEY,
  reason TEXT NOT NULL,
  details_json TEXT,
  first_quarantined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE conflicts (
  conflict_uuid TEXT PRIMARY KEY,
  event_uuid TEXT NOT NULL,
  project_uuid TEXT NOT NULL,
  entity_uuid TEXT,
  conflict_type TEXT NOT NULL,
  details_json TEXT NOT NULL,
  resolved INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE snapshots (
  snapshot_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  stream_id TEXT NOT NULL,
  through_event_uuid TEXT NOT NULL,
  snapshot_kind TEXT NOT NULL,
  snapshot_json TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  FOREIGN KEY (through_event_uuid) REFERENCES log_events(event_uuid)
);
