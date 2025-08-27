CREATE TABLE IF NOT EXISTS users (
  id         UUID        NOT NULL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  name       TEXT        NOT NULL,
  email      TEXT        NOT NULL,
  google_id  TEXT,
  github_id  TEXT
);
