CREATE TABLE IF NOT EXISTS sessions (
  id          TEXT        NOT NULL PRIMARY KEY,
  secret_hash BYTEA       NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL
);
