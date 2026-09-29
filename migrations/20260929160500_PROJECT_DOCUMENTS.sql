-- Add migration script here
CREATE TABLE IF NOT EXISTS documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hash TEXT NOT NULL,
  extension TEXT NOT NULL,
  mtime INTEGER NOT NULL,
  original_name TEXT NOT NULL,
  path TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_document (
  project_id INTEGER NOT NULL,
  document_id INTEGER NOT NULL,
  PRIMARY KEY (project_id, document_id),
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_document_customer (
  project_id INTEGER NOT NULL,
  customer_id INTEGER NOT NULL,
  document_id INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'not_confirmed' CHECK (status IN ('not_confirmed', 'confirmed')),
  PRIMARY KEY (project_id, customer_id, document_id),
  FOREIGN KEY (project_id, document_id) REFERENCES project_document(project_id, document_id) ON DELETE CASCADE,
  FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_documents_hash 
  ON documents(hash);

CREATE INDEX IF NOT EXISTS idx_project_document_document_id 
  ON project_document(document_id);

CREATE INDEX IF NOT EXISTS idx_project_document_customer_document_id 
  ON project_document_customer(document_id);

CREATE INDEX IF NOT EXISTS idx_project_document_customer_customer_id 
  ON project_document_customer(customer_id);
