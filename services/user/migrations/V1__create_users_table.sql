CREATE TABLE IF NOT EXISTS users (
  id         UUID        NOT NULL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  google_id  TEXT
);
