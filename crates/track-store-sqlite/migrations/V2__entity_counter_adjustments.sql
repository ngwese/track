CREATE TABLE entity_counter_adjustments (
  event_uuid TEXT PRIMARY KEY,
  entity_uuid TEXT NOT NULL,
  field_name TEXT NOT NULL,
  delta INTEGER NOT NULL,
  applied_hlc TEXT NOT NULL,
  node_uuid TEXT NOT NULL,
  stream_seq INTEGER NOT NULL,
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE INDEX idx_entity_counter_adjustments_entity_field
  ON entity_counter_adjustments(entity_uuid, field_name);
