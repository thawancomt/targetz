-- Add migration script here
CREATE TABLE IF NOT EXISTS project_status_history(
  from_status TEXT NOT NULL,
  to_status TEXT NOT NULL,
  note TEXT NULL,
  change_ask_by TEXT NULL,
  reason TEXT NULL,
  impact_on_target_deadline
);
