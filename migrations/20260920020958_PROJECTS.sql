-- Add migration script here
CREATE TABLE IF NOT EXISTS projects(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT NULL,
  codename TEXT NULL UNIQUE,
  current_version TEXT NOT NULL,
  budget FLOAT NULL,
  status TEXT NOT NULL DEFAULT "started",
  site_url TEXT NULL,
  start_date TEXT NULL,
  target_deadline TEXT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_DATE,
  updated_at TEXT NULL
);
