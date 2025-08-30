CREATE TABLE IF NOT EXISTS oauth_accounts (
  id               TEXT        NOT NULL PRIMARY KEY,
  provider         INTEGER     NOT NULL,
  provider_user_id TEXT        NOT NULL UNIQUE,
  access_token     TEXT        NULL,
  refresh_token    TEXT        NULL,
  user_id          TEXT        NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
