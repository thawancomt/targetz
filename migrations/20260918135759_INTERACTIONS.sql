-- Add migration script here
CREATE TABLE IF NOT EXISTS interaction(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  interaction_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  status TEXT NOT NULL DEFAULT "contacted",
  note TEXT NULL
);
