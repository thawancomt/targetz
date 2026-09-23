-- Add migration script here
CREATE TABLE IF NOT EXISTS project_status_history(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  from_status TEXT NOT NULL,
  to_status TEXT NOT NULL,
  project_id INTEGER NOT NULL,
  note TEXT NULL,
  change_ask_by TEXT NULL,
  reason TEXT NULL,
  impact_on_target_deadline TEXT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);
