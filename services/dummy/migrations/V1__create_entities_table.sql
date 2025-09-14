CREATE TABLE IF NOT EXISTS entities (
  id         UUID        NOT NULL PRIMARY KEY,
  user_id    UUID        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
