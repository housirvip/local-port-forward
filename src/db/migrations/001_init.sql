CREATE TABLE IF NOT EXISTS rules (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT    NOT NULL DEFAULT '',
  local_port  INTEGER NOT NULL UNIQUE,
  remote_host TEXT    NOT NULL,
  remote_port INTEGER NOT NULL,
  protocol    TEXT    NOT NULL DEFAULT 'auto',
  enabled     INTEGER NOT NULL DEFAULT 1,
  log_enabled INTEGER NOT NULL DEFAULT 1,
  log_body    INTEGER NOT NULL DEFAULT 0,
  created_at  DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS request_logs (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_id           INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  created_at        DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  protocol          TEXT    NOT NULL,
  src_addr          TEXT    NOT NULL,
  http_method       TEXT,
  http_path         TEXT,
  http_status       INTEGER,
  http_req_headers  TEXT,
  http_req_body     TEXT,
  http_resp_headers TEXT,
  http_resp_body    TEXT,
  tcp_preview       TEXT,
  bytes_transferred INTEGER NOT NULL DEFAULT 0,
  duration_ms       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_logs_rule_id    ON request_logs(rule_id);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON request_logs(created_at);
