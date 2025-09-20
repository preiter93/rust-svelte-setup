CREATE TABLE IF NOT EXISTS sessions (
  id          TEXT        NOT NULL PRIMARY KEY,
  secret_hash BYTEA       NOT NULL,
  user_id     UUID        NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL,
  expires_at  TIMESTAMPTZ NOT NULL
);
