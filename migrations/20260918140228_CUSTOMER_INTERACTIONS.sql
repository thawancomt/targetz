-- Add migration script here
CREATE TABLE IF NOT EXISTS customer_interaction(
  interaction_id INTEGER NOT NULL,
  customer_id INTEGER NOT NULL,
  FOREIGN KEY(interaction_id) REFERENCES interaction(id) ON DELETE SET NULL,
  FOREIGN KEY(customer_id) REFERENCES customers(id) ON DELETE SET NULL
);
